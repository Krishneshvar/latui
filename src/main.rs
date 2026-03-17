use latui::app::state::AppState;
use latui::ui;

use tracing::{info, error, debug, Level};
use tracing_appender::rolling;
use xdg::BaseDirectories;

use std::io;


use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
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

#[cfg(unix)]
fn secure_permissions(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700));
}

fn main() -> anyhow::Result<()> {
    let xdg = BaseDirectories::with_prefix("latui");
    
    // Ensure core directories exist and are secure
    if let Ok(data_dir) = xdg.create_data_directory("") {
        #[cfg(unix)]
        secure_permissions(&data_dir);
    }
    if let Ok(state_dir) = xdg.create_state_directory("") {
        #[cfg(unix)]
        secure_permissions(&state_dir);
    }

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

    let mut app = AppState::new();
    
    // Load all registered modes
    info!("Initializing modes...");
    let mode_names: Vec<String> = app.mode_registry.get_mode_order().to_vec();
    for mode_name in &mode_names {
        if let Err(e) = app.mode_registry.switch_mode(mode_name) {
            error!("Failed to switch to mode '{}': {}", mode_name, e);
            continue;
        }
        
        if let Some(mode) = app.mode_registry.get_active_mode_mut() {
            debug!("Loading mode: {} ({})", mode.name(), mode.description());
            if let Err(e) = mode.load() {
                error!("Failed to load mode '{}': {}", mode.name(), e);
            }
        }
    }
    
    // Switch back to default mode
    let default_mode = app.mode_registry.default_mode.clone();
    app.mode_registry.switch_mode(&default_mode)?;
    
    // Initial search with empty query
    if let Some(mode) = app.mode_registry.get_active_mode_mut() {
        debug!("Loading initial results for mode: {}", mode.name());
        app.filtered_items = mode.search("");
    }
    debug!("Initial items loaded: {}", app.filtered_items.len());

    loop {
        terminal.draw(|f| ui::renderer::draw(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match (key.code, key.modifiers) {
                // Character input
                (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                    if is_valid_query_char(c) && app.query.len() < 128 {
                        app.query.push(c);
                        update_results(&mut app);
                    }
                }

                // Backspace
                (KeyCode::Backspace, _) => {
                    app.query.pop();
                    update_results(&mut app);
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
                    update_results(&mut app);
                    info!("Switched to mode: {}", app.mode_registry.active_mode);
                }
                
                (KeyCode::BackTab, _) => {
                    app.mode_registry.previous_mode();
                    app.query.clear();
                    update_results(&mut app);
                    info!("Switched to mode: {}", app.mode_registry.active_mode);
                }

                // Execute selected item
                (KeyCode::Enter, _) => {
                    if let Some(i) = app.list_state.selected() {
                        if let Some(item) = app.filtered_items.get(i).cloned() {
                            if let Some(mode) = app.mode_registry.get_active_mode_mut() {
                                info!("Executing item '{}' in mode '{}'", item.title, mode.name());
                                
                                // Record the selection for usage tracking
                                mode.record_selection(&app.query, &item);
                                
                                // Execute the action
                                if let Err(e) = mode.execute(&item) {
                                    error!("Failed to execute item '{}': {}", item.title, e);
                                    // Continue running instead of crashing
                                } else {
                                    // Exit after successful execution
                                    break;
                                }
                            }
                        }
                    }
                }

                // Exit
                (KeyCode::Esc, _) => {
                    info!("User requested exit via Esc");
                    break;
                }

                _ => {}
            }
        }
    }

    terminal.show_cursor()?;
    Ok(())
}

/// Validates that a character is safe for search queries.
/// Prevents injection attacks and ensures reasonable input.
fn is_valid_query_char(c: char) -> bool {
    // Only allow safe characters for search
    c.is_alphanumeric() || c.is_whitespace() || "-_.$+!*'(),/".contains(c)
}

/// Updates search results based on current query and active mode.
fn update_results(app: &mut AppState) {
    if let Some(mode) = app.mode_registry.get_active_mode_mut() {
        app.filtered_items = mode.search(&app.query);
    }
    app.reset_selection();
}
