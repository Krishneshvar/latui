use crate::app::state::AppState;
use crate::ui;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{Terminal, backend::Backend};
use std::time::Duration;

pub struct AppController {
    // We can add configuration or other dependencies here later
}

impl AppController {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for AppController {
    fn default() -> Self {
        Self::new()
    }
}

impl AppController {
    pub fn run<B: Backend>(
        &self,
        terminal: &mut Terminal<B>,
        app: &mut AppState,
    ) -> anyhow::Result<()> {
        loop {
            terminal
                .draw(|f| ui::renderer::draw(f, app))
                .map_err(|e| anyhow::anyhow!("Draw error: {}", e))?;

            if let Ok(true) = event::poll(Duration::from_millis(100))
                && let Event::Key(key) = event::read()?
            {
                match (key.code, key.modifiers) {
                    // Character input
                    (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                        if is_valid_query_char(c) && app.query.len() < 128 {
                            app.query.push(c);
                            self.update_results(app);
                        }
                    }

                    // Backspace
                    (KeyCode::Backspace, _) => {
                        app.query.pop();
                        self.update_results(app);
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
                        self.update_results(app);
                    }

                    (KeyCode::BackTab, _) => {
                        app.mode_registry.previous_mode();
                        app.query.clear();
                        self.update_results(app);
                    }

                    // Execute selected item
                    (KeyCode::Enter, _) => {
                        if let Some(i) = app.list_state.selected()
                            && let Some(item) = app.filtered_items.get(i).cloned()
                            && let Some(mode) = app.mode_registry.get_active_mode_mut()
                        {
                            // Record the selection for usage tracking
                            mode.record_selection(&app.query, &item);

                            let keep_open = mode.stays_open();

                            // Execute the action
                            match mode.execute(&item) {
                                Ok(()) => {
                                    if !keep_open {
                                        return Ok(());
                                    }
                                    // Stay open: clear query so the picker is ready
                                    // for the next selection (useful for clipboard / emojis).
                                    app.query.clear();
                                    self.update_results(app);
                                }
                                Err(e) => {
                                    tracing::error!(
                                        "Failed to execute item '{}': {}",
                                        item.title,
                                        e
                                    );
                                    crate::core::utils::notify_error(
                                        &format!("Failed to launch {}", item.title),
                                        &e.to_string()
                                    );
                                }
                            }
                        }
                    }

                    // Exit
                    (KeyCode::Esc, _) => {
                        return Ok(());
                    }

                    _ => {}
                }
            }
        }
    }

    fn update_results(&self, app: &mut AppState) {
        if let Some(mode) = app.mode_registry.get_active_mode_mut() {
            app.filtered_items = mode.search(&app.query);
        }
        app.reset_selection();
    }
}

fn is_valid_query_char(c: char) -> bool {
    c.is_alphanumeric() || c.is_whitespace() || "-_.$+!*'(),/@:".contains(c)
}
