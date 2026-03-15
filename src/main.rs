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
pub mod error;

use tracing::{info, error, debug, Level};
use tracing_appender::rolling;
use xdg::BaseDirectories;

use std::io;

use app::state::AppState;
use crate::core::mode::Mode;
use crate::modes::apps::AppsMode;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};

fn init_tracing() -> anyhow::Result<tracing_appender::non_blocking::WorkerGuard> {
    let xdg = BaseDirectories::with_prefix("latui");
    let log_dir = xdg.place_state_file("logs")?;
    let file_appender = rolling::daily(log_dir.parent().unwrap_or(std::path::Path::new("/tmp")), "latui.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_max_level(Level::DEBUG)
        .with_ansi(false)
        .init();

    Ok(guard)
}

fn main() -> anyhow::Result<()> {
    let _guard = init_tracing().map_err(|e| anyhow::anyhow!("Failed to initialize logging: {}", e))?;
    info!("Starting Latui launcher...");

    let res = run_app();

    // Ensure raw mode is correctly disabled regardless of UI panics
    if let Err(e) = crossterm::terminal::disable_raw_mode() {
        error!("Failed to disable raw mode on exit: {}", e);
    }
    if let Err(e) = execute!(io::stdout(), LeaveAlternateScreen) {
        error!("Failed to leave alternate screen: {}", e);
    }

    if let Err(err) = res {
        error!("Fatal application error recorded: {:?}", err);
        return Err(err);
    }
    
    info!("Latui launcher successfully shut down.");
    Ok(())
}

fn run_app() -> anyhow::Result<()> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut mode = AppsMode::new();
    debug!("Loading applications mode");
    mode.load();

    let mut app = AppState::new(Vec::new());
    app.filtered_items = mode.search("");
    debug!("Initial items loaded: {}", app.filtered_items.len());

    loop {
        terminal.draw(|f| ui::renderer::draw(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char(c) => {
                    if is_valid_query_char(c) && app.query.len() < 128 {
                        app.query.push(c);
                        update_results(&mut app, &mut mode);
                    }
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
                            info!("Launching selected item: {}", item.title);
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

    terminal.show_cursor()?;
    Ok(())
}

fn is_valid_query_char(c: char) -> bool {
    // Only allow safe characters for search
    c.is_alphanumeric() || c.is_whitespace() || "-_.$+!*'(),/".contains(c)
}

fn update_results(app: &mut AppState, mode: &mut impl Mode) {
    app.filtered_items = mode.search(&app.query);
    app.reset_selection();
}
