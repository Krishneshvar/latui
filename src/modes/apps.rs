use crate::core::{item::Item, mode::Mode, searchable_item::SearchableItem};
use crate::error::LatuiError;
use crate::cache::apps_cache::{load_cache, save_cache};
use crate::tracking::frequency::FrequencyTracker;
use crate::index::trie::MultiTokenTrie;

use freedesktop_desktop_entry::DesktopEntry;
use walkdir::WalkDir;

use std::path::PathBuf;
use std::process::Command;

pub struct AppsMode {
    items: Vec<SearchableItem>,
    trie: Option<MultiTokenTrie>,
    frequency_tracker: Option<FrequencyTracker>,
    keyword_mapper: crate::config::keywords::KeywordMapper,
    search_engine: crate::search::engine::SearchEngine,
    last_action_time: Option<std::time::Instant>,
}

impl AppsMode {
    pub fn new(
        frequency_tracker: Option<FrequencyTracker>,
        keyword_mapper: crate::config::keywords::KeywordMapper,
    ) -> Self {
        Self {
            items: Vec::new(),
            trie: None,
            frequency_tracker,
            keyword_mapper,
            search_engine: crate::search::engine::SearchEngine::new(),
            last_action_time: None,
        }
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
            tracing::debug!("Scanning directory for desktop files: {:?}", dir);

            let base_dir = dir.clone();
            for entry in WalkDir::new(&dir)
                .into_iter()
                .filter_map(Result::ok)
            {
                let path = entry.path();

                // Validation: Only .desktop files, no symlinks (prevent symlink attacks)
                // Also ensure the path is actually inside our search directories
                if path.extension().map(|e| e == "desktop").unwrap_or(false) 
                    && !path.is_symlink() 
                    && path.starts_with(&base_dir)
                {
                    match DesktopEntry::from_path(path, None::<&[&str]>) {
                        Ok(entry) => {
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
                            metadata: Some(exec.clone()),
                        };

                        // Create SearchableItem with all fields
                        let mut searchable = SearchableItem::new(item)
                            .with_field("name", &name, 10.0);
                        
                        if let Some(gn) = generic_name {
                            searchable = searchable.with_field("generic_name", &gn, 7.0);
                        }
                        
                        for keyword in keywords {
                            searchable = searchable.with_field("keyword", &keyword, 8.0);
                        }
                        
                        for category in categories {
                            searchable = searchable.with_field("category", &category, 5.0);
                        }
                        
                        if let Some(desc) = description {
                            searchable = searchable.with_field("description", &desc, 3.0);
                        }
                        
                        searchable = searchable.with_field("executable", &executable, 2.0);
                        
                        items.push(searchable);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse desktop file {}: {}", path.display(), e);
                    }
                }
            }
        }
        }

        items
    }
}

impl Mode for AppsMode {
    fn name(&self) -> &str { "apps" }
    fn icon(&self) -> &str { "🔥" }
    fn description(&self) -> &str { "Applications" }

    fn load(&mut self) -> Result<(), LatuiError> {
        match load_cache() {
            Ok(cached) => {
                tracing::debug!("Loaded {} items from cache", cached.len());
                self.items = cached;
                // Build trie from cached items
                self.trie = Some(MultiTokenTrie::build(&self.items));
                return Ok(());
            }
            Err(e) => {
                tracing::warn!("Cache load failed, rebuilding: {}", e);
            }
        }

        let items = Self::build_index();

        if let Err(e) = save_cache(&items) {
            tracing::error!("Failed to save built index to cache: {}", e);
        }

        tracing::info!("Indexing complete. Ingested {} applications.", items.len());

        // Build trie from items
        self.trie = Some(MultiTokenTrie::build(&items));
        self.items = items;
        Ok(())
    }

    fn search(&mut self, query: &str) -> Vec<Item> {
        if query.is_empty() {
            let mut scored_all: Vec<(usize, f64)> = self.items.iter().enumerate().map(|(idx, searchable)| {
                let mut score = 0.0;
                if let Some(ref tracker) = self.frequency_tracker {
                    score += tracker.get_total_boost(&searchable.item.id);
                }
                (idx, score)
            }).collect();

            // Sort by score (descending)
            scored_all.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            
            return scored_all.into_iter()
                .map(|(idx, _)| self.items[idx].item.clone())
                .collect();
        }

        use crate::search::tokenizer::Tokenizer;
        
        let start = std::time::Instant::now();
        let tokenizer = Tokenizer::new();
        let q = query.to_lowercase();
        let query_tokens = tokenizer.tokenize(&q);

        // Get candidate indices from trie (fast prefix filtering)
        let mut candidate_indices = if let Some(ref trie) = self.trie {
            if query_tokens.len() > 1 {
                trie.get_multi_token_candidates(&query_tokens)
            } else {
                trie.get_candidates(&q)
            }
        } else {
            (0..self.items.len()).collect()
        };

        // Add candidates from keyword mappings
        if let Some(mapped_apps) = self.keyword_mapper.get_matches(&q) {
            for app_needle in mapped_apps {
                for (idx, item) in self.items.iter().enumerate() {
                    // This is a bit slow, but keyword mappings are usually for a few apps
                    if item.item.title.to_lowercase().contains(app_needle)
                        && !candidate_indices.contains(&idx) {
                            candidate_indices.push(idx);
                        }
                }
            }
        }

        if candidate_indices.is_empty() {
            return Vec::new();
        }

        // Delegate to SearchEngine for candidate scoring
        let candidates: Vec<SearchableItem> = candidate_indices.iter()
            .map(|&idx| self.items[idx].clone())
            .collect();
            
        let mut scored_results = self.search_engine.search_scored(query, &candidates);
        
        // Re-apply boosts to the results
        if let Some(ref tracker) = self.frequency_tracker {
            for (item, score) in scored_results.iter_mut() {
                // Frequency boost (0-100 points)
                *score += tracker.get_frequency_boost(&item.id);
                // Recency boost (0-50 points)
                *score += tracker.get_recency_boost(&item.id);
                // Query-specific boost (0-50 points)
                *score += tracker.get_query_boost(&q, &item.id);
            }
            
            // Re-sort after applying boosts
            scored_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        }

        let results: Vec<Item> = scored_results.into_iter().map(|(item, _)| item).collect();
        tracing::trace!("Search for '{}' completed in {:?} with {} results", query, start.elapsed(), results.len());
        results
    }

    fn execute(&mut self, item: &Item) -> Result<(), LatuiError> {
        // Rate limiting for execution to prevent spamming processes
        if let Some(last) = self.last_action_time
            && last.elapsed() < std::time::Duration::from_millis(500) {
                tracing::warn!("Rate limiting execution for item: {}", item.title);
                return Ok(());
            }
        self.last_action_time = Some(std::time::Instant::now());

        // Record the launch in frequency tracker
        if let Some(ref mut tracker) = self.frequency_tracker
            && let Err(e) = tracker.record_launch(&item.id) {
                tracing::error!("Failed to record launch tracking: {}", e);
            }

        if let Some(cmd) = &item.metadata {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            if parts.is_empty() {
                return Ok(());
            }

            // Security: Avoid shell injection by executing directly if there are no shell metacharacters
            let shell_chars = [';', '&', '|', '<', '>', '(', ')', '$', '`', '\\', '"', '\'', '*', '?', '[', ']', '~', '!'];
            let has_shell_chars = cmd.chars().any(|c| shell_chars.contains(&c));

            let child = if !has_shell_chars {
                // Safe direct execution
                Command::new(parts[0])
                    .args(&parts[1..])
                    .spawn()
            } else {
                // Fallback to shell with warning
                tracing::warn!("Executing command with shell features: {}", cmd);
                Command::new("sh")
                    .arg("-c")
                    .arg(cmd)
                    .spawn()
            };

            if let Err(e) = child {
                tracing::error!("Failed to execute '{}': {}", cmd, e);
                return Err(LatuiError::Io(e));
            }
        } else {
            tracing::warn!("Apps mode received item without metadata (command): {}", item.title);
            return Err(LatuiError::App("Missing command metadata for execution".to_string()));
        }
        Ok(())
    }

    fn record_selection(&mut self, query: &str, item: &Item) {
        // Rate limiting: 5 selections per second max
        if let Some(last) = self.last_action_time
            && last.elapsed() < std::time::Duration::from_millis(200) {
                return;
            }
        self.last_action_time = Some(std::time::Instant::now());

        if let Some(ref mut tracker) = self.frequency_tracker
            && let Err(e) = tracker.record_selection(query, &item.id) {
                tracing::error!("Failed to record selection tracking: {}", e);
            }
    }
}

fn sanitize_exec(exec: &str) -> String {

    exec.split_whitespace()
        .filter(|part| !part.starts_with('%'))
        .collect::<Vec<_>>()
        .join(" ")
}
