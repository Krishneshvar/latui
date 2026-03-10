use crate::core::{action::Action, item::Item, mode::Mode};
use crate::cache::apps_cache::{load_cache, save_cache};

use freedesktop_desktop_entry::DesktopEntry;
use walkdir::WalkDir;

use std::path::PathBuf;
use std::process::Command;

pub struct AppsMode {
    items: Vec<Item>,
}

impl AppsMode {

    pub fn new() -> Self {
        Self {
            items: Vec::new(),
        }
    }

    fn build_index() -> Vec<Item> {

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

                    if let Ok(entry) = DesktopEntry::from_path(path, None::<&[&str]>) {

                        if entry.no_display() {
                            continue;
                        }

                        let name = entry
                            .name::<&str>(&[])
                            .map(|n| n.to_string())
                            .unwrap_or_default();

                        let exec = entry
                            .exec()
                            .map(|e| e.to_string())
                            .unwrap_or_default();

                        if name.is_empty() || exec.is_empty() {
                            continue;
                        }

                        let exec = sanitize_exec(&exec);

                        items.push(Item {
                            id: path.to_string_lossy().to_string(),
                            title: name.clone(),
                            search_text: name.to_lowercase(),
                            description: None,
                            action: Action::Launch(exec),
                        });
                    }
                }
            }
        }

        items
    }
}

impl Mode for AppsMode {

    fn name(&self) -> &str {
        "apps"
    }

    fn load(&mut self) {

        if let Some(cached) = load_cache() {
            self.items = cached;
            return;
        }

        let items = Self::build_index();

        save_cache(&items);

        self.items = items;
    }

    fn search(&self, query: &str) -> Vec<Item> {

        if query.is_empty() {
            return self.items.clone();
        }

        let q = query.to_lowercase();

        self.items
            .iter()
            .filter(|i| i.search_text.starts_with(&q))
            .cloned()
            .collect()
    }

    fn execute(&self, item: &Item) {

        match &item.action {

            Action::Launch(cmd) | Action::Command(cmd) => {

                Command::new("sh")
                    .arg("-c")
                    .arg(cmd)
                    .spawn()
                    .ok();
            }
        }
    }
}

fn sanitize_exec(exec: &str) -> String {

    exec.split_whitespace()
        .filter(|part| !part.starts_with('%'))
        .collect::<Vec<_>>()
        .join(" ")
}
