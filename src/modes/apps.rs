use crate::core::{action::Action, item::Item, mode::Mode, searchable_item::SearchableItem};
use crate::cache::apps_cache::{load_cache, save_cache};
use crate::matcher::fuzzy::FuzzyMatcher;

use freedesktop_desktop_entry::DesktopEntry;
use walkdir::WalkDir;

use std::path::PathBuf;
use std::process::Command;

pub struct AppsMode {
    items: Vec<SearchableItem>,
}

impl AppsMode {

    pub fn new() -> Self {
        Self {
            items: Vec::new(),
        }
    }
    
    /// Score acronym matches
    fn score_acronym_match(&self, query: &str, item: &SearchableItem) -> f64 {
        // Check if query matches any acronym
        for acronym in &item.acronyms {
            if acronym == query {
                // Exact acronym match gets high score with name field weight
                return 250.0 * 10.0; // 2500.0
            } else if acronym.starts_with(query) {
                // Prefix acronym match
                return 200.0 * 10.0; // 2000.0
            }
        }
        0.0
    }

    fn build_index() -> Vec<SearchableItem> {

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

                        // Extract keywords from desktop entry
                        let keywords: Vec<String> = entry
                            .keywords::<&str>(&[])
                            .map(|k| k.iter()
                                .map(|s| s.to_lowercase())
                                .collect())
                            .unwrap_or_default();

                        // Extract categories
                        let categories: Vec<String> = entry
                            .categories()
                            .map(|cats| cats.iter()
                                .map(|s| s.to_lowercase())
                                .collect())
                            .unwrap_or_default();

                        // Extract generic name
                        let generic_name = entry
                            .generic_name::<&str>(&[])
                            .map(|g| g.to_lowercase());

                        // Extract comment/description
                        let description = entry
                            .comment::<&str>(&[])
                            .map(|c| c.to_lowercase());

                        // Extract executable name (first part of exec)
                        let executable = exec
                            .split_whitespace()
                            .next()
                            .unwrap_or(&exec)
                            .to_lowercase();

                        // Create the base Item
                        let item = Item {
                            id: path.to_string_lossy().to_string(),
                            title: name.clone(),
                            search_text: name.to_lowercase(),
                            description: description.clone(),
                            action: Action::Launch(exec.clone()),
                        };

                        // Create SearchableItem with all fields
                        let searchable = SearchableItem::new(
                            item,
                            name.to_lowercase(),
                            keywords,
                            categories,
                            generic_name,
                            description,
                            executable,
                        );

                        items.push(searchable);
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
            return self.items.iter().map(|s| s.item.clone()).collect();
        }

        use crate::search::tokenizer::Tokenizer;
        
        let tokenizer = Tokenizer::new();
        let q = query.to_lowercase();
        let query_tokens = tokenizer.tokenize(&q);

        // Collect candidates with their scores
        let mut scored_items: Vec<(usize, f64)> = Vec::new();

        for (idx, searchable) in self.items.iter().enumerate() {
            let mut best_score: f64 = 0.0;

            // Check acronym match first (high priority)
            let acronym_score = self.score_acronym_match(&q, searchable);
            best_score = best_score.max(acronym_score);

            // Get all weighted fields
            let fields = searchable.get_weighted_fields();

            // Score each field
            for field in fields {
                let field_text = field.text.to_lowercase();
                let mut field_score = 0.0;

                // Exact match (highest priority)
                if field_text == q {
                    field_score = 1000.0;
                }
                // Prefix match
                else if field_text.starts_with(&q) {
                    field_score = 500.0;
                }
                // Token-based matching
                else {
                    // Check if query matches any token exactly
                    let token_exact = field.tokens.iter().any(|t| t == &q);
                    if token_exact {
                        field_score = 400.0;
                    }
                    // Check if query is prefix of any token
                    else if field.tokens.iter().any(|t| t.starts_with(&q)) {
                        field_score = 350.0;
                    }
                    // Word boundary match
                    else if field_text.split_whitespace().any(|word| word.starts_with(&q)) {
                        field_score = 300.0;
                    }
                    // Multi-token match (all query tokens match)
                    else if !query_tokens.is_empty() {
                        let all_match = query_tokens.iter().all(|qt| {
                            field.tokens.iter().any(|ft| ft.contains(qt))
                        });
                        if all_match {
                            field_score = 250.0;
                        }
                    }
                    
                    // Substring match
                    if field_score == 0.0 && field_text.contains(&q) {
                        field_score = 100.0;
                    }
                    
                    // Fuzzy match as fallback
                    if field_score == 0.0 {
                        let mut matcher = FuzzyMatcher::new();
                        let results = matcher.filter(&q, &[&field_text]);
                        if let Some((_, score)) = results.first() {
                            field_score = (*score as f64).min(200.0);
                        }
                    }
                }

                // Apply field weight
                let weighted_score = field_score * field.weight;
                best_score = best_score.max(weighted_score);
            }

            if best_score > 0.0 {
                scored_items.push((idx, best_score));
            }
        }

        // Sort by score (descending)
        scored_items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Return items in sorted order
        scored_items
            .iter()
            .map(|(idx, _)| self.items[*idx].item.clone())
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
