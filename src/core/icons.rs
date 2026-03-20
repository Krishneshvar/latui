use std::path::{Path, PathBuf};
use freedesktop_desktop_entry::DesktopEntry;

/// Resolves an icon name or path to an absolute path on disk.
/// Uses freedesktop-icons for standard icon theme lookup.
pub fn resolve_desktop_icon_path(icon_name: &str) -> Option<PathBuf> {
    let icon_name = icon_name.trim();
    if icon_name.is_empty() {
        return None;
    }

    let direct = Path::new(icon_name);
    if direct.is_absolute() && direct.exists() {
        return Some(direct.to_path_buf());
    }

    let theme = std::env::var("LATUI_ICON_THEME")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .or_else(freedesktop_icons::default_theme_gtk)
        .unwrap_or_else(|| "hicolor".to_string());

    freedesktop_icons::lookup(icon_name)
        .with_size(96)
        .with_scale(1)
        .with_theme(&theme)
        .with_cache()
        .find()
}

/// Resolves the icon path for a specific desktop entry file.
pub fn resolve_icon_from_entry(desktop_path: &Path) -> Option<PathBuf> {
    if !desktop_path.exists() {
        return None;
    }

    let entry = DesktopEntry::from_path(desktop_path, None::<&[&str]>).ok()?;
    let icon_name = entry.icon()?;
    resolve_desktop_icon_path(icon_name)
}
