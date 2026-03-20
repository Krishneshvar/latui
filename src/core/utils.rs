use std::time::{SystemTime, UNIX_EPOCH};
use xdg::BaseDirectories;

/// Current unix timestamp in seconds.
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Centralized XDG base directory for LaTUI.
pub fn latui_xdg() -> BaseDirectories {
    BaseDirectories::with_prefix("latui")
}
