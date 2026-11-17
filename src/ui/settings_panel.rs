//! The settings, in place of the content of the window: the language; under Game, always on top
//! and close with the game; the price-check hotkey; under Search, the pseudo totals; and at the
//! bottom the link to the donation page and the disclaimer.

use egui::text::{LayoutJob, TextFormat};
use egui::{Align, CentralPanel, Frame, Label, Layout, RichText, Ui, UiBuilder, Vec2, vec2};

use super::hotkey_list::{self, HotkeyList, HotkeyListAction, KeysButton};
use super::keys::{self, Emphasis};
use super::theme::{color, size, text};
use super::widgets::{self, Mark};
use crate::hotkey::{HotkeyState, Shortcut};
use crate::i18n::{Message, Translator};
use crate::settings::{LanguageSetting, PseudoTotals, SettingChange, Settings};

/// The space above and under the settings.
const PADDING_Y: f32 = 12.0;
/// The space above each group of choices.
const GROUP_GAP: f32 = 16.0;
/// The space between the choices of a group, and under its label.
const CHOICE_GAP: f32 = 6.0;
/// The space above the second row of a checkbox of a group, closer than the groups (mock-up
/// section "Close with the game (20a1)").
const CHECK_ROW_GAP: f32 = 12.0;
/// The space between the title of a checkbox and its line under it (the same mock-up section).
const LINE_GAP: f32 = 1.0;
/// The space above the line of the donation page, at the bottom of the settings.
const DONATION_GAP: f32 = 22.0;
/// The space between the line of the donation page and the disclaimer under it.
const DISCLAIMER_GAP: f32 = 6.0;

/// The texts of a row of a checkbox: its title, and the line under it that says what it does.
struct CheckRow {
    title: Message,
    line: Message,
}

const ALWAYS_ON_TOP: CheckRow = CheckRow {
    title: Message::SettingsAlwaysOnTop,
    line: Message::SettingsAlwaysOnTopHelp,
};

const CLOSE_WITH_GAME: CheckRow = CheckRow {
    title: Message::SettingsCloseWithGame,
    line: Message::SettingsCloseWithGameHelp,
};

const PSEUDO_TOTALS: CheckRow = CheckRow {
    title: Message::SettingsPseudoTotals,
    line: Message::SettingsPseudoTotalsHelp,
};

/// What the player did in the settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsAction {
    Change(SettingChange),
    /// Opened or closed the list of the keys of the hotkey, with its button.
    ToggleKeys,
    /// Clicked outside the open list of the keys.
    CloseKeys,
}

pub struct SettingsPanel<'a> {
    settings: &'a Settings,
    i18n: &'a Translator,
    hotkey: HotkeyState,
    /// The keys the hotkey holds, which the line of the key taken names: those of the settings,
    /// unless the previous ones couldn't be released (until another choice of keys or the next
    /// launch).
    held: Shortcut,
    keys_open: bool,
}

impl<'a> SettingsPanel<'a> {
    /// The settings, with what the price-check `hotkey` does now; its group, and Close with the
    /// game, are drawn only on a system where the game runs.
    #[must_use]
    pub fn new(settings: &'a Settings, i18n: &'a Translator, hotkey: HotkeyState) -> Self {
        Self {
            settings,
            i18n,
            hotkey,
            held: settings.hotkey().keys,
            keys_open: false,
        }
    }

    /// The keys the hotkey holds, when they aren't those of the settings.
    #[must_use]
    pub fn held_keys(mut self, keys: Shortcut) -> Self {
        self.held = keys;
        self
    }

    /// Opens the list of the keys of the hotkey under its button.
    #[must_use]
    pub fn keys_open(mut self, open: bool) -> Self {
        self.keys_open = open;
        self
    }

    /// The settings no longer drawn: the list of the keys forgets its animation, so that it
    /// doesn't fade out when they open again.
    pub fn forget(ctx: &egui::Context) {
        hotkey_list::forget(ctx);
    }

    /// Draws the settings; what the player did, if anything.
    pub fn show(self, ui: &mut Ui) -> Option<SettingsAction> {
        CentralPanel::no_frame()
            .frame(
                Frame::NONE
                    .fill(color::BAND_B)
                    .inner_margin(vec2(size::BAND_PADDING, PADDING_Y)),
            )
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                ui.label(
                    RichText::new(self.i18n.text(Message::SettingsTitle))
                        .font(text::title())
                        .color(color::TEXT),
                );
                let language = self.language(ui);
                self.group_label(ui, Message::SettingsGame);
                let always_on_top = self.always_on_top(ui);
                let (close_with_game, hotkey) = if crate::HAS_THE_GAME {
                    (self.close_with_game(ui), self.hotkey(ui))
                } else {
                    (None, None)
                };
                let pseudo_totals = self.pseudo_totals(ui);
                self.donation(ui);
                self.disclaimer(ui);
                // A change wins over the click outside the list of the keys, which it closes too.
                language
                    .or(always_on_top)
                    .or(close_with_game)
                    .or(pseudo_totals)
                    .map(SettingsAction::Change)
                    .or(hotkey)
            })
            .inner
    }

    /// The label above a group of choices, after the space above the group.
    fn group_label(&self, ui: &mut Ui, message: Message) {
        ui.add_space(GROUP_GAP);
        widgets::small_title(ui, &self.i18n.text(message), color::WEAK);
    }

    fn language(&self, ui: &mut Ui) -> Option<SettingChange> {
        self.group_label(ui, Message::SettingsLanguage);

        let mut change = None;
        for setting in LanguageSetting::ALL {
            ui.add_space(CHOICE_GAP);
            let name = self.i18n.text(match setting {
                LanguageSetting::System => Message::LanguageSystem,
                LanguageSetting::English => Message::LanguageEnglish,
                LanguageSetting::French => Message::LanguageFrench,
            });
            let mark = Mark::Radio {
                selected: self.settings.language() == setting,
            };
            if widgets::choice(ui, mark, &name).clicked() {
                change = Some(SettingChange::Language(setting));
            }
        }
        change
    }

    /// The first row of the group Game.
    fn always_on_top(&self, ui: &mut Ui) -> Option<SettingChange> {
        ui.add_space(CHOICE_GAP);
        let checked = self.settings.always_on_top();
        self.check_row(ui, checked, &ALWAYS_ON_TOP)
            .then_some(SettingChange::AlwaysOnTop(!checked))
    }

    /// The second row of the group Game, under Always on top.
    fn close_with_game(&self, ui: &mut Ui) -> Option<SettingChange> {
        ui.add_space(CHECK_ROW_GAP);
        let checked = self.settings.close_with_game();
        self.check_row(ui, checked, &CLOSE_WITH_GAME)
            .then_some(SettingChange::CloseWithGame(!checked))
    }

    /// The group Search, its one row: the pseudo totals shown or hidden.
    fn pseudo_totals(&self, ui: &mut Ui) -> Option<SettingChange> {
        self.group_label(ui, Message::SettingsSearch);
        ui.add_space(CHOICE_GAP);
        let shown = self.settings.pseudo_totals() == PseudoTotals::Shown;
        let toggled = if shown {
            PseudoTotals::Hidden
        } else {
            PseudoTotals::Shown
        };
        self.check_row(ui, shown, &PSEUDO_TOTALS)
            .then_some(SettingChange::PseudoTotals(toggled))
    }

    /// A row of a checkbox: the checkbox and the title of `row`, then its line under the title;
    /// whether it was clicked.
    fn check_row(&self, ui: &mut Ui, checked: bool, row: &CheckRow) -> bool {
        let title = self.i18n.text(row.title);
        let clicked = widgets::choice(ui, Mark::Check { checked }, &title).clicked();
        line_under_title(ui, faint_line(self.i18n.text(row.line)));
        clicked
    }

    /// The price-check hotkey: its checkbox, the button of its keys on the right of the row and
    /// their list under it, then a line under its title, what it does, or in the color of the
    /// warnings that its key is taken.
    fn hotkey(&self, ui: &mut Ui) -> Option<SettingsAction> {
        self.group_label(ui, Message::SettingsHotkey);
        ui.add_space(CHOICE_GAP);
        let on = self.settings.hotkey().on;
        let shortcut = self.settings.hotkey().keys;
        let hotkey = keys::joined(&keys::named(&shortcut.keys(), self.i18n));
        let label = self.i18n.text(Message::SettingsHotkeyPriceCheck);
        let (checked, button) = ui
            .horizontal(|ui| {
                let checked = widgets::choice(ui, Mark::Check { checked: on }, &label).clicked();
                let button = ui
                    .with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let tooltip = self.i18n.text(Message::SettingsHotkeyChange);
                        KeysButton {
                            keys: &hotkey,
                            is_on: on,
                            is_open: self.keys_open,
                            tooltip: &tooltip,
                        }
                        .show(ui)
                    })
                    .inner;
                (checked, button)
            })
            .inner;
        let list = HotkeyList::new(shortcut, self.i18n).show(ui.ctx(), &button, self.keys_open);
        let line = if self.hotkey == HotkeyState::Taken {
            let held = keys::joined(&keys::named(&self.held.keys(), self.i18n));
            let copy = keys::joined(&keys::COPY);
            keys::two_lines(
                &self.i18n.hotkey_taken(&held),
                &self.i18n.hotkey_taken_settings(&copy),
                &Emphasis {
                    strong: &[&held, &copy],
                    regular: TextFormat::simple(text::small(), color::WARNING),
                    bold: TextFormat::simple(text::small_strong(), color::WARNING),
                },
            )
        } else {
            faint_line(self.i18n.text(Message::SettingsHotkeyHelp))
        };
        line_under_title(ui, line);
        hotkey_action(checked.then_some(!on), button.clicked(), list)
    }

    /// "Exile Market is free." then the link to the donation page, which opens it in the
    /// browser: a donation unlocks nothing.
    fn donation(&self, ui: &mut Ui) {
        ui.add_space(DONATION_GAP);
        ui.horizontal(|ui| {
            // The space of the text, not of the layout, before the link.
            ui.spacing_mut().item_spacing.x = 0.0;
            let intro = format!("{} ", self.i18n.donation_intro());
            ui.add(Label::new(
                RichText::new(intro).font(text::small()).color(color::FAINT),
            ));
            widgets::link(
                ui,
                &self.i18n.text(Message::DonationLink),
                crate::DONATION_PAGE,
            );
        });
    }

    fn disclaimer(&self, ui: &mut Ui) {
        ui.add_space(DISCLAIMER_GAP);
        ui.add(
            Label::new(
                RichText::new(self.i18n.text(Message::Disclaimer))
                    .font(text::small())
                    .color(color::FAINT),
            )
            .wrap(),
        );
    }
}

/// The line under the title of a checkbox, from the start of the title to the right edge of the
/// settings, under the keys of the hotkey too.
fn line_under_title(ui: &mut Ui, line: LayoutJob) {
    ui.add_space(LINE_GAP);
    let mut rect = ui.available_rect_before_wrap();
    rect.min.x += widgets::CHOICE_TEXT_LEFT;
    ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
        ui.add(Label::new(line).wrap());
    });
}

/// What a checkbox does, in faint text.
fn faint_line(text: String) -> LayoutJob {
    LayoutJob::simple_format(text, TextFormat::simple(text::small(), color::FAINT))
}

/// What the player did with the hotkey in a frame: the checkbox `turned` on or off, the button of
/// the keys `toggled`, and what they did with the `list`. The click on the button that closes the
/// list is outside it too: the button wins, so that the list doesn't close then open again. The
/// checkbox wins over the click outside the list, which its change closes too.
fn hotkey_action(
    turned: Option<bool>,
    toggled: bool,
    list: Option<HotkeyListAction>,
) -> Option<SettingsAction> {
    if let Some(on) = turned {
        return Some(SettingsAction::Change(SettingChange::HotkeyOn(on)));
    }
    if toggled {
        return Some(SettingsAction::ToggleKeys);
    }
    list.map(|action| match action {
        HotkeyListAction::Choose(keys) => SettingsAction::Change(SettingChange::HotkeyKeys(keys)),
        HotkeyListAction::Close => SettingsAction::CloseKeys,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The line of the donation page alone, drawn frame after frame in one context.
    struct DonationLine {
        ctx: egui::Context,
        settings: Settings,
        i18n: Translator,
    }

    impl DonationLine {
        fn new(language: crate::language::Language) -> Self {
            let ctx = egui::Context::default();
            crate::ui::install(&ctx);
            Self {
                ctx,
                settings: Settings::default(),
                i18n: Translator::new(language),
            }
        }

        /// One frame with `events`, in a column of `width`: the rectangle of the line, and the
        /// addresses the frame opens.
        fn frame(&self, events: Vec<egui::Event>, width: f32) -> (egui::Rect, Vec<String>) {
            let panel = SettingsPanel::new(&self.settings, &self.i18n, HotkeyState::Off);
            let input = egui::RawInput {
                events,
                ..Default::default()
            };
            let mut line = egui::Rect::NOTHING;
            let output = crate::ui::fixtures::run_frame(&self.ctx, input, |ui| {
                ui.vertical(|ui| {
                    ui.set_max_width(width);
                    panel.donation(ui);
                    line = ui.min_rect();
                });
            });
            let opened = output
                .platform_output
                .commands
                .into_iter()
                .filter_map(|command| match command {
                    egui::OutputCommand::OpenUrl(open) => Some(open.url),
                    _ => None,
                })
                .collect();
            (line, opened)
        }

        /// The addresses opened by a click at `at`.
        fn click(&self, at: egui::Pos2) -> Vec<String> {
            let button = |pressed| egui::Event::PointerButton {
                pos: at,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            };
            self.frame(vec![egui::Event::PointerMoved(at)], WIDE);
            self.frame(vec![button(true)], WIDE);
            self.frame(vec![button(false)], WIDE).1
        }

        /// The addresses opened by `key`, pressed then released.
        fn press(&self, key: egui::Key) -> Vec<String> {
            let event = |pressed| egui::Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            };
            let mut opened = self.frame(vec![event(true)], WIDE).1;
            opened.extend(self.frame(vec![event(false)], WIDE).1);
            opened
        }
    }

    /// A column wide enough for the line.
    const WIDE: f32 = 400.0;

    /// The width of the line of the donation page in `language`, in a column of `width`.
    fn donation_width(language: crate::language::Language, width: f32) -> f32 {
        DonationLine::new(language).frame(vec![], width).0.width()
    }

    #[test]
    fn a_click_on_the_link_opens_the_donation_page() {
        let donation = DonationLine::new(crate::language::Language::English);
        let (rect, _) = donation.frame(vec![], WIDE);

        // The link ends the line, under the space above it; its start is the sentence before it.
        let row = rect.bottom() - 4.0;
        let link = egui::pos2(rect.right() - 4.0, row);
        assert_eq!(donation.click(link), vec![crate::DONATION_PAGE.to_owned()]);
        let sentence = egui::pos2(rect.left() + 4.0, row);
        assert_eq!(donation.click(sentence), Vec::<String>::new());
    }

    #[test]
    fn the_link_reached_with_the_keyboard_opens_the_donation_page_with_enter() {
        let donation = DonationLine::new(crate::language::Language::English);
        donation.frame(vec![], WIDE);

        assert_eq!(donation.press(egui::Key::Tab), Vec::<String>::new());
        assert_eq!(
            donation.press(egui::Key::Enter),
            vec![crate::DONATION_PAGE.to_owned()]
        );
    }

    #[test]
    fn the_line_of_the_donation_page_fits_the_narrow_window_in_every_language() {
        // The narrowest window, less the padding of the settings on both sides.
        let width = crate::MIN_WINDOW_SIZE[0] - 2.0 * size::BAND_PADDING;
        for language in crate::language::Language::ALL {
            let line = donation_width(language, width);
            assert!(line <= width + 0.5, "{line} of {width} in {language:?}");
        }
    }

    #[test]
    fn the_lines_of_the_checkboxes_hold_on_one_line_in_the_narrow_window() {
        let ctx = egui::Context::default();
        crate::ui::install(&ctx);
        // The narrowest window, less the padding of the settings and the checkbox before the line.
        let width =
            crate::MIN_WINDOW_SIZE[0] - 2.0 * size::BAND_PADDING - widgets::CHOICE_TEXT_LEFT;
        for language in crate::language::Language::ALL {
            let i18n = Translator::new(language);
            for message in [ALWAYS_ON_TOP.line, CLOSE_WITH_GAME.line] {
                let mut line = 0.0;
                crate::ui::fixtures::run_frame(&ctx, egui::RawInput::default(), |ui| {
                    line = ui
                        .painter()
                        .layout_no_wrap(i18n.text(message), text::small(), color::FAINT)
                        .size()
                        .x;
                });
                assert!(
                    line <= width,
                    "{line} of {width}: {message:?} in {language:?}"
                );
            }
        }
    }

    #[test]
    fn keys_chosen_in_the_list_change_the_setting() {
        let keys = Shortcut::CHOICES[1];
        assert_eq!(
            hotkey_action(None, false, Some(HotkeyListAction::Choose(keys))),
            Some(SettingsAction::Change(SettingChange::HotkeyKeys(keys)))
        );
    }

    #[test]
    fn the_button_that_closes_the_list_isnt_a_click_outside_it() {
        assert_eq!(
            hotkey_action(None, true, Some(HotkeyListAction::Close)),
            Some(SettingsAction::ToggleKeys)
        );
        assert_eq!(
            hotkey_action(None, false, Some(HotkeyListAction::Close)),
            Some(SettingsAction::CloseKeys)
        );
    }

    #[test]
    fn the_checkbox_turns_the_hotkey_on_or_off() {
        assert_eq!(
            hotkey_action(Some(false), false, None),
            Some(SettingsAction::Change(SettingChange::HotkeyOn(false)))
        );
        assert_eq!(hotkey_action(None, false, None), None);
    }

    #[test]
    fn the_checkbox_clicked_while_the_list_is_open_changes_the_setting() {
        // A change of a setting closes the list too (`OpenLists::settings`).
        assert_eq!(
            hotkey_action(Some(false), false, Some(HotkeyListAction::Close)),
            Some(SettingsAction::Change(SettingChange::HotkeyOn(false)))
        );
    }
}
