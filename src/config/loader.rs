use xdg::BaseDirectories;
use std::path::PathBuf;

/// Load configuration from user's config directory
pub fn load_user_config_path() -> Option<PathBuf> {
    let xdg = BaseDirectories::with_prefix("latui");
    xdg.find_config_file("keywords.toml")
}


