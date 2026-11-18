//! The warning band under the title bar: what doesn't work, one sentence per warning.

use egui::{Frame, Label, Panel, RichText, Ui, vec2};

use super::theme::{color, size, text};

/// The space above and under the text.
const PADDING_Y: f32 = 10.0;
/// The space between two warnings.
const GAP: f32 = 4.0;

pub struct WarningBand {
    warnings: Vec<String>,
}

impl WarningBand {
    #[must_use]
    pub fn new(warnings: Vec<String>) -> Self {
        Self { warnings }
    }

    /// Shown after the title bar, so that it sits right under it; nothing without a warning.
    pub fn show(self, ui: &mut Ui) {
        if self.warnings.is_empty() {
            return;
        }
        Panel::top("warning-band")
            .resizable(false)
            .frame(
                Frame::NONE
                    .fill(color::WARNING_BAND)
                    .inner_margin(vec2(size::BAND_PADDING, PADDING_Y)),
            )
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = GAP;
                for warning in self.warnings {
                    let width = ui.fonts_mut(|fonts| {
                        fonts
                            .layout_no_wrap(warning.clone(), text::warning(), color::WARNING_TEXT)
                            .size()
                            .x
                    });
                    let warning = written(warning, width, ui.available_width());
                    ui.add(
                        Label::new(
                            RichText::new(warning)
                                .font(text::warning())
                                .color(color::WARNING_TEXT),
                        )
                        .wrap(),
                    );
                }
            });
    }
}

/// `warning` as the band writes it in `room` points, `width` wide on one line: whole when it
/// fits, else each of its sentences on a line of its own, so that none is cut in the middle while
/// it has room ("Dernière mise à jour il y a 3h." under "La réponse de la base des prix est
/// illisible.").
fn written(warning: String, width: f32, room: f32) -> String {
    if width <= room {
        return warning;
    }
    warning
        .split_inclusive(". ")
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    const UNREADABLE: &str =
        "La réponse de la base des prix est illisible. Dernière mise à jour il y a 3h.";

    #[test]
    fn a_warning_that_fits_stays_on_one_line() {
        assert_eq!(written(UNREADABLE.to_owned(), 300.0, 300.0), UNREADABLE);
    }

    #[test]
    fn a_warning_too_wide_puts_each_sentence_on_its_own_line() {
        assert_eq!(
            written(UNREADABLE.to_owned(), 301.0, 300.0),
            "La réponse de la base des prix est illisible.\nDernière mise à jour il y a 3h."
        );
    }

    #[test]
    fn a_warning_of_one_sentence_too_wide_stays_whole_to_wrap() {
        let warning = "Prix illisibles, nouvelle tentative…";
        assert_eq!(written(warning.to_owned(), 301.0, 300.0), warning);
    }
}
