//! What the limits know, kept in a file of the cache directory (`trade-limits.json`): written
//! when a search leaves and after each answer, and read at the launch, so that closing the
//! application and launching it again forgets neither the requests it sent, nor the limits
//! and the restriction the trade site told, nor the last search.
//!
//! A file in doubt (a clock of the system set back since, a file written by hand) is read on
//! the safe side: a time of the past that would come after the launch is read as the launch,
//! and a window lasts a day at most. Later is the safe side in a window, and the wrong one in
//! the counts of the trade site, from which the requests it counted for sure are taken to
//! find the hits of another tool. The clock is the one of the whole file: once it shows that
//! the times are read later than they were (one of them after the launch, or a file written
//! by a clock behind the clock of the system), no request of the file is taken from those
//! counts any more. Only the end of a restriction more than a day ahead, which no answer can
//! ask, is ignored, as the wait of a refusal is (`kept_wait`): it would stop the searches for
//! as long as the clock was set back.

use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::clock::Time;
use super::{Hit, Known, LONGEST_HEADER_TIME, LONGEST_REFUSAL_WAIT, MARGIN, PolicyLimits, Window};
use crate::files;
use crate::logging;

/// Changed with the shape of [`Kept`], and with the order of the policies in it: a file of
/// another version is ignored.
const VERSION: u32 = 1;
/// The file of the limits in the cache directory.
const KEPT_FILE: &str = "trade-limits.json";

/// What the file keeps: what the limits know (`K`), borrowed to be written, owned once read.
#[derive(Debug, Serialize, Deserialize)]
struct Kept<K> {
    /// How far the clock of the limits was behind the clock of the system when the file was
    /// written: its times are that much later on the clock of the system, the one the next
    /// launch starts from.
    behind_system: Duration,
    known: K,
}

impl Kept<Known> {
    /// What the file knew, as the limits of an application launched at `launch` count it: on
    /// the safe side when in doubt.
    fn read_at(self, launch: Time) -> AtLaunch {
        let mut reading = Reading {
            behind_system: self.behind_system,
            launch,
            after_launch: 0,
        };
        let Known {
            policies,
            last_search,
        } = self.known;
        let mut known = Known {
            policies: policies.map(|policy| reading.policy(policy)),
            last_search: last_search.map(|last| reading.past(last)),
        };
        if reading.reads_later() {
            // Read more recent than it was sent, a request taken from the counts of the
            // trade site would hide a hit of another tool.
            let requests = known
                .policies
                .iter_mut()
                .flat_map(|policy| &mut policy.sent);
            for request in requests {
                request.counted = false;
            }
        }
        AtLaunch {
            known,
            after_launch: reading.after_launch,
        }
    }
}

/// What a file knew, as a launch reads it.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct AtLaunch {
    pub known: Known,
    /// How many of its times of the past came after the launch, and were read as the launch:
    /// the clock of the system was set back since, or the file written by hand.
    pub after_launch: usize,
}

/// The reading of a file at a launch: where each of its times goes.
struct Reading {
    /// How far behind the clock of the system the file was written.
    behind_system: Duration,
    launch: Time,
    /// The times of the past found after the launch so far.
    after_launch: usize,
}

impl Reading {
    /// A time of the past, on the clock of the system: the launch at the latest.
    fn past(&mut self, time: Time) -> Time {
        let read = time + self.behind_system;
        if read > self.launch {
            self.after_launch += 1;
        }
        read.min(self.launch)
    }

    /// Whether the times of the file are read later than they were, as far as the file read
    /// so far shows: written by a clock behind by more than the margin, every time is told
    /// later, those from before it fell behind too late; with a time after the launch, the
    /// clock of the system was set back, under every time of the file.
    fn reads_later(&self) -> bool {
        self.behind_system > MARGIN || self.after_launch > 0
    }

    /// A request, at the times read.
    fn hit(&mut self, hit: Hit<Time>) -> Hit<Time> {
        Hit {
            sent: self.past(hit.sent),
            answered: self.past(hit.answered),
            counted: hit.counted,
        }
    }

    /// The limits of a policy: its requests in the order the windows count on, whatever the
    /// file tells, without those no window counts any more.
    fn policy(&mut self, policy: PolicyLimits) -> PolicyLimits {
        let PolicyLimits {
            sent,
            windows,
            blocked_until,
        } = policy;
        let mut sent: Vec<Hit<Time>> = sent.into_iter().map(|hit| self.hit(hit)).collect();
        sent.sort_by_key(|hit| hit.answered);
        let windows = windows
            .into_iter()
            .map(|window| Window {
                hits: window.hits,
                period: window.period.min(LONGEST_HEADER_TIME),
                others: window.others,
                told: self.past(window.told),
            })
            .collect();
        let blocked_until = blocked_until
            .and_then(|until| restriction_end(until + self.behind_system, self.launch));
        let mut policy = PolicyLimits {
            sent,
            windows,
            blocked_until,
        };
        policy.forget_before(self.launch);
        policy
    }
}

/// The end `until` of a restriction, as limits launched at `launch` read it: none when it
/// is further ahead than an answer can tell (a day, and the margin), which the log tells.
fn restriction_end(until: Time, launch: Time) -> Option<Time> {
    let left = until.saturating_duration_since(launch);
    if left > LONGEST_REFUSAL_WAIT {
        log::warn!(
            "Trade site: the restriction kept in {KEPT_FILE} would last {} s, more than an \
             answer asks: ignored",
            left.as_secs()
        );
        return None;
    }
    Some(until)
}

/// The file of the limits in a cache directory.
#[derive(Debug)]
pub(super) struct LimitsFile {
    directory: PathBuf,
    /// The log already says that the limits can't be kept, until they are again.
    told_unkept: bool,
}

impl LimitsFile {
    /// The file of the limits in `directory`.
    pub fn in_directory(directory: PathBuf) -> Self {
        Self {
            directory,
            told_unkept: false,
        }
    }

    /// Where the file is.
    fn path(&self) -> PathBuf {
        self.directory.join(KEPT_FILE)
    }

    /// What the file knows, for limits launched at `launch`: nothing without a file of this
    /// version, nor from a file that can't be read, which the log tells, as it tells a file
    /// whose times are read later than they were.
    pub fn read(&self, launch: Time) -> Option<AtLaunch> {
        let kept: Kept<Known> = match files::read_versioned_json(&self.path(), VERSION) {
            Ok(kept) => kept?,
            Err(error) => {
                log::warn!(
                    "{} can't be read, the limits of the trade site it may keep are ignored: \
                     {error}",
                    logging::without_home(&self.path()).display()
                );
                return None;
            }
        };
        let behind_system = kept.behind_system;
        let read = kept.read_at(launch);
        let requests: usize = read
            .known
            .policies
            .iter()
            .map(|policy| policy.sent.len())
            .sum();
        log::info!("Trade site: the limits kept in {KEPT_FILE}, {requests} requests counted");
        if behind_system > MARGIN {
            log::info!(
                "Trade site: {KEPT_FILE} was written by a clock {} s behind the clock of the \
                 system (a sleep of the system?): its times are read that much later",
                behind_system.as_secs()
            );
        }
        if read.after_launch > 0 {
            log::warn!(
                "Trade site: times kept in {KEPT_FILE} come after the launch, {} of them (the \
                 clock of the system set back since?): read as the launch",
                read.after_launch
            );
        }
        Some(read)
    }

    /// Keeps `known`, told by a clock `behind_system` behind the clock of the system; a
    /// failure is logged once, and what the limits know then ends with the application.
    pub fn keep(&mut self, known: &Known, behind_system: Duration) {
        let kept = Kept {
            behind_system,
            known,
        };
        match files::write_versioned_json(&self.directory, KEPT_FILE, VERSION, &kept) {
            Ok(()) => self.told_unkept = false,
            // Once: every answer would say the same.
            Err(error) if !self.told_unkept => {
                log::warn!(
                    "The limits of the trade site couldn't be kept in {}: {error}",
                    logging::without_home(&self.path()).display()
                );
                self.told_unkept = true;
            }
            Err(_) => {}
        }
    }
}

#[cfg(test)]
mod tests;
