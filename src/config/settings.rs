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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
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
    if config.general.theme != "inline" {
        let theme_name = config.general.theme.clone();
        if let Some(theme_cfg) = load_theme(&theme_name) {
            // Re-parse the user config to identify what was actually there
            // Reuse the content we already read
            if let Some(content) = raw_toml
                && let Ok(toml_value) = toml::from_str::<toml::Value>(&content) {
                    let mut final_config = theme_cfg;
                    
                    if let Some(general) = toml_value.get("general")
                        && let Ok(c) = general.clone().try_into() { final_config.general = c; }
                    if let Some(layout) = toml_value.get("layout")
                      && let Ok(c) = layout.clone().try_into() { final_config.layout = c; }
                    if let Some(navbar) = toml_value.get("navbar")
                        && let Ok(c) = navbar.clone().try_into() { final_config.navbar = c; }
                    if let Some(search) = toml_value.get("search")
                        && let Ok(c) = search.clone().try_into() { final_config.search = c; }
                    if let Some(results) = toml_value.get("results")
                        && let Ok(c) = results.clone().try_into() { final_config.results = c; }
                    if let Some(modes) = toml_value.get("modes")
                        && let Ok(c) = modes.clone().try_into() { final_config.modes = c; }
                    
                    return final_config;
                }
            return theme_cfg;
        }
    }

    config
}

fn load_theme(name: &str) -> Option<AppConfig> {
    // 1. Try ~/.config/latui/themes/<name>.toml
    let xdg = latui_xdg();
    if let Some(theme_path) = xdg.find_config_file(format!("themes/{}.toml", name))
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

fn default_true() -> bool {
    true
}

fn default_icon_size() -> u16 {
    24
}

fn default_icon_scale() -> u16 {
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
