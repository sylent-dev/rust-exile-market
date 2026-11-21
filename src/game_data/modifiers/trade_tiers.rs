//! The tiers of the modifiers of an item copied from the in-game trade, found from their values
//! alone: its candidates, its best reading, then the other readings that put lines in doubt,
//! why a line is left without tier, and how the search ended.

use super::ModifierTiers;
use super::best_reading::{AffixesLeft, BestReading, Cut, Room, best_reading};
use super::candidates::{AffixSet, Candidate, candidates};
use super::copied::Copied;
use super::doubts::{self, Doubts, Outcome};
use super::item_tables::{ItemBase, ItemTables};
use super::split_lines::AddedTiers;
use crate::game_data::lines::modifier_stats;
use crate::item::{Affix, Modifier, ModifierKind, ModifierOrigin, NoTier};
use crate::language::Language;

/// How many prefixes and suffixes an item can have: those of its rarity, with those its runes
/// and implicits add or take away (`+1 Suffix Modifier allowed`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AffixLimits {
    pub prefixes: u32,
    pub suffixes: u32,
    /// Whether the fill of the item is its count alone, without most nor open place: a
    /// corrupted rare waystone, whose limits are no fixed number of each affix.
    pub counted_only: bool,
}

impl AffixLimits {
    /// At most `prefixes` prefixes and `suffixes` suffixes, a fill with its most.
    #[cfg(test)]
    #[must_use]
    pub const fn new(prefixes: u32, suffixes: u32) -> Self {
        Self {
            prefixes,
            suffixes,
            counted_only: false,
        }
    }
}

/// An item copied from the in-game trade, as the tiers of its modifiers are found from their
/// values.
#[derive(Debug, Clone, Copy)]
pub struct TradeItem<'a> {
    pub base: ItemBase<'a>,
    /// How many prefixes and suffixes it can have.
    pub limits: AffixLimits,
    /// Whether it is sanctified: each of its values can have left its tier, and none is found.
    pub is_sanctified: bool,
}

/// The tiers of a modifier copied from the in-game trade, found from its values.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct FoundTiers {
    /// The indexes among the modifiers of the item of the lines read as one modifier: one
    /// line, or the consecutive lines of a hybrid; for a line that adds up two modifiers, the
    /// lines of the one of `affixes`, the line itself or with the other lines of its hybrid.
    pub(super) modifiers: std::ops::Range<usize>,
    /// Each affix the modifier can be, with its tiers on the base, the copied one the worst
    /// that holds its values: a prefix or a suffix, or both when the item has room for either
    /// (the `18% increased Rarity of Items found` of a helmet), the prefix first, as
    /// [`crate::item::TierDoubt::EitherAffix`] reads them. The ranges of a tier, and
    /// `counts`, follow the line matches of [`modifier_stats`] for each of these modifiers in
    /// their order.
    pub(super) affixes: Vec<(Affix, ModifierTiers)>,
    /// For a line that adds up two modifiers of its stat, the other one: a pure modifier of
    /// that line (the pure 170–179 of the `258% increased Physical Damage` of a bow, the 75–79
    /// of the hybrid with `+195 to Accuracy Rating` in `affixes`); of two pure modifiers of a
    /// prefix and a suffix, the suffix. One of both may be in two neighbouring tiers
    /// ([`ModifierTiers::neighbour`]).
    pub(super) added: Option<AddedTiers>,
}

/// What the search of the tiers of an item copied from the in-game trade tells of its lines.
#[derive(Debug, Clone, PartialEq, Default)]
pub(super) struct TradeTiers {
    /// The tiers found, in the order of their lines.
    pub(super) found: Vec<FoundTiers>,
    /// The indexes among the modifiers of the item of the lines searched and left without
    /// tier, in their order, each with why.
    pub(super) without_tier: Vec<(usize, NoTier)>,
}

/// The tiers of the explicit `modifiers` of `item`, rare or magic, copied from the in-game
/// trade, found from their values in the tables of its base among the tiers its level allows;
/// the other modifiers count as the affix their header tells. A
/// modifier no tier holds isn't among those found, nor counted as a prefix or a suffix: a stat
/// the tables don't know; nor a line adding up two modifiers of the same stat in more than one
/// way (two ways in two neighbouring tiers of one of them being one), with the other lines of
/// its hybrids, which take their room.
///
/// Only the explicit modifiers without header are read, one line each, as the parser reads
/// such a copy: a modifier whose header the parser doesn't know isn't a line of the in-game
/// trade, the caller leaves it out. Each line is read alone, unless a line has no tier that
/// way, or the item more prefixes or suffixes than its limits, or two fractured modifiers (a
/// Fracturing Orb fractures one modifier of an item not fractured yet): the consecutive lines
/// of one origin are then read as one modifier when they fit a hybrid of the tables (the
/// fractured `41% increased Energy Shield` and `+34 to maximum Mana` of a tiara). An affix the
/// full prefixes or suffixes rule out is left out, unless no reading fits them, the tables
/// being then out of step with the game. A value in several tiers takes the worst of them.
///
/// A line no other reading holds is split into two modifiers of two types with its stat, as
/// the game writes them on one line (the `258% increased Physical Damage` of a bow, a pure
/// 179% and the 79% of the hybrid with `+195 to Accuracy Rating`): a pure modifier, and
/// another, pure or a hybrid whose other lines follow or precede it with the same origin; the
/// split counts two affixes, and a reading with fewer split lines is better: a line a modifier
/// holds alone is split only when the room of the item needs it, or when the other line of the
/// hybrid has no tier alone (the `9% increased Armour` of a body armour, under every pure tier,
/// with `+100 to maximum Life`: the hybrid T6 and the pure T7). Two ways to split a line that
/// differ only by the tier of one of its modifiers, two neighbouring tiers, are one, that
/// modifier in both (`248%` with `+195 to Accuracy Rating`: the hybrid T1 and the pure T1 or
/// T2).
///
/// A reading never holds two modifiers of one type, which the game can't roll: a stat the
/// game writes on two lines, of two marks (`172% increased Physical Damage` and `79% increased
/// Physical Damage (desecrated)` on a bow), isn't twice its pure modifier, but that modifier
/// and a hybrid with the lines next to one of them; where each line can only be the pure
/// modifier, both stay without tier.
///
/// When the application can't tell, it doesn't decide (owner's rule on 2026-11-10): a line
/// stays without tier, with the other lines of its modifiers, when two readings as good read
/// it otherwise (a rarity split into a prefix of tier 1 and a suffix of tier 2, or of tier 2
/// and tier 1; a line a hybrid joins with the line before it, or another with the line after
/// it), the other modifiers being read as the first of them does, and when another reading
/// fits the item as well, which [`doubts::doubts`] tells: two pure modifiers or their
/// hybrid, a line adding up modifiers in another way. When such readings leave another room
/// to a line that can be a prefix or a suffix, it keeps both its affixes. A search that stops
/// at one of its bounds ([`super::best_reading::MAX_READINGS`], `MAX_STEPS`) leaves every line
/// without tier, as does one that doesn't meet its best reading among the others: it can't
/// vouch for the reading it would show.
///
/// A hybrid whose lines don't follow each other is never read as one modifier, a modifier
/// being read on consecutive lines ([`FoundTiers::modifiers`]): its lines stay without tier,
/// also when the room of the item leaves no other reading.
///
/// A line left without tier says why ([`lines_without_tier`]): in doubt, when modifiers of
/// the tables hold it and nothing tells which rolled, or held by no modifier.
///
/// No tier is found on a sanctified item, whose values can each have left their tier, for
/// the one next to it as well (the `+74 to maximum Mana` of the owner's sceptre, a T5
/// `80–89`, falls in the T6): nothing in such a copy tells which did (owner's choice on
/// 2026-11-11). Its lines aren't searched, nor those of an item whose base the tables don't
/// know: none of them is told without tier.
#[must_use]
pub(super) fn trade_tiers(item: &TradeItem<'_>, modifiers: &[Modifier]) -> TradeTiers {
    if item.is_sanctified {
        return TradeTiers::default();
    }
    let Some(tables) = ItemTables::of_game(item.base) else {
        return TradeTiers::default();
    };
    let ItemBase {
        language,
        base_type,
        ..
    } = item.base;
    let (tiers, effort) = trade_tiers_in(&tables, language, modifiers, item.limits);
    log::debug!(
        "Game data, {base_type} copied from the in-game trade: read in {} readings and {} steps",
        effort.readings,
        effort.steps
    );
    effort.ending.log(base_type, item.limits);
    tiers
}

/// What the search of the tiers of an item copied from the in-game trade took, against its
/// bounds, and how it ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Effort {
    /// The readings of its lines looked at, [`super::best_reading::MAX_READINGS`] at most.
    readings: usize,
    /// The steps of the search of the other readings ([`doubts::doubts`]).
    steps: usize,
    ending: Ending,
}

/// How the search of the tiers of an item copied from the in-game trade ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ending {
    /// The readings stopped at [`super::best_reading::MAX_READINGS`], some not looked at: no
    /// line is read, and the other readings aren't searched.
    ReadingsCut,
    /// With a best reading, then as the search of the other readings ended: a line keeps
    /// its tier as that end tells, and with every affix unless the best reading was met.
    OtherReadings(Outcome),
}

impl Ending {
    /// Logs what the search couldn't vouch for on an item of `base_type`, which can have
    /// `limits` prefixes and suffixes.
    fn log(self, base_type: &str, limits: AffixLimits) {
        let AffixLimits {
            prefixes, suffixes, ..
        } = limits;
        match self {
            Self::OtherReadings(Outcome::BestMet) => {}
            Self::ReadingsCut => log::warn!(
                "Game data, {base_type} copied from the in-game trade: the readings of its lines couldn't all be looked at, its lines left without tier"
            ),
            Self::OtherReadings(Outcome::Cut) => log::warn!(
                "Game data, {base_type} copied from the in-game trade: the other readings of its lines couldn't all be looked at, its lines left without tier"
            ),
            Self::OtherReadings(Outcome::BestNotMet) => log::warn!(
                "Game data, {base_type} copied from the in-game trade: the other readings of its lines don't hold its best one, its lines left without tier"
            ),
            Self::OtherReadings(Outcome::OnlyAnotherFits) => log::debug!(
                "Game data, {base_type} copied from the in-game trade: its best reading doesn't fit {prefixes} prefixes and {suffixes} suffixes, others do, the lines they don't all read the same left without tier"
            ),
            Self::OtherReadings(Outcome::NoneFits) => log::warn!(
                "Game data, {base_type} copied from the in-game trade: no reading of its lines fits {prefixes} prefixes and {suffixes} suffixes"
            ),
        }
    }
}

/// Whether `modifier` is a line of the in-game trade whose tier is searched from its values:
/// an explicit modifier without header, revealed.
fn is_searched(modifier: &Modifier) -> bool {
    modifier.kind == ModifierKind::Explicit(None)
        && modifier.tier.is_none()
        && !modifier.is_unrevealed
        && !modifier.lines.is_empty()
}

/// For each of `modifiers`, its lines as copied when it is a line of the in-game trade to
/// read: one searched ([`is_searched`]), with its stat of the game data.
fn lines_to_read(language: Language, modifiers: &[Modifier]) -> Vec<Option<Copied<'static>>> {
    modifiers
        .iter()
        .map(|modifier| {
            is_searched(modifier)
                .then(|| Copied::of(modifier, &modifier_stats(language, modifier, None)))
                .flatten()
        })
        .collect()
}

/// The lines of `modifiers` searched ([`is_searched`]) that `found` leaves without tier, each
/// with why: in doubt when `is_in_doubt` says so of a line to read, one with its stat of the
/// game data (`to_read`, [`lines_to_read`]); held by no modifier otherwise, a line without
/// such a stat among them.
fn lines_without_tier(
    modifiers: &[Modifier],
    to_read: &[Option<Copied<'_>>],
    found: &[FoundTiers],
    is_in_doubt: impl Fn(usize) -> bool,
) -> Vec<(usize, NoTier)> {
    let has_tier = |line| found.iter().any(|found| found.modifiers.contains(&line));
    let has_stat = |line: usize| to_read.get(line).is_some_and(Option::is_some);
    modifiers
        .iter()
        .enumerate()
        .filter(|&(line, modifier)| is_searched(modifier) && !has_tier(line))
        .map(|(line, _)| {
            let why = if has_stat(line) && is_in_doubt(line) {
                NoTier::InDoubt
            } else {
                NoTier::NoModifier
            };
            (line, why)
        })
        .collect()
}

/// [`trade_tiers`] among `tables`, for lines written in `language`: those of the game, or
/// those of a test, so that the search is tested on tables a patch of the game doesn't change;
/// with what the search took.
///
/// A line left without tier is in doubt when a candidate holds it, also one the room of the
/// item rules out, or another reading leaves it so, and every line to read is when the search
/// stops at a bound or doesn't meet its best reading; a line no candidate holds and no reading
/// in the room of the item reads is held by no modifier, as far as the search tells: a hybrid
/// whose lines are apart is no candidate, and no reading when the item has no room for it.
fn trade_tiers_in(
    tables: &ItemTables<'_>,
    language: Language,
    modifiers: &[Modifier],
    limits: AffixLimits,
) -> (TradeTiers, Effort) {
    let copied = lines_to_read(language, modifiers);
    let candidates = candidates(tables, &copied);
    let room = room_left(limits, modifiers);
    let fractured = modifiers
        .iter()
        .filter(|modifier| modifier.origin == Some(ModifierOrigin::Fractured))
        .count();
    let best = match best_reading(&candidates, modifiers.len(), room, fractured) {
        Ok(best) => best,
        // Cut at the bound of its readings, the search has no best reading: no line keeps a
        // tier it can't vouch for.
        Err(Cut { readings }) => {
            let effort = Effort {
                readings,
                steps: 0,
                ending: Ending::ReadingsCut,
            };
            let tiers = TradeTiers {
                found: Vec::new(),
                without_tier: lines_without_tier(modifiers, &copied, &[], |_| true),
            };
            return (tiers, effort);
        }
    };
    let doubts = doubts::doubts(tables, &copied, &best);
    let effort = Effort {
        readings: best.readings,
        steps: doubts.steps,
        ending: Ending::OtherReadings(doubts.outcome),
    };
    let found = found_tiers(&best, &doubts);
    let is_in_doubt = |line: usize| {
        let is_held = candidates
            .iter()
            .any(|candidate| candidate.modifiers.contains(&line));
        is_held || doubts.in_doubt.contains(&line)
    };
    let without_tier = lines_without_tier(modifiers, &copied, &found, is_in_doubt);
    let tiers = TradeTiers {
        found,
        without_tier,
    };
    (tiers, effort)
}

/// The room `limits` leave to the lines read from their values: without those of `modifiers`
/// whose header tells the affix.
fn room_left(limits: AffixLimits, modifiers: &[Modifier]) -> Room {
    let left = |limit: u32, affix| {
        let known = modifiers
            .iter()
            .filter(|modifier| modifier.kind == ModifierKind::Explicit(Some(affix)))
            .count();
        usize::try_from(limit)
            .unwrap_or(usize::MAX)
            .saturating_sub(known)
    };
    Room {
        prefixes: left(limits.prefixes, Affix::Prefix),
        suffixes: left(limits.suffixes, Affix::Suffix),
    }
}

/// The tiers the `best` reading gives, once the other readings told their `doubts`: without
/// the lines in doubt, and each modifier that can be a prefix or a suffix with the affixes
/// the item has room for.
fn found_tiers(best: &BestReading<'_>, doubts: &Doubts) -> Vec<FoundTiers> {
    // A modifier that can be both keeps the affixes the others leave room for
    // ([`Room::affixes_left`], both when the reading doesn't fit the room): both too when a
    // reading as good, or one in doubt, leaves another room.
    let affixes_left = if best.room_in_doubt || doubts.other_room {
        AffixesLeft::BOTH
    } else {
        best.affixes_left()
    };
    // A line two readings as good read otherwise, or another reading leaves in doubt, and the
    // other lines of its modifiers: without tier, the others read as the best reading does.
    let in_doubt = |candidate: &Candidate<'_>| {
        let mut lines = best.ambiguous.iter().chain(&doubts.in_doubt);
        lines.any(|line| candidate.modifiers.contains(line))
    };
    best.candidates()
        .filter(|candidate| !in_doubt(candidate))
        .map(|candidate| {
            let kept = |affix| match candidate.set() {
                AffixSet::Only(_) => true,
                AffixSet::Both => affixes_left.has(affix),
            };
            FoundTiers {
                modifiers: candidate.modifiers.clone(),
                // Without the affix the item has no room for, whose tables hold the values
                // too: a prefix, the prefixes being full.
                affixes: candidate
                    .affixes
                    .iter()
                    .filter(|(affix, _)| kept(*affix))
                    .cloned()
                    .collect(),
                added: candidate.added.clone(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod game_tests;
