use std::fs;
use std::time::{Instant, UNIX_EPOCH};

use super::*;
use crate::files::TestDirectory;
use crate::trade::kept_wait::tests::assert_about;
use crate::trade::limits::clock::Clock;
use crate::trade::limits::tests::{SECOND, hit, search_headers};
use crate::trade::limits::{Limits, MARGIN, Policy};

/// When the first launch of these tests starts, in seconds of the clock of the system.
const FIRST_LAUNCH: u64 = 1_000_000_000;

/// The limits of an application launched at `start`, `seconds` after the first launch on the
/// clock of the system, with the cache directory `directory`.
fn launched(directory: &TestDirectory, start: Instant, seconds: u64) -> Limits {
    let system = UNIX_EPOCH + Duration::from_secs(FIRST_LAUNCH + seconds);
    Limits::starting(Clock::starting(start, system), Some(directory.0.clone()))
}

/// Five searches, one a second from `start`, which fill the window of 5 per 10 s.
fn search_five_times(limits: &mut Limits, start: Instant) {
    for second in 0..5 {
        let now = start + second * SECOND;
        limits.sent(Policy::Search, hit(now));
        let state = format!("{}:10:0,{0}:60:0,{0}:300:0,{0}:21600:0", second + 1);
        limits.answered(Policy::Search, &search_headers(&state), now);
    }
}

/// The time `seconds` after the first launch.
fn time(seconds: u64) -> Time {
    Time::of_second(FIRST_LAUNCH + seconds)
}

/// A request answered at once, `seconds` after the first launch, as the limits keep it.
fn kept_hit(seconds: u64) -> Hit<Time> {
    Hit {
        sent: time(seconds),
        answered: time(seconds),
        counted: true,
    }
}

/// What the file of limits that know `search` of the searches, and nothing else, gives to an
/// application launched `launch` seconds after the first one.
fn read_at(search: PolicyLimits, launch: u64) -> PolicyLimits {
    let kept = Kept {
        behind_system: Duration::ZERO,
        known: Known {
            policies: [search, PolicyLimits::default(), PolicyLimits::default()],
            last_search: None,
        },
    };
    let [search, fetch, exchange] = kept.read_at(time(launch)).policies;
    assert_eq!(
        [fetch, exchange],
        [PolicyLimits::default(), PolicyLimits::default()]
    );
    search
}

/// A window of 5 per `period`, told `told` seconds after the first launch.
fn window(period: Duration, told: u64) -> Window {
    Window {
        hits: 5,
        period,
        others: 2,
        told: time(told),
    }
}

/// The case of Windows, where an `Instant` can't tell a time before the start of the system.
#[test]
fn a_request_sent_before_a_restart_of_the_system_still_counts() {
    let directory = TestDirectory::new("limits-kept-requests");
    let start = Instant::now();
    let mut limits = launched(&directory, start, 0);
    search_five_times(&mut limits, start);
    limits.keep_behind(Duration::ZERO);

    // Launched again 6 s after the first search, the system restarted meanwhile: its
    // `Instant`s start over, and none tells the time of that search.
    let next_launch = launched(&directory, start, 6);

    // 5 per 10 s: the first search leaves the window 10 s after it left.
    assert_eq!(
        next_launch.ready_at(Policy::Search, start),
        start + 4 * SECOND + MARGIN
    );
    // The other policies have their own limits.
    assert_eq!(next_launch.ready_at(Policy::Fetch, start), start);
}

#[test]
fn the_last_search_is_kept_for_the_next_launch() {
    let directory = TestDirectory::new("limits-kept-last-search");
    let start = Instant::now();
    let mut limits = launched(&directory, start, 0);
    assert_eq!(limits.since_last_search(start), None);
    limits.searched(start + 2 * SECOND);
    assert_eq!(limits.since_last_search(start + 3 * SECOND), Some(SECOND));
    limits.keep_behind(Duration::ZERO);

    let next_launch = launched(&directory, start, 6);

    assert_eq!(next_launch.since_last_search(start), Some(4 * SECOND));
}

#[test]
fn a_restriction_is_kept_for_the_next_launch() {
    let directory = TestDirectory::new("limits-kept-restriction");
    let start = Instant::now();
    let mut limits = launched(&directory, start, 0);
    limits.answered(
        Policy::Search,
        &search_headers("6:10:60,6:60:0,6:300:0,6:21600:0"),
        start,
    );
    limits.keep_behind(Duration::ZERO);

    let next_launch = launched(&directory, start, 20);

    assert_eq!(
        next_launch.ready_at(Policy::Search, start),
        start + 40 * SECOND + MARGIN
    );
}

/// On Linux, the clock of the limits stops while the system sleeps: told as they are, the
/// searches made before the sleep would look 8 hours older at the next launch, and be
/// forgotten.
#[test]
fn the_times_of_a_clock_behind_the_system_are_read_that_much_later() {
    let directory = TestDirectory::new("limits-kept-behind");
    let start = Instant::now();
    let mut limits = launched(&directory, start, 0);
    search_five_times(&mut limits, start);
    let sleep = Duration::from_hours(8);
    limits.keep_behind(sleep);

    let next_launch = launched(&directory, start, sleep.as_secs() + 6);

    assert_eq!(
        next_launch.ready_at(Policy::Search, start),
        start + 4 * SECOND + MARGIN
    );
}

#[test]
fn a_time_of_the_past_after_the_launch_is_read_as_the_launch() {
    let kept = Kept {
        behind_system: 15 * SECOND,
        known: Known {
            policies: [
                PolicyLimits {
                    // Read at 85 s and 105 s, the launch at 100 s.
                    sent: vec![kept_hit(70), kept_hit(90)],
                    windows: vec![window(10 * SECOND, 95)],
                    blocked_until: None,
                },
                PolicyLimits::default(),
                PolicyLimits::default(),
            ],
            last_search: Some(time(90)),
        },
    };

    let known = kept.read_at(time(100));

    assert_eq!(
        known,
        Known {
            policies: [
                PolicyLimits {
                    sent: vec![kept_hit(85), kept_hit(100)],
                    windows: vec![window(10 * SECOND, 100)],
                    blocked_until: None,
                },
                PolicyLimits::default(),
                PolicyLimits::default(),
            ],
            last_search: Some(time(100)),
        }
    );
}

#[test]
fn a_window_read_from_the_file_lasts_a_day_at_most() {
    let day = Duration::from_hours(24);
    let kept = PolicyLimits {
        windows: vec![window(day, 0), window(day + SECOND, 0), window(30 * day, 0)],
        ..PolicyLimits::default()
    };

    let read = read_at(kept, 10);

    assert_eq!(
        read.windows,
        [window(day, 0), window(day, 0), window(day, 0)]
    );
}

#[test]
fn the_end_of_a_restriction_more_than_a_day_ahead_is_ignored() {
    let restricted_until = |seconds: u64| PolicyLimits {
        blocked_until: Some(time(seconds)),
        ..PolicyLimits::default()
    };
    // The longest restriction an answer can tell, at the launch: a day, and the margin.
    let longest = 100 + LONGEST_REFUSAL_WAIT.as_secs();
    assert_eq!(longest, 100 + 24 * 3600 + 1);

    assert_eq!(read_at(restricted_until(160), 100), restricted_until(160));
    assert_eq!(
        read_at(restricted_until(longest), 100),
        restricted_until(longest)
    );
    assert_eq!(
        read_at(restricted_until(longest + 1), 100),
        PolicyLimits::default()
    );
    // Over at the launch: kept, and without effect.
    assert_eq!(read_at(restricted_until(40), 100), restricted_until(40));
}

/// The windows find the requests inside them by their order.
#[test]
fn the_requests_of_the_file_are_read_in_the_order_of_their_answers() {
    let kept = PolicyLimits {
        sent: vec![kept_hit(50), kept_hit(30), kept_hit(40)],
        ..PolicyLimits::default()
    };

    let read = read_at(kept, 100);

    assert_eq!(read.sent, [kept_hit(30), kept_hit(40), kept_hit(50)]);
}

/// A change of the shape of the file without a new version would lose the requests counted
/// at an update of the application, or misread them.
#[test]
fn the_file_keeps_the_shape_of_its_version() {
    const FILE: &str = concat!(
        r#"{"version":1,"value":{"behind_system":{"secs":3,"nanos":0},"known":{"policies":["#,
        r#"{"sent":[{"sent":{"secs":1000000000,"nanos":0},"#,
        r#""answered":{"secs":1000000000,"nanos":0},"counted":true}],"#,
        r#""windows":[{"hits":5,"period":{"secs":10,"nanos":0},"others":2,"#,
        r#""told":{"secs":1000000001,"nanos":0}}],"#,
        r#""blocked_until":{"secs":1000000061,"nanos":0}},"#,
        r#"{"sent":[],"windows":[],"blocked_until":null},"#,
        r#"{"sent":[],"windows":[],"blocked_until":null}],"#,
        r#""last_search":{"secs":1000000000,"nanos":0}}}}"#
    );
    let known = Known {
        policies: [
            PolicyLimits {
                sent: vec![kept_hit(0)],
                windows: vec![window(10 * SECOND, 1)],
                blocked_until: Some(time(61)),
            },
            PolicyLimits::default(),
            PolicyLimits::default(),
        ],
        last_search: Some(time(0)),
    };
    let directory = TestDirectory::new("limits-kept-shape");
    let mut file = LimitsFile::in_directory(directory.0.clone());

    file.keep(&known, 3 * SECOND);

    let path = directory.0.join("trade-limits.json");
    assert_eq!(fs::read_to_string(&path).expect("a file"), FILE);
    // Read 3 s later, as the clock of the system tells them.
    assert_eq!(
        file.read(time(1000)),
        Known {
            policies: [
                PolicyLimits {
                    sent: vec![kept_hit(3)],
                    windows: vec![window(10 * SECOND, 4)],
                    blocked_until: Some(time(64)),
                },
                PolicyLimits::default(),
                PolicyLimits::default(),
            ],
            last_search: Some(time(3)),
        }
    );
}

#[test]
fn a_file_that_cant_be_read_or_of_another_version_knows_nothing() {
    let directory = TestDirectory::new("limits-kept-unreadable");
    let start = Instant::now();
    let mut limits = launched(&directory, start, 0);
    search_five_times(&mut limits, start);
    limits.searched(start);
    limits.keep_behind(Duration::ZERO);
    let path = directory.0.join(KEPT_FILE);
    let written = fs::read_to_string(&path).expect("a file");
    assert!(launched(&directory, start, 6).ready_at(Policy::Search, start) > start);

    for contents in [
        "{",
        r#"{"version":1,"value":{"known":"?"}}"#,
        &written.replacen(r#""version":1"#, r#""version":2"#, 1),
    ] {
        fs::write(&path, contents).expect("a file");

        let next_launch = launched(&directory, start, 6);

        assert_eq!(
            next_launch.ready_at(Policy::Search, start),
            start,
            "{contents}"
        );
        assert_eq!(next_launch.since_last_search(start), None, "{contents}");
    }
}

#[test]
fn limits_that_cant_be_kept_still_hold_until_the_application_closes() {
    let directory = TestDirectory::new("limits-kept-unwritable");
    fs::create_dir_all(&directory.0).expect("a directory");
    // A file has the name of the cache directory: nothing can be written in it.
    let blocked = directory.0.join("blocked");
    fs::write(&blocked, "").expect("a file");
    let start = Instant::now();
    let clock = Clock::starting(start, UNIX_EPOCH + Duration::from_secs(FIRST_LAUNCH));

    for mut limits in [
        Limits::starting(clock, Some(blocked.clone())),
        Limits::starting(clock, None),
    ] {
        search_five_times(&mut limits, start);
        limits.keep();
        limits.keep();

        assert_eq!(
            limits.ready_at(Policy::Search, start + 4 * SECOND),
            start + 10 * SECOND + MARGIN
        );
    }
    // Nothing was kept: the file is still a file, with no directory in its place.
    assert!(blocked.is_file());
}

/// With the clocks of the system, as the application reads them: the same wait, but for the
/// moment between two readings of the clocks.
#[test]
fn the_limits_kept_by_a_launch_are_those_of_the_next_one() {
    let directory = TestDirectory::new("limits-kept-clocks");
    let mut limits = Limits::kept_in(Some(directory.0.clone()));
    let start = Instant::now();
    search_five_times(&mut limits, start);
    limits.searched(start);
    limits.keep();

    let next_launch = Limits::kept_in(Some(directory.0.clone()));

    let now = start + 4 * SECOND;
    assert_about(
        Some(next_launch.ready_at(Policy::Search, now)),
        limits.ready_at(Policy::Search, now),
    );
    let since = next_launch.since_last_search(now).expect("a search");
    assert!(3 * SECOND < since && since < 5 * SECOND, "{since:?}");
}
