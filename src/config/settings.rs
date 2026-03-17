use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::warn;
use xdg::BaseDirectories;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct UserSettings {
    #[serde(default)]
    pub modes: ModesSettings,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ModesSettings {
    #[serde(default)]
    pub apps: AppsModeSettings,
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
    let xdg = BaseDirectories::with_prefix("latui");
    xdg.find_config_file("config.toml")
}

pub fn load_user_settings() -> Option<UserSettings> {
    let path = load_user_settings_path()?;
    match std::fs::read_to_string(&path) {
        Ok(content) => match toml::from_str::<UserSettings>(&content) {
            Ok(config) => Some(config),
            Err(error) => {
                warn!(
                    "Failed to parse settings file {}: {}",
                    path.display(),
                    error
                );
                None
            }
        },
        Err(error) => {
            warn!("Failed to read settings file {}: {}", path.display(), error);
            None
        }
    }
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
    let mut dirs = Vec::new();

    if let Ok(home) = std::env::var("HOME") {
        dirs.push(format!("{home}/.local/share/applications"));
    }

    dirs.push("/usr/local/share/applications".to_string());
    dirs.push("/usr/share/applications".to_string());

    dirs
}
