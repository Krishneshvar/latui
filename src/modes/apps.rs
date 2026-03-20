use crate::cache::apps_cache::{APPS_CACHE_SCHEMA_VERSION, load_cache, save_cache};
use crate::config::settings::{AppsIconRenderMode, AppsModeSettings};
use crate::core::{item::Item, mode::Mode, searchable_item::SearchableItem};
use crate::error::LatuiError;
use crate::index::trie::MultiTokenTrie;
use crate::tracking::frequency::FrequencyTracker;

use freedesktop_desktop_entry::DesktopEntry;
use image::ImageReader;
use image::imageops::FilterType;
use walkdir::WalkDir;

use serde::Serialize;
use std::collections::{HashSet, hash_map::DefaultHasher};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Apps launcher mode backed by freedesktop desktop entries.
pub struct AppsMode {
    items: Vec<SearchableItem>,
    trie: Option<MultiTokenTrie>,
    frequency_tracker: Option<FrequencyTracker>,
    keyword_mapper: crate::config::keywords::KeywordMapper,
    search_engine: crate::search::engine::SearchEngine,
    last_action_time: Option<std::time::Instant>,
    settings: AppsModeSettings,
    current_desktop_envs: Vec<String>,
    icon_resolver: Option<AppIconResolver>,
}

impl AppsMode {
    pub fn new(
        frequency_tracker: Option<FrequencyTracker>,
        keyword_mapper: crate::config::keywords::KeywordMapper,
        settings: AppsModeSettings,
    ) -> Self {
        let icon_resolver = if settings.icons.enabled {
            Some(AppIconResolver::new(&settings))
        } else {
            None
        };

        Self {
            items: Vec::new(),
            trie: None,
            frequency_tracker,
            keyword_mapper,
            search_engine: crate::search::engine::SearchEngine::new(),
            last_action_time: None,
            settings,
            current_desktop_envs: current_desktop_envs(),
            icon_resolver,
        }
    }

    fn build_index(&self) -> Vec<SearchableItem> {
        let mut items = Vec::new();
        let mut seen_app_ids = HashSet::new();

        for dir in self.desktop_dirs() {
            if !dir.exists() {
                continue;
            }

            tracing::debug!("Scanning directory for desktop files: {:?}", dir);
            let base_dir = dir.clone();

            for entry in WalkDir::new(&dir)
                .follow_links(false)
                .into_iter()
                .filter_map(Result::ok)
            {
                let path = entry.path();
                if !is_safe_desktop_file(path, &base_dir) {
                    continue;
                }

                match DesktopEntry::from_path(path, None::<&[&str]>) {
                    Ok(desktop_entry) => {
                        if !self.should_include_desktop_entry(&desktop_entry) {
                            continue;
                        }

                        let name = desktop_entry
                            .name::<&str>(&[])
                            .map(|n| n.to_string())
                            .unwrap_or_default();

                        let exec = desktop_entry
                            .exec()
                            .map(|e| e.to_string())
                            .unwrap_or_default();

                        if name.is_empty() || exec.is_empty() {
                            continue;
                        }

                        let desktop_id = desktop_entry.id().to_string();

                        let exec = sanitize_exec(&exec);
                        if exec.is_empty() {
                            continue;
                        }

                        let path_string = path.to_string_lossy().to_string();
                        let filter_haystacks = [
                            desktop_id.as_str(),
                            name.as_str(),
                            exec.as_str(),
                            path_string.as_str(),
                        ];

                        if !self.matches_app_filters(&filter_haystacks) {
                            continue;
                        }

                        let dedup_key = if desktop_id.is_empty() {
                            path_string.to_lowercase()
                        } else {
                            desktop_id.to_lowercase()
                        };
                        if !seen_app_ids.insert(dedup_key) {
                            continue;
                        }

                        let keywords: Vec<String> = desktop_entry
                            .keywords::<&str>(&[])
                            .map(|k| k.iter().map(|s| s.to_lowercase()).collect())
                            .unwrap_or_default();

                        let categories: Vec<String> = desktop_entry
                            .categories()
                            .map(|cats| {
                                cats.iter()
                                    .map(|s| s.to_lowercase())
                                    .filter(|s| !s.is_empty())
                                    .collect()
                            })
                            .unwrap_or_default();

                        let icon = self.resolve_item_icon(
                            desktop_entry.icon(),
                            &name,
                            &categories,
                            &filter_haystacks,
                        );

                        let generic_name = desktop_entry
                            .generic_name::<&str>(&[])
                            .map(|g| g.to_string());
                        let description = desktop_entry.comment::<&str>(&[]).map(|c| c.to_string());
                        let icon_name = desktop_entry.icon().map(|i| i.to_string());

                        let executable = exec
                            .split_whitespace()
                            .next()
                            .unwrap_or(&exec)
                            .to_lowercase();

                        let item = Item {
                            id: path_string,
                            title: name.clone(),
                            search_text: name.to_lowercase(),
                            description: description.clone(),
                            icon,
                            metadata: Some(exec.clone()),
                        };

                        let mut searchable =
                            SearchableItem::new(item).with_field("name", &name, 10.0);

                        if let Some(gn) = generic_name {
                            searchable = searchable.with_field("generic_name", &gn, 7.0);
                        }

                        for keyword in keywords {
                            searchable = searchable.with_field("keyword", &keyword, 8.0);
                        }

                        for category in &categories {
                            searchable = searchable.with_field("category", category, 5.0);
                        }

                        if let Some(desc) = &description {
                            searchable = searchable.with_field("description", desc, 3.0);
                        }

                        searchable = searchable.with_field("executable", &executable, 2.0);

                        if let Some(icon_name) = icon_name {
                            searchable = searchable.with_field("icon", &icon_name, 1.5);
                        }

                        if !desktop_id.is_empty() {
                            searchable = searchable.with_field("desktop_id", &desktop_id, 6.0);
                        }

                        items.push(searchable);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse desktop file {}: {}", path.display(), e);
                    }
                }
            }
        }

        items
    }

    fn desktop_dirs(&self) -> Vec<PathBuf> {
        self.settings
            .desktop_dirs
            .iter()
            .filter_map(|dir| {
                let trimmed = dir.trim();
                if trimmed.is_empty() {
                    None
                } else if trimmed == "~" {
                    std::env::var("HOME").ok().map(PathBuf::from)
                } else if let Some(rest) = trimmed.strip_prefix("~/") {
                    std::env::var("HOME")
                        .ok()
                        .map(|home| PathBuf::from(home).join(rest))
                } else {
                    Some(PathBuf::from(trimmed))
                }
            })
            .collect()
    }

    fn should_include_desktop_entry(&self, entry: &DesktopEntry) -> bool {
        if entry.no_display() || entry.hidden() {
            return false;
        }

        if let Some(entry_type) = entry.type_()
            && !entry_type.eq_ignore_ascii_case("Application")
        {
            return false;
        }

        if self.settings.skip_terminal_apps && entry.terminal() {
            return false;
        }

        if let Some(only_show_in) = entry.only_show_in()
            && !only_show_in.is_empty()
        {
            if self.current_desktop_envs.is_empty() {
                return false;
            }
            if !desktop_list_matches(&self.current_desktop_envs, &only_show_in) {
                return false;
            }
        }

        if let Some(not_show_in) = entry.not_show_in()
            && !not_show_in.is_empty()
            && desktop_list_matches(&self.current_desktop_envs, &not_show_in)
        {
            return false;
        }

        true
    }

    fn matches_app_filters(&self, haystacks: &[&str]) -> bool {
        if !self.settings.include.is_empty()
            && !matches_any_pattern(&self.settings.include, haystacks)
        {
            return false;
        }

        if matches_any_pattern(&self.settings.exclude, haystacks) {
            return false;
        }

        true
    }

    fn resolve_item_icon(
        &self,
        icon_source: Option<&str>,
        title: &str,
        _categories: &[String],
        haystacks: &[&str],
    ) -> Option<String> {
        if !self.settings.icons.enabled {
            return None;
        }

        if !self.settings.icons.include.is_empty()
            && !matches_any_pattern(&self.settings.icons.include, haystacks)
        {
            return None;
        }

        if matches_any_pattern(&self.settings.icons.exclude, haystacks) {
            return None;
        }

        self.icon_resolver
            .as_ref()
            .and_then(|resolver| resolver.render(icon_source, title))
    }

    fn compute_cache_key(&self) -> String {
        let desktop_fingerprint = self.compute_desktop_fingerprint();
        let material = AppsCacheKeyMaterial {
            desktop_fingerprint,
            desktop_dirs: self.settings.desktop_dirs.clone(),
            include: self.settings.include.clone(),
            exclude: self.settings.exclude.clone(),
            skip_terminal_apps: self.settings.skip_terminal_apps,
            current_desktop_envs: self.current_desktop_envs.clone(),
            icons_enabled: self.settings.icons.enabled,
            icons_theme: self
                .icon_resolver
                .as_ref()
                .map(|r| r.theme.clone())
                .unwrap_or_else(|| "disabled".to_string()),
            icons_size: self.settings.icons.size,
            icons_scale: self.settings.icons.scale,
            icons_prefer_svg: self.settings.icons.prefer_svg,
            icons_render_mode: self.settings.icons.render_mode.clone(),
            icons_fallback: self.settings.icons.fallback.clone(),
            icons_include: self.settings.icons.include.clone(),
            icons_exclude: self.settings.icons.exclude.clone(),
        };

        let serialized = serde_json::to_string(&material).unwrap_or_default();
        let mut hasher = DefaultHasher::new();
        serialized.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    fn compute_desktop_fingerprint(&self) -> DesktopFingerprint {
        let mut file_count = 0u64;
        let mut newest_mtime = 0u64;
        let mut total_size = 0u64;
        let mut path_hash = 0u64;

        for dir in self.desktop_dirs() {
            if !dir.exists() {
                continue;
            }

            let base_dir = dir.clone();
            for entry in WalkDir::new(&dir)
                .follow_links(false)
                .into_iter()
                .filter_map(Result::ok)
            {
                let path = entry.path();
                if !is_safe_desktop_file(path, &base_dir) {
                    continue;
                }

                if let Ok(metadata) = std::fs::metadata(path) {
                    file_count = file_count.saturating_add(1);
                    total_size = total_size.saturating_add(metadata.len());

                    let modified = metadata
                        .modified()
                        .ok()
                        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|dur| dur.as_secs())
                        .unwrap_or(0);
                    newest_mtime = newest_mtime.max(modified);

                    let mut file_hasher = DefaultHasher::new();
                    path.to_string_lossy().hash(&mut file_hasher);
                    metadata.len().hash(&mut file_hasher);
                    modified.hash(&mut file_hasher);
                    path_hash ^= file_hasher.finish();
                }
            }
        }

        DesktopFingerprint {
            file_count,
            newest_mtime,
            total_size,
            path_hash,
        }
    }
}

impl Mode for AppsMode {
    fn name(&self) -> &str {
        "apps"
    }

    fn icon(&self) -> &str {
        "🔥"
    }

    fn description(&self) -> &str {
        "Applications"
    }

    fn load(&mut self) -> Result<(), LatuiError> {
        let cache_key = self.compute_cache_key();

        match load_cache() {
            Ok(cached)
                if cached.schema_version == APPS_CACHE_SCHEMA_VERSION
                    && cached.cache_key == cache_key =>
            {
                tracing::debug!(
                    "Loaded {} items from fresh cache (schema {})",
                    cached.apps.len(),
                    cached.schema_version
                );
                self.items = cached.apps;
                self.trie = Some(MultiTokenTrie::build(&self.items));
                return Ok(());
            }
            Ok(cached) => {
                tracing::info!(
                    "Apps cache is stale (schema={}, key_match={}), rebuilding",
                    cached.schema_version,
                    cached.cache_key == cache_key,
                );
            }
            Err(e) => {
                tracing::warn!("Cache load failed, rebuilding: {}", e);
            }
        }

        let items = self.build_index();

        if let Err(e) = save_cache(&items, &cache_key) {
            tracing::error!("Failed to save built index to cache: {}", e);
        }

        tracing::info!("Indexing complete. Ingested {} applications.", items.len());

        self.trie = Some(MultiTokenTrie::build(&items));
        self.items = items;
        Ok(())
    }

    fn search(&mut self, query: &str) -> Vec<Item> {
        if query.is_empty() {
            let mut scored_all: Vec<(usize, f64)> = self
                .items
                .iter()
                .enumerate()
                .map(|(idx, searchable)| {
                    let mut score = 0.0;
                    if let Some(ref tracker) = self.frequency_tracker {
                        score += tracker.get_total_boost(&searchable.item.id);
                    }
                    (idx, score)
                })
                .collect();

            scored_all.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            return scored_all
                .into_iter()
                .map(|(idx, _)| self.items[idx].item.clone())
                .collect();
        }

        use crate::search::tokenizer::Tokenizer;

        let start = std::time::Instant::now();
        let tokenizer = Tokenizer::new();
        let q = query.to_lowercase();
        let query_tokens = tokenizer.tokenize(&q);

        let mut candidate_indices = if let Some(ref trie) = self.trie {
            if query_tokens.len() > 1 {
                trie.get_multi_token_candidates(&query_tokens)
            } else {
                trie.get_candidates(&q)
            }
        } else {
            (0..self.items.len()).collect()
        };

        let mut candidate_set: HashSet<usize> = candidate_indices.iter().copied().collect();

        if let Some(mapped_apps) = self.keyword_mapper.get_matches(&q) {
            for app_needle in mapped_apps {
                for (idx, item) in self.items.iter().enumerate() {
                    if item.item.title.to_lowercase().contains(app_needle)
                        && candidate_set.insert(idx)
                    {
                        candidate_indices.push(idx);
                    }
                }
            }
        }

        if candidate_indices.is_empty() {
            return Vec::new();
        }

        let candidates: Vec<SearchableItem> = candidate_indices
            .iter()
            .map(|&idx| self.items[idx].clone())
            .collect();

        let mut scored_results = self.search_engine.search_scored(query, &candidates);

        if let Some(ref tracker) = self.frequency_tracker {
            for (item, score) in &mut scored_results {
                *score += tracker.get_frequency_boost(&item.id);
                *score += tracker.get_recency_boost(&item.id);
                *score += tracker.get_query_boost(&q, &item.id);
            }

            scored_results
                .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        }

        let results: Vec<Item> = scored_results.into_iter().map(|(item, _)| item).collect();
        tracing::trace!(
            "Search for '{}' completed in {:?} with {} results",
            query,
            start.elapsed(),
            results.len()
        );
        results
    }

    fn execute(&mut self, item: &Item) -> Result<(), LatuiError> {
        if let Some(last) = self.last_action_time
            && last.elapsed() < std::time::Duration::from_millis(500)
        {
            tracing::warn!("Rate limiting execution for item: {}", item.title);
            return Ok(());
        }
        self.last_action_time = Some(std::time::Instant::now());

        if let Some(ref mut tracker) = self.frequency_tracker
            && let Err(e) = tracker.record_launch(&item.id)
        {
            tracing::error!("Failed to record launch tracking: {}", e);
        }

        if let Some(cmd) = &item.metadata {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            if parts.is_empty() {
                return Ok(());
            }

            let shell_chars = [
                ';', '&', '|', '<', '>', '(', ')', '$', '`', '\\', '"', '\'', '*', '?', '[', ']',
                '~', '!',
            ];
            let has_shell_chars = cmd.chars().any(|c| shell_chars.contains(&c));

            let child = if !has_shell_chars {
                Command::new(parts[0]).args(&parts[1..]).spawn()
            } else {
                tracing::warn!("Executing command with shell features: {}", cmd);
                Command::new("sh").arg("-c").arg(cmd).spawn()
            };

            if let Err(e) = child {
                tracing::error!("Failed to execute '{}': {}", cmd, e);
                return Err(LatuiError::Io(e));
            }
        } else {
            tracing::warn!(
                "Apps mode received item without metadata (command): {}",
                item.title
            );
            return Err(LatuiError::App(
                "Missing command metadata for execution".to_string(),
            ));
        }
        Ok(())
    }

    fn record_selection(&mut self, query: &str, item: &Item) {
        if let Some(last) = self.last_action_time
            && last.elapsed() < std::time::Duration::from_millis(200)
        {
            return;
        }
        self.last_action_time = Some(std::time::Instant::now());

        if let Some(ref mut tracker) = self.frequency_tracker
            && let Err(e) = tracker.record_selection(query, &item.id)
        {
            tracing::error!("Failed to record selection tracking: {}", e);
        }
    }
}

#[derive(Debug, Serialize)]
struct DesktopFingerprint {
    file_count: u64,
    newest_mtime: u64,
    total_size: u64,
    path_hash: u64,
}

#[derive(Debug, Serialize)]
struct AppsCacheKeyMaterial {
    desktop_fingerprint: DesktopFingerprint,
    desktop_dirs: Vec<String>,
    include: Vec<String>,
    exclude: Vec<String>,
    skip_terminal_apps: bool,
    current_desktop_envs: Vec<String>,
    icons_enabled: bool,
    icons_theme: String,
    icons_size: u16,
    icons_scale: u16,
    icons_prefer_svg: bool,
    icons_render_mode: AppsIconRenderMode,
    icons_fallback: String,
    icons_include: Vec<String>,
    icons_exclude: Vec<String>,
}

#[derive(Clone)]
struct AppIconResolver {
    theme: String,
    size: u16,
    scale: u16,
    prefer_svg: bool,
    fallback: String,
    render_mode: AppsIconRenderMode,
}

impl AppIconResolver {
    fn new(settings: &AppsModeSettings) -> Self {
        let theme = settings
            .icons
            .theme
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .or_else(|| {
                std::env::var("LATUI_ICON_THEME")
                    .ok()
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty())
            })
            .or_else(freedesktop_icons::default_theme_gtk)
            .unwrap_or_else(|| "hicolor".to_string());

        Self {
            theme,
            size: settings.icons.size.max(1),
            scale: settings.icons.scale.max(1),
            prefer_svg: settings.icons.prefer_svg,
            fallback: settings.icons.fallback.clone(),
            render_mode: settings.icons.render_mode.clone(),
        }
    }

    fn render(&self, icon_source: Option<&str>, title: &str) -> Option<String> {
        let icon_name = icon_source.and_then(normalize_icon_source);
        let icon_path = icon_name.as_ref().and_then(|name| self.lookup_path(name));

        let path_hint = icon_path.as_ref().and_then(|path| {
            path.file_stem()
                .and_then(|stem| stem.to_str())
                .map(|stem| stem.to_lowercase())
        });

        let hint = path_hint.or_else(|| icon_name.as_ref().and_then(|name| icon_hint(name)));

        let rendered = match self.render_mode {
            AppsIconRenderMode::IconName => {
                icon_name.as_ref().map(|value| compact_icon_name(value))
            }
            AppsIconRenderMode::Thumbnail => self.thumbnail_icon(
                icon_name.as_deref(),
                hint.as_deref(),
                icon_path.as_deref(),
                title,
            ),
        };

        rendered.or_else(|| {
            if self.fallback.is_empty() {
                None
            } else {
                Some(self.fallback.clone())
            }
        })
    }

    fn thumbnail_icon(
        &self,
        icon_name: Option<&str>,
        icon_hint: Option<&str>,
        icon_path: Option<&Path>,
        title: &str,
    ) -> Option<String> {
        if let Some(path) = icon_path
            && let Some(icon) = icon_braille_thumbnail(path)
        {
            return Some(icon);
        }

        if let Some(hint) = icon_hint
            && let Some(icon) = icon_from_brand_hint(hint)
        {
            return Some(icon.to_string());
        }

        if let Some(icon_name) = icon_name
            && let Some(icon) = icon_name_badge(icon_name)
        {
            return Some(icon);
        }

        title_initials(title)
    }

    fn lookup_path(&self, icon_name: &str) -> Option<PathBuf> {
        let path = Path::new(icon_name);
        if path.is_absolute() && path.exists() {
            return Some(path.to_path_buf());
        }

        let mut lookup = freedesktop_icons::lookup(icon_name)
            .with_size(self.size)
            .with_scale(self.scale)
            .with_theme(&self.theme)
            .with_cache();

        if self.prefer_svg {
            lookup = lookup.force_svg();
        }

        lookup.find()
    }
}

fn sanitize_exec(exec: &str) -> String {
    exec.split_whitespace()
        .filter(|part| !part.starts_with('%'))
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_safe_desktop_file(path: &Path, base_dir: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("desktop"))
        .unwrap_or(false)
        && !path.is_symlink()
        && path.starts_with(base_dir)
}

fn desktop_list_matches(current_envs: &[String], desktops: &[&str]) -> bool {
    if current_envs.is_empty() {
        return false;
    }

    let mut wanted = Vec::new();
    for desktop in desktops {
        let trimmed = desktop.trim();
        if trimmed.is_empty() {
            continue;
        }
        wanted.push(trimmed.to_lowercase());
    }

    current_envs
        .iter()
        .any(|current| wanted.iter().any(|target| current == target))
}

fn current_desktop_envs() -> Vec<String> {
    std::env::var("XDG_CURRENT_DESKTOP")
        .ok()
        .map(|raw| {
            raw.split([':', ';', ','])
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| value.to_lowercase())
                .collect()
        })
        .unwrap_or_default()
}

fn matches_any_pattern(patterns: &[String], haystacks: &[&str]) -> bool {
    if patterns.is_empty() {
        return false;
    }

    let lowered_haystacks: Vec<String> =
        haystacks.iter().map(|value| value.to_lowercase()).collect();

    patterns.iter().any(|pattern| {
        let needle = pattern.trim().to_lowercase();
        !needle.is_empty()
            && lowered_haystacks
                .iter()
                .any(|haystack| haystack.contains(&needle))
    })
}

fn normalize_icon_source(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn icon_hint(value: &str) -> Option<String> {
    let path = Path::new(value);
    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
        let stem = stem.trim().to_lowercase();
        if !stem.is_empty() {
            return Some(stem);
        }
    }

    let lowered = value.trim().to_lowercase();
    if lowered.is_empty() {
        None
    } else {
        Some(lowered)
    }
}

fn icon_name_badge(value: &str) -> Option<String> {
    let mut initials = String::new();
    for segment in value
        .split(|c: char| !c.is_alphanumeric())
        .filter(|segment| !segment.is_empty())
    {
        if let Some(ch) = segment.chars().next() {
            initials.push(ch.to_ascii_uppercase());
        }
        if initials.len() >= 2 {
            break;
        }
    }

    if initials.is_empty() {
        None
    } else {
        Some(format!("[{initials}]"))
    }
}

fn compact_icon_name(value: &str) -> String {
    let base = icon_hint(value).unwrap_or_else(|| "icon".to_string());
    let mut short: String = base.chars().take(8).collect();
    if base.chars().count() > 8 {
        short.pop();
        short.push('+');
    }
    format!("[{short}]")
}

fn title_initials(title: &str) -> Option<String> {
    let letters: String = title
        .split_whitespace()
        .filter_map(|word| word.chars().find(|c| c.is_alphanumeric()))
        .take(2)
        .collect();

    if letters.is_empty() {
        None
    } else {
        Some(format!("[{}]", letters.to_uppercase()))
    }
}

fn icon_from_brand_hint(hint: &str) -> Option<&'static str> {
    let key = hint.to_lowercase();

    if contains_any(&key, &["google-chrome", "chrome", "chromium"]) {
        return Some("Ⓒ");
    }
    if contains_any(&key, &["brave"]) {
        return Some("🅱");
    }
    if contains_any(&key, &["firefox", "floorp", "librewolf"]) {
        return Some("🦊");
    }
    if contains_any(&key, &["vivaldi"]) {
        return Some("Ⓥ");
    }
    if contains_any(&key, &["code", "vscode", "vscodium", "codium"]) {
        return Some("⌘");
    }
    if contains_any(&key, &["neovim", "nvim"]) {
        return Some("Ⓝ");
    }
    if contains_any(&key, &["jetbrains", "idea", "pycharm", "webstorm"]) {
        return Some("ⓙ");
    }
    if contains_any(&key, &["emacs"]) {
        return Some("Ⓔ");
    }
    if contains_any(&key, &["vim"]) {
        return Some("Ⓥ");
    }
    if contains_any(
        &key,
        &[
            "terminal",
            "alacritty",
            "kitty",
            "wezterm",
            "console",
            "xterm",
        ],
    ) {
        return Some("⌨");
    }
    if contains_any(&key, &["file", "thunar", "nautilus", "dolphin", "pcmanfm"]) {
        return Some("🗂");
    }
    if contains_any(&key, &["spotify"]) {
        return Some("Ⓢ");
    }

    None
}

fn contains_any(input: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|pattern| input.contains(pattern))
}

fn icon_braille_thumbnail(path: &Path) -> Option<String> {
    if path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("svg"))
    {
        return None;
    }

    let image = ImageReader::open(path)
        .ok()?
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()?
        .to_rgba8();

    let scaled = image::imageops::resize(&image, 6, 4, FilterType::Triangle);
    let mut output = String::new();

    for cell_x in 0..3u32 {
        let mut bits = 0u8;
        for y in 0..4u32 {
            for x in 0..2u32 {
                let px = scaled.get_pixel(cell_x * 2 + x, y).0;
                if pixel_active(px[0], px[1], px[2], px[3]) {
                    bits |= braille_bit(x, y);
                }
            }
        }

        let ch = char::from_u32(0x2800 + u32::from(bits)).unwrap_or(' ');
        output.push(ch);
    }

    if output.chars().all(|ch| ch == '\u{2800}') {
        None
    } else {
        Some(output)
    }
}

fn braille_bit(x: u32, y: u32) -> u8 {
    match (x, y) {
        (0, 0) => 1 << 0,
        (0, 1) => 1 << 1,
        (0, 2) => 1 << 2,
        (1, 0) => 1 << 3,
        (1, 1) => 1 << 4,
        (1, 2) => 1 << 5,
        (0, 3) => 1 << 6,
        (1, 3) => 1 << 7,
        _ => 0,
    }
}

fn pixel_active(r: u8, g: u8, b: u8, a: u8) -> bool {
    if a < 28 {
        return false;
    }

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let saturation = max.saturating_sub(min);
    let luminance =
        ((54u16 * u16::from(r)) + (183u16 * u16::from(g)) + (19u16 * u16::from(b))) / 256u16;

    saturation > 15 || luminance < 205 || (a > 200 && luminance < 245)
}

impl Default for AppsMode {
    fn default() -> Self {
        Self::new(
            None,
            crate::config::keywords::KeywordMapper::new(),
            AppsModeSettings::default(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::keywords::KeywordMapper;
    use crate::config::settings::{AppsIconSettings, AppsModeSettings};

    #[test]
    fn test_sanitize_exec_removes_desktop_field_codes() {
        let exec = "firefox %U --new-window";
        assert_eq!(sanitize_exec(exec), "firefox --new-window");
    }

    #[test]
    fn test_pattern_matching_is_case_insensitive() {
        let patterns = vec!["fire".to_string()];
        let hay = ["Firefox", "browser"];
        assert!(matches_any_pattern(&patterns, &hay));
    }

    #[test]
    fn test_compact_icon_name_truncates() {
        let rendered = compact_icon_name("very-long-icon-name");
        assert!(rendered.starts_with('['));
        assert!(rendered.ends_with(']'));
        assert!(rendered.len() <= 10);
    }

    #[test]
    fn test_title_initials_generation() {
        assert_eq!(title_initials("Visual Studio").as_deref(), Some("[VS]"));
        assert_eq!(title_initials("").as_deref(), None);
    }

    #[test]
    fn test_icon_name_badge_generation() {
        assert_eq!(icon_name_badge("google-chrome").as_deref(), Some("[GC]"));
        assert_eq!(icon_name_badge("").as_deref(), None);
    }

    #[test]
    fn test_cache_key_changes_with_icon_settings() {
        let base = AppsModeSettings::default();

        let mode1 = AppsMode::new(None, KeywordMapper::new(), base.clone());
        let key1 = mode1.compute_cache_key();

        let mut changed = base;
        changed.icons = AppsIconSettings {
            size: 48,
            ..AppsIconSettings::default()
        };

        let mode2 = AppsMode::new(None, KeywordMapper::new(), changed);
        let key2 = mode2.compute_cache_key();

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_mode_creation_defaults() {
        let mode = AppsMode::new(None, KeywordMapper::new(), AppsModeSettings::default());
        assert_eq!(mode.name(), "apps");
        assert_eq!(mode.icon(), "🔥");
    }

    #[test]
    fn test_render_mode_icon_name_uses_icon_label() {
        let mut settings = AppsModeSettings::default();
        settings.icons.render_mode = AppsIconRenderMode::IconName;

        let resolver = AppIconResolver::new(&settings);
        let rendered = resolver.render(Some("firefox"), "Firefox").unwrap();
        assert!(rendered.starts_with('['));
    }
}
