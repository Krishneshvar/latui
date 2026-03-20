use latui::app::state::AppState;
use latui::config::keywords::KeywordMapper;
use latui::config::loader::load_user_config_path;
use latui::config::settings::load_user_settings;
use latui::modes::{
    apps::AppsMode, clipboard::ClipboardMode, custom::CustomMode, emojis::EmojisMode, files::FilesMode, run::RunMode,
};
use latui::tracking::frequency::FrequencyTracker;
use std::fs;

use tracing::{Level, debug, error, info};
use tracing_appender::rolling;
use latui::core::utils::latui_xdg;
use std::io;

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode},
};

use ratatui::{Terminal, backend::CrosstermBackend};

fn init_tracing() -> anyhow::Result<tracing_appender::non_blocking::WorkerGuard> {
    let xdg = latui_xdg();
    let log_dir = xdg.place_state_file("logs")?;
    let file_appender = rolling::daily(
        log_dir.parent().unwrap_or(std::path::Path::new("/tmp")),
        "latui.log",
    );
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
    let xdg = latui_xdg();

    // Ensure core directories exist and are secure
    if let Ok(data_dir) = xdg.create_data_directory("") {
        #[cfg(unix)]
        secure_permissions(&data_dir);
    }
    if let Ok(state_dir) = xdg.create_state_directory("") {
        #[cfg(unix)]
        secure_permissions(&state_dir);
    }

    let _guard =
        init_tracing().map_err(|e| anyhow::anyhow!("Failed to initialize logging: {}", e))?;
    
    // Set panic hook to ensure terminal restoration
    std::panic::set_hook(Box::new(|panic_info| {
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = execute!(std::io::stdout(), LeaveAlternateScreen);
        eprintln!("Latui Panic: {}", panic_info);
        tracing::error!("FATAL PANIC: {}", panic_info);
    }));

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
    app.detect_image_support();

    // Initialize common components
    let xdg = latui_xdg();
    let frequency_tracker = match xdg.place_data_file("usage.db") {
        Ok(db_path) => match FrequencyTracker::new(&db_path) {
            Ok(mut tracker) => {
                let _ = tracker.cleanup(30);
                Some(tracker)
            }
            Err(e) => {
                error!("Failed to initialize usage tracker: {}", e);
                None
            }
        },
        Err(e) => {
            error!("Failed to generate usage tracking path: {}", e);
            None
        }
    };

    let mut keyword_mapper = KeywordMapper::with_defaults();
    if let Some(path) = load_user_config_path()
        && let Ok(content) = fs::read_to_string(&path)
        && let Ok(custom_mapper) = KeywordMapper::from_toml(&content)
    {
        keyword_mapper = custom_mapper;
        info!("Loaded custom keywords from {:?}", path);
    }

    // Load the full application configuration (including themes)
    app.config = load_user_settings();
    let apps_settings = app.config.modes.apps.clone();

    // Register built-in modes with injected dependencies
    app.mode_registry.register(
        "apps",
        Box::new(AppsMode::new(
            frequency_tracker,
            keyword_mapper,
            apps_settings,
        )),
    );
    app.mode_registry.register("run", Box::new(RunMode::new()));
    app.mode_registry
        .register("files", Box::new(FilesMode::new()));
    app.mode_registry
        .register("clipboard", Box::new(ClipboardMode::new()));
    app.mode_registry
        .register("emojis", Box::new(EmojisMode::new()));

    // Register custom modes from configuration
    for (id, custom_config) in app.config.modes.custom.clone() {
        app.mode_registry.register(
            &id,
            Box::new(CustomMode::new(id.clone(), custom_config)),
        );
    }

    // Load all registered modes
    info!("Initializing modes...");
    for mode_name in app.mode_registry.get_mode_order().to_vec() {
        if let Err(e) = app.mode_registry.switch_mode(&mode_name) {
            error!("Failed to switch to mode '{}': {}", mode_name, e);
            continue;
        }

        if let Some(mode) = app.mode_registry.get_active_mode_mut() {
            debug!("Loading mode: {}", mode.name());
            if let Err(e) = mode.load() {
                error!("Failed to load mode '{}': {}", mode.name(), e);
            }
        }
    }

    // Switch back to default mode and initial search
    let default_mode = app.mode_registry.default_mode.clone();
    app.mode_registry.switch_mode(&default_mode)?;
    if let Some(mode) = app.mode_registry.get_active_mode_mut() {
        app.filtered_items = mode.search("");
    }

    let controller = latui::app::controller::AppController::new();
    let res = controller.run(&mut terminal, &mut app);

    terminal.show_cursor()?;
    res
}
