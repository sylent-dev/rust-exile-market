//! The wait the trade site asked for by refusing a request, shared by the two threads that
//! call it: a search refused makes the download of the item names wait, and the other way
//! round, since both reach the same site from the same address. Its end is kept in the cache
//! directory and read there again before each request ([`KeptWait`]): the next launch and
//! another Exile Market wait too.

use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Instant;

use super::kept_wait::KeptWait;

/// The file of the end of the wait in the cache directory.
const KEPT_FILE: &str = "trade-refusal.json";

/// The time before which no request is sent to the trade site, if it refused one. Its clones
/// share the same time. Outside the tests, only [`Refusal::kept_in`] builds one, so that the
/// application can't be handed a wait that forgets its cache directory.
///
/// Behind a mutex rather than sent through a channel: each thread reads it before each of
/// its requests, and a refusal of one must stop the next request of the other.
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(Default))]
pub struct Refusal(Arc<Mutex<KeptWait>>);

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
        let wait = KeptWait::read_from(directory, KEPT_FILE);
        Self(Arc::new(Mutex::new(wait)))
    }

    /// The end of the wait this Exile Market knows, while it is after `now`: what a request
    /// would wait for, told without reading the file.
    #[must_use]
    pub fn until(&self, now: Instant) -> Option<Instant> {
        self.wait().until(now)
    }

    /// The end of the wait, while it is after `now`, its file read again first: asked before
    /// each request, since another Exile Market may have been refused meanwhile.
    #[must_use]
    pub fn until_read_again(&self, now: Instant) -> Option<Instant> {
        self.wait().until_read_again(now)
    }

    /// Waits until `until` at least, and keeps that end for the next launch: a wait already
    /// longer stays, in memory and in its file, whichever Exile Market wrote it.
    pub fn extend(&self, until: Instant) {
        // The file is written under the lock, before the other thread can extend the wait in
        // turn: of the waits of this Exile Market, it holds the longest.
        self.wait().extend(until);
    }

    /// The wait, for one thread at a time.
    fn wait(&self) -> MutexGuard<'_, KeptWait> {
        // A thread that panicked wrote a whole wait or none: the wait still holds.
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
pub(super) mod tests {
    use std::time::Duration;

    use super::*;
    use crate::files::TestDirectory;
    use crate::trade::kept_wait::tests::{assert_about, leave_file};

    /// The refusal of an application launched with the cache directory `directory`.
    pub fn launched_with(directory: &TestDirectory) -> Refusal {
        Refusal::kept_in(Some(directory.0.clone()))
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

    /// Two Exile Market on one cache directory: two processes, which share the file alone.
    #[test]
    fn a_refusal_is_kept_for_the_next_launch_and_for_another_exile_market() {
        let directory = TestDirectory::new("refusal-kept");
        let now = Instant::now();
        let refusal = launched_with(&directory);
        let other = launched_with(&directory);

        refusal.extend(now + Duration::from_mins(30));

        assert_about(other.until_read_again(now), now + Duration::from_mins(30));
        let next_launch = launched_with(&directory);
        assert_about(next_launch.until(now), now + Duration::from_mins(30));
    }

    /// The file as `specs.md` names it and as the version 0.2.1 wrote it: renamed, the wait
    /// under way would be lost at an update of the application.
    #[test]
    fn the_wait_of_a_refusal_is_read_from_the_file_of_its_name() {
        let directory = TestDirectory::new("refusal-file-name");
        let now = Instant::now();
        leave_file(&directory, "trade-refusal.json", 1);

        let refusal = launched_with(&directory);

        assert_about(refusal.until(now), now + Duration::from_mins(30));
    }
}
