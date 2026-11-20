//! What the drawing computes before it draws: the width of the text of a stat, the tooltip of a
//! range field and how it is drawn, the bar of the tier of a waystone.

use super::*;
use crate::tier_rolls::TierSpan;

#[test]
fn the_text_of_a_filter_without_range_field_runs_over_its_column() {
    let without = filter_text_width(300.0, None);
    assert_eq!(without, 300.0 - CHECKBOX_COLUMN - COLUMN_GAP);
    let narrow = Limit::minimum(52.0);
    assert!(!narrow.is_wide());
    assert_eq!(
        without - filter_text_width(300.0, Some(narrow)),
        COLUMN_GAP + RANGE_COLUMN
    );
    let wide = Limit::minimum(2250.0);
    assert!(wide.is_wide());
    assert_eq!(
        without - filter_text_width(300.0, Some(wide)),
        COLUMN_GAP + WIDE_RANGE_COLUMN
    );
    // Never below zero in a row too narrow.
    assert_eq!(filter_text_width(10.0, Some(narrow)), 0.0);
}

#[test]
fn the_tooltip_of_a_range_field_tells_its_rolls_and_its_better_side() {
    let i18n = Translator::new(crate::language::Language::English);
    let range = Some(Range {
        min: 15.0,
        max: 25.0,
    });
    let tooltip = |half, rolls, tiers| {
        let limit = Limit {
            value: 18.0,
            filled: Filled::Half(half),
            bounds: range,
            rolls,
            tiers,
        };
        tooltip_lines(limit, &i18n)
    };
    let rolls = TooltipLine::new("Rolls 15 to 25".to_owned(), color::TEXT);
    let lower = TooltipLine::new("Lower is better".to_owned(), color::WEAK);

    assert_eq!(
        tooltip(Half::Minimum, range, None),
        std::slice::from_ref(&rolls)
    );
    assert_eq!(
        tooltip(Half::Maximum, range, None),
        [rolls.clone(), lower.clone()]
    );
    // A line of several numbers.
    assert_eq!(
        tooltip(Half::Maximum, None, None),
        std::slice::from_ref(&lower)
    );
    assert_eq!(tooltip(Half::Minimum, None, None), []);
    // The required level of a rare for leveling, searched with a maximum: lower is better.
    assert_eq!(
        tooltip_lines(Limit::maximum(40.0), &i18n),
        std::slice::from_ref(&lower)
    );
    // A prefix or suffix: the tiers of its rolls right under them, small and faint.
    let tiers = TooltipLine {
        text: "T5–T1".to_owned(),
        tint: color::FAINT,
        is_small: true,
    };
    assert_eq!(
        tooltip(
            Half::Maximum,
            range,
            Some(TierSpan {
                lowest: 5,
                highest: 1,
            })
        ),
        [rolls, tiers, lower]
    );
}

#[test]
fn the_bar_of_the_tier_of_a_waystone_goes_from_tier_1_to_tier_16() {
    assert!(waystone_tier_share(1).abs() < f32::EPSILON);
    assert!((waystone_tier_share(16) - 1.0).abs() < f32::EPSILON);
    assert!((waystone_tier_share(12) - 11.0 / 15.0).abs() < 1e-6);
    // Never out of the bar.
    assert!(waystone_tier_share(0).abs() < f32::EPSILON);
    assert!((waystone_tier_share(17) - 1.0).abs() < f32::EPSILON);
}

#[test]
fn the_fill_of_an_affix_is_lit_while_a_place_can_be_open() {
    let i18n = Translator::new(crate::language::Language::English);
    let words = |room| fill_words(AffixFill { filled: 2, room }, &i18n);
    let room = |max, open_at_most| Some(AffixRoom { max, open_at_most });

    assert_eq!(words(room(3, None)), ("2 of 3 · 1 open".to_owned(), true));
    assert_eq!(words(room(2, None)), ("2 of 2".to_owned(), false));
    // Modifiers without affix take places the fill doesn't count: one open at most, or none.
    assert_eq!(
        words(room(3, Some(1))),
        ("2 of 3 · up to 1 open".to_owned(), true)
    );
    assert_eq!(words(room(3, Some(0))), ("2 of 3".to_owned(), false));
    // A corrupted waystone: its count alone.
    assert_eq!(words(None), ("2".to_owned(), false));
}

/// A range field drawn alone, its minimum `80`, for the filter of the base.
struct LoneField {
    ctx: egui::Context,
    filter: Filter,
    time: f64,
}

/// A range field as a frame drew it: where it is, the fills of the meshes drawn, and whether a
/// half has the focus once the frame is drawn.
struct FieldFrame {
    rect: Rect,
    has_band_a: bool,
    has_control: bool,
    is_focused: bool,
}

impl LoneField {
    fn new() -> Self {
        let ctx = egui::Context::default();
        crate::ui::install(&ctx);
        let filter = Filter {
            checked: true,
            minimum: "80".to_owned(),
            maximum: String::new(),
            bounds: None,
        };
        Self {
            ctx,
            filter,
            time: 0.0,
        }
    }

    /// One frame of the field of `limit`, after `events`.
    fn frame(&mut self, limit: Limit, events: Vec<egui::Event>) -> FieldFrame {
        // Short of the longest click.
        self.time += 0.1;
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(420.0, 200.0))),
            time: Some(self.time),
            events,
            ..Default::default()
        };
        let mut rect = Rect::NOTHING;
        let filter = &mut self.filter;
        let output = crate::ui::fixtures::run_frame(&self.ctx, input, |ui| {
            // In a row, as `filter_row` draws it.
            ui.horizontal(|ui| {
                rect = range_field(ui, FilterKey::BaseType, filter, limit).rect;
            });
        });
        let colors: Vec<Color32> = output
            .shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                Shape::Mesh(mesh) => Some(mesh.vertices.iter().map(|vertex| vertex.color)),
                _ => None,
            })
            .flatten()
            .collect();
        FieldFrame {
            rect,
            has_band_a: colors.contains(&color::BAND_A),
            has_control: colors.contains(&color::CONTROL),
            is_focused: self.ctx.memory(|memory| memory.focused().is_some()),
        }
    }
}

#[test]
fn a_range_field_keeps_its_column_and_a_line_of_numbers_high() {
    let mut field = LoneField::new();
    for (value, column) in [(52.0, RANGE_COLUMN), (2250.0, WIDE_RANGE_COLUMN)] {
        let rect = field.frame(Limit::minimum(value), vec![]).rect;
        let line = field
            .ctx
            .fonts_mut(|fonts| fonts.row_height(&text::range_field()));
        // The line of numbers rounded up to a whole pixel, as egui lays it out, then 2 points
        // above and below.
        assert_eq!(rect.size(), vec2(column, line.ceil() + 4.0));
    }
}

#[test]
fn a_range_field_takes_the_fill_of_the_controls_in_the_frame_it_gets_the_keyboard() {
    let mut field = LoneField::new();
    let limit = Limit::minimum(80.0);
    let at_rest = field.frame(limit, vec![]);
    assert!(at_rest.has_band_a && !at_rest.has_control && !at_rest.is_focused);
    // In the middle of the minimum.
    let at = at_rest.rect.left_center() + vec2(RANGE_COLUMN / 4.0, 0.0);
    let button = |pressed| egui::Event::PointerButton {
        pos: at,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    };
    let frames = [
        field.frame(limit, vec![egui::Event::PointerMoved(at)]),
        field.frame(limit, vec![button(true)]),
        field.frame(limit, vec![button(false)]),
    ];
    assert!(frames.iter().any(|frame| frame.is_focused));
    for frame in frames {
        assert_eq!(
            (frame.has_control, frame.has_band_a),
            (frame.is_focused, !frame.is_focused)
        );
    }
}
