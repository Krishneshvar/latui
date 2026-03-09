use std::path::PathBuf;

use walkdir::WalkDir;
use freedesktop_desktop_entry::DesktopEntry;

use crate::core::{action::Action, item::Item};

pub fn load_apps() -> Vec<Item> {
    let mut items = Vec::new();

    let dirs = vec![
        "/usr/share/applications",
        "/usr/local/share/applications",
    ];

    let home = std::env::var("HOME").unwrap_or_default();
    let user_dir = format!("{}/.local/share/applications", home);

    let mut all_dirs: Vec<PathBuf> = dirs.into_iter().map(PathBuf::from).collect();
    all_dirs.push(PathBuf::from(user_dir));

    for dir in all_dirs {
        if !dir.exists() {
            continue;
        }

        for entry in WalkDir::new(dir)
            .into_iter()
            .filter_map(Result::ok)
        {
            let path = entry.path();

            if path.extension().map(|e| e == "desktop").unwrap_or(false) {

                if let Ok(entry) = DesktopEntry::from_path(path) {

                    if entry.no_display() {
                        continue;
                    }

                    let name = entry.name(None).unwrap_or("").to_string();

                    let exec = entry.exec().unwrap_or("").to_string();

                    if name.is_empty() || exec.is_empty() {
                        continue;
                    }

                    let exec = sanitize_exec(&exec);

                    items.push(Item {
                        id: path.to_string_lossy().to_string(),
                        title: name,
                        description: None,
                        score: 0,
                        action: Action::Launch(exec),
                    });
                }
            }
        }
    }

    items
}

fn sanitize_exec(exec: &str) -> String {
    exec.split_whitespace()
        .filter(|part| !part.starts_with('%'))
        .collect::<Vec<_>>()
        .join(" ")
}
