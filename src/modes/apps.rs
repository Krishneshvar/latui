use crate::core::{action::Action, item::Item, mode::Mode, searchable_item::SearchableItem};
use crate::error::LatuiError;
use crate::cache::apps_cache::{load_cache, save_cache};
use crate::matcher::fuzzy::FuzzyMatcher;
use crate::search::typo::TypoTolerance;
use crate::tracking::frequency::FrequencyTracker;
use crate::index::trie::MultiTokenTrie;

use freedesktop_desktop_entry::DesktopEntry;
use walkdir::WalkDir;

use std::path::PathBuf;
use std::process::Command;

pub struct AppsMode {
    items: Vec<SearchableItem>,
    trie: Option<MultiTokenTrie>,
    typo_tolerance: TypoTolerance,
    frequency_tracker: Option<FrequencyTracker>,
    keyword_mapper: crate::config::keywords::KeywordMapper,
    fuzzy_matcher: FuzzyMatcher,
    last_action_time: Option<std::time::Instant>,
}

impl AppsMode {
    pub fn new() -> Self {
        // Initialize frequency tracker
        let mut frequency_tracker = Self::init_frequency_tracker();
        
        // Cleanup old stats (keep 30 days)
        if let Some(ref mut tracker) = frequency_tracker {
            if let Err(e) = tracker.cleanup(30) {
                tracing::warn!("Failed to cleanup old usage data: {}", e);
            }
        }
        
        Self {
            items: Vec::new(),
            trie: None,
            typo_tolerance: TypoTolerance::new(),
            frequency_tracker,
            keyword_mapper: Self::init_keyword_mapper(),
            fuzzy_matcher: FuzzyMatcher::new(),
            last_action_time: None,
        }
    }
    
    /// Initialize frequency tracker with database
    fn init_frequency_tracker() -> Option<FrequencyTracker> {
        use xdg::BaseDirectories;
        
        let xdg = BaseDirectories::with_prefix("latui");
        let db_path = match xdg.place_data_file("usage.db") {
            Ok(p) => p,
            Err(e) => {
                tracing::error!("Failed to generate usage tracking path: {}", e);
                return None;
            }
        };
        
        match FrequencyTracker::new(&db_path) {
            Ok(tracker) => Some(tracker),
            Err(e) => {
                tracing::error!("Failed to initialize usage tracker DB: {}", e);
                None
            }
        }
    }

    fn init_keyword_mapper() -> crate::config::keywords::KeywordMapper {
        use crate::config::keywords::KeywordMapper;
        use crate::config::loader::load_user_config_path;
        use std::fs;

        let mapper = KeywordMapper::with_defaults();

        if let Some(path) = load_user_config_path() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    match KeywordMapper::from_toml(&content) {
                        Ok(custom_mapper) => {
                            // Merge custom mappings into defaults
                            // For simplicity, we can just use the custom_mapper if it successfully parsed
                            // or merge them if KeywordMapper supported merging.
                            // Let's at least log it.
                            tracing::info!("Loaded custom keywords from {:?}", path);
                            return custom_mapper;
                        }
                        Err(e) => tracing::error!("Failed to parse keywords TOML: {}", e),
                    }
                }
                Err(e) => tracing::error!("Failed to read keywords file: {}", e),
            }
        }

        mapper
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
                            action: Action::Launch(exec.clone()),
                        };

                        // Create SearchableItem with all fields
                        match SearchableItem::new(
                            item,
                            name.to_lowercase(),
                            keywords,
                            categories,
                            generic_name,
                            description,
                            executable,
                        ) {
                            Ok(searchable) => {
                                items.push(searchable);
                            }
                            Err(e) => {
                                tracing::warn!("Failed to ingest {}: {}", path.display(), e);
                            }
                        }
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
        if query.is_empty() || query.len() > 128 {
            return if query.is_empty() {
                let mut scored_all: Vec<(usize, f64)> = self.items.iter().enumerate().map(|(idx, searchable)| {
                    let mut score = 0.0;
                    if let Some(ref tracker) = self.frequency_tracker {
                        score += tracker.get_total_boost(&searchable.item.id);
                    }
                    (idx, score)
                }).collect();

                // Sort by score (descending)
                scored_all.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                
                scored_all.into_iter()
                    .map(|(idx, _)| self.items[idx].item.clone())
                    .collect()
            } else {
                Vec::new()
            };
        }

        use crate::search::tokenizer::Tokenizer;
        
        let start = std::time::Instant::now();
        let tokenizer = Tokenizer::new();
        let q = query.to_lowercase();
        let query_tokens = tokenizer.tokenize(&q);

        // Get candidate indices from trie (fast prefix filtering)
        let mut candidate_indices = if let Some(ref trie) = self.trie {
            if query_tokens.len() > 1 {
                // For multi-token queries, use AND logic (all tokens must match a prefix)
                trie.get_multi_token_candidates(&query_tokens)
            } else {
                // For single token, just get candidates for the query
                trie.get_candidates(&q)
            }
        } else {
            // Fallback: search all items if trie not built
            (0..self.items.len()).collect()
        };

        // Add candidates from keyword mappings (e.g., "browser" -> "firefox")
        if let Some(mapped_apps) = self.keyword_mapper.get_matches(&q) {
            for app_needle in mapped_apps {
                for (idx, item) in self.items.iter().enumerate() {
                    if item.name.to_lowercase().contains(app_needle) {
                        if !candidate_indices.contains(&idx) {
                            candidate_indices.push(idx);
                        }
                    }
                }
            }
        }

        tracing::trace!("Search query '{}' yielded {} candidates", query, candidate_indices.len());

        // If no candidates from trie, return empty
        if candidate_indices.is_empty() {
            return Vec::new();
        }

        // Collect candidates with their scores (only score trie candidates)
        let mut scored_items: Vec<(usize, f64)> = Vec::new();

        for idx in candidate_indices {
            let searchable = &self.items[idx];
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
                    
                    // Typo tolerance
                    if field_score == 0.0 {
                        // Check typo match against field text
                        if let Some(typo_score) = self.typo_tolerance.score(&q, &field_text) {
                            field_score = typo_score;
                        }
                        // Also check against individual tokens
                        else {
                            for token in field.tokens.iter() {
                                if let Some(typo_score) = self.typo_tolerance.score(&q, token) {
                                    field_score = field_score.max(typo_score);
                                }
                            }
                        }
                    }
                    
                    // Substring match
                    if field_score == 0.0 && field_text.contains(&q) {
                        field_score = 100.0;
                    }
                    
                    // Fuzzy match as fallback
                    if field_score == 0.0 {
                        let results = self.fuzzy_matcher.filter(&q, &[&field_text]);
                        if let Some((_, score)) = results.first() {
                            field_score = (*score as f64).min(200.0);
                        }
                    }
                }

                // Apply field weight
                let weighted_score = field_score * field.weight;
                best_score = best_score.max(weighted_score);
            }

            // Add frequency and recency boosts
            if let Some(ref tracker) = self.frequency_tracker {
                let app_id = &searchable.item.id;
                
                // Frequency boost (0-100 points)
                let frequency_boost = tracker.get_frequency_boost(app_id);
                
                // Recency boost (0-50 points)
                let recency_boost = tracker.get_recency_boost(app_id);
                
                // Query-specific boost (0-50 points)
                let query_boost = tracker.get_query_boost(&q, app_id);
                
                // Add all boosts to the score
                best_score += frequency_boost + recency_boost + query_boost;
            }

            if best_score > 0.0 {
                scored_items.push((idx, best_score));
            }
        }

        // Sort by score (descending)
        scored_items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Return items in sorted order
        let results: Vec<Item> = scored_items
            .iter()
            .map(|(idx, _)| self.items[*idx].item.clone())
            .collect();

        tracing::trace!("Search for '{}' completed in {:?} with {} results", query, start.elapsed(), results.len());
        results
    }

    fn execute(&mut self, item: &Item) -> Result<(), LatuiError> {
        // Rate limiting for execution to prevent spamming processes
        if let Some(last) = self.last_action_time {
            if last.elapsed() < std::time::Duration::from_millis(500) {
                tracing::warn!("Rate limiting execution for item: {}", item.title);
                return Ok(());
            }
        }
        self.last_action_time = Some(std::time::Instant::now());

        // Record the launch in frequency tracker
        if let Some(ref mut tracker) = self.frequency_tracker {
            if let Err(e) = tracker.record_launch(&item.id) {
                tracing::error!("Failed to record launch tracking: {}", e);
            }
        }

        match &item.action {
            Action::Launch(cmd) | Action::Command(cmd) => {
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
            }
            _ => {
                tracing::warn!("Apps mode received unsupported action type: {:?}", item.action);
                return Err(LatuiError::App("Unsupported action type for Apps mode".to_string()));
            }
        }
        Ok(())
    }

    fn record_selection(&mut self, query: &str, item: &Item) {
        // Rate limiting: 5 selections per second max
        if let Some(last) = self.last_action_time {
            if last.elapsed() < std::time::Duration::from_millis(200) {
                return;
            }
        }
        self.last_action_time = Some(std::time::Instant::now());

        if let Some(ref mut tracker) = self.frequency_tracker {
            if let Err(e) = tracker.record_selection(query, &item.id) {
                tracing::error!("Failed to record selection tracking: {}", e);
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
