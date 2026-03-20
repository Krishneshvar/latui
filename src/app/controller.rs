//! Event loop and input handling for the `LaTUI` application.
//!
//! This module coordinates the flow of the application by processing terminal
//! events, updating the state based on user input, and triggering UI redraws.

use crate::app::state::AppState;
use crate::error::LatuiError;
use crate::ui;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{backend::Backend, Terminal};
use std::time::Duration;

/// Processes the main event loop for the application.
///
/// This function listens for keyboard input, updates the user's query,
/// navigates results, and executes actions. It returns when the user
/// either picks an item or exits via `Esc`.
pub fn run<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut AppState,
) -> Result<(), LatuiError> {
    loop {
        terminal
            .draw(|f| ui::renderer::draw(f, app))
            .map_err(|e| LatuiError::Draw(e.to_string()))?;

        if matches!(event::poll(Duration::from_millis(100)), Ok(true))
            && let Event::Key(key) =
                event::read().map_err(|e| LatuiError::Event(e.to_string()))?
            && !handle_key_event(app, key)? {
                return Ok(());
            }
    }
}

/// Internal helper to handle a key event and update application state.
/// Returns `Ok(true)` to keep running, or `Ok(false)` to exit the application.
pub fn handle_key_event(app: &mut AppState, key: event::KeyEvent) -> Result<bool, LatuiError> {
    match (key.code, key.modifiers) {
        // Character input
        (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
            if is_valid_query_char(c) && app.query.len() < 128 {
                app.query.push(c);
                update_results(app);
            }
        }

        // Backspace
        (KeyCode::Backspace, _) => {
            app.query.pop();
            update_results(app);
        }

        // Navigation
        (KeyCode::Down, _) => {
            app.next();
        }

        (KeyCode::Up, _) => {
            app.previous();
        }

        // Mode switching
        (KeyCode::Tab, KeyModifiers::NONE) => {
            app.mode_registry.next_mode();
            app.query.clear();
            update_results(app);
        }

        (KeyCode::BackTab, _) => {
            app.mode_registry.previous_mode();
            app.query.clear();
            update_results(app);
        }

        // Execute selected item
        (KeyCode::Enter, _) => {
            if let Some(i) = app.list_state.selected()
                && let Some(item) = app.filtered_items.get(i).cloned()
                && let Some(mode) = app.mode_registry.get_active_mode_mut()
            {
                // Record the selection for usage tracking
                mode.record_selection(&app.query, &item);

                let stays_open = mode.stays_open();

                // Execute the action
                match mode.execute(&item) {
                    Ok(()) => {
                        if !stays_open {
                            return Ok(false);
                        }
                        // Stay open: clear query so the picker is ready
                        app.query.clear();
                        update_results(app);
                    }
                    Err(e) => {
                        tracing::error!(
                            item = %item.title,
                            error = %e,
                            "Failed to execute item"
                        );
                        crate::core::utils::notify_error(
                            &format!("Failed to launch {}", item.title),
                            &e.to_string(),
                        );
                    }
                }
            }
        }

        // Exit
        (KeyCode::Esc, _) => {
            return Ok(false);
        }

        _ => {}
    }
    Ok(true)
}

/// Synchronizes search results with the current application query.
fn update_results(app: &mut AppState) {
    if let Some(mode) = app.mode_registry.get_active_mode_mut() {
        app.filtered_items = mode.search(&app.query);
    }
    app.reset_selection();
}

/// Returns true if the character is permitted in search queries.
fn is_valid_query_char(c: char) -> bool {
    c.is_alphanumeric()
        || c == ' '
        || matches!(
            c,
            '-' | '_'
                | '.'
                | '$'
                | '+'
                | '!'
                | '*'
                | '\''
                | '('
                | ')'
                | ','
                | '/'
                | '@'
                | ':'
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_query_chars() {
        assert!(is_valid_query_char('a'));
        assert!(is_valid_query_char('Z'));
        assert!(is_valid_query_char('0'));
        assert!(is_valid_query_char(' '));
        assert!(is_valid_query_char('-'));
        assert!(is_valid_query_char('_'));
        assert!(is_valid_query_char('.'));
        assert!(is_valid_query_char('/'));
        assert!(is_valid_query_char('@'));
        assert!(is_valid_query_char(':'));
    }

    #[test]
    fn test_invalid_query_chars() {
        assert!(!is_valid_query_char('\n'));
        assert!(!is_valid_query_char('\t'));
        assert!(!is_valid_query_char(';'));
        assert!(!is_valid_query_char('<'));
        assert!(!is_valid_query_char('>'));
        assert!(!is_valid_query_char('?'));
        assert!(!is_valid_query_char('|'));
        assert!(!is_valid_query_char('\\'));
    }

    use crossterm::event::{KeyEvent, KeyEventKind, KeyEventState};

    fn format_key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn test_controller_typing() {
        let mut app = AppState::new();
        // Type 'a'
        handle_key_event(&mut app, format_key(KeyCode::Char('a'))).unwrap();
        assert_eq!(app.query, "a");
        
        // Type 'p'
        handle_key_event(&mut app, format_key(KeyCode::Char('p'))).unwrap();
        assert_eq!(app.query, "ap");

        // Backspace
        handle_key_event(&mut app, format_key(KeyCode::Backspace)).unwrap();
        assert_eq!(app.query, "a");
    }

    #[test]
    fn test_controller_tab_switching() {
        let mut app = AppState::new();
        let _initial_mode = app.mode_registry.active_mode.clone();

        // This might not change if only one mode relies on defaults, but if there are multiple, it should.
        // Just calling it verifies it won't crash.
        handle_key_event(&mut app, format_key(KeyCode::Tab)).unwrap();
    }
}
