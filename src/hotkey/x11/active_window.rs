//! The active window of the desktop followed, so that the hotkey is grabbed only while it is
//! the game.

use std::collections::HashSet;

use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::{
    Atom, AtomEnum, ChangeWindowAttributesAux, ConnectionExt as _, EventMask, PropertyNotifyEvent,
    Window,
};
use x11rb::rust_connection::RustConnection;

use crate::game_window::{is_game_class, is_game_title};
use crate::hotkey::grab::Active;
use crate::logging;
use crate::x11::{Result, Session};

use super::window_class;

/// Follows the active window of the desktop on a thread of its own, and tells `game` what it is:
/// at the start, then at each change of the active window or of its class or title (a window may
/// be titled after it is shown), until `game` returns `false`. When the desktop doesn't tell its
/// active window (checked at the start), or the X server is lost, `game` is told once that it
/// can't be followed ([`Active::NotFollowed`]), and the thread ends.
///
/// # Errors
///
/// When the X server can't be reached: told at once, before the thread starts.
pub fn follow_game(mut game: impl FnMut(Active) -> bool + Send + 'static) -> Result<()> {
    let watch = ActiveWindow::connect()?;
    std::thread::Builder::new()
        .name("game-window".to_owned())
        .spawn(move || {
            if let Err(error) = watch.follow(&mut game) {
                log::warn!("The game window can't be followed: {error}");
                game(Active::NotFollowed);
            }
        })?;
    Ok(())
}

/// The atoms the active window is followed with.
#[derive(Debug, Clone, Copy)]
struct WatchAtoms {
    /// `_NET_ACTIVE_WINDOW`, on the root window.
    active_window: Atom,
    /// `_NET_WM_NAME`, the title of a window in UTF-8.
    net_wm_name: Atom,
    /// `UTF8_STRING`, the type of `_NET_WM_NAME`.
    utf8_string: Atom,
}

/// The connection that follows the active window.
struct ActiveWindow {
    session: Session,
    atoms: WatchAtoms,
}

impl ActiveWindow {
    /// Connects to the X server, with the atoms the active window is followed with.
    fn connect() -> Result<Self> {
        let session = Session::connect()?;
        let atoms = WatchAtoms {
            active_window: session.atoms().active_window,
            net_wm_name: session.atoms().net_wm_name,
            utf8_string: session.atoms().utf8_string,
        };
        Ok(Self { session, atoms })
    }

    /// Follows the active window until `game` returns `false`.
    ///
    /// # Errors
    ///
    /// When the desktop doesn't tell its active window, or the X server is lost.
    fn follow(&self, game: &mut impl FnMut(Active) -> bool) -> Result<()> {
        // Once, for the bug reports: the window over the game leans on what the window manager
        // does with the active window and the focus, checked on KWin only.
        match self.window_manager() {
            Ok(name) => log::info!("Window manager: {name}"),
            Err(error) => log::info!("Window manager: unknown ({error})"),
        }
        // A desktop started after the application leaves the hotkey grabbed for every window.
        if !self.session.supports(self.atoms.active_window)? {
            return Err("the desktop doesn't tell its active window (_NET_SUPPORTED)".into());
        }
        // Listened to before the active window is read, so that no change is missed.
        self.listen(self.session.root(), EventMask::PROPERTY_CHANGE)?
            .check()?;
        let mut followed = None;
        // The classes of the windows already written to the log.
        let mut classes_told = HashSet::new();
        loop {
            let active = known_active(self.session.active_window()?, self.session.root());
            if active != followed {
                // The windows may be gone: their errors are dropped.
                if let Some(window) = followed {
                    self.listen(window, EventMask::NO_EVENT)?.ignore_error();
                }
                if let Some(window) = active {
                    self.listen(window, EventMask::PROPERTY_CHANGE)?
                        .ignore_error();
                }
                followed = active;
            }
            // None: the desktop tells no active window, which may be the window over the game
            // keeping the keyboard (`crate::overlay::keyboard`), or a window outside X11.
            let kind = active.map_or(Active::Unknown, |window| {
                match self.is_game(window, &mut classes_told) {
                    Ok(true) => Active::Game,
                    Ok(false) => Active::Other,
                    Err(error) => {
                        log::info!("The active window can't be read: {error}");
                        Active::Other
                    }
                }
            });
            if !game(kind) {
                return Ok(());
            }
            self.wait_for_change(followed)?;
        }
    }

    /// The name of the window manager: the `_NET_WM_NAME` of the window it names on the root
    /// window (`_NET_SUPPORTING_WM_CHECK`, EWMH), e.g. `KWin` or `GNOME Shell`.
    fn window_manager(&self) -> Result<String> {
        let connection = self.session.connection();
        let check = self.session.intern(b"_NET_SUPPORTING_WM_CHECK")?;
        let window = connection
            .get_property(false, self.session.root(), check, AtomEnum::WINDOW, 0, 1)?
            .reply()?
            .value32()
            .and_then(|mut windows| windows.next());
        let Some(window) = window else {
            return Ok("none told (_NET_SUPPORTING_WM_CHECK)".to_owned());
        };
        let name = connection
            .get_property(
                false,
                window,
                self.atoms.net_wm_name,
                self.atoms.utf8_string,
                0,
                256,
            )?
            .reply()?
            .value;
        Ok(window_manager_name(&name))
    }

    /// Waits for a change that may change whether the active window is the game.
    fn wait_for_change(&self, followed: Option<Window>) -> Result<()> {
        loop {
            if let Event::PropertyNotify(event) = self.session.connection().wait_for_event()?
                && matters(&event, self.session.root(), followed, self.atoms)
            {
                return Ok(());
            }
        }
    }

    /// Sets the events of `window` this connection gets.
    fn listen(
        &self,
        window: Window,
        events: EventMask,
    ) -> Result<x11rb::cookie::VoidCookie<'_, RustConnection>> {
        Ok(self.session.connection().change_window_attributes(
            window,
            &ChangeWindowAttributesAux::new().event_mask(events),
        )?)
    }

    /// Whether `window` is the game, by its class or its title; the class of a window told by its
    /// title, the game or not, is written to the log the first time it is active, never its
    /// title (the page of a browser), so that the class of a client of the game not known yet can
    /// be found there.
    fn is_game(&self, window: Window, classes_told: &mut HashSet<String>) -> Result<bool> {
        // The helper of the window over the game, active in its place on a single screen
        // (`crate::overlay`): the hotkey stays grabbed, and its copy gives the game back first.
        if self.session.game_behind(window)?.is_some() {
            return Ok(true);
        }
        let class = self.session.window_class(window)?;
        if is_game_class(&class) {
            return Ok(true);
        }
        let is_game = is_game_title(&self.session.window_title(window)?);
        let class = window_class(&class);
        if !classes_told.contains(&class) {
            // The class of the game told by its title (a client run by Wine, whose class is
            // unknown): the bug report keeps it in the log as the game's.
            if is_game {
                log::info!("The active window, of class {class}, is the game");
            } else {
                log::info!("The active window, of class {class}, isn't the game");
            }
            classes_told.insert(class);
        }
        Ok(is_game)
    }
}

/// The active window read, none for the root window, which a desktop may write there: listened to
/// for its active window, it must never be followed as one, whose events would be turned off once
/// it is no longer active.
fn known_active(active: Option<Window>, root: Window) -> Option<Window> {
    active.filter(|&window| window != root)
}

/// Whether `event` may change whether the active window is the game: the active window of `root`,
/// or the class or the title of the window `followed`.
fn matters(
    event: &PropertyNotifyEvent,
    root: Window,
    followed: Option<Window>,
    atoms: WatchAtoms,
) -> bool {
    if event.window == root {
        return event.atom == atoms.active_window;
    }
    Some(event.window) == followed
        && [
            AtomEnum::WM_CLASS.into(),
            AtomEnum::WM_NAME.into(),
            atoms.net_wm_name,
        ]
        .contains(&event.atom)
}

/// The name of a window manager from its `_NET_WM_NAME`, as the log writes the text of another
/// program: one short line.
fn window_manager_name(name: &[u8]) -> String {
    if name.is_empty() {
        return "unnamed".to_owned();
    }
    logging::plain(&String::from_utf8_lossy(name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use x11rb::protocol::xproto;

    const ROOT: Window = 0x410;

    const FOLLOWED: Window = 0x0060_0001;
    const ATOMS: WatchAtoms = WatchAtoms {
        active_window: 300,
        net_wm_name: 302,
        utf8_string: 303,
    };

    /// A change of `atom` on `window`.
    fn changed(window: Window, atom: Atom) -> PropertyNotifyEvent {
        PropertyNotifyEvent {
            response_type: xproto::PROPERTY_NOTIFY_EVENT,
            sequence: 0,
            window,
            atom,
            time: 0,
            state: xproto::Property::NEW_VALUE,
        }
    }

    #[test]
    fn follows_the_active_window_of_the_root_window_only() {
        assert!(matters(
            &changed(ROOT, ATOMS.active_window),
            ROOT,
            None,
            ATOMS
        ));
        // `_NET_CLIENT_LIST` and the others on the root window.
        assert!(!matters(&changed(ROOT, 400), ROOT, None, ATOMS));
        // The title of the root window.
        assert!(!matters(
            &changed(ROOT, ATOMS.net_wm_name),
            ROOT,
            Some(ROOT),
            ATOMS
        ));
    }

    #[test]
    fn follows_the_class_and_the_titles_of_the_active_window() {
        for atom in [
            AtomEnum::WM_CLASS.into(),
            AtomEnum::WM_NAME.into(),
            ATOMS.net_wm_name,
        ] {
            assert!(matters(
                &changed(FOLLOWED, atom),
                ROOT,
                Some(FOLLOWED),
                ATOMS
            ));
        }
    }

    #[test]
    fn leaves_the_other_properties_and_the_windows_no_longer_active() {
        // `_NET_WM_USER_TIME`, written at each key.
        assert!(!matters(
            &changed(FOLLOWED, 401),
            ROOT,
            Some(FOLLOWED),
            ATOMS
        ));
        // The title of a window that was active before.
        assert!(!matters(
            &changed(FOLLOWED + 1, ATOMS.net_wm_name),
            ROOT,
            Some(FOLLOWED),
            ATOMS
        ));
        assert!(!matters(
            &changed(FOLLOWED, ATOMS.net_wm_name),
            ROOT,
            None,
            ATOMS
        ));
    }

    #[test]
    fn never_follows_the_root_window_as_the_active_window() {
        assert_eq!(known_active(Some(ROOT), ROOT), None);
        assert_eq!(known_active(Some(FOLLOWED), ROOT), Some(FOLLOWED));
        assert_eq!(known_active(None, ROOT), None);
    }

    #[test]
    fn the_name_of_the_window_manager_is_written_as_one_short_line() {
        assert_eq!(window_manager_name(b"KWin"), "KWin");
        assert_eq!(window_manager_name(b"GNOME\nShell"), "GNOME Shell");
        assert_eq!(window_manager_name(b""), "unnamed");
        assert!(window_manager_name(&[b'a'; 5_000]).chars().count() < 250);
    }
}
