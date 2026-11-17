//! The windows of Path of Exile 2: how a window of the game is told from the other windows, by
//! its class or its exact title, alike for the hotkey ([`crate::hotkey`]) and the bug report
//! ([`crate::bug_report`]); and whether the game is open, so that the application can close with
//! it (`specs.md`, Settings).
//!
//! The game is open while one of its windows is, not only active, and closed once they have all
//! been gone for [`CLOSED_AFTER`]. The windows are read on a thread of the system, which tells
//! what it saw with the instant it saw it; the window takes it in and decides. On Linux from the
//! window list of X11, on Windows from its top-level windows; never on macOS, where the game has
//! no client.

use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::{Duration, Instant};

#[cfg(target_os = "linux")]
mod x11;

#[cfg(target_os = "linux")]
use self::x11 as system;

/// The exact title of the window of the game: the title Exiled Exchange 2 looks for
/// (`attachByTitle`), the standalone client and the one of Steam alike on Windows, and the
/// standalone client run by Wine on Linux, whose class is unknown. Never a part of the title: the
/// page of a browser about the game is titled `Path of Exile 2 - …`.
#[cfg(any(target_os = "linux", target_os = "windows"))]
pub const GAME_TITLE: &str = "Path of Exile 2";

/// The class of the window of the game run by Steam through Proton, logged by step 14a1 in the
/// owner's game on 2026-11-01; the bug report keeps it in the log, as the name of the game.
pub const GAME_CLASS: &str = "steam_app_2694490";

/// The class of a window of the File Explorer of Windows, titled after its folder: the folder of
/// the game is titled `Path of Exile 2` too, and Ctrl+D there must stay to it (a copy sent there
/// would copy the files selected).
#[cfg(any(target_os = "windows", all(test, target_os = "linux")))]
const EXPLORER_CLASS: &str = "CabinetWClass";

/// Whether a window of another application, of `title` and `class`, is the game on Windows: its
/// exact title, unless it is a folder of the File Explorer.
#[cfg(any(target_os = "windows", all(test, target_os = "linux")))]
pub fn is_game_window(title: &str, class: &str) -> bool {
    title == GAME_TITLE && class != EXPLORER_CLASS
}

/// Whether a window title, as X11 keeps it, is the title of the game, exactly.
#[cfg(target_os = "linux")]
pub fn is_game_title(title: &[u8]) -> bool {
    title == GAME_TITLE.as_bytes()
}

/// Whether a `WM_CLASS` of X11, an instance name then a class name each ended by a null byte, is
/// the class of the game.
#[cfg(target_os = "linux")]
pub fn is_game_class(property: &[u8]) -> bool {
    property
        .split(|&byte| byte == 0)
        .any(|name| name == GAME_CLASS.as_bytes())
}

/// How long the windows of the game must stay gone before it is told closed, so that a window the
/// game makes again (a change of its display mode, maybe) isn't taken for its end (owner,
/// 2026-11-17): the application closes up to this long after the game.
pub const CLOSED_AFTER: Duration = Duration::from_secs(5);

/// Whether a window of the game was open, as the thread of the system read the windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seen {
    pub is_open: bool,
    /// When the windows were read.
    pub at: Instant,
}

/// What the game did, as the window is told.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    /// A window of the game opened, the first since the start or since it closed.
    Opened,
    /// Its windows have been gone for [`CLOSED_AFTER`].
    Closed,
}

/// Whether the game is open, from what the windows told.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum Presence {
    /// Not seen open since the start, or since it closed.
    #[default]
    Closed,
    Open,
    /// Its windows gone since `since`, not long enough to tell it closed.
    Gone {
        since: Instant,
    },
}

impl Presence {
    /// Takes in what the windows told: first the closing its instant reached, then what it
    /// changes, so that the changes come in the order they happened.
    fn take(&mut self, seen: Seen) -> impl Iterator<Item = Change> {
        let due = self.check(seen.at);
        due.into_iter().chain(self.apply(seen))
    }

    fn apply(&mut self, seen: Seen) -> Option<Change> {
        match (*self, seen.is_open) {
            (Self::Closed, true) => {
                *self = Self::Open;
                Some(Change::Opened)
            }
            // Back before it was told closed: the same game.
            (Self::Gone { .. }, true) => {
                *self = Self::Open;
                None
            }
            (Self::Open, false) => {
                *self = Self::Gone { since: seen.at };
                None
            }
            (Self::Open, true) | (Self::Closed | Self::Gone { .. }, false) => None,
        }
    }

    /// The closing, once the windows of the game have been gone for [`CLOSED_AFTER`] at `now`.
    fn check(&mut self, now: Instant) -> Option<Change> {
        let deadline = self.deadline()?;
        (now >= deadline).then(|| {
            *self = Self::Closed;
            Change::Closed
        })
    }

    /// When the game will be told closed, if its windows stay gone.
    fn deadline(self) -> Option<Instant> {
        match self {
            Self::Gone { since } => Some(since + CLOSED_AFTER),
            Self::Closed | Self::Open => None,
        }
    }
}

/// Whether the game is open, followed for as long as this value lives.
pub struct GameWindowWatch {
    /// What the thread of the system read; none once it stopped, or on a system without it.
    seen: Option<Receiver<Seen>>,
    presence: Presence,
}

impl GameWindowWatch {
    /// Starts reading the windows of the system, `wake` called at each change. When they can't
    /// be read, the reason goes to the log once, and the game is never told open.
    pub fn start(wake: impl Fn() + Send + 'static) -> Self {
        let (sender, receiver) = mpsc::channel();
        #[cfg(target_os = "linux")]
        if let Err(error) = system::follow(sender, wake) {
            log::warn!("Whether the game is open can't be followed: {error}");
        }
        #[cfg(not(target_os = "linux"))]
        drop((sender, wake));
        Self {
            seen: Some(receiver),
            presence: Presence::default(),
        }
    }

    /// Takes in what the windows told since the last call, then the closing `now` reaches: the
    /// changes of the game, in their order, each written to the log. Once the thread of the
    /// system stops, the game is never told closed, as it is never seen again.
    pub fn follow(&mut self, now: Instant) -> Vec<Change> {
        let mut changes = Vec::new();
        let mut is_lost = false;
        if let Some(seen) = &self.seen {
            loop {
                match seen.try_recv() {
                    Ok(seen) => changes.extend(self.presence.take(seen)),
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        is_lost = true;
                        break;
                    }
                }
            }
        }
        if is_lost {
            self.seen = None;
            self.presence = Presence::Closed;
        }
        changes.extend(self.presence.check(now));
        for change in &changes {
            match change {
                Change::Opened => log::info!("Path of Exile 2 opened"),
                Change::Closed => log::info!("Path of Exile 2 closed"),
            }
        }
        changes
    }

    /// How long until the game may be told closed, so that the window wakes then.
    pub fn next_check(&self, now: Instant) -> Option<Duration> {
        Some(self.presence.deadline()?.saturating_duration_since(now))
    }
}

#[cfg(test)]
mod tests;
