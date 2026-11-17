use super::*;

#[test]
fn the_gear_opens_the_settings_then_closes_them() {
    assert_eq!(View::Item.toggled(View::Settings), View::Settings);
    assert_eq!(View::Settings.toggled(View::Settings), View::Item);
}

#[test]
fn the_bug_opens_the_bug_report_and_the_gear_the_settings_in_its_place() {
    assert_eq!(View::Item.toggled(View::BugReport), View::BugReport);
    assert_eq!(View::BugReport.toggled(View::BugReport), View::Item);
    assert_eq!(View::BugReport.toggled(View::Settings), View::Settings);
    assert_eq!(View::Settings.toggled(View::BugReport), View::BugReport);
}

#[test]
fn a_copied_item_closes_the_bug_report() {
    let copied = ClipboardEvent::Copied {
        item: CopiedItem::from_text("Item Class: Rings\nRarity: Rare\nWoe Loop\nRuby Ring")
            .unwrap(),
        hotkey: None,
    };

    assert_eq!(View::BugReport.after(&copied), View::Item);
    assert_eq!(
        View::BugReport.after(&ClipboardEvent::Unreadable),
        View::BugReport
    );
}

#[test]
fn a_copied_item_closes_the_settings() {
    let copied = ClipboardEvent::Copied {
        item: CopiedItem::from_text("Item Class: Rings\nRarity: Rare\nWoe Loop\nRuby Ring")
            .unwrap(),
        hotkey: None,
    };

    assert_eq!(View::Settings.after(&copied), View::Item);
    assert_eq!(View::Item.after(&copied), View::Item);
}

#[test]
fn the_state_of_the_clipboard_keeps_the_view() {
    for event in [ClipboardEvent::Unreadable, ClipboardEvent::Readable] {
        assert_eq!(View::Settings.after(&event), View::Settings);
        assert_eq!(View::Item.after(&event), View::Item);
    }
}

#[test]
fn the_application_quits_when_the_game_closes_once_close_with_the_game_is_checked() {
    use game_window::Change;
    let mut settings = Settings::default();

    assert!(!quits_with_the_game(&[Change::Closed], &settings));
    settings.apply(SettingChange::CloseWithGame(true));
    assert!(quits_with_the_game(&[Change::Closed], &settings));
    assert!(quits_with_the_game(
        &[Change::Opened, Change::Closed],
        &settings
    ));
    assert!(!quits_with_the_game(&[Change::Opened], &settings));
    assert!(!quits_with_the_game(
        &[Change::Closed, Change::Opened],
        &settings
    ));
    assert!(!quits_with_the_game(&[], &settings));
}
