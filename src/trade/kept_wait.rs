//! A wait before the next request to the trade site, whose end is kept in a file of the cache
//! directory: closing the application and launching it again doesn't end the wait, and the
//! file is read again before each request, so that two Exile Market that run at once, on the
//! same cache directory, share it too.

use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime};

use serde::{Deserialize, Serialize};

use super::limits::LONGEST_REFUSAL_WAIT;
use crate::files;
use crate::logging;

/// Changed with the shape of [`KeptEnd`]: a file of another version is ignored.
const VERSION: u32 = 1;

/// A wait as this Exile Market knows it, and the file that keeps its end.
#[derive(Debug)]
#[cfg_attr(test, derive(Default))]
pub(super) struct KeptWait {
    /// Its end, on the clock that never goes back.
    until: Option<Instant>,
    /// The end its file told when last read or written: a file that still tells it holds
    /// nothing new, and its time, read again on the other clock, would differ by a moment.
    end_in_file: Option<SystemTime>,
    /// Where its end is kept; without a cache directory, the wait ends with the application.
    file: Option<WaitFile>,
    /// The log already says that the file can't be read, until it reads again.
    told_unreadable: bool,
}

/// The file `name` of the cache directory `directory`, which keeps the end of a wait.
#[derive(Debug)]
struct WaitFile {
    directory: PathBuf,
    name: &'static str,
}

/// The end of a wait as its file keeps it: a time of the clock of the system, the only one
/// that goes on from one launch to the next, and that two Exile Market share.
#[derive(Debug, Serialize, Deserialize)]
struct KeptEnd {
    until: SystemTime,
}

impl KeptWait {
    /// The wait kept in the file `name` of `directory`, when it isn't over, and every wait
    /// to come kept there in turn. Without a directory, a wait ends with the application.
    pub fn read_from(directory: Option<PathBuf>, name: &'static str) -> Self {
        let mut wait = Self {
            until: None,
            end_in_file: None,
            file: directory.map(|directory| WaitFile { directory, name }),
            told_unreadable: false,
        };
        // Read at the launch, so that the log tells a wait under way from its first lines.
        wait.read_again();
        wait
    }

    /// The end of the wait this Exile Market knows, while it is after `now`: what a request
    /// would wait for, told without reading the file.
    pub fn until(&self, now: Instant) -> Option<Instant> {
        self.until.filter(|&until| until > now)
    }

    /// The end of the wait, while it is after `now`, its file read again first: asked before
    /// each request, since another Exile Market may have written a wait meanwhile.
    pub fn until_read_again(&mut self, now: Instant) -> Option<Instant> {
        self.read_again();
        self.until(now)
    }

    /// Waits until `until` at least, and keeps that end in the file: a wait already longer
    /// stays, in memory and in the file, whichever Exile Market wrote it.
    pub fn extend(&mut self, until: Instant) {
        self.read_again();
        if self.until.is_some_and(|known| known >= until) {
            return;
        }
        self.until = Some(until);
        if let Some(written) = self.file.as_ref().and_then(|file| file.keep(until)) {
            self.end_in_file = Some(written);
        }
    }

    /// Takes in the wait of the file when it changed and is the longest of the two: a file
    /// that tells a shorter wait, or none, ends nothing.
    fn read_again(&mut self) {
        let Some(file) = &self.file else { return };
        let kept = match file.read() {
            Ok(kept) => {
                self.told_unreadable = false;
                kept
            }
            Err(error) => {
                // Once: every request would say the same, until a wait is written again.
                if !self.told_unreadable {
                    log::warn!(
                        "{} can't be read, the wait it may keep is ignored: {error}",
                        logging::without_home(&file.path()).display()
                    );
                    self.told_unreadable = true;
                }
                return;
            }
        };
        let Some(kept) = kept else { return };
        if self.end_in_file == Some(kept) {
            return;
        }
        self.end_in_file = Some(kept);
        let left = match wait_left(kept, SystemTime::now()) {
            WaitLeft::For(left) => left,
            WaitLeft::Over => return,
            WaitLeft::TooLong(left) => {
                log::warn!(
                    "Trade site: the wait kept in {} would last {} s, more than a refusal \
                     asks: ignored",
                    file.name,
                    left.as_secs()
                );
                return;
            }
        };
        let until = Instant::now() + left;
        if self.until.is_none_or(|known| until > known) {
            log::info!(
                "Trade site: the wait kept in {}, {} s left",
                file.name,
                left.as_secs()
            );
            self.until = Some(until);
        }
    }
}

impl WaitFile {
    /// Where the file is.
    fn path(&self) -> PathBuf {
        self.directory.join(self.name)
    }

    /// The end of the wait the file keeps, if there is a file of this version; an error when
    /// it can't be read.
    fn read(&self) -> io::Result<Option<SystemTime>> {
        let kept: Option<KeptEnd> = files::read_versioned_json(&self.path(), VERSION)?;
        Ok(kept.map(|kept| kept.until))
    }

    /// Keeps the end `until` of the wait, and gives it as the file tells it; a failure is
    /// logged, and the wait then ends with the application.
    fn keep(&self, until: Instant) -> Option<SystemTime> {
        let left = until.saturating_duration_since(Instant::now());
        let written = SystemTime::now()
            .checked_add(left)
            .ok_or_else(|| io::Error::other("its end is beyond the clock of the system"))
            .and_then(|until| {
                files::write_versioned_json(&self.directory, self.name, VERSION, &KeptEnd { until })
                    .map(|()| until)
            });
        written
            .inspect_err(|error| {
                log::warn!(
                    "The wait of the trade site couldn't be kept in {}: {error}",
                    logging::without_home(&self.path()).display()
                );
            })
            .ok()
    }
}

/// What is left of a wait kept in a file.
#[derive(Debug, PartialEq, Eq)]
enum WaitLeft {
    /// Nothing: its end is past.
    Over,
    /// This long.
    For(Duration),
    /// This long, more than a refusal can ask: the time of a clock set back since, or of a
    /// file written by hand, which is ignored.
    TooLong(Duration),
}

/// What is left at `clock` of a wait kept until `until`.
fn wait_left(until: SystemTime, clock: SystemTime) -> WaitLeft {
    // An error when `until` is before `clock`: the wait is over.
    match until.duration_since(clock) {
        Err(_) => WaitLeft::Over,
        Ok(left) if left > LONGEST_REFUSAL_WAIT => WaitLeft::TooLong(left),
        Ok(left) => WaitLeft::For(left),
    }
}

#[cfg(test)]
pub(super) mod tests {
    use std::fs;

    use super::*;
    use crate::files::TestDirectory;

    /// The file of the waits of these tests.
    const FILE: &str = "wait.json";

    /// The wait of an Exile Market launched with the cache directory `directory`.
    fn launched_with(directory: &TestDirectory) -> KeptWait {
        KeptWait::read_from(Some(directory.0.clone()), FILE)
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
    fn a_wait_is_kept_for_the_next_launch() {
        let directory = TestDirectory::new("wait-kept");
        let now = Instant::now();
        assert_eq!(launched_with(&directory).until(now), None);

        launched_with(&directory).extend(now + Duration::from_mins(30));

        let next_launch = launched_with(&directory);
        assert_about(next_launch.until(now), now + Duration::from_mins(30));
    }

    #[test]
    fn a_shorter_wait_doesnt_replace_the_wait_kept() {
        let directory = TestDirectory::new("wait-shorter");
        let now = Instant::now();
        let mut wait = launched_with(&directory);

        wait.extend(now + Duration::from_mins(30));
        wait.extend(now + Duration::from_mins(5));

        assert_eq!(wait.until(now), Some(now + Duration::from_mins(30)));
        let next_launch = launched_with(&directory);
        assert_about(next_launch.until(now), now + Duration::from_mins(30));
    }

    #[test]
    fn a_wait_kept_is_extended_by_a_longer_one_only() {
        let directory = TestDirectory::new("wait-extended");
        let now = Instant::now();
        launched_with(&directory).extend(now + Duration::from_mins(30));
        let mut wait = launched_with(&directory);

        wait.extend(now + Duration::from_mins(5));
        assert_about(wait.until(now), now + Duration::from_mins(30));

        wait.extend(now + Duration::from_hours(2));
        assert_eq!(wait.until(now), Some(now + Duration::from_hours(2)));
        let next_launch = launched_with(&directory);
        assert_about(next_launch.until(now), now + Duration::from_hours(2));
    }

    #[test]
    fn a_wait_that_cant_be_kept_still_holds_until_the_application_closes() {
        let directory = TestDirectory::new("wait-unwritable");
        fs::create_dir_all(&directory.0).expect("a directory");
        // A file has the name of the cache directory: nothing can be written in it.
        let blocked = directory.0.join("blocked");
        fs::write(&blocked, "").expect("a file");
        let now = Instant::now();

        for mut wait in [
            KeptWait::read_from(Some(blocked.clone()), FILE),
            KeptWait::read_from(None, FILE),
        ] {
            assert_eq!(wait.until(now), None);

            wait.extend(now + Duration::from_mins(30));

            assert_eq!(
                wait.until_read_again(now),
                Some(now + Duration::from_mins(30))
            );
        }
        // Nothing was kept: the file is still a file, with no directory in its place.
        assert!(blocked.is_file());
    }

    /// The security audit of v0.2.1 (2026-11-21): two Exile Market at once, each launched
    /// before the refusal of the other, read the file at their launch only.
    #[test]
    fn the_wait_of_another_exile_market_is_read_again_before_a_request() {
        let directory = TestDirectory::new("wait-other");
        let now = Instant::now();
        let mut wait = launched_with(&directory);
        let mut other = launched_with(&directory);

        other.extend(now + Duration::from_mins(30));

        assert_about(wait.until_read_again(now), now + Duration::from_mins(30));
        // Known from then on, as a wait of its own.
        assert_about(wait.until(now), now + Duration::from_mins(30));
    }

    #[test]
    fn a_shorter_wait_doesnt_replace_the_wait_of_another_exile_market() {
        let directory = TestDirectory::new("wait-other-shorter");
        let now = Instant::now();
        let mut wait = launched_with(&directory);
        let mut other = launched_with(&directory);
        other.extend(now + Duration::from_mins(30));

        wait.extend(now + Duration::from_mins(5));

        assert_about(wait.until(now), now + Duration::from_mins(30));
        let next_launch = launched_with(&directory);
        assert_about(next_launch.until(now), now + Duration::from_mins(30));
    }

    #[test]
    fn a_longer_wait_replaces_the_wait_of_another_exile_market() {
        let directory = TestDirectory::new("wait-other-longer");
        let now = Instant::now();
        let mut wait = launched_with(&directory);
        let mut other = launched_with(&directory);
        other.extend(now + Duration::from_mins(5));

        wait.extend(now + Duration::from_mins(30));

        assert_about(other.until_read_again(now), now + Duration::from_mins(30));
    }

    /// Its time, read from the file on the other clock, is a moment later each time.
    #[test]
    fn the_file_it_wrote_doesnt_move_the_wait_known() {
        let directory = TestDirectory::new("wait-own-file");
        let now = Instant::now();
        let until = now + Duration::from_mins(30);
        let mut wait = launched_with(&directory);

        wait.extend(until);

        assert_eq!(wait.until_read_again(now), Some(until));
        assert_eq!(wait.until_read_again(now), Some(until));
    }

    /// Writes in `directory` the file of a wait that ends in `left`, whatever it kept: as an
    /// Exile Market of a version that didn't read the file before writing it.
    fn overwrite_file(directory: &TestDirectory, left: Duration) {
        let until = SystemTime::now() + left;
        files::write_versioned_json(&directory.0, FILE, VERSION, &KeptEnd { until })
            .expect("a file");
    }

    #[test]
    fn a_file_that_tells_a_shorter_wait_or_none_ends_no_wait_known() {
        let directory = TestDirectory::new("wait-file-shorter");
        let now = Instant::now();
        let until = now + Duration::from_mins(30);
        let mut wait = launched_with(&directory);
        wait.extend(until);

        overwrite_file(&directory, Duration::from_mins(5));
        assert_eq!(wait.until_read_again(now), Some(until));

        fs::remove_file(directory.0.join(FILE)).expect("a file to remove");
        assert_eq!(wait.until_read_again(now), Some(until));

        fs::write(directory.0.join(FILE), "not JSON").expect("a file");
        assert_eq!(wait.until_read_again(now), Some(until));
    }

    /// A wait written, then a longer one, then the first again: each change of the file is
    /// read, and the file read last doesn't hide the one before.
    #[test]
    fn each_change_of_the_file_is_read_and_the_longest_wait_stays() {
        let directory = TestDirectory::new("wait-file-changes");
        let now = Instant::now();
        let mut wait = launched_with(&directory);

        overwrite_file(&directory, Duration::from_mins(5));
        assert_about(wait.until_read_again(now), now + Duration::from_mins(5));
        overwrite_file(&directory, Duration::from_hours(2));
        assert_about(wait.until_read_again(now), now + Duration::from_hours(2));
        overwrite_file(&directory, Duration::from_mins(5));
        assert_about(wait.until_read_again(now), now + Duration::from_hours(2));
    }

    /// The time of a clock set back, or of a file written by hand, in the file of another
    /// Exile Market: ignored as at the launch, and the wait it writes next is read.
    #[test]
    fn a_wait_too_long_in_the_file_is_ignored_and_the_next_one_is_read() {
        let directory = TestDirectory::new("wait-file-too-long");
        let now = Instant::now();
        let mut wait = launched_with(&directory);

        overwrite_file(&directory, LONGEST_REFUSAL_WAIT + Duration::from_mins(1));
        assert_eq!(wait.until_read_again(now), None);
        overwrite_file(&directory, Duration::from_mins(30));
        assert_about(wait.until_read_again(now), now + Duration::from_mins(30));
    }

    /// Writes the file `name` of `version` a launch left in `directory`, as the version 1
    /// writes it, with a wait that ends in 30 minutes.
    pub fn leave_file(directory: &TestDirectory, name: &str, version: u32) {
        let until = (SystemTime::now() + Duration::from_mins(30))
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("after 1970");
        let json = format!(
            r#"{{"version":{version},"value":{{"until":{{"secs_since_epoch":{},"nanos_since_epoch":{}}}}}}}"#,
            until.as_secs(),
            until.subsec_nanos()
        );
        fs::create_dir_all(&directory.0).expect("a directory");
        fs::write(directory.0.join(name), json).expect("a file");
    }

    /// A change of the shape of the file without a new version would lose the wait under way
    /// at an update of the application.
    #[test]
    fn the_file_of_the_version_is_read_as_a_launch_wrote_it() {
        let directory = TestDirectory::new("wait-version");
        let now = Instant::now();
        leave_file(&directory, FILE, VERSION);

        let wait = launched_with(&directory);

        assert_about(wait.until(now), now + Duration::from_mins(30));
    }

    #[test]
    fn the_file_of_another_version_keeps_no_wait() {
        let directory = TestDirectory::new("wait-other-version");
        leave_file(&directory, FILE, VERSION + 1);

        assert_eq!(launched_with(&directory).until(Instant::now()), None);
    }

    /// A time of the clock of the system, as a launch reads it.
    fn clock() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_795_000_000)
    }

    #[test]
    fn a_wait_kept_lasts_what_is_left_of_it_at_the_launch() {
        let left = Duration::from_mins(5);

        assert_eq!(wait_left(clock() + left, clock()), WaitLeft::For(left));
    }

    #[test]
    fn a_wait_whose_end_is_past_is_told_over() {
        assert_eq!(
            wait_left(clock() - Duration::from_secs(1), clock()),
            WaitLeft::Over
        );
        assert_eq!(wait_left(SystemTime::UNIX_EPOCH, clock()), WaitLeft::Over);
    }

    #[test]
    fn a_wait_longer_than_a_refusal_asks_is_told_too_long() {
        let longest = clock() + LONGEST_REFUSAL_WAIT;

        let second = Duration::from_secs(1);
        let year = Duration::from_hours(24 * 365);

        assert_eq!(
            wait_left(longest, clock()),
            WaitLeft::For(LONGEST_REFUSAL_WAIT)
        );
        assert_eq!(
            wait_left(longest + second, clock()),
            WaitLeft::TooLong(LONGEST_REFUSAL_WAIT + second)
        );
        assert_eq!(wait_left(clock() + year, clock()), WaitLeft::TooLong(year));
    }

    #[test]
    fn a_file_that_cant_be_read_keeps_no_wait_and_is_written_again() {
        let directory = TestDirectory::new("wait-unreadable");
        fs::create_dir_all(&directory.0).expect("a directory");
        for unreadable in [
            "",
            "not JSON",
            r#"{"version": 1, "value": {"until": "tomorrow"}}"#,
            r#"{"version": 1, "value": {"until": {"secs_since_epoch": 18446744073709551615, "nanos_since_epoch": 4294967295}}}"#,
        ] {
            fs::write(directory.0.join(FILE), unreadable).expect("a file");
            let now = Instant::now();

            let mut wait = launched_with(&directory);
            assert_eq!(wait.until(now), None, "{unreadable}");

            wait.extend(now + Duration::from_mins(30));
            let next_launch = launched_with(&directory);
            assert_about(next_launch.until(now), now + Duration::from_mins(30));
        }
    }

    /// A file that can't be read stays so until a refusal writes it again: without the
    /// mark, each request would write the same line to the log.
    #[test]
    fn a_file_that_cant_be_read_is_told_once_and_again_after_it_was_read() {
        let directory = TestDirectory::new("wait-unreadable-told");
        fs::create_dir_all(&directory.0).expect("a directory");
        fs::write(directory.0.join(FILE), "not JSON").expect("a file");
        let now = Instant::now();

        let mut wait = launched_with(&directory);
        assert!(wait.told_unreadable);
        assert_eq!(wait.until_read_again(now), None);
        assert!(wait.told_unreadable, "still unreadable, already told");

        wait.extend(now + Duration::from_mins(30));
        assert_eq!(
            wait.until_read_again(now),
            Some(now + Duration::from_mins(30))
        );
        assert!(!wait.told_unreadable, "read again: the next time is told");

        let break_file = || fs::write(directory.0.join(FILE), "not JSON").expect("a file");
        break_file();
        assert_eq!(
            wait.until_read_again(now),
            Some(now + Duration::from_mins(30))
        );
        assert!(wait.told_unreadable);

        // A file gone is read too: one that can't be read after it is a new event.
        fs::remove_file(directory.0.join(FILE)).expect("a file to remove");
        assert_eq!(
            wait.until_read_again(now),
            Some(now + Duration::from_mins(30))
        );
        assert!(!wait.told_unreadable, "no file: the next time is told");
        break_file();
        assert_eq!(
            wait.until_read_again(now),
            Some(now + Duration::from_mins(30))
        );
        assert!(wait.told_unreadable);
    }

    #[test]
    fn a_wait_over_or_too_long_in_the_file_is_ignored_at_the_launch() {
        let directory = TestDirectory::new("wait-ignored");
        let now = SystemTime::now();
        for until in [
            now - Duration::from_secs(1),
            now + LONGEST_REFUSAL_WAIT + Duration::from_mins(1),
        ] {
            files::write_versioned_json(&directory.0, FILE, VERSION, &KeptEnd { until })
                .expect("a file");

            let wait = launched_with(&directory);

            assert_eq!(wait.until(Instant::now()), None, "{until:?}");
        }
    }
}
