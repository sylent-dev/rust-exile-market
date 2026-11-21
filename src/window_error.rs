//! The message of a window that can't open (specifications, Error handling; mock-up section
//! "Window can't open (17a)"). The application can't draw anything then: the message goes to a
//! dialog of the system (`crate::system_dialog`), to the terminal and to the log.

use std::path::Path;

use crate::i18n::Translator;

/// Why the window couldn't open, as far as the player can act on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cause {
    /// No X11 display: neither X11 nor XWayland (Linux only, where the window always goes
    /// through X11).
    Display,
    /// No OpenGL 2.1, the least the renderer of egui draws with.
    OpenGl,
    /// Anything else, to report.
    Other,
}

impl Cause {
    /// The cause of `error`, returned by `eframe::run_native`.
    #[must_use]
    pub fn of(error: &eframe::Error) -> Self {
        match error {
            // The event loop of winit, forced to X11, fails without its connection to the
            // display (`XNotSupported`): the other systems always have their display.
            eframe::Error::WinitEventLoop(_) if cfg!(target_os = "linux") => Self::Display,
            eframe::Error::Glutin(_)
            | eframe::Error::NoGlutinConfigs(..)
            | eframe::Error::OpenGL(_) => Self::OpenGl,
            _ => Self::Other,
        }
    }
}

/// The text of the message: what failed and what to do, then the log for a cause to report,
/// then the error itself, in paragraphs.
#[must_use]
pub fn message(translator: &Translator, cause: Cause, error: &str, log: Option<&Path>) -> String {
    let mut paragraphs = translator.window_error(cause).to_vec();
    if let (Cause::Other, Some(log)) = (cause, log) {
        paragraphs.push(translator.window_error_log(&log.display().to_string()));
    }
    paragraphs.push(translator.window_error_details(&without_source(error)));
    paragraphs.join("\n\n")
}

/// `error` without the place in the source of winit that a winit error writes (`os error at
/// /home/…/.cargo/…/mod.rs:788: Failed to open connection to X server`), of no use to the player:
/// the log keeps the whole error.
fn without_source(error: &str) -> String {
    for (at, _) in error.match_indices(" at ") {
        let place = &error[at + " at ".len()..];
        // The place is one path, without a space.
        let Some(rs) = place.find(".rs:").filter(|&rs| !place[..rs].contains(' ')) else {
            continue;
        };
        let line = &place[rs + ".rs:".len()..];
        let after_line = line.trim_start_matches(|c: char| c.is_ascii_digit());
        if after_line.len() == line.len() {
            continue;
        }
        if let Some(rest) = after_line.strip_prefix(": ") {
            return format!("{}: {rest}", &error[..at]);
        }
    }
    error.to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::Language;

    #[test]
    #[cfg(target_os = "linux")]
    fn a_failed_event_loop_on_linux_is_a_missing_display() {
        let error = eframe::Error::WinitEventLoop(winit::error::EventLoopError::RecreationAttempt);
        assert_eq!(Cause::of(&error), Cause::Display);
    }

    #[test]
    fn a_failed_creation_of_the_application_is_a_cause_to_report() {
        let error = eframe::Error::AppCreation("no state".into());
        assert_eq!(Cause::of(&error), Cause::Other);
    }

    #[test]
    fn the_message_of_a_missing_display_says_what_to_install_then_the_error() {
        let translator = Translator::new(Language::English);
        let text = message(
            &translator,
            Cause::Display,
            "X11 not supported",
            Some(Path::new(
                "/home/player/.local/state/exile-market/exile-market.log",
            )),
        );
        assert_eq!(
            text,
            "Exile Market can't open its window: it needs X11, or XWayland on a Wayland \
             desktop.\n\n\
             Install XWayland (the package xwayland on most distributions), then launch Exile \
             Market again.\n\n\
             Details: X11 not supported"
        );
    }

    #[test]
    fn the_message_of_a_missing_opengl_asks_to_update_the_driver() {
        let translator = Translator::new(Language::French);
        let text = message(&translator, Cause::OpenGl, "no configs", None);
        assert_eq!(
            text,
            "Exile Market ne peut pas ouvrir sa fenêtre\u{a0}: il lui faut OpenGL 2.1 ou plus \
             récent, que le pilote graphique ne fournit pas.\n\n\
             Mettez à jour le pilote graphique, puis relancez Exile Market.\n\n\
             Détails\u{a0}: no configs"
        );
    }

    #[test]
    fn another_cause_asks_to_report_it_with_the_log() {
        let translator = Translator::new(Language::English);
        let text = message(
            &translator,
            Cause::Other,
            "failed",
            Some(Path::new("/logs/exile-market.log")),
        );
        assert_eq!(
            text,
            "Exile Market can't open its window.\n\n\
             Report it on GitHub with its log, so that it can be fixed: \
             https://github.com/sylent-dev/rust-exile-market/issues\n\n\
             Log: /logs/exile-market.log\n\n\
             Details: failed"
        );
    }

    #[test]
    fn the_details_leave_out_the_place_in_the_source_of_winit() {
        assert_eq!(
            without_source(
                "winit EventLoopError: os error at /home/runner/.cargo/registry/src/\
                 winit-0.30.13/src/platform_impl/linux/mod.rs:788: Failed to open connection \
                 to X server"
            ),
            "winit EventLoopError: os error: Failed to open connection to X server"
        );
        assert_eq!(without_source("failed at the start"), "failed at the start");
        // Only a place with a line number, after any number of other " at ".
        assert_eq!(
            without_source("lost at sea: os error at src/x11.rs:12: no display"),
            "lost at sea: os error: no display"
        );
        assert_eq!(
            without_source("read at file.rs:abc: kept"),
            "read at file.rs:abc: kept"
        );
    }

    #[test]
    fn another_cause_without_a_log_file_still_gives_the_error() {
        let translator = Translator::new(Language::English);
        let text = message(&translator, Cause::Other, "failed", None);
        assert!(!text.contains("Log:"), "{text}");
        assert!(text.ends_with("Details: failed"), "{text}");
    }
}
