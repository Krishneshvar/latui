/// Top-level error type for all LaTUI operations.
///
/// Library code returns this type; `main.rs` uses `anyhow::Result` only
/// at the binary boundary to attach final context before display.
#[derive(Debug, thiserror::Error)]
pub enum LatuiError {
    // ── Subsystem errors (structured, matchable) ──────────────────────────

    #[error("Database error: {0}")]
    Database(#[from] crate::tracking::database::DatabaseError),

    #[error("Cache error: {0}")]
    Cache(#[from] CacheError),

    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Execution error for '{command}': {source}")]
    Execution {
        command: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Terminal drawing error: {0}")]
    Draw(String),

    #[error("Terminal event error: {0}")]
    Event(String),

    // ── Targeted string errors (only where no structured type fits) ───────

    /// XDG base-directory resolution failed. Stores the underlying message
    /// because `xdg::BaseDirectoriesError` does not implement `std::error::Error`
    /// in all supported versions.
    #[error("XDG path error: {0}")]
    Xdg(String),

    /// Catch-all for mode-level logic errors that do not fit a narrower
    /// variant. Prefer adding a specific variant when extending the enum.
    #[error("Application error: {0}")]
    App(String),
}

// ── Configuration errors ─────────────────────────────────────────────────────

/// Errors that arise while loading or validating LaTUI configuration.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to read config file '{path}': {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse config file '{path}': {source}")]
    Parse {
        path: String,
        #[source]
        source: toml::de::Error,
    },

    #[error("Failed to parse keyword mappings: {0}")]
    Keywords(toml::de::Error),

    #[error("Theme '{name}' not found in user config dirs or bundled themes")]
    ThemeNotFound { name: String },

    #[error("Failed to read theme file '{path}': {source}")]
    ThemeRead {
        path: String,
        #[source]
        source: std::io::Error,
    },
}

// ── Cache errors ─────────────────────────────────────────────────────────────

/// Errors that arise while reading or writing the application index cache.
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("IO error during cache operation: {0}")]
    Io(#[from] std::io::Error),

    #[error("Cache serialization error: {0}")]
    Json(#[from] serde_json::Error),
}
