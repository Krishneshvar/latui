use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{warn, info};
use crate::config::theme::AppConfig;
use crate::ui::bundled_themes;
use crate::core::utils::latui_xdg;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ModesSettings {
    #[serde(default)]
    pub apps: AppsModeSettings,
    #[serde(default)]
    pub custom: std::collections::HashMap<String, CustomModeConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CustomModeConfig {
    pub name: String,
    pub icon: String,
    pub description: String,
    pub list_cmd: String,
    pub exec_cmd: String,
    #[serde(default)]
    pub stays_open: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppsModeSettings {
    #[serde(default = "default_desktop_dirs")]
    pub desktop_dirs: Vec<String>,
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub skip_terminal_apps: bool,
    #[serde(default)]
    pub icons: AppsIconSettings,
}

impl Default for AppsModeSettings {
    fn default() -> Self {
        Self {
            desktop_dirs: default_desktop_dirs(),
            include: Vec::new(),
            exclude: Vec::new(),
            skip_terminal_apps: false,
            icons: AppsIconSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
pub enum AppsIconRenderMode {
    #[default]
    Thumbnail,
    IconName,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppsIconSettings {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default = "default_icon_size")]
    pub size: u16,
    #[serde(default = "default_icon_scale")]
    pub scale: u16,
    #[serde(default)]
    pub prefer_svg: bool,
    #[serde(default = "default_icon_fallback")]
    pub fallback: String,
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub render_mode: AppsIconRenderMode,
}

impl Default for AppsIconSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            theme: None,
            size: default_icon_size(),
            scale: default_icon_scale(),
            prefer_svg: false,
            fallback: default_icon_fallback(),
            include: Vec::new(),
            exclude: Vec::new(),
            render_mode: AppsIconRenderMode::Thumbnail,
        }
    }
}

pub fn load_user_settings_path() -> Option<PathBuf> {
    let xdg = latui_xdg();
    xdg.find_config_file("config.toml")
}

/// Loads the user configuration and applies the selected theme cascade.
pub fn load_user_settings() -> AppConfig {
    let mut config = AppConfig::default();

    // 1. Try to load user's config.toml
    let raw_toml = load_user_settings_path().and_then(|path| {
        std::fs::read_to_string(&path)
            .map_err(|e| warn!("Failed to read config file {}: {}", path.display(), e))
            .ok()
    });

    if let Some(ref content) = raw_toml {
        match toml::from_str::<AppConfig>(content) {
            Ok(user_cfg) => {
                config = user_cfg;
                info!("Loaded user configuration");
            }
            Err(e) => {
                warn!("Failed to parse config file: {}. Using defaults.", e);
            }
        }
    }

    // 2. Resolve theme if not "inline"
    let theme_name = &config.general.theme;
    if theme_name != "inline" && let Some(theme_cfg) = load_theme(theme_name) {
        // Re-parse the user config to identify what was actually there
        // Reuse the content we already read
        if let Some(content) = raw_toml
            && let Ok(user_toml_value) = toml::from_str::<toml::Value>(&content) {
                // Start with theme defaults
                let mut final_toml = toml::Value::try_from(theme_cfg.clone())
                    .unwrap_or_else(|_| toml::Value::Table(toml::map::Map::default()));
                
                // Deep merge user overrides on top of theme
                crate::core::utils::merge_toml(&mut final_toml, user_toml_value);
                
                // Convert back to AppConfig
                if let Ok(merged_cfg) = final_toml.try_into() {
                    return merged_cfg;
                }
        }
        return theme_cfg;
    }

    config
}

fn load_theme(name: &str) -> Option<AppConfig> {
    // 1. Try ~/.config/latui/themes/<name>.toml
    let xdg = latui_xdg();
    if let Some(theme_path) = xdg.find_config_file(format!("themes/{name}.toml"))
        && let Ok(content) = std::fs::read_to_string(&theme_path)
            && let Ok(cfg) = toml::from_str::<AppConfig>(&content) {
                info!("Loaded theme '{}' from {}", name, theme_path.display());
                return Some(cfg);
            }

    // 2. Try absolute path if name looks like a path
    let path = Path::new(name);
    if path.is_absolute() && path.exists()
        && let Ok(content) = std::fs::read_to_string(path)
            && let Ok(cfg) = toml::from_str::<AppConfig>(&content) {
                info!("Loaded theme from absolute path {}", path.display());
                return Some(cfg);
            }

    // 3. Try bundled themes
    let bundled = match name {
        "dark" => Some(bundled_themes::DARK),
        "light" => Some(bundled_themes::LIGHT),
        _ => None,
    };

    if let Some(content) = bundled
        && let Ok(cfg) = toml::from_str::<AppConfig>(content) {
            info!("Loaded bundled theme '{}'", name);
            return Some(cfg);
        }

    warn!("Theme '{}' not found", name);
    None
}

const fn default_true() -> bool {
    true
}

const fn default_icon_size() -> u16 {
    24
}

const fn default_icon_scale() -> u16 {
    1
}

fn default_icon_fallback() -> String {
    "📦".to_string()
}

fn default_desktop_dirs() -> Vec<String> {
    let mut dirs: Vec<String> = Vec::new();
    let xdg = latui_xdg();

    if let Some(home) = xdg.get_data_home() {
        dirs.push(home.join("applications").to_string_lossy().to_string());
    }

    for dir in xdg.get_data_dirs() {
        dirs.push(dir.join("applications").to_string_lossy().to_string());
    }

    dirs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modes_settings_defaults() {
        let toml_str = "";
        let settings: ModesSettings = toml::from_str(toml_str).unwrap();

        assert!(settings.apps.include.is_empty());
        assert!(!settings.apps.skip_terminal_apps);
        assert!(settings.apps.icons.enabled);
        assert_eq!(settings.apps.icons.render_mode, AppsIconRenderMode::Thumbnail);
        assert_eq!(settings.custom.len(), 0);
    }

    #[test]
    fn test_modes_settings_overrides() {
        let toml_str = r#"
        [apps]
        skip_terminal_apps = true
        exclude = ["kitty"]

        [apps.icons]
        render_mode = "icon_name"
        size = 32

        [custom.my_power_menu]
        name = "Power"
        icon = "⏻"
        description = "Session management"
        list_cmd = "echo 'Shutdown\nReboot'"
        exec_cmd = "echo {}"
        stays_open = false
        "#;
        let settings: ModesSettings = toml::from_str(toml_str).unwrap();

        assert!(settings.apps.skip_terminal_apps);
        assert_eq!(settings.apps.exclude.len(), 1);
        assert_eq!(settings.apps.icons.size, 32);
        assert_eq!(settings.apps.icons.render_mode, AppsIconRenderMode::IconName);
        
        assert_eq!(settings.custom.len(), 1);
        let custom = settings.custom.get("my_power_menu").unwrap();
        assert_eq!(custom.name, "Power");
        assert_eq!(custom.exec_cmd, "echo {}");
    }
}
