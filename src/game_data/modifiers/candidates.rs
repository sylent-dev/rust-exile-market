//! The candidates of the reading of an item copied from the in-game trade: each line, and each
//! run of consecutive lines, that a modifier of the tables holds, or that adds up two.

use super::ModifierTiers;
use super::copied::{Copied, CopiedTier};
use super::item_tables::ItemTables;
use super::lookup::{FoundModifier, Tie, find};
use super::split_lines::{AddedTiers, splits};
use super::tables::{Table, TableAffix};
use crate::item::{Affix, ModifierOrigin};

/// Consecutive modifiers of an item copied from the in-game trade read as one, and the affixes
/// whose tables hold their values; or one of these lines split into two modifiers.
pub(super) struct Candidate<'a> {
    pub(super) modifiers: std::ops::Range<usize>,
    /// Whether its modifiers are fractured.
    pub(super) fractured: bool,
    /// A prefix, a suffix, or both, never none; one for a split line.
    pub(super) affixes: Vec<(Affix, ModifierTiers)>,
    /// The other modifier of a split line.
    pub(super) added: Option<AddedTiers>,
    /// The types of its modifiers that are sure, two modifiers of one type never being on an
    /// item: both for a split line; for the others its type when modifiers of that type alone
    /// hold its lines, in each affix it can be, none otherwise
    /// ([`FoundModifier::modifier_type`]; a line modifiers of two types hold as a prefix and
    /// as a suffix).
    pub(super) types: Vec<&'a str>,
}

impl Candidate<'_> {
    /// The affixes its modifier can be; for a split line, the one of the modifier of every
    /// line.
    pub(super) fn set(&self) -> AffixSet {
        match self.affixes.as_slice() {
            [(affix, _)] => AffixSet::Only(*affix),
            _ => AffixSet::Both,
        }
    }

    /// The affixes each of its modifiers can be: two modifiers for a split line.
    pub(super) fn sets(&self) -> impl Iterator<Item = AffixSet> {
        let added = self.added.as_ref().map(|added| AffixSet::Only(added.affix));
        std::iter::once(self.set()).chain(added)
    }

    /// How many modifiers it makes.
    pub(super) fn count(&self) -> usize {
        1 + usize::from(self.added.is_some())
    }

    /// Whether it reads one line alone, as one modifier.
    pub(super) fn is_alone(&self) -> bool {
        self.modifiers.len() == 1 && self.added.is_none()
    }

    /// Whether a modifier of it is of the type of a modifier of `other`, as far as their
    /// types are sure: an item never has both.
    pub(super) fn shares_a_type_with(&self, other: &Self) -> bool {
        self.types.iter().any(|held| other.types.contains(held))
    }
}

/// The affixes a modifier can be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AffixSet {
    Only(Affix),
    Both,
}

/// The most lines a modifier of `tables` has: a hybrid of three lines in those of the game.
fn most_lines(tables: &[Table]) -> usize {
    tables
        .iter()
        .map(|table| table.lines.len())
        .max()
        .unwrap_or(1)
}

/// Every modifier or run of modifiers of an item whose values `tables` hold, the tier the
/// worst among those the level of the item allows, `copied` giving for each modifier of the
/// item its lines to read (none for a modifier that isn't read, or whose lines have no stat of
/// the game data): one modifier, or up to as many as the lines of a hybrid, of the same
/// origin; then each way one of their lines adds up two modifiers ([`splits`]). The shorter
/// first at each start.
pub(super) fn candidates<'a>(
    tables: &ItemTables<'a>,
    copied: &[Option<Copied<'_>>],
) -> Vec<Candidate<'a>> {
    let most_lines = most_lines(tables.all);
    let mut candidates = Vec::new();
    for (start, first) in copied.iter().enumerate() {
        // A run starts at a modifier to read, and goes on while the next ones are of its
        // origin.
        let Some(origin) = first.as_ref().map(|first| first.origin) else {
            continue;
        };
        let mut lines = Vec::new();
        // The index among the modifiers of the item of the modifier of each line.
        let mut owners = Vec::new();
        // A bound of the work only: a longer run has more lines than any modifier of the
        // tables, and makes no candidate.
        let run = copied.iter().skip(start).take(most_lines);
        for (end, part) in (start + 1..).zip(run) {
            let Some(part) = part.as_ref().filter(|part| part.origin == origin) else {
                break;
            };
            lines.extend(part.lines.iter().cloned());
            owners.extend(std::iter::repeat_n(end - 1, part.lines.len()));
            let joined = Copied {
                lines: lines.clone(),
                ranges: None,
                tier: CopiedTier::Unknown,
                origin,
            };
            candidates.extend(run_candidates(tables, &joined, &owners, start..end));
        }
    }
    candidates
}

/// The candidates of the run of modifiers `modifiers` of the item, read as the lines `joined`,
/// `owners` giving the index among the modifiers of the item of the modifier of each line: the
/// modifier that holds them all, with each affix whose tables hold them; then each way one of
/// these lines adds up two modifiers.
fn run_candidates<'a>(
    tables: &ItemTables<'a>,
    joined: &Copied<'_>,
    owners: &[usize],
    modifiers: std::ops::Range<usize>,
) -> Vec<Candidate<'a>> {
    let fractured = joined.origin == Some(ModifierOrigin::Fractured);
    let found: Vec<(Affix, FoundModifier<'a>)> = [
        (Affix::Prefix, TableAffix::Prefix),
        (Affix::Suffix, TableAffix::Suffix),
    ]
    .into_iter()
    .filter_map(|(affix, table_affix)| {
        Some((affix, find(tables, table_affix, joined, Tie::Worst)?))
    })
    .collect();
    let whole = (!found.is_empty()).then(|| Candidate {
        modifiers: modifiers.clone(),
        fractured,
        types: shared_type(&found).into_iter().collect(),
        affixes: found
            .into_iter()
            .map(|(affix, found)| (affix, found.tiers))
            .collect(),
        added: None,
    });
    // A modifier of one line only may be split: the line the game writes for both.
    let has_one_line = |owner: usize| owners.iter().filter(|&&other| other == owner).count() == 1;
    let split = owners
        .iter()
        .enumerate()
        .filter(|&(_, &owner)| has_one_line(owner))
        .flat_map(|(summed, &owner)| splits(tables, joined, (summed, owner)))
        .map(|split| Candidate {
            modifiers: modifiers.clone(),
            fractured,
            affixes: vec![(split.affix, split.tiers)],
            added: Some(split.added),
            types: split.types.to_vec(),
        });
    whole.into_iter().chain(split).collect()
}

/// The type of the modifiers that hold the lines of a candidate, `found` for each affix it
/// can be: none when those of an affix are of two types, or those of the prefix of another
/// type than those of the suffix.
fn shared_type<'a>(found: &[(Affix, FoundModifier<'a>)]) -> Option<&'a str> {
    let mut types = found.iter().map(|(_, found)| found.modifier_type);
    let first = types.next()??;
    types.all(|held| held == Some(first)).then_some(first)
}

#[cfg(test)]
mod tests;
