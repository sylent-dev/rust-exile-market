use super::*;

#[test]
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
