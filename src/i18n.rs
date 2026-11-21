//! Interface translations: the Fluent messages of `locales/`, embedded in the executable.
//!
//! The code never names a message by a string: the messages without arguments are the
//! variants of [`Message`], and each message with arguments has its own method of
//! [`Translator`], the only place that knows the names of its arguments.

use std::ops::Range;
use std::time::Duration;

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};

use crate::filters::Property;
use crate::item::{Affix, Catalyst, Lord, QualityKind, WaystoneProperty};
use crate::language::{Language, PerLanguage};
use crate::market::Failure;
use crate::price::Direction;
use crate::trade::pseudo::Pseudo;
use crate::window_error::Cause;

// `allow`, not `expect`: the lint is quiet in the tests, whose `use super::*` reads them too.
#[allow(
    clippy::wildcard_imports,
    reason = "the identifiers of the messages, read by the methods of the translator"
)]
use ids::*;
pub use ids::{DOWNLOAD_QUOTE_COUNT, Message};

mod ids;

/// The Fluent messages of each language, embedded in the executable.
const MESSAGES: PerLanguage<&str> = PerLanguage {
    english: include_str!("../locales/en.ftl"),
    french: include_str!("../locales/fr.ftl"),
};

/// The Fluent messages of `language`.
fn messages(language: Language) -> &'static str {
    MESSAGES.of(language)
}

/// A sentence with a link in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Linked {
    pub text: String,
    /// The bytes of the words of the link in `text`; empty when it has none.
    pub link: Range<usize>,
}

/// The range of the offers counted in an estimate, written in one currency: from `cheapest` to
/// `dearest` `currency`, "2 to 3 div".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrittenRange {
    pub cheapest: String,
    pub dearest: String,
    /// The abbreviation of the currency, the same in every language: `div`, `ex`.
    pub currency: &'static str,
}

/// The tags of the item block with a number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberTag {
    Quality,
    Sockets,
    ItemLevel,
}

/// The parts of the requirements line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Requirement {
    Level,
    Strength,
    Dexterity,
    Intelligence,
}

/// A translated sentence cut around one of its arguments, which the caller writes apart, in
/// bold or in its color: "Rise of the Abyssal has started." is `""`, the league, then
/// `" has started."`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CutSentence {
    pub before: String,
    pub after: String,
}

impl CutSentence {
    /// The whole sentence, `argument` in its place.
    #[must_use]
    pub fn whole(&self, argument: &str) -> String {
        format!("{}{argument}{}", self.before, self.after)
    }
}

/// The character [`Translator::around`] puts in place of an argument: no translation writes
/// it, so that the place of the argument is exact whatever the sentence says.
const PLACE: &str = "\u{1}";

/// Gives the texts of the interface in one language.
pub struct Translator {
    bundle: FluentBundle<FluentResource>,
    language: Language,
}

impl Translator {
    /// # Panics
    ///
    /// If the embedded messages of `language` can't be parsed, which the tests rule out.
    #[must_use]
    pub fn new(language: Language) -> Self {
        let resource = FluentResource::try_new(messages(language).to_owned())
            .unwrap_or_else(|(_, errors)| panic!("invalid messages for {language:?}: {errors:?}"));
        let mut bundle = FluentBundle::new(vec![language.identifier()]);
        // Without the Unicode isolation marks around the arguments, which egui would draw as boxes.
        bundle.set_use_isolating(false);
        bundle
            .add_resource(resource)
            .unwrap_or_else(|errors| panic!("duplicate messages for {language:?}: {errors:?}"));
        Self { bundle, language }
    }

    /// The language of its messages, the language of the interface.
    #[must_use]
    pub fn language(&self) -> Language {
        self.language
    }

    /// The text of `message`.
    #[must_use]
    pub fn text(&self, message: Message) -> String {
        self.format(message.id(), None)
    }

    /// The line of the status bar: the state of the prices, then of the trade site.
    #[must_use]
    pub fn status_bar(&self, prices: &str, trade: &str) -> String {
        let mut args = FluentArgs::new();
        args.set("prices", prices);
        args.set("trade", trade);
        self.format(STATUS_BAR, Some(&args))
    }

    /// The state of the prices in the status bar, with their age (`12min ago`).
    #[must_use]
    pub fn status_prices(&self, age: &str) -> String {
        self.format_age(STATUS_PRICES, age)
    }

    /// The state of the prices in the status bar, with their age, while the price database
    /// can't be read for `failure`.
    #[must_use]
    pub fn status_prices_failed(&self, failure: Failure, age: &str) -> String {
        let id = match failure {
            Failure::Unreachable => STATUS_PRICES_OFFLINE,
            Failure::Unreadable => STATUS_PRICES_UNREADABLE,
        };
        self.format_age(id, age)
    }

    /// The warning of a price database that can't be read for `failure`, with the age of the
    /// prices.
    #[must_use]
    pub fn prices_failed(&self, failure: Failure, age: &str) -> String {
        let id = match failure {
            Failure::Unreachable => PRICES_UNREACHABLE,
            Failure::Unreadable => PRICES_UNREADABLE,
        };
        self.format_age(id, age)
    }

    /// The warning of a price database that can't be read for `failure`, while no price is
    /// known.
    #[must_use]
    pub fn prices_unavailable(&self, failure: Failure) -> String {
        self.text(match failure {
            Failure::Unreachable => Message::PricesUnavailable,
            Failure::Unreadable => Message::PricesUnreadableRetrying,
        })
    }

    /// How long ago something happened, rounded down: `12min ago`, `3h ago`, `2d ago`.
    #[must_use]
    pub fn age(&self, age: Duration) -> String {
        let minutes = age.as_secs() / 60;
        let (id, name, count) = match minutes {
            0 => return self.format(AGE_NOW, None),
            1..60 => (AGE_MINUTES, "minutes", minutes),
            60..1440 => (AGE_HOURS, "hours", minutes / 60),
            _ => (AGE_DAYS, "days", minutes / 1440),
        };
        let mut args = FluentArgs::new();
        args.set(name, count);
        self.format(id, Some(&args))
    }

    /// A wait, rounded up so that it never ends too soon: `8s` under a minute, `4min` from it.
    #[must_use]
    pub fn wait(&self, wait: Duration) -> String {
        let seconds = wait.as_secs() + u64::from(wait.subsec_nanos() > 0);
        let (id, name, count) = if seconds < 60 {
            (WAIT_SECONDS, "seconds", seconds)
        } else {
            (WAIT_MINUTES, "minutes", seconds.div_ceil(60))
        };
        let mut args = FluentArgs::new();
        args.set(name, count);
        self.format(id, Some(&args))
    }

    /// The title of the download of the prices, around the name of the league.
    #[must_use]
    pub fn download_title(&self) -> CutSentence {
        self.around(DOWNLOAD_TITLE, "league", FluentArgs::new())
    }

    /// The sentence `index` under the bar of the download of the prices, taken modulo their
    /// number.
    #[must_use]
    pub fn download_quote(&self, index: usize) -> String {
        // The modulo keeps the index in the array.
        self.format(DOWNLOAD_QUOTES[index % DOWNLOAD_QUOTE_COUNT], None)
    }

    /// The pill that unfolds the stats of the item: "Show 6 stats".
    #[must_use]
    pub fn show_stats(&self, count: u32) -> String {
        self.format_number(SHOW_STATS, "count", count)
    }

    /// The number of active filters, beside the pill of the stats: "3 filters".
    #[must_use]
    pub fn active_filters(&self, count: u32) -> String {
        self.format_number(ACTIVE_FILTERS, "count", count)
    }

    /// The fill of the prefixes or suffixes of an item: "2 of 3 · 1 open".
    #[must_use]
    pub fn affix_fill(&self, filled: u32, max: u32) -> String {
        let mut args = FluentArgs::new();
        args.set("filled", filled);
        args.set("max", max);
        let open = max.saturating_sub(filled);
        if open == 0 {
            return self.format(AFFIX_FILL, Some(&args));
        }
        args.set("open", open);
        self.format(AFFIX_FILL_OPEN, Some(&args))
    }

    /// The fill of the prefixes or suffixes of an item some of whose modifiers have no affix
    /// known, and take places it doesn't count: at most `open` are open, "1 of 3 · up to 1
    /// open"; the count alone once none can be.
    #[must_use]
    pub fn affix_fill_up_to(&self, filled: u32, max: u32, open: u32) -> String {
        let mut args = FluentArgs::new();
        args.set("filled", filled);
        args.set("max", max);
        if open == 0 {
            return self.format(AFFIX_FILL, Some(&args));
        }
        args.set("open", open);
        self.format(AFFIX_FILL_UP_TO, Some(&args))
    }

    /// The affix of a modifier of the lord `lord` after the ranges of its tier, in gray, when
    /// the line can be his or an ordinary modifier: "Amanamu Prefix".
    #[must_use]
    pub fn lord_affix(&self, affix: Affix, lord: Lord) -> String {
        let mut args = FluentArgs::new();
        // The French messages select on the name: "de Kurgal", "d'Amanamu".
        args.set("lord", lord.name());
        let id = match affix {
            Affix::Prefix => LORD_PREFIX,
            Affix::Suffix => LORD_SUFFIX,
        };
        self.format(id, Some(&args))
    }

    /// The rolls a range field accepts, in its tooltip: "Rolls 40 to 60"; `low` and `high`
    /// written by [`crate::format::number`].
    #[must_use]
    pub fn filter_rolls(&self, low: &str, high: &str) -> String {
        let mut args = FluentArgs::new();
        args.set("low", low);
        args.set("high", high);
        self.format(FILTER_ROLLS, Some(&args))
    }

    /// The tooltip of "Hybrid" under the line that adds up the part of a hybrid: its other
    /// `lines`, "With +195 to Accuracy Rating", two of them joined by "and".
    #[must_use]
    pub fn hybrid_with(&self, lines: &[&str]) -> String {
        let lines = lines
            .iter()
            .map(|&line| line.to_owned())
            .reduce(|first, second| {
                let mut args = FluentArgs::new();
                args.set("first", first);
                args.set("second", second);
                self.format(HYBRID_LINES_AND, Some(&args))
            })
            .unwrap_or_default();
        let mut args = FluentArgs::new();
        args.set("lines", lines);
        self.format(HYBRID_WITH, Some(&args))
    }

    /// The tooltip of "Hybrid" under the other lines of a hybrid: the `ranges` of its part in
    /// the `line` that adds it up, "With 75–79 of 258% increased Physical Damage".
    #[must_use]
    pub fn hybrid_part(&self, ranges: &str, line: &str) -> String {
        let mut args = FluentArgs::new();
        args.set("ranges", ranges);
        args.set("line", line);
        self.format(HYBRID_PART, Some(&args))
    }

    /// The filter of the base type of an item: "Base: Gemini Bow".
    #[must_use]
    pub fn filter_base_type(&self, base_type: &str) -> String {
        let mut args = FluentArgs::new();
        args.set("base", base_type);
        self.format(FILTER_BASE_TYPE, Some(&args))
    }

    /// Under the filter of the base type unchecked, the item class searched in its place:
    /// "Class searched: Bows".
    #[must_use]
    pub fn filter_item_class(&self, class: &str) -> String {
        let mut args = FluentArgs::new();
        args.set("class", class);
        self.format(FILTER_ITEM_CLASS, Some(&args))
    }

    /// The filter of a property of an item, with its `value` written by
    /// [`crate::format::number`]: "Item level: 81", "Quality: 24%", "Physical DPS: 245"; a
    /// percentage of a waystone with its sign, "Item rarity: +11%".
    #[must_use]
    pub fn filter_property(&self, property: Property, value: &str) -> String {
        let id = match property {
            Property::ItemLevel => FILTER_ITEM_LEVEL,
            Property::Sockets | Property::GemSockets => FILTER_SOCKETS,
            Property::GemLevel => FILTER_GEM_LEVEL,
            Property::Quality => FILTER_QUALITY,
            Property::RequiredLevel => FILTER_REQUIRED_LEVEL,
            Property::PhysicalDps => FILTER_PHYSICAL_DPS,
            Property::ElementalDps => FILTER_ELEMENTAL_DPS,
            Property::WaystoneTier => FILTER_WAYSTONE_TIER,
            Property::Waystone(property) => match property {
                WaystoneProperty::Revives => FILTER_WAYSTONE_REVIVES,
                WaystoneProperty::ItemRarity => FILTER_WAYSTONE_ITEM_RARITY,
                WaystoneProperty::PackSize => FILTER_WAYSTONE_PACK_SIZE,
                WaystoneProperty::MonsterRarity => FILTER_WAYSTONE_MONSTER_RARITY,
                WaystoneProperty::MonsterEffectiveness => FILTER_WAYSTONE_MONSTER_EFFECTIVENESS,
                WaystoneProperty::DropChance => FILTER_WAYSTONE_DROP_CHANCE,
            },
        };
        let mut args = FluentArgs::new();
        args.set("value", value);
        self.format(id, Some(&args))
    }

    /// The filter of the quality of a catalyst, short with its kind, its `value` written by
    /// [`crate::format::number`]: "Mana quality: 40%"; a kind the application doesn't know
    /// with the words of the game, "Quality (Speed Modifiers): 20%".
    #[must_use]
    pub fn filter_catalyst_quality(&self, kind: &QualityKind, value: &str) -> String {
        let mut args = FluentArgs::new();
        args.set("kind", kind.catalyst.map_or("other", catalyst_id));
        // Also for a known kind: a selector missing from a file falls back to the words.
        args.set("words", kind.words.as_str());
        args.set("value", value);
        self.format(FILTER_CATALYST_QUALITY, Some(&args))
    }

    /// The filter of a pseudo stat, with its `value` written by [`crate::format::number`]:
    /// "+83% total Elemental Resistance", "Unrevealed modifiers: 1".
    #[must_use]
    pub fn filter_pseudo(&self, pseudo: Pseudo, value: &str) -> String {
        let id = match pseudo {
            Pseudo::ElementalResistance => PSEUDO_ELEMENTAL_RESISTANCE,
            Pseudo::ChaosResistance => PSEUDO_CHAOS_RESISTANCE,
            Pseudo::Life => PSEUDO_LIFE,
            Pseudo::Mana => PSEUDO_MANA,
            Pseudo::Strength => PSEUDO_STRENGTH,
            Pseudo::Dexterity => PSEUDO_DEXTERITY,
            Pseudo::Intelligence => PSEUDO_INTELLIGENCE,
            Pseudo::AllAttributes => PSEUDO_ALL_ATTRIBUTES,
            Pseudo::UnrevealedModifiers => PSEUDO_UNREVEALED_MODIFIERS,
        };
        let mut args = FluentArgs::new();
        args.set("value", value);
        self.format(id, Some(&args))
    }

    /// The number of offers behind the price of a unique: "23 offers".
    #[must_use]
    pub fn offers(&self, count: u32) -> String {
        self.format_number(OFFERS, "count", count)
    }

    /// The size of a copied stack, on the right of the price title: "Stack of 7".
    #[must_use]
    pub fn stack_of(&self, size: u32) -> String {
        self.format_number(STACK_OF, "size", size)
    }

    /// The unit price of a stack, from its amounts in the three currencies joined by the
    /// caller.
    #[must_use]
    pub fn unit_price(&self, prices: &str) -> String {
        let mut args = FluentArgs::new();
        args.set("prices", prices);
        self.format(UNIT_PRICE, Some(&args))
    }

    /// What one Divine or Chaos Orb is worth in Exalted Orbs, under its name in its tooltip:
    /// "1 = 712 Exalted Orbs"; `amount` as the price block writes it, its plural chosen by the
    /// number written.
    #[must_use]
    pub fn orb_rate_in_exalted(&self, amount: &str) -> String {
        let mut args = FluentArgs::new();
        args.set("amount", amount);
        args.set("count", FluentValue::try_number(amount));
        self.format(ORB_RATE_IN_EXALTED, Some(&args))
    }

    /// How many Exalted Orbs make a Divine Orb, under the name of the Exalted Orb in its
    /// tooltip: "712 = 1 Divine Orb".
    #[must_use]
    pub fn orb_rate_for_divine(&self, amount: &str) -> String {
        let mut args = FluentArgs::new();
        args.set("amount", amount);
        self.format(ORB_RATE_FOR_DIVINE, Some(&args))
    }

    /// The trend of a price: its change in percent, rounded, over `days` days.
    #[must_use]
    pub fn trend(&self, direction: Direction, percent: u32, days: u32) -> String {
        let id = match direction {
            Direction::Rise => TREND_RISE,
            Direction::Fall => TREND_FALL,
            Direction::Stable => TREND_STABLE,
        };
        let mut args = FluentArgs::new();
        args.set("percent", percent);
        args.set("days", days);
        self.format(id, Some(&args))
    }

    /// The line of low reliability of the price of a unique on few offers, `count` of them.
    #[must_use]
    pub fn low_reliability_offers(&self, count: u32) -> String {
        self.format_number(LOW_RELIABILITY_OFFERS, "count", count)
    }

    /// The line of low reliability of an item of the currency exchange priced on few trades,
    /// `count` of them.
    #[must_use]
    pub fn low_reliability_trades(&self, count: u32) -> String {
        self.format_number(LOW_RELIABILITY_TRADES, "count", count)
    }

    /// The similar offers found, on the right of the title of an estimated price: "18 similar".
    #[must_use]
    pub fn similar(&self, total: u64) -> String {
        self.format_number(SIMILAR, "count", total)
    }

    /// The line of low reliability of an estimate from few offers, the `counted` ones.
    #[must_use]
    pub fn low_reliability_similar(&self, counted: usize) -> String {
        self.format_number(LOW_RELIABILITY_SIMILAR, "count", counted)
    }

    /// How an estimate from all the `counted` offers is made, of the kind of the quality of a
    /// catalyst searched, with the `range` of the offers: "Median of the 3 similar offers of mana
    /// quality, 40 to 300 ex."
    #[must_use]
    pub fn estimate_median(
        &self,
        counted: usize,
        searched_catalyst: Option<Catalyst>,
        range: Option<&WrittenRange>,
    ) -> String {
        let note = self.format_estimate(MEDIAN, counted, searched_catalyst);
        self.estimate_note(note, range)
    }

    /// How an estimate from the `counted` cheapest offers is made, of the kind of the quality
    /// of a catalyst searched, with the `range` of the offers.
    #[must_use]
    pub fn estimate_median_cheapest(
        &self,
        counted: usize,
        searched_catalyst: Option<Catalyst>,
        range: Option<&WrittenRange>,
    ) -> String {
        let note = self.format_estimate(MEDIAN_CHEAPEST, counted, searched_catalyst);
        self.estimate_note(note, range)
    }

    /// The `note` of an estimate as a sentence, with the `range` of its offers when given.
    fn estimate_note(&self, note: String, range: Option<&WrittenRange>) -> String {
        let mut args = FluentArgs::new();
        args.set("note", note);
        let Some(range) = range else {
            return self.format(ESTIMATE_NOTE, Some(&args));
        };
        args.set("cheapest", range.cheapest.as_str());
        args.set("dearest", range.dearest.as_str());
        args.set("currency", range.currency);
        self.format(ESTIMATE_NOTE_RANGE, Some(&args))
    }

    /// The kind of the quality of an offer, after its price in the list: "Attack quality".
    #[must_use]
    pub fn offer_quality_kind(&self, catalyst: Catalyst) -> String {
        self.format_kind(OFFER_QUALITY_KIND, catalyst)
    }

    /// The price block while the prices of `league` are downloading.
    #[must_use]
    pub fn prices_downloading(&self, league: &str) -> String {
        self.format_league(PRICES_DOWNLOADING, league)
    }

    /// The price block of a fixed-name item missing from the prices of `league`.
    #[must_use]
    pub fn no_price_found(&self, league: &str) -> String {
        self.format_league(NO_PRICE_FOUND, league)
    }

    /// A tag of the item block with a number: the quality (`Q 20%`), the sockets, the item
    /// level.
    #[must_use]
    pub fn tag(&self, tag: NumberTag, value: u32) -> String {
        let id = match tag {
            NumberTag::Quality => TAG_QUALITY,
            NumberTag::Sockets => TAG_SOCKETS,
            NumberTag::ItemLevel => TAG_ITEM_LEVEL,
        };
        self.format_number(id, "value", value)
    }

    /// The level of a gem, after its item class under its name: "Level 2".
    #[must_use]
    pub fn gem_level(&self, level: u32) -> String {
        self.format_number(GEM_LEVEL, "value", level)
    }

    /// The stack tag of the item block: its size and its largest size.
    #[must_use]
    pub fn stack_tag(&self, size: u32, max: u32) -> String {
        let mut args = FluentArgs::new();
        args.set("size", size);
        args.set("max", max);
        self.format(TAG_STACK, Some(&args))
    }

    /// The summary of an item that grants skills, e.g. a staff, from the skills joined by
    /// the caller.
    #[must_use]
    pub fn granted_skill(&self, skill: &str) -> String {
        let mut args = FluentArgs::new();
        args.set("skill", skill);
        self.format(GRANTED_SKILL, Some(&args))
    }

    /// The requirements line around its parts, which the caller writes each in its color:
    /// `"Requires "`, then the parts.
    #[must_use]
    pub fn requirements_line(&self) -> CutSentence {
        self.around(REQUIRES, "parts", FluentArgs::new())
    }

    /// A part of the requirements line: the level, or an attribute.
    #[must_use]
    pub fn requirement(&self, requirement: Requirement, value: u32) -> String {
        let id = match requirement {
            Requirement::Level => REQUIREMENT_LEVEL,
            Requirement::Strength => REQUIREMENT_STRENGTH,
            Requirement::Dexterity => REQUIREMENT_DEXTERITY,
            Requirement::Intelligence => REQUIREMENT_INTELLIGENCE,
        };
        self.format_number(id, "value", value)
    }

    /// The tooltip of the league block, with the mode of the league (`Softcore`).
    #[must_use]
    pub fn league_tooltip(&self, mode: &str) -> String {
        let mut args = FluentArgs::new();
        args.set("mode", mode);
        self.format(LEAGUE_TOOLTIP, Some(&args))
    }

    /// The bubble of a new league, around its name.
    #[must_use]
    pub fn new_league(&self) -> CutSentence {
        self.around(NEW_LEAGUE, "league", FluentArgs::new())
    }

    /// The bubble of a newer version of `app`, around the version: "Exile Market 0.2.0 is
    /// available."
    #[must_use]
    pub fn update_available(&self, app: &str) -> CutSentence {
        let mut args = FluentArgs::new();
        args.set("app", app);
        self.around(UPDATE_AVAILABLE, "version", args)
    }

    /// A newer version in the status bar of the settings, a link: "0.2.0 available".
    #[must_use]
    pub fn status_update(&self, version: &str) -> String {
        let mut args = FluentArgs::new();
        args.set("version", version);
        self.format(STATUS_UPDATE, Some(&args))
    }

    /// The key of the hotkey taken by another application, under the invitation of the empty
    /// window: "Ctrl+D is taken by another application."
    #[must_use]
    pub fn hotkey_taken(&self, hotkey: &str) -> String {
        let mut args = FluentArgs::new();
        args.set("hotkey", hotkey);
        self.format(HOTKEY_TAKEN, Some(&args))
    }

    /// The checkbox that joins the item shown to the bug report, around its name, in bold.
    #[must_use]
    pub fn bug_report_join_item(&self) -> CutSentence {
        self.around(BUG_REPORT_JOIN_ITEM, "name", FluentArgs::new())
    }

    /// What a screen reader says of a pill of the choice of the log, `session` its word.
    #[must_use]
    pub fn bug_report_log_session(&self, session: &str) -> String {
        let mut args = FluentArgs::new();
        args.set("session", session);
        self.format(BUG_REPORT_LOG_SESSION, Some(&args))
    }

    /// The message of a window that can't open for `cause`: what failed, then what to do.
    #[must_use]
    pub fn window_error(&self, cause: Cause) -> [String; 2] {
        let (failed, fix) = match cause {
            Cause::Display => (WINDOW_ERROR_DISPLAY, WINDOW_ERROR_DISPLAY_FIX),
            Cause::OpenGl => (WINDOW_ERROR_OPENGL, WINDOW_ERROR_OPENGL_FIX),
            Cause::Other => (WINDOW_ERROR_OTHER, WINDOW_ERROR_OTHER_FIX),
        };
        let mut args = FluentArgs::new();
        args.set("app", crate::APP_NAME);
        args.set("issues", concat!(env!("CARGO_PKG_REPOSITORY"), "/issues"));
        [
            self.format(failed, Some(&args)),
            self.format(fix, Some(&args)),
        ]
    }

    /// The log file under the message of a window that can't open for a cause to report.
    #[must_use]
    pub fn window_error_log(&self, path: &str) -> String {
        let mut args = FluentArgs::new();
        args.set("path", path);
        self.format(WINDOW_ERROR_LOG, Some(&args))
    }

    /// The error itself, last in the message of a window that can't open.
    #[must_use]
    pub fn window_error_details(&self, error: &str) -> String {
        let mut args = FluentArgs::new();
        args.set("error", error);
        self.format(WINDOW_ERROR_DETAILS, Some(&args))
    }

    /// The message of a launch while the application already runs: that it is open, then where
    /// its window may be.
    #[must_use]
    pub fn already_open(&self) -> [String; 2] {
        let mut args = FluentArgs::new();
        args.set("app", crate::APP_NAME);
        [
            self.format(ALREADY_OPEN, Some(&args)),
            self.format(ALREADY_OPEN_WHERE, None),
        ]
    }

    /// The key of the hotkey taken by another application, in the settings, on the line after
    /// [`Self::hotkey_taken`]: where to choose another, and the keys that still `copy`.
    #[must_use]
    pub fn hotkey_taken_settings(&self, copy: &str) -> String {
        let mut args = FluentArgs::new();
        args.set("copy", copy);
        self.format(HOTKEY_TAKEN_SETTINGS, Some(&args))
    }

    /// The author in the status bar while the settings are open, after the version: "Made by
    /// Sylent".
    #[must_use]
    pub fn made_by(&self) -> String {
        let mut args = FluentArgs::new();
        args.set("author", crate::AUTHOR);
        self.format(STATUS_MADE_BY, Some(&args))
    }

    /// The sentence of the settings before the link to the donation page: "Exile Market is
    /// free."
    #[must_use]
    pub fn donation_intro(&self) -> String {
        let mut args = FluentArgs::new();
        args.set("app", crate::APP_NAME);
        self.format(DONATION_INTRO, Some(&args))
    }

    /// The state of the trade site in the status bar: the age of its last search.
    #[must_use]
    pub fn status_trade(&self, age: &str) -> String {
        self.format_age(STATUS_TRADE, age)
    }

    /// The state of the trade site in the status bar during a long wait before the next
    /// search, written by [`Self::wait`].
    #[must_use]
    pub fn status_trade_wait(&self, wait: Duration) -> String {
        self.format_wait(STATUS_TRADE_WAIT, wait)
    }

    /// The age of the offers, under their title: "searched 2min ago".
    #[must_use]
    pub fn offers_searched(&self, age: &str) -> String {
        self.format_age(OFFERS_SEARCHED, age)
    }

    /// The offers shown, of those found: "10 of 13 online".
    #[must_use]
    pub fn offers_count(&self, shown: u64, total: u64) -> String {
        let mut args = FluentArgs::new();
        args.set("shown", shown);
        args.set("total", total);
        self.format(OFFERS_COUNT, Some(&args))
    }

    /// The tooltip of the countdown before the next search.
    #[must_use]
    pub fn offers_next_search_tooltip(&self, wait: Duration) -> String {
        self.format_wait(OFFERS_NEXT_SEARCH_TOOLTIP, wait)
    }

    /// The line under the title of the offers during a long wait.
    #[must_use]
    pub fn offers_limit(&self, wait: Duration) -> String {
        self.format_wait(OFFERS_LIMIT, wait)
    }

    /// The line under the title during the wait of a refusal of the trade site that names no
    /// limit (HTTP 403), `wait` left.
    #[must_use]
    pub fn offers_forbidden(&self, wait: Duration) -> String {
        self.format_wait(OFFERS_FORBIDDEN, wait)
    }

    /// What to do after a refusal of the trade site that names no limit, its words that open
    /// the bug report told.
    #[must_use]
    pub fn offers_forbidden_hint(&self) -> Linked {
        self.linked(OFFERS_FORBIDDEN_HINT, OFFERS_REPORT)
    }

    /// What to do after a search the trade site turned down, its words that open the bug
    /// report told.
    #[must_use]
    pub fn offers_refused_hint(&self) -> Linked {
        self.linked(OFFERS_REFUSED_HINT, OFFERS_REPORT)
    }

    /// What to do after an answer of the trade site that can't be read, its words that open
    /// the bug report told.
    #[must_use]
    pub fn offers_unreadable_hint(&self) -> Linked {
        self.linked(OFFERS_UNREADABLE_HINT, OFFERS_REPORT_START)
    }

    /// The message `id`, its argument `report` the message `link`, and where that one is in it.
    fn linked(&self, id: &str, link: &str) -> Linked {
        let words = self.format(link, None);
        let sentence = self.around(id, "report", FluentArgs::new());
        let start = sentence.before.len();
        Linked {
            text: sentence.whole(&words),
            link: start..start + words.len(),
        }
    }

    fn format_wait(&self, id: &str, wait: Duration) -> String {
        let mut args = FluentArgs::new();
        args.set("wait", self.wait(wait));
        self.format(id, Some(&args))
    }

    fn format_league(&self, id: &str, league: &str) -> String {
        let mut args = FluentArgs::new();
        args.set("league", league);
        self.format(id, Some(&args))
    }

    fn format_number(
        &self,
        id: &str,
        name: &'static str,
        value: impl Into<FluentValue<'static>>,
    ) -> String {
        let mut args = FluentArgs::new();
        args.set(name, value);
        self.format(id, Some(&args))
    }

    /// The note of an estimate, with the kind of the quality searched when there is one.
    fn format_estimate(
        &self,
        note: EstimateNote,
        counted: usize,
        searched_catalyst: Option<Catalyst>,
    ) -> String {
        let Some(catalyst) = searched_catalyst else {
            return self.format_number(note.plain, "count", counted);
        };
        let mut args = FluentArgs::new();
        args.set("count", counted);
        args.set("quality", self.format_kind(QUALITY_OF_KIND, catalyst));
        self.format(note.of_kind, Some(&args))
    }

    fn format_kind(&self, id: &str, catalyst: Catalyst) -> String {
        let mut args = FluentArgs::new();
        args.set("kind", catalyst_id(catalyst));
        self.format(id, Some(&args))
    }

    fn format_age(&self, id: &str, age: &str) -> String {
        let mut args = FluentArgs::new();
        args.set("age", age);
        self.format(id, Some(&args))
    }

    /// The message `id` formatted with `args`, cut around the argument `name`, which the
    /// translation writes once. Without the argument, which the tests rule out, the whole
    /// message comes before it.
    fn around(&self, id: &str, name: &'static str, mut args: FluentArgs<'_>) -> CutSentence {
        args.set(name, PLACE);
        let line = self.format(id, Some(&args));
        let Some((before, after)) = line.split_once(PLACE) else {
            debug_assert!(false, "no {name} in the message {id}");
            return CutSentence {
                before: line,
                after: String::new(),
            };
        };
        debug_assert!(!after.contains(PLACE), "{name} twice in the message {id}");
        CutSentence {
            before: before.to_owned(),
            after: after.to_owned(),
        }
    }

    /// The text of the message `id`; the id itself when the message is missing, which the
    /// tests rule out.
    fn format(&self, id: &str, args: Option<&FluentArgs>) -> String {
        let Some(pattern) = self
            .bundle
            .get_message(id)
            .and_then(|message| message.value())
        else {
            debug_assert!(false, "missing message {id}");
            return id.to_owned();
        };
        let mut errors = Vec::new();
        let text = self
            .bundle
            .format_pattern(pattern, args, &mut errors)
            .into_owned();
        debug_assert!(errors.is_empty(), "errors in message {id}: {errors:?}");
        text
    }
}

/// The selector of a kind of catalyst in the messages.
fn catalyst_id(catalyst: Catalyst) -> &'static str {
    match catalyst {
        Catalyst::Life => "life",
        Catalyst::Mana => "mana",
        Catalyst::Defence => "defence",
        Catalyst::Physical => "physical",
        Catalyst::Fire => "fire",
        Catalyst::Cold => "cold",
        Catalyst::Lightning => "lightning",
        Catalyst::Chaos => "chaos",
        Catalyst::Attack => "attack",
        Catalyst::Caster => "caster",
        Catalyst::Speed => "speed",
        Catalyst::Attribute => "attribute",
        Catalyst::Minion => "minion",
    }
}

#[cfg(test)]
mod tests;
