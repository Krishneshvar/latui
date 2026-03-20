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

/// Recursively merges two TOML tables.
/// Values in 'overrides' replace values in 'base'.
pub fn merge_toml(base: &mut toml::Value, overrides: toml::Value) {
    match (base, overrides) {
        (toml::Value::Table(base_table), toml::Value::Table(overrides_table)) => {
            for (key, val) in overrides_table {
                if let Some(base_val) = base_table.get_mut(&key) {
                    merge_toml(base_val, val);
                } else {
                    base_table.insert(key, val);
                }
            }
        }
        (base_val, overrides_val) => {
            *base_val = overrides_val;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_toml_basic() {
        let mut base = toml::Value::Table(toml::toml! {
            [section]
            a = 1
            b = 2
        });
        let overrides = toml::Value::Table(toml::toml! {
            [section]
            b = 3
            c = 4
        });
        
        merge_toml(&mut base, overrides);
        
        assert_eq!(base["section"]["a"].as_integer(), Some(1));
        assert_eq!(base["section"]["b"].as_integer(), Some(3));
        assert_eq!(base["section"]["c"].as_integer(), Some(4));
    }

    #[test]
    fn test_merge_toml_replaces_type() {
        let mut base = toml::Value::Table(toml::toml! {
            val = "string"
        });
        let overrides = toml::Value::Table(toml::toml! {
            val = 42
        });
        
        merge_toml(&mut base, overrides);
        
        assert_eq!(base["val"].as_integer(), Some(42));
    }

    #[test]
    fn test_current_timestamp() {
        let ts = current_timestamp();
        assert!(ts > 1_700_000_000); // Sanity check
    }
}

/// Sends a desktop notification if notify-send is available.
/// Used for graceful error reporting when the TUI is closed or as a popup.
pub fn notify_error(title: &str, message: &str) {
    let _ = std::process::Command::new("notify-send")
        .arg("-a")
        .arg("LaTUI")
        .arg("-i")
        .arg("dialog-error")
        .arg(title)
        .arg(message)
        .spawn();
}
