use super::*;

#[test]
#[cfg(any(target_os = "linux", target_os = "windows"))]
fn the_game_is_told_by_its_exact_title_out_of_the_file_explorer() {
    assert!(is_game_window("Path of Exile 2", "POEWindowClass"));
    // A page of a browser about the game.
    assert!(!is_game_window(
        "Path of Exile 2 - Wiki",
        "Chrome_WidgetWin_1"
    ));
    // The folder of the game.
    assert!(!is_game_window("Path of Exile 2", "CabinetWClass"));
}

#[test]
#[cfg(target_os = "linux")]
fn tells_the_game_by_its_class_under_proton() {
    assert!(is_game_class(b"steam_app_2694490\0steam_app_2694490\0"));
    assert!(is_game_class(b"steam_app_2694490"));
}

#[test]
#[cfg(target_os = "linux")]
fn tells_the_game_by_its_exact_title_only() {
    assert!(is_game_title(b"Path of Exile 2"));
    // Path of Exile 1, a page of a browser about the game.
    assert!(!is_game_title(b"Path of Exile"));
    assert!(!is_game_title(b"Path of Exile 2 - Mozilla Firefox"));
    assert!(!is_game_title(b""));
}

#[test]
#[cfg(target_os = "linux")]
fn tells_another_window_by_its_class() {
    // Path of Exile 1, the Steam client, a class that only starts like the game's.
    assert!(!is_game_class(b"steam_app_238960\0steam_app_238960\0"));
    assert!(!is_game_class(b"steam\0steam\0"));
    assert!(!is_game_class(b"steam_app_26944900\0"));
    assert!(!is_game_class(b""));
}

/// What the windows told, `seconds` after `start`.
fn seen(start: Instant, seconds: f32, is_open: bool) -> Seen {
    Seen {
        is_open,
        at: start + Duration::from_secs_f32(seconds),
    }
}

/// The changes of a game whose windows told `readings`, then checked at `now_seconds`.
fn changes(start: Instant, readings: &[(f32, bool)], now_seconds: f32) -> Vec<Change> {
    let mut presence = Presence::default();
    let mut changes: Vec<_> = readings
        .iter()
        .flat_map(|&(seconds, is_open)| {
            presence
                .take(seen(start, seconds, is_open))
                .collect::<Vec<_>>()
        })
        .collect();
    changes.extend(presence.check(start + Duration::from_secs_f32(now_seconds)));
    changes
}

#[test]
fn a_game_seen_open_is_told_opened_once() {
    let start = Instant::now();
    assert_eq!(
        changes(start, &[(0.0, true), (1.0, true)], 2.0),
        [Change::Opened]
    );
}

#[test]
fn a_game_is_told_closed_once_its_windows_have_been_gone_for_five_seconds() {
    let start = Instant::now();
    let readings = [(0.0, true), (10.0, false)];
    assert_eq!(changes(start, &readings, 14.9), [Change::Opened]);
    assert_eq!(
        changes(start, &readings, 15.0),
        [Change::Opened, Change::Closed]
    );
}

#[test]
fn a_window_of_the_game_back_within_five_seconds_is_the_same_game() {
    let start = Instant::now();
    let readings = [(0.0, true), (10.0, false), (14.0, true)];
    assert_eq!(changes(start, &readings, 60.0), [Change::Opened]);
}

#[test]
fn a_game_never_seen_open_is_never_told_closed() {
    let start = Instant::now();
    assert_eq!(changes(start, &[(0.0, false), (1.0, false)], 60.0), []);
}

#[test]
fn a_game_opened_again_after_its_closing_is_told_closed_then_opened() {
    let start = Instant::now();
    // The window woken late: the closing due at 15 s comes before the game opened at 20 s.
    let readings = [(0.0, true), (10.0, false), (20.0, true)];
    assert_eq!(
        changes(start, &readings, 21.0),
        [Change::Opened, Change::Closed, Change::Opened]
    );
}

#[test]
fn the_window_wakes_when_the_closing_is_due_only_while_the_game_is_gone() {
    let start = Instant::now();
    let (sender, receiver) = mpsc::channel();
    let mut watch = GameWindowWatch {
        seen: Some(receiver),
        presence: Presence::default(),
    };
    let at = |seconds| start + Duration::from_secs(seconds);
    sender.send(seen(start, 0.0, true)).unwrap();
    assert_eq!(watch.follow(at(1)), [Change::Opened]);
    assert_eq!(watch.next_check(at(1)), None);
    sender.send(seen(start, 10.0, false)).unwrap();
    assert_eq!(watch.follow(at(11)), []);
    assert_eq!(watch.next_check(at(11)), Some(Duration::from_secs(4)));
    assert_eq!(watch.follow(at(15)), [Change::Closed]);
    assert_eq!(watch.next_check(at(15)), None);
}

#[test]
fn the_game_is_never_told_closed_once_its_windows_can_no_longer_be_read() {
    let start = Instant::now();
    let (sender, receiver) = mpsc::channel();
    let mut watch = GameWindowWatch {
        seen: Some(receiver),
        presence: Presence::default(),
    };
    sender.send(seen(start, 0.0, true)).unwrap();
    sender.send(seen(start, 10.0, false)).unwrap();
    // The thread of the system stopped: the X server lost.
    drop(sender);
    assert_eq!(
        watch.follow(start + Duration::from_secs(11)),
        [Change::Opened]
    );
    assert_eq!(watch.follow(start + Duration::from_secs(60)), []);
    assert_eq!(watch.next_check(start + Duration::from_secs(60)), None);
}
