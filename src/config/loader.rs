use std::path::PathBuf;
use crate::core::utils::latui_xdg;

/// Load configuration from user's config directory
pub fn load_user_config_path() -> Option<PathBuf> {
    let xdg = latui_xdg();
    xdg.find_config_file("keywords.toml")
}
