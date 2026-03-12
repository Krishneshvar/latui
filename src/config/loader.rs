use std::path::PathBuf;

/// Load configuration from user's config directory
pub fn load_user_config() -> Option<PathBuf> {
    // TODO: Load from ~/.config/latui/keywords.toml
    None
}

/// Get the default config path
pub fn default_config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(format!("{}/.config/latui/keywords.toml", home))
}
