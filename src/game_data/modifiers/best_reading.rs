//! The best reading of the lines of an item copied from the in-game trade: every way to choose
//! among its candidates that holds no type twice, ranked, within the room the item has for
//! prefixes and suffixes.

use super::candidates::{AffixSet, Candidate};
use crate::item::Affix;

/// The most ways to read the lines of an item copied from the in-game trade that are looked
/// at: a handful on a real item, each hybrid the lines may be doubling them, as does each
/// line that may be left without tier for its type ([`Unread`]; 60 at most on the items drawn
/// at random by `trade_tiers/game_tests/random_items.rs`, 2026-11-21), the choices
/// [`Search::is_reading`] refuses counted among them, though they aren't readings. Past them,
/// no line is read: the best of the readings looked at may not be the best of all.
pub(super) const MAX_READINGS: usize = 4096;

/// How many prefixes and suffixes the modifiers read from their values can add to the item.
#[derive(Debug, Clone, Copy)]
pub(super) struct Room {
    pub(super) prefixes: usize,
    pub(super) suffixes: usize,
}

impl Room {
    /// The bounds of how many of the modifiers that can be both are prefixes, the others
    /// suffixes, when the `chosen` candidates fit the room; none when they don't.
    fn bounds(self, chosen: &[usize], candidates: &[Candidate<'_>]) -> Option<(usize, usize)> {
        let (mut prefixes, mut suffixes, mut both) = (0_usize, 0_usize, 0_usize);
        let sets = chosen
            .iter()
            .filter_map(|&index| candidates.get(index))
            .flat_map(Candidate::sets);
        for set in sets {
            match set {
                AffixSet::Only(Affix::Prefix) => prefixes += 1,
                AffixSet::Only(Affix::Suffix) => suffixes += 1,
                AffixSet::Both => both += 1,
            }
        }
        let lowest = (both + suffixes).saturating_sub(self.suffixes);
        let highest = both.min(self.prefixes.checked_sub(prefixes)?);
        (lowest <= highest).then_some((lowest, highest))
    }

    /// The affixes a modifier that can be both is left with, while the others of the `chosen`
    /// candidates fit the room; both when the candidates don't fit it.
    pub(super) fn affixes_left(
        self,
        chosen: &[usize],
        candidates: &[Candidate<'_>],
    ) -> AffixesLeft {
        let both = chosen
            .iter()
            .filter_map(|&index| candidates.get(index))
            .flat_map(Candidate::sets)
            .filter(|&set| set == AffixSet::Both)
            .count();
        self.bounds(chosen, candidates)
            .map_or(AffixesLeft::BOTH, |(lowest, highest)| AffixesLeft {
                prefix: highest >= 1,
                suffix: lowest < both,
            })
    }
}

/// The affixes a modifier that can be a prefix or a suffix is left with, once the other
/// modifiers of a reading have taken their room on the item; neither for a reading without
/// such a modifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct AffixesLeft {
    prefix: bool,
    suffix: bool,
}

impl AffixesLeft {
    /// A prefix or a suffix: nothing tells which.
    pub(super) const BOTH: Self = Self {
        prefix: true,
        suffix: true,
    };

    /// Whether such a modifier can be of `affix`.
    pub(super) fn has(self, affix: Affix) -> bool {
        match affix {
            Affix::Prefix => self.prefix,
            Affix::Suffix => self.suffix,
        }
    }
}

/// How good a reading is, the better the greater: one fractured modifier at most, then the
/// room of the item, then the lines read with a tier, then the lines not split, then the
/// lines read alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ReadingKey {
    fractured: bool,
    room: bool,
    /// How many lines are read with a tier: a line only a hybrid holds is read in it
    /// (`30% increased Physical Damage` of a weapon, the pure modifier starting at 40%), and
    /// one no tier holds split into two modifiers.
    read: usize,
    /// How many lines aren't split into two modifiers: a line that a modifier holds alone
    /// isn't split (`18% increased Rarity of Items found` would also fit two of 9%).
    whole: usize,
    /// How many modifiers the lines make, those without tier counted: the lines alone
    /// first. Where a hybrid holds them too in the room of the item, the other readings
    /// leave them without tier ([`super::doubts::doubts`]), whichever is read here.
    modifiers: usize,
}

/// The best reading of the lines of an item among its candidates, and what the readings as
/// good as it leave in doubt.
pub(super) struct BestReading<'a> {
    candidates: &'a [Candidate<'a>],
    /// The indexes among `candidates` of those it reads, in the order of their lines.
    chosen: Vec<usize>,
    /// How many prefixes and suffixes the lines can add to the item.
    pub(super) room: Room,
    /// Whether it fits that room.
    pub(super) fits_room: bool,
    /// The lines that a reading as good reads otherwise, with the other lines of their
    /// modifiers: the indexes among the modifiers of the item.
    pub(super) ambiguous: Vec<usize>,
    /// Whether a reading as good leaves another room to the modifiers that can be a prefix or
    /// a suffix.
    pub(super) room_in_doubt: bool,
    /// The readings looked at to find it, [`MAX_READINGS`] at most.
    pub(super) readings: usize,
}

impl BestReading<'_> {
    /// The candidates it reads, in the order of their lines.
    pub(super) fn candidates(&self) -> impl Iterator<Item = &Candidate<'_>> {
        self.chosen
            .iter()
            .filter_map(|&index| self.candidates.get(index))
    }

    /// The affixes a modifier of it that can be both is left with ([`Room::affixes_left`]).
    pub(super) fn affixes_left(&self) -> AffixesLeft {
        self.room.affixes_left(&self.chosen, self.candidates)
    }
}

/// The search of the best reading stopped at [`MAX_READINGS`], some readings not looked at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Cut {
    /// The readings it looked at before it stopped.
    pub(super) readings: usize,
}

/// The best reading of the `count` modifiers of an item among `candidates`, within `room`,
/// `fractured` of the modifiers being fractured: the best of those that hold no type twice
/// ([`Search::is_reading`]). Past [`MAX_READINGS`], none: a reading not looked at may be as
/// good as the best of the others, or better.
pub(super) fn best_reading<'a>(
    candidates: &'a [Candidate<'a>],
    count: usize,
    room: Room,
    fractured: usize,
) -> Result<BestReading<'a>, Cut> {
    let mut search = Search::new(candidates, count, room, fractured);
    search.walk(0, &mut Vec::new());
    match search.best {
        Some((key, chosen)) if !search.cut => Ok(BestReading {
            candidates,
            chosen,
            room,
            fits_room: key.room,
            ambiguous: search.ambiguous,
            room_in_doubt: search.room_in_doubt,
            readings: search.readings,
        }),
        // Cut, the search holds the best of the readings it looked at only, which it doesn't
        // give. Not cut, it always holds one: each line read alone, or left without tier where
        // no candidate reads it so, or where an earlier line has taken its type.
        Some(_) | None => Err(Cut {
            readings: search.readings,
        }),
    }
}

/// When a reading may leave a modifier of the item without tier.
#[derive(Clone, Copy)]
enum Unread<'a> {
    /// In any reading: no candidate reads it alone.
    Freely,
    /// In none: a candidate reads it alone, the lines being read alone first.
    Never,
    /// Only in a reading that holds a type of this candidate, the one that reads it alone,
    /// which can't be read with it: a stat the game writes on two lines, of two marks, each
    /// of them the pure modifier of that stat by its value.
    WhenItsTypeIsTaken(&'a Candidate<'a>),
}

/// The ways to read the modifiers of an item, looked at one by one: for each, the candidates
/// chosen, the other modifiers without tier.
struct Search<'a> {
    candidates: &'a [Candidate<'a>],
    /// For each modifier of the item, the candidates that start at it, the shorter first.
    starting: Vec<Vec<usize>>,
    /// For each modifier of the item, when a reading may leave it without tier.
    unread: Vec<Unread<'a>>,
    room: Room,
    /// How many modifiers of the item are fractured.
    fractured: usize,
    readings: usize,
    /// Whether the search stopped at [`MAX_READINGS`], some readings not looked at.
    cut: bool,
    /// The best reading so far, the first one on a tie.
    best: Option<(ReadingKey, Vec<usize>)>,
    /// The lines that a reading as good as the best one reads otherwise
    /// ([`BestReading::ambiguous`]).
    ambiguous: Vec<usize>,
    /// Whether a reading as good as the best one leaves another room
    /// ([`BestReading::room_in_doubt`]).
    room_in_doubt: bool,
}

impl<'a> Search<'a> {
    /// A search among `candidates` that has looked at no reading yet, of the `count` modifiers
    /// of an item, `fractured` of them fractured, within `room`.
    fn new(candidates: &'a [Candidate<'a>], count: usize, room: Room, fractured: usize) -> Self {
        let mut starting = vec![Vec::new(); count];
        for (index, candidate) in candidates.iter().enumerate() {
            if let Some(starting) = starting.get_mut(candidate.modifiers.start) {
                starting.push(index);
            }
        }
        let unread = starting
            .iter()
            .enumerate()
            .map(|(modifier, starting)| unread(candidates, modifier, starting))
            .collect();
        Self {
            candidates,
            starting,
            unread,
            room,
            fractured,
            readings: 0,
            cut: false,
            best: None,
            ambiguous: Vec::new(),
            room_in_doubt: false,
        }
    }

    /// Every reading from the modifier `position` on, after the candidates `chosen`, at most
    /// [`MAX_READINGS`]: the lines alone first, a modifier left without tier where it may be
    /// ([`Unread`]), as one no candidate starts at.
    fn walk(&mut self, position: usize, chosen: &mut Vec<usize>) {
        if self.readings >= MAX_READINGS {
            self.cut = true;
            return;
        }
        let Some(starting) = self.starting.get(position) else {
            self.readings += 1;
            self.keep(chosen);
            return;
        };
        let starting = starting.clone();
        let may_be_unread = match self.unread.get(position) {
            Some(Unread::Freely | Unread::WhenItsTypeIsTaken(_)) | None => true,
            Some(Unread::Never) => false,
        };
        if may_be_unread {
            self.walk(position + 1, chosen);
        }
        for index in starting {
            let Some(end) = self.candidates.get(index).map(|c| c.modifiers.end) else {
                continue;
            };
            chosen.push(index);
            self.walk(end, chosen);
            chosen.pop();
        }
    }

    /// Whether the candidates `chosen` make a reading of the item: no type twice among their
    /// modifiers, the game rolling one modifier of a type at most; and a line that a candidate
    /// reads alone left without tier only where a chosen candidate has taken a type of that
    /// one ([`Unread::WhenItsTypeIsTaken`]), never to make room for the others.
    fn is_reading(&self, chosen: &[usize]) -> bool {
        let chosen = || {
            chosen
                .iter()
                .filter_map(|&index| self.candidates.get(index))
        };
        let has_a_type_twice = chosen().enumerate().any(|(place, candidate)| {
            let mut after = chosen().skip(place + 1);
            after.any(|other| other.shares_a_type_with(candidate))
        });
        let leaves_a_line_unread =
            self.unread
                .iter()
                .enumerate()
                .any(|(modifier, unread)| match *unread {
                    Unread::Freely | Unread::Never => false,
                    // Not read, the line isn't read by `alone` either: no chosen candidate is it.
                    Unread::WhenItsTypeIsTaken(alone) => {
                        let is_read =
                            chosen().any(|candidate| candidate.modifiers.contains(&modifier));
                        let is_taken =
                            chosen().any(|candidate| candidate.shares_a_type_with(alone));
                        !is_read && !is_taken
                    }
                });
        !has_a_type_twice && !leaves_a_line_unread
    }

    /// Ranks the reading of the candidates `chosen` against the best one so far, when they
    /// make one ([`Self::is_reading`]): kept when better; as good, the lines both read
    /// otherwise are ambiguous, and their room in doubt when they leave another one.
    fn keep(&mut self, chosen: &[usize]) {
        if !self.is_reading(chosen) {
            return;
        }
        let (mut read, mut joined, mut split) = (0, 0, 0);
        // The fractured modifiers, the lines alone counted.
        let mut fractured = self.fractured;
        for candidate in chosen
            .iter()
            .filter_map(|&index| self.candidates.get(index))
        {
            let lines = candidate.modifiers.len();
            read += lines;
            joined += lines - 1;
            split += candidate.count() - 1;
            if candidate.fractured {
                // Its fractured lines are among those of the item: as many modifiers as it makes.
                fractured = fractured + candidate.count() - lines;
            }
        }
        let key = ReadingKey {
            fractured: fractured <= 1,
            room: self.room.bounds(chosen, self.candidates).is_some(),
            read,
            whole: self.starting.len() - split,
            modifiers: self.starting.len() - joined + split,
        };
        match &self.best {
            Some((best, _)) if key < *best => {}
            // As good as the best one: nothing tells which of them rolled, and the order of
            // the lines, which made one the first, doesn't.
            Some((best, reading)) if key == *best => {
                let lines = differently_read(self.candidates, reading, chosen);
                for line in lines {
                    if !self.ambiguous.contains(&line) {
                        self.ambiguous.push(line);
                    }
                }
                let affixes_left = |reading| self.room.affixes_left(reading, self.candidates);
                if affixes_left(reading) != affixes_left(chosen) {
                    self.room_in_doubt = true;
                }
            }
            _ => {
                self.best = Some((key, chosen.to_vec()));
                self.ambiguous.clear();
                self.room_in_doubt = false;
            }
        }
    }
}

/// When a reading may leave without tier the modifier `modifier` of an item, the candidates
/// of the indexes `starting` among `candidates` starting at it: never when one of them reads
/// it alone, unless that one shares a type with a candidate of other lines, which a reading
/// can't hold with it.
fn unread<'a>(candidates: &'a [Candidate<'a>], modifier: usize, starting: &[usize]) -> Unread<'a> {
    let mut starting = starting.iter().filter_map(|&index| candidates.get(index));
    let Some(alone) = starting.find(|candidate| candidate.is_alone()) else {
        return Unread::Freely;
    };
    let is_of_other_lines = |other: &Candidate<'_>| !other.modifiers.contains(&modifier);
    if candidates
        .iter()
        .any(|other| is_of_other_lines(other) && other.shares_a_type_with(alone))
    {
        Unread::WhenItsTypeIsTaken(alone)
    } else {
        Unread::Never
    }
}

/// The lines that the readings `one` and `other`, as good, read otherwise: those of the
/// candidates only one of them has, a line split into other modifiers (the `30% increased
/// Rarity of Items found` of a tiara, a prefix T1 and a suffix T2, or a T2 and a T1), joined
/// with another line, or read by one of them only, with the other lines of these modifiers,
/// which would otherwise be read alone.
fn differently_read(candidates: &[Candidate<'_>], one: &[usize], other: &[usize]) -> Vec<usize> {
    let only_one = one.iter().filter(|index| !other.contains(index));
    let only_other = other.iter().filter(|index| !one.contains(index));
    only_one
        .chain(only_other)
        .filter_map(|&index| candidates.get(index))
        .flat_map(|candidate| candidate.modifiers.clone())
        .collect()
}

#[cfg(test)]
mod tests;
