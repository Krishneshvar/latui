use std::fs;
use std::path::PathBuf;

use walkdir::WalkDir;

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
                if let Some(item) = parse_desktop_file(path) {
                    items.push(item);
                }
            }
        }
    }

    items
}

fn parse_desktop_file(path: &std::path::Path) -> Option<Item> {
    let content = fs::read_to_string(path).ok()?;

    let mut name = None;
    let mut exec = None;
    let mut no_display = false;

    for line in content.lines() {

        if line.starts_with("Name=") && name.is_none() {
            name = Some(line.trim_start_matches("Name=").to_string());
        }

        if line.starts_with("Exec=") && exec.is_none() {
            exec = Some(line.trim_start_matches("Exec=").to_string());
        }

        if line.starts_with("NoDisplay=true") {
            no_display = true;
        }
    }

    if no_display {
        return None;
    }

    let name = name?;
    let exec = exec?;

    let exec = sanitize_exec(&exec);

    Some(Item {
        id: path.to_string_lossy().to_string(),
        title: name,
        description: None,
        score: 0,
        action: Action::Launch(exec),
    })
}

fn sanitize_exec(exec: &str) -> String {
    exec.split_whitespace()
        .filter(|part| !part.starts_with('%'))
        .collect::<Vec<_>>()
        .join(" ")
}
