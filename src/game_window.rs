//! The windows of Path of Exile 2: how a window of the game is told from the other windows, by
//! its class or its exact title, alike for the hotkey ([`crate::hotkey`]) and the bug report
//! ([`crate::bug_report`]).

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

#[cfg(all(test, any(target_os = "linux", target_os = "windows")))]
mod tests;
