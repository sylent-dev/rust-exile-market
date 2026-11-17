//! The game seen open under Windows: one of its top-level windows, shown or minimized, read once
//! a second.

use std::io;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::{Duration, Instant};

use winsafe::HWND;

use super::{Seen, is_game_window};

/// How often the windows are read. Windows tells their changes only to a hook
/// (`SetWinEventHook`), whose callback can't be written without `unsafe`: read once a second
/// instead, a few hundred windows in a millisecond or so, the closing waiting
/// [`super::CLOSED_AFTER`] anyway.
const WINDOWS_POLL: Duration = Duration::from_secs(1);

/// Reads the windows of the desktop on a thread of its own, and sends whether one of them is the
/// game: at the start, then at each change, until the receiver is dropped. When they can't be
/// read, the reason goes to the log once, and the thread ends.
///
/// # Errors
///
/// When the thread can't start.
pub fn follow(seen: Sender<Seen>, wake: impl Fn() + Send + 'static) -> io::Result<()> {
    thread::Builder::new()
        .name("game-open".to_owned())
        .spawn(move || {
            let mut was_open = None;
            loop {
                let at = Instant::now();
                let is_open = match is_game_open() {
                    Ok(is_open) => is_open,
                    Err(error) => {
                        log::warn!("Whether the game is open can't be followed: {error}");
                        return;
                    }
                };
                if was_open != Some(is_open) {
                    if seen.send(Seen { is_open, at }).is_err() {
                        return;
                    }
                    wake();
                    was_open = Some(is_open);
                }
                thread::sleep(WINDOWS_POLL);
            }
        })?;
    Ok(())
}

/// Whether one of the top-level windows is the game ([`is_game_window`]), read for the windows of
/// the other applications only (the title of a window of this process is read through a message
/// to the thread of the window, which this one would wait for).
fn is_game_open() -> winsafe::SysResult<bool> {
    let mut is_open = false;
    // Every window, never stopped early: a callback that stops it makes `EnumWindows` fail.
    winsafe::EnumWindows(|window: HWND| {
        if !is_open && !crate::overlay::is_ours(&window) {
            let title = window.GetWindowText().unwrap_or_default();
            let class = window.GetClassName().unwrap_or_default();
            is_open = is_game_window(&title, &class);
        }
        true
    })?;
    Ok(is_open)
}
