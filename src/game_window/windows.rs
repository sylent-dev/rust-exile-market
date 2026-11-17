//! The game seen open under Windows: one of its top-level windows, shown or minimized (never a
//! hidden one), read once a second.

use std::io;
use std::thread;
use std::time::{Duration, Instant};

use winsafe::HWND;

use super::{ReadingSender, Seen, is_game_window};

/// How often the windows are read. Windows tells their changes only to a hook
/// (`SetWinEventHook`), whose callback can't be written without `unsafe`: read once a second
/// instead, the closing waiting [`super::CLOSED_AFTER`] anyway, so that the application closes
/// up to a second later than under X11.
const WINDOWS_POLL: Duration = Duration::from_secs(1);

/// Reads the windows of the desktop on a thread of its own, and sends whether one of them is the
/// game: at the start, then at each change, until the receiver is dropped. When they can't be
/// read, the reason goes to the log once, and the thread ends.
///
/// # Errors
///
/// When the thread can't start.
pub(super) fn follow(mut readings: ReadingSender<impl Fn() + Send + 'static>) -> io::Result<()> {
    thread::Builder::new()
        .name("game-open".to_owned())
        .spawn(move || {
            loop {
                let at = Instant::now();
                let is_open = match is_game_open() {
                    Ok(is_open) => is_open,
                    Err(error) => {
                        log::warn!("Whether the game is open can't be followed: {error}");
                        return;
                    }
                };
                if readings.send(Seen { is_open, at }).is_err() {
                    return;
                }
                thread::sleep(WINDOWS_POLL);
            }
        })?;
    Ok(())
}

/// Whether one of the visible top-level windows is the game ([`is_game_window`]), read for the
/// windows of the other applications only (the title of a window of this process is read through
/// a message to the thread of the window, which this one would wait for). A minimized window is
/// visible; a hidden one, which a program may keep titled after the game, isn't the game open.
fn is_game_open() -> winsafe::SysResult<bool> {
    let mut is_open = false;
    // Every window, never stopped early: a callback that stops it makes `EnumWindows` fail.
    winsafe::EnumWindows(|window: HWND| {
        if !is_open && window.IsWindowVisible() && !crate::overlay::is_ours(&window) {
            let title = window.GetWindowText().unwrap_or_default();
            let class = window.GetClassName().unwrap_or_default();
            is_open = is_game_window(&title, &class);
        }
        true
    })?;
    Ok(is_open)
}
