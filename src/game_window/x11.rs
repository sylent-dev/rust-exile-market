//! The game seen open under X11: one of its windows in the window list of the desktop
//! (`_NET_CLIENT_LIST`, EWMH), minimized or not, read again at each change of the list and of the
//! active window (a window of the game titled after it is listed is seen once it is active).

use std::time::Instant;

use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::{
    Atom, AtomEnum, ChangeWindowAttributesAux, ConnectionExt as _, EventMask, PropertyNotifyEvent,
    Window,
};

use super::{Reading, ReadingSender, is_game_class, is_game_title};
use crate::x11::{Result, Session};

/// Reads the windows of the desktop on a thread of its own, and sends whether one of them is the
/// game: at the start, then at each change, until the receiver is dropped. When the desktop
/// doesn't tell its windows, or the X server is lost, the reason goes to the log once, and the
/// thread ends.
///
/// # Errors
///
/// When the X server can't be reached, or the thread can't start: told at once.
pub(super) fn follow(mut readings: ReadingSender<impl Fn() + Send + 'static>) -> Result<()> {
    let session = Session::connect()?;
    let client_list = session.intern(b"_NET_CLIENT_LIST")?;
    std::thread::Builder::new()
        .name("game-open".to_owned())
        .spawn(move || {
            if let Err(error) = follow_windows(&session, client_list, &mut readings) {
                log::warn!("Whether the game is open can't be followed: {error}");
            }
        })?;
    Ok(())
}

/// Sends whether a window of the game is open at each change, until the receiver is dropped.
///
/// # Errors
///
/// When the desktop doesn't tell its windows, or the X server is lost.
fn follow_windows(
    session: &Session,
    client_list: Atom,
    readings: &mut ReadingSender<impl Fn()>,
) -> Result<()> {
    if !session.supports(client_list)? {
        return Err("the desktop doesn't tell its windows (_NET_SUPPORTED)".into());
    }
    // Listened to before the windows are read, so that no change is missed. Each connection
    // keeps its own events of the root window: the hotkey listens on its own.
    session
        .connection()
        .change_window_attributes(
            session.root(),
            &ChangeWindowAttributesAux::new().event_mask(EventMask::PROPERTY_CHANGE),
        )?
        .check()?;
    let lists = [client_list, session.atoms().active_window];
    loop {
        let at = Instant::now();
        let is_open = windows(session, client_list)?
            .into_iter()
            .any(|window| is_game(session, window));
        if readings.send(Reading { is_open, at }).is_err() {
            return Ok(());
        }
        loop {
            if let Event::PropertyNotify(event) = session.connection().wait_for_event()?
                && is_list_change(&event, session.root(), lists)
            {
                break;
            }
        }
    }
}

/// The windows the desktop lists, `_NET_CLIENT_LIST` on the root window.
fn windows(session: &Session, client_list: Atom) -> Result<Vec<Window>> {
    // Read whole: the server sends the property up to its length.
    let list = session
        .connection()
        .get_property(
            false,
            session.root(),
            client_list,
            AtomEnum::WINDOW,
            0,
            u32::MAX,
        )?
        .reply()?;
    Ok(list.value32().into_iter().flatten().collect())
}

/// Whether `window` is the game, by its class or its title; a window gone since the list was
/// read isn't, nor one whose names can't be read: the X server lost fails the next read of the
/// list.
fn is_game(session: &Session, window: Window) -> bool {
    session
        .window_class(window)
        .is_ok_and(|class| is_game_class(&class))
        || session
            .window_title(window)
            .is_ok_and(|title| is_game_title(&title))
}

/// Whether `event` may change whether the game is open: a change of `lists`, the window list or
/// the active window, on `root`.
fn is_list_change(event: &PropertyNotifyEvent, root: Window, lists: [Atom; 2]) -> bool {
    event.window == root && lists.contains(&event.atom)
}

#[cfg(test)]
mod tests {
    use super::*;
    use x11rb::protocol::xproto;

    const ROOT: Window = 0x410;
    const CLIENT_LIST: Atom = 310;
    const ACTIVE_WINDOW: Atom = 300;

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
    fn the_windows_are_read_again_when_the_list_or_the_active_window_changes() {
        let lists = [CLIENT_LIST, ACTIVE_WINDOW];
        assert!(is_list_change(&changed(ROOT, CLIENT_LIST), ROOT, lists));
        assert!(is_list_change(&changed(ROOT, ACTIVE_WINDOW), ROOT, lists));
        // `_NET_CLIENT_LIST_STACKING`, changed at each window raised, and the same atom on
        // another window.
        assert!(!is_list_change(&changed(ROOT, 311), ROOT, lists));
        assert!(!is_list_change(
            &changed(0x0060_0001, CLIENT_LIST),
            ROOT,
            lists
        ));
    }
}
