mod app;
mod core;
mod ui;
mod modes;
mod cache;
mod index;
mod matcher;
mod search;
mod config;
mod tracking;

use std::io;

use app::state::AppState;
use crate::core::mode::Mode;
use crate::modes::apps::AppsMode;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};

fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut mode = AppsMode::new();
    mode.load();

    let mut app = AppState::new(Vec::new());
    app.filtered_items = mode.search("");

    loop {
        terminal.draw(|f| ui::renderer::draw(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char(c) => {
                    app.query.push(c);
                    update_results(&mut app, &mut mode);
                }

                KeyCode::Backspace => {
                    app.query.pop();
                    update_results(&mut app, &mut mode);
                }

                KeyCode::Down => {
                    app.next();
                }

                KeyCode::Up => {
                    app.previous();
                }

                KeyCode::Enter => {
                    if let Some(i) = app.list_state.selected() {
                        if let Some(item) = app.filtered_items.get(i) {
                            // Record the selection
                            mode.record_selection(&app.query, item);
                            // Execute the app
                            mode.execute(item);
                        }
                    }
                }

                KeyCode::Esc => break,

                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn update_results(app: &mut AppState, mode: &mut impl Mode) {
    app.filtered_items = mode.search(&app.query);
    app.reset_selection();
}
