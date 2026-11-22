//! What the limits know, kept in a file of the cache directory (`trade-limits.json`): written
//! after each answer and read at the launch, so that closing the application and launching it
//! again forgets neither the requests it sent, nor the limits and the restriction the trade
//! site told, nor the last search.
//!
//! A file in doubt (a clock of the system set back since, a file written by hand) is read on
//! the safe side: a time of the past that would come after the launch is read as the launch,
//! and a window lasts a day at most. Only the end of a restriction more than a day ahead,
//! which no answer can ask, is ignored, as the wait of a refusal is (`kept_wait`): it would
//! stop the searches for as long as the clock was set back.

use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::clock::Time;
use super::{Hit, Known, LONGEST_HEADER_TIME, LONGEST_REFUSAL_WAIT, PolicyLimits, Window};
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
        let Self {
            behind_system,
            known: Known {
                policies,
                last_search,
            },
        } = self;
        // A time of the past on the clock of the system: the launch at the latest.
        let past = |time: Time| (time + behind_system).min(launch);
        let policies = policies.map(|policy| {
            let PolicyLimits {
                sent,
                windows,
                blocked_until,
            } = policy;
            let mut sent: Vec<Hit<Time>> = sent
                .into_iter()
                .map(|hit| Hit {
                    sent: past(hit.sent),
                    answered: past(hit.answered),
                    counted: hit.counted,
                })
                .collect();
            // The order the windows count on, whatever the file tells.
            sent.sort_by_key(|hit| hit.answered);
            let windows = windows
                .into_iter()
                .map(|window| Window {
                    hits: window.hits,
                    period: window.period.min(LONGEST_HEADER_TIME),
                    others: window.others,
                    told: past(window.told),
                })
                .collect();
            let blocked_until =
                blocked_until.and_then(|until| restriction_end(until + behind_system, launch));
            PolicyLimits {
                sent,
                windows,
                blocked_until,
            }
        });
        Known {
            policies,
            last_search: last_search.map(past),
        }
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
    pub fn read(&self, launch: Time) -> Known {
        let kept: Kept<Known> = match files::read_versioned_json(&self.path(), VERSION) {
            Ok(Some(kept)) => kept,
            Ok(None) => return Known::default(),
            Err(error) => {
                log::warn!(
                    "{} can't be read, the limits of the trade site it may keep are ignored: \
                     {error}",
                    logging::without_home(&self.path()).display()
                );
                return Known::default();
            }
        };
        let known = kept.read_at(launch);
        let requests: usize = known.policies.iter().map(|policy| policy.sent.len()).sum();
        log::info!("Trade site: the limits kept in {KEPT_FILE}, {requests} requests counted");
        known
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
