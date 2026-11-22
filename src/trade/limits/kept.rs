//! What the limits know, kept in a file of the cache directory (`trade-limits.json`): written
//! after each answer and read at the launch, so that closing the application and launching it
//! again forgets neither the requests it sent, nor the limits and the restriction the trade
//! site told, nor the last search.
//!
//! A file in doubt (a clock of the system set back since, a file written by hand) is read on
//! the safe side: a time of the past that would come after the launch is read as the launch,
//! and a window lasts a day at most. A request whose time the reading moves stays in the
//! windows, where later is the safe side, and is no longer taken from the counts of the trade
//! site, where it would hide the hits of another tool. Only the end of a restriction more
//! than a day ahead, which no answer can ask, is ignored, as the wait of a refusal is
//! (`kept_wait`): it would stop the searches for as long as the clock was set back.

use std::cell::Cell;
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
    fn read_at(self, launch: Time) -> Known {
        let reading = Reading {
            behind_system: self.behind_system,
            launch,
            after_launch: Cell::new(0),
        };
        let Known {
            policies,
            last_search,
        } = self.known;
        let known = Known {
            policies: policies.map(|policy| reading.policy(policy)),
            last_search: last_search.map(|last| reading.past(last)),
        };
        let after_launch = reading.after_launch.get();
        if after_launch > 0 {
            log::warn!(
                "Trade site: {after_launch} times kept in {KEPT_FILE} come after the launch \
                 (the clock of the system set back since?): read as the launch"
            );
        }
        known
    }
}

/// The reading of a file at a launch: where each of its times goes.
struct Reading {
    /// How far behind the clock of the system the file was written.
    behind_system: Duration,
    launch: Time,
    /// The times of the past found after the launch so far.
    after_launch: Cell<usize>,
}

impl Reading {
    /// A time of the past, on the clock of the system: the launch at the latest.
    fn past(&self, time: Time) -> Time {
        let read = time + self.behind_system;
        if read > self.launch {
            self.after_launch.set(self.after_launch.get() + 1);
        }
        read.min(self.launch)
    }

    /// A request, no longer counted for sure once the reading moves the time it left at:
    /// taken from the counts of the trade site, a request read too recent would hide a hit
    /// of another tool. A clock behind by more than the margin moves every time: the requests
    /// sent before it fell behind are read more recent than they are.
    fn hit(&self, hit: Hit<Time>) -> Hit<Time> {
        let all_later = self.behind_system > MARGIN;
        let after_launch = hit.sent + self.behind_system > self.launch;
        Hit {
            sent: self.past(hit.sent),
            answered: self.past(hit.answered),
            counted: hit.counted && !all_later && !after_launch,
        }
    }

    /// The limits of a policy: its requests in the order the windows count on, whatever the
    /// file tells, without those no window counts any more.
    fn policy(&self, policy: PolicyLimits) -> PolicyLimits {
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
    /// version, nor from a file that can't be read, which the log tells.
    pub fn read(&self, launch: Time) -> Option<Known> {
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
        let known = kept.read_at(launch);
        let requests: usize = known.policies.iter().map(|policy| policy.sent.len()).sum();
        log::info!("Trade site: the limits kept in {KEPT_FILE}, {requests} requests counted");
        Some(known)
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
