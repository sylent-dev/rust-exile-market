//! The wait the trade site asked for by refusing a request, shared by the two threads that
//! call it: a search refused makes the download of the item names wait, and the other way
//! round, since both reach the same site from the same address. Its end is kept in the cache
//! directory, so that closing the application and launching it again doesn't end the wait.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant, SystemTime};

use serde::{Deserialize, Serialize};

use super::limits::LONGEST_REFUSAL_WAIT;
use crate::files;
use crate::logging;

/// The file of the end of the wait in the cache directory.
const KEPT_FILE: &str = "trade-refusal.json";
/// Changed with the shape of [`Kept`]: a file of another version is ignored.
const KEPT_VERSION: u32 = 1;

/// The time before which no request is sent to the trade site, if it refused one. Its clones
/// share the same time.
#[derive(Debug, Clone, Default)]
pub struct Refusal(Arc<Shared>);

/// What the clones of a [`Refusal`] share.
#[derive(Debug, Default)]
struct Shared {
    /// Behind a mutex rather than sent through a channel: each thread reads it before each
    /// of its requests, and a refusal of one must stop the next request of the other.
    until: Mutex<Option<Instant>>,
    /// The cache directory, where the end of the wait is kept for the next launch; without
    /// one, the wait ends with the application.
    directory: Option<PathBuf>,
}

/// The end of a wait as its file keeps it: a time of the clock of the system, the only one
/// that goes on from one launch to the next.
#[derive(Debug, Serialize, Deserialize)]
struct Kept {
    until: SystemTime,
}

impl Refusal {
    /// The wait the last launch kept in `directory`, when it isn't over, and every wait to
    /// come kept there in turn.
    #[must_use]
    pub fn kept_in(directory: Option<PathBuf>) -> Self {
        let Some(directory) = directory else {
            log::warn!(
                "No cache directory: the wait of a refusal of the trade site will end with \
                 the application"
            );
            return Self::default();
        };
        let left = read_kept(&directory).and_then(|kept| wait_left(kept.until, SystemTime::now()));
        if let Some(left) = left {
            log::info!(
                "Trade site: the wait of a refusal kept from the last launch, {} s left",
                left.as_secs()
            );
        }
        Self(Arc::new(Shared {
            until: Mutex::new(left.map(|left| Instant::now() + left)),
            directory: Some(directory),
        }))
    }

    /// The end of the wait, while it is after `now`.
    #[must_use]
    pub fn until(&self, now: Instant) -> Option<Instant> {
        // A thread that panicked wrote a whole time or none: the wait still holds.
        let until = *self.0.until.lock().unwrap_or_else(PoisonError::into_inner);
        until.filter(|&until| until > now)
    }

    /// Waits until `until` at least: a wait already longer stays.
    pub fn extend(&self, until: Instant) {
        let mut known = self.0.until.lock().unwrap_or_else(PoisonError::into_inner);
        if known.is_some_and(|known| known >= until) {
            return;
        }
        *known = Some(until);
        // Written before the other thread can extend the wait in turn: the file always
        // holds the longest one.
        if let Some(directory) = &self.0.directory {
            keep(directory, until);
        }
    }
}

/// The wait left at `clock` of a wait kept until `until`: none once it is over, nor when it
/// is longer than a refusal can ask, the time of a clock set back since, or of a file written
/// by hand.
fn wait_left(until: SystemTime, clock: SystemTime) -> Option<Duration> {
    // An error when `until` is before `clock`: the wait is over.
    let left = until.duration_since(clock).ok()?;
    if left > LONGEST_REFUSAL_WAIT {
        log::warn!(
            "Trade site: the wait of a refusal kept from the last launch would last {} s, \
             more than a refusal asks: ignored",
            left.as_secs()
        );
        return None;
    }
    Some(left)
}

/// What the file of `directory` keeps, if it has one that can be read.
fn read_kept(directory: &Path) -> Option<Kept> {
    let path = directory.join(KEPT_FILE);
    files::read_versioned_json(&path, KEPT_VERSION).unwrap_or_else(|error| {
        log::warn!(
            "{} can't be read, the wait of a refusal it may keep is ignored: {error}",
            logging::without_home(&path).display()
        );
        None
    })
}

/// Keeps the end `until` of the wait in `directory` for the next launch; a failure is logged,
/// and the wait then ends with the application.
fn keep(directory: &Path, until: Instant) {
    let left = until.saturating_duration_since(Instant::now());
    let written = SystemTime::now()
        .checked_add(left)
        .ok_or_else(|| io::Error::other("its end is beyond the clock of the system"))
        .and_then(|until| {
            files::write_versioned_json(directory, KEPT_FILE, KEPT_VERSION, &Kept { until })
        });
    if let Err(error) = written {
        log::warn!(
            "The wait of a refusal of the trade site couldn't be kept in {}: {error}",
            logging::without_home(directory).display()
        );
    }
}

#[cfg(test)]
pub(super) mod tests {
    use std::fs;

    use super::*;
    use crate::files::TestDirectory;

    /// The refusal of an application launched with the cache directory `directory`.
    pub fn launched_with(directory: &TestDirectory) -> Refusal {
        Refusal::kept_in(Some(directory.0.clone()))
    }

    /// `until` is `expected`, but for the moment between two readings of the clocks.
    pub fn assert_about(until: Option<Instant>, expected: Instant) {
        let moment = Duration::from_secs(1);
        assert!(
            until.is_some_and(|until| until + moment > expected && until < expected + moment),
            "{until:?}, expected about {expected:?}"
        );
    }

    #[test]
    fn the_clones_share_the_longest_wait() {
        let now = Instant::now();
        let refusal = Refusal::default();
        let other = refusal.clone();
        assert_eq!(other.until(now), None);

        refusal.extend(now + Duration::from_mins(30));
        other.extend(now + Duration::from_mins(5));

        assert_eq!(other.until(now), Some(now + Duration::from_mins(30)));
        assert_eq!(refusal.until(now + Duration::from_mins(30)), None);
    }

    #[test]
    fn a_wait_is_kept_for_the_next_launch() {
        let directory = TestDirectory::new("refusal-kept");
        let now = Instant::now();
        assert_eq!(launched_with(&directory).until(now), None);

        launched_with(&directory).extend(now + Duration::from_mins(30));

        let next_launch = launched_with(&directory);
        assert_about(next_launch.until(now), now + Duration::from_mins(30));
    }

    #[test]
    fn a_shorter_wait_doesnt_replace_the_wait_kept() {
        let directory = TestDirectory::new("refusal-shorter");
        let now = Instant::now();
        let refusal = launched_with(&directory);

        refusal.extend(now + Duration::from_mins(30));
        refusal.extend(now + Duration::from_mins(5));

        let next_launch = launched_with(&directory);
        assert_about(next_launch.until(now), now + Duration::from_mins(30));
    }

    #[test]
    fn a_wait_kept_is_extended_by_a_longer_one_only() {
        let directory = TestDirectory::new("refusal-extended");
        let now = Instant::now();
        launched_with(&directory).extend(now + Duration::from_mins(30));
        let refusal = launched_with(&directory);

        refusal.extend(now + Duration::from_mins(5));
        assert_about(refusal.until(now), now + Duration::from_mins(30));

        refusal.extend(now + Duration::from_hours(2));
        assert_eq!(refusal.until(now), Some(now + Duration::from_hours(2)));
        let next_launch = launched_with(&directory);
        assert_about(next_launch.until(now), now + Duration::from_hours(2));
    }

    /// A time of the clock of the system, as a launch reads it.
    fn clock() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_795_000_000)
    }

    #[test]
    fn a_wait_kept_lasts_what_is_left_of_it_at_the_launch() {
        let left = Duration::from_mins(5);

        assert_eq!(wait_left(clock() + left, clock()), Some(left));
    }

    #[test]
    fn a_wait_over_at_the_launch_is_ignored() {
        assert_eq!(wait_left(clock() - Duration::from_secs(1), clock()), None);
        assert_eq!(wait_left(SystemTime::UNIX_EPOCH, clock()), None);
    }

    #[test]
    fn a_wait_longer_than_a_refusal_asks_is_ignored() {
        let longest = clock() + LONGEST_REFUSAL_WAIT;

        assert_eq!(wait_left(longest, clock()), Some(LONGEST_REFUSAL_WAIT));
        assert_eq!(wait_left(longest + Duration::from_secs(1), clock()), None);
        assert_eq!(
            wait_left(clock() + Duration::from_hours(24 * 365), clock()),
            None
        );
    }

    #[test]
    fn a_file_that_cant_be_read_keeps_no_wait_and_is_written_again() {
        let directory = TestDirectory::new("refusal-unreadable");
        fs::create_dir_all(&directory.0).expect("a directory");
        for unreadable in [
            "",
            "not JSON",
            r#"{"version": 1, "value": {"until": "tomorrow"}}"#,
            r#"{"version": 1, "value": {"until": {"secs_since_epoch": 18446744073709551615, "nanos_since_epoch": 4294967295}}}"#,
        ] {
            fs::write(directory.0.join(KEPT_FILE), unreadable).expect("a file");
            let now = Instant::now();

            let refusal = launched_with(&directory);
            assert_eq!(refusal.until(now), None, "{unreadable}");

            refusal.extend(now + Duration::from_mins(30));
            let next_launch = launched_with(&directory);
            assert_about(next_launch.until(now), now + Duration::from_mins(30));
        }
    }

    #[test]
    fn a_wait_over_or_too_long_in_the_file_is_ignored_at_the_launch() {
        let directory = TestDirectory::new("refusal-ignored");
        let now = SystemTime::now();
        for until in [
            now - Duration::from_secs(1),
            now + LONGEST_REFUSAL_WAIT + Duration::from_mins(1),
        ] {
            files::write_versioned_json(&directory.0, KEPT_FILE, KEPT_VERSION, &Kept { until })
                .expect("a file");

            let refusal = launched_with(&directory);

            assert_eq!(refusal.until(Instant::now()), None, "{until:?}");
        }
    }
}
