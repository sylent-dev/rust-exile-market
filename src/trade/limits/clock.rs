//! The time the limits count in: the clock of the system, read once when the limits are
//! built, then moved on by the clock that never goes back, so that it never goes back either.
//!
//! An [`Instant`] alone can't tell a time before the start of the system on Windows, where it
//! is counted from that start (`Instant { t: Duration }` read from `QueryPerformanceCounter`,
//! whose `checked_sub` gives `None`; `library/std/src/sys/pal/windows/time.rs` of Rust 1.95,
//! read on 2026-11-22): a request sent before a restart couldn't be told as one. A [`Time`]
//! can, down to 1970.

use std::ops::Add;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// A time of the [`Clock`] of the limits, counted from 1970 (the Unix epoch).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Time(Duration);

impl Time {
    /// The time `period` earlier; `None` before 1970.
    pub fn checked_sub(self, period: Duration) -> Option<Self> {
        self.0.checked_sub(period).map(Self)
    }

    /// How long this time is after `earlier`; nothing when it is before.
    pub fn saturating_duration_since(self, earlier: Self) -> Duration {
        self.0.saturating_sub(earlier.0)
    }
}

impl Add<Duration> for Time {
    type Output = Self;

    /// The time `period` later, the last time that can be told at most: a wrong number
    /// can't overflow it.
    fn add(self, period: Duration) -> Self {
        Self(self.0.saturating_add(period))
    }
}

/// The clock of the limits: the time the clock of the system told at its start, moved on
/// since by the clock that never goes back.
#[derive(Debug, Clone, Copy)]
pub(super) struct Clock {
    /// Its start, on the clock that never goes back…
    start: Instant,
    /// …and the time the clock of the system told then.
    started_at: Time,
}

impl Clock {
    /// The clock that starts now, at the time the clock of the system tells.
    pub fn starting_now() -> Self {
        Self::starting(Instant::now(), SystemTime::now())
    }

    /// The clock that starts at `start`, when the clock of the system told `system`.
    fn starting(start: Instant, system: SystemTime) -> Self {
        // A clock of the system set before 1970 counts from 1970: the times after the start
        // stay in order, which is all one launch needs.
        let since_epoch = system.duration_since(UNIX_EPOCH).unwrap_or_default();
        Self {
            start,
            started_at: Time(since_epoch),
        }
    }

    /// The time of `instant`, before the start of the clock too.
    pub fn time_of(&self, instant: Instant) -> Time {
        // One of the two is nothing: an `Instant` gives no negative time.
        let after = instant.saturating_duration_since(self.start);
        let before = self.start.saturating_duration_since(instant);
        Time(
            self.started_at
                .0
                .saturating_add(after)
                .saturating_sub(before),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECOND: Duration = Duration::from_secs(1);

    /// A clock started at `start`, a billion seconds after 1970.
    fn clock(start: Instant) -> Clock {
        Clock::starting(start, UNIX_EPOCH + 1_000_000_000 * SECOND)
    }

    #[test]
    fn a_time_moves_on_with_the_clock_that_never_goes_back() {
        let start = Instant::now();
        let clock = clock(start);

        assert_eq!(clock.time_of(start), Time(1_000_000_000 * SECOND));
        assert_eq!(
            clock.time_of(start + 90 * SECOND),
            Time(1_000_000_090 * SECOND)
        );
    }

    #[test]
    fn an_instant_before_the_start_of_the_clock_has_its_time_too() {
        let earlier = Instant::now();
        let clock = clock(earlier + 90 * SECOND);

        assert_eq!(clock.time_of(earlier), Time(999_999_910 * SECOND));
    }

    /// What an `Instant` can't do on Windows, and the limits kept in a file need.
    #[test]
    fn a_time_goes_back_before_the_start_of_the_system() {
        let now = Instant::now();
        // Longer than any system has run: 946,080,000 s.
        let thirty_years = 30 * Duration::from_hours(24 * 365);

        assert_eq!(
            clock(now).time_of(now).checked_sub(thirty_years),
            Some(Time(53_920_000 * SECOND))
        );
    }

    #[test]
    fn a_time_before_1970_cant_be_told() {
        let time = Time(10 * SECOND);

        assert_eq!(time.checked_sub(10 * SECOND), Some(Time(Duration::ZERO)));
        assert_eq!(time.checked_sub(11 * SECOND), None);
    }

    #[test]
    fn a_clock_of_the_system_before_1970_counts_from_1970() {
        let start = Instant::now();
        let clock = Clock::starting(start, UNIX_EPOCH - SECOND);

        assert_eq!(clock.time_of(start), Time(Duration::ZERO));
        assert_eq!(clock.time_of(start + SECOND), Time(SECOND));
    }

    #[test]
    fn the_time_between_two_times_is_nothing_when_they_are_swapped() {
        let earlier = Time(10 * SECOND);
        let later = Time(25 * SECOND);

        assert_eq!(later.saturating_duration_since(earlier), 15 * SECOND);
        assert_eq!(earlier.saturating_duration_since(later), Duration::ZERO);
        assert!(earlier < later);
    }

    #[test]
    fn a_time_too_far_to_be_told_is_the_last_one() {
        let last = Time(Duration::MAX);

        assert_eq!(last + SECOND, last);
        assert_eq!(Time(10 * SECOND) + 5 * SECOND, Time(15 * SECOND));
    }

    /// Between two readings of the clock of the system, whatever the speed of the machine.
    #[test]
    fn a_clock_starting_now_tells_the_time_of_the_system() {
        let since_epoch = |system: SystemTime| system.duration_since(UNIX_EPOCH).ok();
        let before = since_epoch(SystemTime::now());

        let clock = Clock::starting_now();

        let after = since_epoch(SystemTime::now());
        let told = Some(clock.time_of(clock.start).0);
        assert!(
            before <= told && told <= after,
            "{before:?} {told:?} {after:?}"
        );
    }
}
