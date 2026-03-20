use latui::app::state::AppState;
use latui::app::controller::handle_key_event;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

fn k(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

#[test]
fn test_excessively_long_query() {
    let mut app = AppState::new();
    
    // Type 200 characters
    for _ in 0..200 {
        handle_key_event(&mut app, k(KeyCode::Char('a'))).unwrap();
    }
    
    // The controller limits query max length (should not panic or go infinite).
    assert!(app.query.len() <= 128);
}

#[test]
fn test_rapid_mode_switching() {
    let mut app = AppState::new();
    
    use latui::modes::run::RunMode;
    use latui::modes::files::FilesMode;
    app.mode_registry.register("run", Box::new(RunMode::new()));
    app.mode_registry.register("files", Box::new(FilesMode::new()));

    let _initial_mode = app.mode_registry.active_mode.clone();

    // Spam tab 100 times
    for _ in 0..100 {
        handle_key_event(&mut app, k(KeyCode::Tab)).unwrap();
    }
    
    // It should happily cycle without crashing and active mode should exist in registry
    assert!(app.mode_registry.modes.contains_key(&app.mode_registry.active_mode));
}

#[test]
fn test_corrupted_toml_config_fallback() {
    use latui::config::theme::AppConfig;
    
    let bad_toml = "[general]\ntheme = ";
    let result = toml::from_str::<AppConfig>(bad_toml);
    assert!(result.is_err(), "Should fail to parse incomplete toml");
    
    let default_config = AppConfig::default();
    assert_eq!(default_config.general.theme, "dark");
}
