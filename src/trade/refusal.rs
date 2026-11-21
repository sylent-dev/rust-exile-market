//! The wait the trade site asked for by refusing a request, shared by the two threads that
//! call it: a search refused makes the download of the item names wait, and the other way
//! round, since both reach the same site from the same address. Its end is kept in the cache
//! directory, so that closing the application and launching it again doesn't end the wait, and
//! read there again before each request, so that two Exile Market that run at once, on the
//! same cache directory, share it too.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant, SystemTime};

use serde::{Deserialize, Serialize};

use super::limits::LONGEST_REFUSAL_WAIT;
use crate::files;
use crate::logging;

/// The file of the end of the wait in the cache directory.
const KEPT_FILE: &str = "trade-refusal.json";
/// Changed with the shape of [`KeptWait`]: a file of another version is ignored.
const KEPT_VERSION: u32 = 1;

/// The time before which no request is sent to the trade site, if it refused one. Its clones
/// share the same time. Outside the tests, only [`Refusal::kept_in`] builds one, so that the
/// application can't be handed a wait that forgets its cache directory.
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(Default))]
pub struct Refusal(Arc<SharedWait>);

/// What the clones of a [`Refusal`] share.
#[derive(Debug)]
#[cfg_attr(test, derive(Default))]
struct SharedWait {
    /// Behind a mutex rather than sent through a channel: each thread reads it before each
    /// of its requests, and a refusal of one must stop the next request of the other.
    wait: Mutex<Wait>,
    /// The cache directory, where the end of the wait is kept for the next launch and for
    /// another Exile Market; without one, the wait ends with the application.
    directory: Option<PathBuf>,
}

/// The wait as this Exile Market knows it.
#[derive(Debug, Default)]
struct Wait {
    /// Its end, on the clock that never goes back.
    until: Option<Instant>,
    /// The end its file told when last read or written: a file that still tells it holds
    /// nothing new, and its time, read again on the other clock, would differ by a moment.
    in_file: Option<SystemTime>,
}

/// The end of a wait as its file keeps it: a time of the clock of the system, the only one
/// that goes on from one launch to the next, and that two Exile Market share.
#[derive(Debug, Serialize, Deserialize)]
struct KeptWait {
    until: SystemTime,
}

impl Refusal {
    /// The wait kept in `directory`, when it isn't over, and every wait to come kept there
    /// in turn. Without a directory, a wait ends with the application.
    #[must_use]
    pub fn kept_in(directory: Option<PathBuf>) -> Self {
        if directory.is_none() {
            log::warn!(
                "No cache directory: the wait of a refusal of the trade site will end with \
                 the application"
            );
        }
        let shared = SharedWait {
            wait: Mutex::default(),
            directory,
        };
        // Read at the launch, so that the log tells a wait under way from its first lines.
        drop(shared.read_again());
        Self(Arc::new(shared))
    }

    /// The end of the wait this Exile Market knows, while it is after `now`: what a request
    /// would wait for, told without reading the file.
    #[must_use]
    pub fn until(&self, now: Instant) -> Option<Instant> {
        self.0.known().until.filter(|&until| until > now)
    }

    /// The end of the wait, while it is after `now`, its file read again first: asked before
    /// each request, since another Exile Market may have been refused meanwhile.
    #[must_use]
    pub fn until_read_again(&self, now: Instant) -> Option<Instant> {
        self.0.read_again().until.filter(|&until| until > now)
    }

    /// Waits until `until` at least, and keeps that end for the next launch: a wait already
    /// longer stays, in memory and in its file, whichever Exile Market wrote it.
    pub fn extend(&self, until: Instant) {
        let mut wait = self.0.read_again();
        if wait.until.is_some_and(|known| known >= until) {
            return;
        }
        wait.until = Some(until);
        // Written before the other thread can extend the wait in turn: the file always
        // holds the longest one.
        if let Some(directory) = &self.0.directory
            && let Some(written) = keep(directory, until)
        {
            wait.in_file = Some(written);
        }
    }
}

impl SharedWait {
    /// The wait this Exile Market knows.
    fn known(&self) -> MutexGuard<'_, Wait> {
        // A thread that panicked wrote a whole wait or none: the wait still holds.
        self.wait.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The wait known once its file is read again, the longest of the two: a file that tells
    /// a shorter wait, or none, ends nothing.
    fn read_again(&self) -> MutexGuard<'_, Wait> {
        let mut wait = self.known();
        let kept = self.directory.as_deref().and_then(read_kept);
        if let Some(KeptWait { until: kept }) = kept
            && wait.in_file != Some(kept)
        {
            wait.in_file = Some(kept);
            let until = wait_left(kept, SystemTime::now()).map(|left| Instant::now() + left);
            if let Some(until) = until
                && wait.until.is_none_or(|known| until > known)
            {
                log::info!(
                    "Trade site: the wait of a refusal read from its file, {} s left",
                    until.saturating_duration_since(Instant::now()).as_secs()
                );
                wait.until = Some(until);
            }
        }
        wait
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
            "Trade site: the wait of a refusal read from its file would last {} s, more than \
             a refusal asks: ignored",
            left.as_secs()
        );
        return None;
    }
    Some(left)
}

/// What the file of `directory` keeps, if it has one that can be read.
fn read_kept(directory: &Path) -> Option<KeptWait> {
    let path = directory.join(KEPT_FILE);
    files::read_versioned_json(&path, KEPT_VERSION).unwrap_or_else(|error| {
        log::warn!(
            "{} can't be read, the wait of a refusal it may keep is ignored: {error}",
            logging::without_home(&path).display()
        );
        None
    })
}

/// Keeps the end `until` of the wait in `directory`, and gives it as its file tells it; a
/// failure is logged, and the wait then ends with the application.
fn keep(directory: &Path, until: Instant) -> Option<SystemTime> {
    let left = until.saturating_duration_since(Instant::now());
    let written = SystemTime::now()
        .checked_add(left)
        .ok_or_else(|| io::Error::other("its end is beyond the clock of the system"))
        .and_then(|until| {
            files::write_versioned_json(directory, KEPT_FILE, KEPT_VERSION, &KeptWait { until })
                .map(|()| until)
        });
    written
        .inspect_err(|error| {
            log::warn!(
                "The wait of a refusal of the trade site couldn't be kept in {}: {error}",
                logging::without_home(directory).display()
            );
        })
        .ok()
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

    #[test]
    fn a_wait_that_cant_be_kept_still_holds_until_the_application_closes() {
        let directory = TestDirectory::new("refusal-unwritable");
        fs::create_dir_all(&directory.0).expect("a directory");
        // A file has the name of the cache directory: nothing can be written in it.
        let blocked = directory.0.join("blocked");
        fs::write(&blocked, "").expect("a file");
        let now = Instant::now();

        for refusal in [
            Refusal::kept_in(Some(blocked.clone())),
            Refusal::kept_in(None),
        ] {
            assert_eq!(refusal.until(now), None);

            refusal.extend(now + Duration::from_mins(30));

            assert_eq!(refusal.until(now), Some(now + Duration::from_mins(30)));
        }
        // Nothing was kept: the file is still a file, with no directory in its place.
        assert!(blocked.is_file());
    }

    /// Writes in `directory` the file of a wait that ends in `left`, whatever it kept: as an
    /// Exile Market of a version that didn't read the file before writing it.
    fn overwrite_file(directory: &TestDirectory, left: Duration) {
        let until = SystemTime::now() + left;
        files::write_versioned_json(&directory.0, KEPT_FILE, KEPT_VERSION, &KeptWait { until })
            .expect("a file");
    }

    /// The security audit of v0.2.1 (2026-11-21): two Exile Market at once, each launched
    /// before the refusal of the other, read the file at their launch only.
    #[test]
    fn the_wait_of_another_exile_market_is_read_again_before_a_request() {
        let directory = TestDirectory::new("refusal-other");
        let now = Instant::now();
        let refusal = launched_with(&directory);
        let other = launched_with(&directory);

        other.extend(now + Duration::from_mins(30));

        assert_about(refusal.until_read_again(now), now + Duration::from_mins(30));
        // Known from then on, as a wait of its own.
        assert_about(refusal.until(now), now + Duration::from_mins(30));
    }

    #[test]
    fn a_shorter_wait_doesnt_replace_the_wait_of_another_exile_market() {
        let directory = TestDirectory::new("refusal-other-shorter");
        let now = Instant::now();
        let refusal = launched_with(&directory);
        let other = launched_with(&directory);
        other.extend(now + Duration::from_mins(30));

        refusal.extend(now + Duration::from_mins(5));

        assert_about(refusal.until(now), now + Duration::from_mins(30));
        let next_launch = launched_with(&directory);
        assert_about(next_launch.until(now), now + Duration::from_mins(30));
    }

    #[test]
    fn a_longer_wait_replaces_the_wait_of_another_exile_market() {
        let directory = TestDirectory::new("refusal-other-longer");
        let now = Instant::now();
        let refusal = launched_with(&directory);
        let other = launched_with(&directory);
        other.extend(now + Duration::from_mins(5));

        refusal.extend(now + Duration::from_mins(30));

        assert_about(other.until_read_again(now), now + Duration::from_mins(30));
    }

    /// Its time, read from the file on the other clock, is a moment later each time.
    #[test]
    fn the_file_it_wrote_doesnt_move_the_wait_known() {
        let directory = TestDirectory::new("refusal-own-file");
        let now = Instant::now();
        let until = now + Duration::from_mins(30);
        let refusal = launched_with(&directory);

        refusal.extend(until);

        assert_eq!(refusal.until_read_again(now), Some(until));
        assert_eq!(refusal.until_read_again(now), Some(until));
    }

    #[test]
    fn a_file_that_tells_a_shorter_wait_or_none_ends_no_wait_known() {
        let directory = TestDirectory::new("refusal-file-shorter");
        let now = Instant::now();
        let until = now + Duration::from_mins(30);
        let refusal = launched_with(&directory);
        refusal.extend(until);

        overwrite_file(&directory, Duration::from_mins(5));
        assert_eq!(refusal.until_read_again(now), Some(until));

        fs::remove_file(directory.0.join(KEPT_FILE)).expect("a file to remove");
        assert_eq!(refusal.until_read_again(now), Some(until));

        fs::write(directory.0.join(KEPT_FILE), "not JSON").expect("a file");
        assert_eq!(refusal.until_read_again(now), Some(until));
    }

    /// A wait written, then a longer one, then the first again: each change of the file is
    /// read, and the file read last doesn't hide the one before.
    #[test]
    fn each_change_of_the_file_is_read_and_the_longest_wait_stays() {
        let directory = TestDirectory::new("refusal-file-changes");
        let now = Instant::now();
        let refusal = launched_with(&directory);

        overwrite_file(&directory, Duration::from_mins(5));
        assert_about(refusal.until_read_again(now), now + Duration::from_mins(5));
        overwrite_file(&directory, Duration::from_hours(2));
        assert_about(refusal.until_read_again(now), now + Duration::from_hours(2));
        overwrite_file(&directory, Duration::from_mins(5));
        assert_about(refusal.until_read_again(now), now + Duration::from_hours(2));
    }

    /// The time of a clock set back, or of a file written by hand, in the file of another
    /// Exile Market: ignored as at the launch, and the wait it writes next is read.
    #[test]
    fn a_wait_too_long_in_the_file_is_ignored_and_the_next_one_is_read() {
        let directory = TestDirectory::new("refusal-file-too-long");
        let now = Instant::now();
        let refusal = launched_with(&directory);

        overwrite_file(&directory, LONGEST_REFUSAL_WAIT + Duration::from_mins(1));
        assert_eq!(refusal.until_read_again(now), None);
        overwrite_file(&directory, Duration::from_mins(30));
        assert_about(refusal.until_read_again(now), now + Duration::from_mins(30));
    }

    /// Writes the file of `version` a launch left in `directory`, as the version 1 writes it,
    /// with a wait that ends in 30 minutes.
    fn leave_file(directory: &TestDirectory, version: u32) {
        let until = (SystemTime::now() + Duration::from_mins(30))
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("after 1970");
        let json = format!(
            r#"{{"version":{version},"value":{{"until":{{"secs_since_epoch":{},"nanos_since_epoch":{}}}}}}}"#,
            until.as_secs(),
            until.subsec_nanos()
        );
        fs::create_dir_all(&directory.0).expect("a directory");
        fs::write(directory.0.join(KEPT_FILE), json).expect("a file");
    }

    /// A change of the shape of the file without a new version would lose the wait under way
    /// at an update of the application.
    #[test]
    fn the_file_of_the_version_is_read_as_a_launch_wrote_it() {
        let directory = TestDirectory::new("refusal-version");
        let now = Instant::now();
        leave_file(&directory, KEPT_VERSION);

        let refusal = launched_with(&directory);

        assert_about(refusal.until(now), now + Duration::from_mins(30));
    }

    #[test]
    fn the_file_of_another_version_keeps_no_wait() {
        let directory = TestDirectory::new("refusal-other-version");
        leave_file(&directory, KEPT_VERSION + 1);

        assert_eq!(launched_with(&directory).until(Instant::now()), None);
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
            files::write_versioned_json(&directory.0, KEPT_FILE, KEPT_VERSION, &KeptWait { until })
                .expect("a file");

            let refusal = launched_with(&directory);

            assert_eq!(refusal.until(Instant::now()), None, "{until:?}");
        }
    }
}
