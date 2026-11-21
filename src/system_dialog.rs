//! The dialog of the system, the one way left to tell the player something when the application
//! has no window to draw in: why its window can't open (`crate::window_error`), or that it already
//! runs (`crate::single_instance`). `MessageBox` on Windows, kdialog or zenity on Linux, which open
//! under Wayland too, an alert on macOS.

#[cfg(target_os = "linux")]
use std::borrow::Cow;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::process::{Command, Stdio};

/// The sign of a dialog of the system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    /// Something failed.
    Error,
    /// Nothing failed: the player is told something.
    Information,
}

/// Shows `message` in the terminal, then in a dialog of the system with `sign`, which waits for
/// the player.
pub fn show(message: &str, sign: Sign) {
    eprintln!("{message}");
    if !dialog(message, sign) {
        log::warn!("No dialog of the system could show the message");
    }
}

/// The dialog of Windows; true once shown.
#[cfg(target_os = "windows")]
fn dialog(message: &str, sign: Sign) -> bool {
    use winsafe::{HWND, co, prelude::Handle};
    let icon = match sign {
        Sign::Error => co::MB::ICONERROR,
        Sign::Information => co::MB::ICONINFORMATION,
    };
    HWND::NULL
        .MessageBox(
            message,
            crate::APP_NAME,
            co::MB::OK | icon | co::MB::SETFOREGROUND,
        )
        .is_ok()
}

/// An alert of macOS, through `AppleScript`, its texts given as arguments so that nothing in them
/// needs escaping; true once shown.
#[cfg(target_os = "macos")]
fn dialog(message: &str, sign: Sign) -> bool {
    let alert = match sign {
        Sign::Error => "display alert (item 1 of argv) message (item 2 of argv) as critical",
        Sign::Information => {
            "display alert (item 1 of argv) message (item 2 of argv) as informational"
        }
    };
    Command::new("osascript")
        .args([
            "-e",
            "on run argv",
            "-e",
            alert,
            "-e",
            "end run",
            crate::APP_NAME,
            message,
        ])
        // It writes the button clicked.
        .stdout(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

/// No dialog on the other systems: the terminal and the log only.
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn dialog(_message: &str, _sign: Sign) -> bool {
    false
}

/// The first dialog tool of the desktop that runs; true once one has shown it.
#[cfg(target_os = "linux")]
fn dialog(message: &str, sign: Sign) -> bool {
    let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
    linux_tools(&desktop).into_iter().any(|tool| {
        Command::new(tool.program())
            .args(
                tool.arguments(message, sign)
                    .iter()
                    .map(AsRef::<str>::as_ref),
            )
            .stdout(Stdio::null())
            .status()
            .is_ok_and(|status| shown(status.code()))
    })
}

/// Whether a dialog tool that exited with `code` showed its dialog: 0 once closed with OK, 1 with
/// the cross; any other code, or none (killed by a signal), is a failure, and the next tool is
/// tried.
#[cfg(target_os = "linux")]
fn shown(code: Option<i32>) -> bool {
    matches!(code, Some(0 | 1))
}

/// A dialog tool of Linux.
#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tool {
    /// Of KDE (Qt).
    Kdialog,
    /// Of GNOME (GTK), found on most other desktops too.
    Zenity,
}

#[cfg(target_os = "linux")]
impl Tool {
    fn program(self) -> &'static str {
        match self {
            Self::Kdialog => "kdialog",
            Self::Zenity => "zenity",
        }
    }

    fn arguments(self, message: &str, sign: Sign) -> Vec<Cow<'_, str>> {
        match self {
            Self::Kdialog => vec![
                "--title".into(),
                crate::APP_NAME.into(),
                kdialog_kind(sign).into(),
                kdialog_text(message).into(),
            ],
            // Without markup: a `<` or a `&` of the error would otherwise break the text.
            Self::Zenity => [
                zenity_kind(sign),
                "--no-markup",
                "--title",
                crate::APP_NAME,
                "--text",
                message,
            ]
            .map(Cow::from)
            .into(),
        }
    }
}

/// `message` written for kdialog, which reads its text as the rich text of Qt when it looks like
/// some (checked on KDE Plasma, 2026-11-20: `<b>` and `<i>` shown bold and italic): inside
/// `<qt>`, a tag at its start, so that Qt always reads it as rich text, its `&`, `<` and `>`
/// escaped, so that none is taken for markup, and its line breaks as `<br>`, which rich text
/// would otherwise join into one line.
#[cfg(target_os = "linux")]
fn kdialog_text(message: &str) -> String {
    let mut text = String::from("<qt>");
    for character in message.chars() {
        match character {
            '&' => text.push_str("&amp;"),
            '<' => text.push_str("&lt;"),
            '>' => text.push_str("&gt;"),
            '\n' => text.push_str("<br>"),
            _ => text.push(character),
        }
    }
    text.push_str("</qt>");
    text
}

/// The kind of dialog of kdialog that shows `sign`.
#[cfg(target_os = "linux")]
fn kdialog_kind(sign: Sign) -> &'static str {
    match sign {
        Sign::Error => "--error",
        Sign::Information => "--msgbox",
    }
}

/// The kind of dialog of zenity that shows `sign`.
#[cfg(target_os = "linux")]
fn zenity_kind(sign: Sign) -> &'static str {
    match sign {
        Sign::Error => "--error",
        Sign::Information => "--info",
    }
}

/// The tools tried in order: the one of the desktop first (`XDG_CURRENT_DESKTOP`, e.g. `KDE` or
/// `ubuntu:GNOME`), so that the dialog looks like the others there.
#[cfg(target_os = "linux")]
fn linux_tools(desktop: &str) -> [Tool; 2] {
    if desktop
        .split(':')
        .any(|name| name.eq_ignore_ascii_case("KDE"))
    {
        [Tool::Kdialog, Tool::Zenity]
    } else {
        [Tool::Zenity, Tool::Kdialog]
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;

    #[test]
    fn a_dialog_tool_that_fails_lets_the_next_one_be_tried() {
        assert!(shown(Some(0)));
        assert!(shown(Some(1)));
        assert!(!shown(Some(255)));
        assert!(!shown(None));
    }

    #[test]
    fn the_dialog_tool_of_the_desktop_comes_first() {
        assert_eq!(linux_tools("KDE"), [Tool::Kdialog, Tool::Zenity]);
        assert_eq!(linux_tools("ubuntu:GNOME"), [Tool::Zenity, Tool::Kdialog]);
        assert_eq!(linux_tools(""), [Tool::Zenity, Tool::Kdialog]);
    }

    #[test]
    fn zenity_shows_the_message_without_markup() {
        assert_eq!(
            Tool::Zenity.arguments("a < b & c\n\nd", Sign::Error),
            [
                "--error",
                "--no-markup",
                "--title",
                "Exile Market",
                "--text",
                "a < b & c\n\nd"
            ]
        );
    }

    #[test]
    fn kdialog_shows_the_message_as_written_in_its_rich_text() {
        assert_eq!(
            Tool::Kdialog.arguments("Details: <b>a</b> & c > d\n\nLog: /logs/a.log", Sign::Error),
            [
                "--title",
                "Exile Market",
                "--error",
                "<qt>Details: &lt;b&gt;a&lt;/b&gt; &amp; c &gt; d<br><br>Log: /logs/a.log</qt>"
            ]
        );
    }

    #[test]
    fn a_message_that_tells_no_failure_has_the_sign_of_information() {
        assert_eq!(
            Tool::Kdialog.arguments("text", Sign::Information),
            ["--title", "Exile Market", "--msgbox", "<qt>text</qt>"]
        );
        assert_eq!(
            Tool::Zenity.arguments("text", Sign::Information),
            [
                "--info",
                "--no-markup",
                "--title",
                "Exile Market",
                "--text",
                "text"
            ]
        );
    }
}
