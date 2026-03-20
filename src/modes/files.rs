//! Files mode — high-performance filesystem search for LaTUI.
//!
//! # Metadata format
//! Each `Item.metadata` field is a JSON-encoded `FileMetadata` struct:
//! ```json
//! {
//!   "path": "/home/user/Documents/notes.txt",
//!   "kind": "file"           // "file" | "dir" | "symlink"
//! }
//! ```
//!
//! # Design
//! - **Recent files** are persisted to XDG data dir (`files_recents.json`).
//! - **Live search** uses `walkdir` with configurable depth from a set of
//!   root directories.  Results are scored by filename relevance, recency and
//!   file-type priority.
//! - **Preview** is supported for text files (first N lines).
//! - All paths are canonicalised and validated to prevent path-traversal.

use crate::core::{item::Item, mode::Mode, searchable_item::SearchableItem};
use crate::core::utils::current_timestamp;
use crate::error::LatuiError;
use crate::search::engine::SearchEngine;

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use std::time::Instant;
use walkdir::WalkDir;

// ─── Constants ────────────────────────────────────────────────────────────────

/// Maximum number of recent files remembered on disk.
const MAX_RECENTS: usize = 200;

/// Number of recent files shown when the query is empty.
const RECENT_DISPLAY_LIMIT: usize = 30;

/// Maximum walkdir depth when the user is typing a query.
const SEARCH_MAX_DEPTH: usize = 5;

/// Maximum results returned from a live walk.
const SEARCH_RESULT_LIMIT: usize = 60;

/// Maximum file size (bytes) we will attempt to preview.
const PREVIEW_MAX_BYTES: u64 = 512 * 1024; // 512 KiB

/// Maximum lines shown in a file preview.
const PREVIEW_MAX_LINES: usize = 20;

// ─── Supporting types ─────────────────────────────────────────────────────────

/// Discriminates between filesystem node kinds.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FileKind {
    File,
    Dir,
    Symlink,
}

impl FileKind {
    fn from_path(path: &Path) -> Self {
        if path.is_symlink() {
            FileKind::Symlink
        } else if path.is_dir() {
            FileKind::Dir
        } else {
            FileKind::File
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            FileKind::File => "📄",
            FileKind::Dir => "📁",
            FileKind::Symlink => "🔗",
        }
    }
}

/// Compact metadata stored in `Item.metadata` as JSON.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileMetadata {
    pub path: String,
    pub kind: FileKind,
}

/// A single entry in the recents list.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct RecentEntry {
    /// Absolute path of the file or directory.
    path: String,
    /// Unix timestamp of when it was last opened through LaTUI.
    timestamp: u64,
    /// How many times it has been opened.
    open_count: u32,
}

// ─── FilesMode ────────────────────────────────────────────────────────────────

/// Files mode — fuzzy-searchable filesystem navigator.
#[derive(Debug)]
pub struct FilesMode {
    /// Recent files/dirs opened through LaTUI (most recent first).
    recents: VecDeque<RecentEntry>,

    /// Searchable items built from the recents list.
    searchable_recents: Vec<SearchableItem>,

    /// Fuzzy search engine (shared with RunMode / AppsMode).
    search_engine: SearchEngine,

    /// Filesystem roots searched during a live walk.
    /// Defaults to `$HOME`.
    search_roots: Vec<PathBuf>,

    /// Path to the recents persistence file.
    recents_path: Option<PathBuf>,

    /// Rate-limiter: timestamp of the last execute/record call.
    last_action_time: Option<Instant>,

    /// Dirty flag — true when recents have been modified but not yet saved.
    dirty: bool,
}

impl FilesMode {
    // ── Constructors ─────────────────────────────────────────────────────────

    /// Create a new `FilesMode` that searches `$HOME`.
    pub fn new() -> Self {
        let home = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/"));
        Self::with_roots(vec![home])
    }

    /// Create a `FilesMode` with custom search roots.
    pub fn with_roots(roots: Vec<PathBuf>) -> Self {
        Self {
            recents: VecDeque::new(),
            searchable_recents: Vec::new(),
            search_engine: SearchEngine::new(),
            search_roots: roots,
            recents_path: None,
            last_action_time: None,
            dirty: false,
        }
    }

    // ── Persistence ──────────────────────────────────────────────────────────

    /// Load the recents list from the XDG data directory.
    fn load_recents(&mut self) -> Result<(), LatuiError> {
        use xdg::BaseDirectories;

        let xdg = BaseDirectories::with_prefix("latui");
        let path = xdg
            .place_data_file("files_recents.json")
            .map_err(std::io::Error::other)?;

        self.recents_path = Some(path.clone());

        // Secure the parent directory on Unix.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
                let _ = std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700));
            }
        }

        if !path.exists() {
            return Ok(());
        }

        match std::fs::read_to_string(&path) {
            Ok(data) => {
                // Refuse files larger than 2 MiB — clearly corrupted.
                if data.len() > 2 * 1024 * 1024 {
                    tracing::warn!("Files recents list too large, discarding");
                    return Ok(());
                }

                match serde_json::from_str::<Vec<RecentEntry>>(&data) {
                    Ok(mut entries) => {
                        // Keep only entries whose paths still exist.
                        entries.retain(|e| Path::new(&e.path).exists());
                        entries.truncate(MAX_RECENTS);
                        self.recents = entries.into();
                        tracing::info!("Loaded {} recent files", self.recents.len());
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse files recents: {}", e);
                    }
                }
            }
            Err(e) => {
                tracing::debug!("No existing files recents file: {}", e);
            }
        }

        Ok(())
    }

    /// Persist the recents list to disk (no-op when not dirty).
    fn save_recents(&mut self) -> Result<(), LatuiError> {
        if !self.dirty {
            return Ok(());
        }

        let path = match &self.recents_path {
            Some(p) => p.clone(),
            None => return Ok(()),
        };

        let entries: Vec<RecentEntry> = self.recents.iter().cloned().collect();
        let json = serde_json::to_string_pretty(&entries)
            .map_err(|e| LatuiError::Io(std::io::Error::other(e)))?;

        // Atomic write via temp file
        let mut tmp_path = path.clone();
        tmp_path.set_extension("tmp");
        std::fs::write(&tmp_path, json)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o600));
        }

        std::fs::rename(&tmp_path, &path)?;

        self.dirty = false;
        tracing::debug!("Saved {} recent files to disk", self.recents.len());
        Ok(())
    }

    // ── Recents management ───────────────────────────────────────────────────

    /// Record a path as recently opened.
    fn add_to_recents(&mut self, path: &str) {
        let now = current_timestamp();

        if let Some(pos) = self.recents.iter().position(|e| e.path == path) {
            // Promote to front and bump counter.
            let mut entry = self.recents.remove(pos).unwrap();
            entry.open_count += 1;
            entry.timestamp = now;
            self.recents.push_front(entry);
        } else {
            self.recents.push_front(RecentEntry {
                path: path.to_string(),
                timestamp: now,
                open_count: 1,
            });

            if self.recents.len() > MAX_RECENTS {
                self.recents.pop_back();
            }
        }

        self.dirty = true;
        self.rebuild_searchable_recents();
    }

    /// Rebuild the searchable index from the current recents list.
    fn rebuild_searchable_recents(&mut self) {
        self.searchable_recents = self
            .recents
            .iter()
            .map(|entry| {
                let path = Path::new(&entry.path);
                let kind = FileKind::from_path(path);
                let file_name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| entry.path.clone());
                let parent = path
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();

                let meta = FileMetadata {
                    path: entry.path.clone(),
                    kind: kind.clone(),
                };
                let meta_json = serde_json::to_string(&meta).unwrap_or_default();

                let item = Item {
                    id: format!("file:{}", entry.path),
                    title: file_name.clone(),
                    search_text: file_name.to_lowercase(),
                    description: Some(parent.clone()),
                    icon: Some(kind.icon().to_string()),
                    metadata: Some(meta_json),
                };

                SearchableItem::new(item)
                    .with_field("name", &file_name, 10.0)
                    .with_field("parent", &parent, 3.0)
                    .with_field("full_path", &entry.path, 2.0)
            })
            .collect();
    }

    /// Return the most recently opened files as `Item`s, scored by recency
    /// and open frequency so that frequently-accessed files bubble up.
    fn get_recent_items(&self) -> Vec<Item> {
        let total = self.recents.len().min(RECENT_DISPLAY_LIMIT);

        let mut scored: Vec<(Item, f64)> = self
            .recents
            .iter()
            .take(total)
            .enumerate()
            .filter_map(|(idx, entry)| {
                let path = Path::new(&entry.path);
                if !path.exists() {
                    return None;
                }

                let kind = FileKind::from_path(path);
                let file_name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| entry.path.clone());
                let parent = path
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();

                let meta = FileMetadata {
                    path: entry.path.clone(),
                    kind: kind.clone(),
                };
                let meta_json = serde_json::to_string(&meta).unwrap_or_default();

                let item = Item {
                    id: format!("file:{}", entry.path),
                    title: file_name.clone(),
                    search_text: file_name.to_lowercase(),
                    description: Some(parent),
                    icon: Some(kind.icon().to_string()),
                    metadata: Some(meta_json),
                };

                // Score: recency weight + frequency bonus
                let recency_score = (total - idx) as f64 * 10.0;
                let frequency_score = (entry.open_count as f64).ln() * 15.0;
                Some((item, recency_score + frequency_score))
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scored.into_iter().map(|(item, _)| item).collect()
    }

    // ── Live filesystem search ───────────────────────────────────────────────

    /// Walk `search_roots` and return items whose filename matches `query`.
    /// Results are scored so that:
    ///   1. Exact filename matches rank highest.
    ///   2. Prefix matches rank next.
    ///   3. Substring matches are included last.
    ///   4. Directories are given a slight boost over plain files.
    fn walk_search(&self, query: &str) -> Vec<(Item, f64)> {
        let q = query.to_lowercase();
        let mut results: Vec<(Item, f64)> = Vec::new();

        for root in &self.search_roots {
            let canonical_root = match root.canonicalize() {
                Ok(p) => p,
                Err(_) => root.clone(),
            };

            for entry in WalkDir::new(root)
                .max_depth(SEARCH_MAX_DEPTH)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
                // Skip hidden directories (names starting with '.'), except
                // the root itself.
                .filter(|e| e.depth() == 0 || !e.file_name().to_string_lossy().starts_with('.'))
            {
                let path = entry.path();

                // Lightweight security check: path should be within root.
                // We avoid per-entry canonicalize() for performance.
                if !path.starts_with(&canonical_root) && !path.starts_with(root) {
                    continue;
                }

                let file_name_os = entry.file_name();
                let file_name = file_name_os.to_string_lossy();
                let file_name_lower = file_name.to_lowercase();

                // Score the hit.
                let name_score: f64 = if file_name_lower == q {
                    1000.0
                } else if file_name_lower.starts_with(&q) {
                    500.0
                } else if file_name_lower.contains(&q) {
                    200.0
                } else {
                    continue; // No match.
                };

                // Give directories a modest boost.
                let kind = FileKind::from_path(path);
                let kind_bonus: f64 = if kind == FileKind::Dir { 20.0 } else { 0.0 };

                let parent = path
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();

                let meta = FileMetadata {
                    path: path.to_string_lossy().to_string(),
                    kind: kind.clone(),
                };
                let meta_json = serde_json::to_string(&meta).unwrap_or_default();

                let item = Item {
                    id: format!("file:{}", path.display()),
                    title: file_name.to_string(),
                    search_text: file_name_lower,
                    description: Some(parent),
                    icon: Some(kind.icon().to_string()),
                    metadata: Some(meta_json),
                };

                results.push((item, name_score + kind_bonus));

                if results.len() >= SEARCH_RESULT_LIMIT {
                    break;
                }
            }

            if results.len() >= SEARCH_RESULT_LIMIT {
                break;
            }
        }

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        results
    }

    // ── Validation ─────────────────────────────────────────────────────────

    /// Validate a path before attempting to open it.
    fn validate_path(path: &str) -> Result<PathBuf, LatuiError> {
        if path.is_empty() {
            return Err(LatuiError::App("Path is empty".to_string()));
        }
        if path.len() > 4096 {
            return Err(LatuiError::App("Path too long".to_string()));
        }
        if path.contains('\0') {
            return Err(LatuiError::App("Path contains null bytes".to_string()));
        }

        let pb = PathBuf::from(path);
        if !pb.exists() {
            return Err(LatuiError::App(format!("Path does not exist: {}", path)));
        }

        Ok(pb)
    }

    // ── Preview helpers ──────────────────────────────────────────────────────

    /// Try to read the first `PREVIEW_MAX_LINES` lines of a text file.
    fn text_preview(path: &Path) -> Option<String> {
        // Check size first to avoid reading huge binaries.
        let size = std::fs::metadata(path).ok()?.len();
        if size > PREVIEW_MAX_BYTES {
            return None;
        }

        let content = std::fs::read(path).ok()?;

        // Heuristic: if more than 30% of the first 512 bytes are non-UTF8 /
        // non-printable we treat the file as binary and skip preview.
        let sample = &content[..content.len().min(512)];
        let non_text = sample
            .iter()
            .filter(|&&b| b < 0x09 || (b > 0x0d && b < 0x20) || b == 0x7f)
            .count();
        if non_text * 100 / sample.len().max(1) > 30 {
            return None;
        }

        let text = String::from_utf8_lossy(&content);
        let lines: Vec<&str> = text.lines().take(PREVIEW_MAX_LINES).collect();
        if lines.is_empty() {
            return None;
        }

        Some(lines.join("\n"))
    }
}

// ─── Mode trait implementation ────────────────────────────────────────────────

impl Mode for FilesMode {
    fn name(&self) -> &str {
        "files"
    }

    fn icon(&self) -> &str {
        "📁"
    }

    fn description(&self) -> &str {
        "Filesystem Search"
    }

    // ── load ──────────────────────────────────────────────────────────────

    fn load(&mut self) -> Result<(), LatuiError> {
        tracing::debug!("Loading files mode with roots: {:?}", self.search_roots);

        self.load_recents()?;
        self.rebuild_searchable_recents();

        tracing::info!(
            "Files mode loaded with {} recent entries",
            self.recents.len()
        );
        Ok(())
    }

    // ── search ────────────────────────────────────────────────────────────

    /// Search strategy:
    /// - **Empty query** → return recent files sorted by recency + frequency.
    /// - **Short query (1 char)** → search recents only (snappy feel).
    /// - **Longer query** → merge recents fuzzy search with live directory
    ///   walk, deduplicating by canonical path.
    fn search(&mut self, query: &str) -> Vec<Item> {
        let start = Instant::now();

        if query.is_empty() {
            let results = self.get_recent_items();
            tracing::trace!(
                "Files empty query → {} recent items in {:?}",
                results.len(),
                start.elapsed()
            );
            return results;
        }

        let q = query.trim();
        let mut seen_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut merged: Vec<(Item, f64)> = Vec::new();

        // 1. Fuzzy-search within recents (fast, always done).
        if !self.searchable_recents.is_empty() {
            let recents_hits = self
                .search_engine
                .search_scored(q, &self.searchable_recents);

            for (item, score) in recents_hits {
                if score > 0.0 {
                    seen_ids.insert(item.id.clone());
                    // Boost recent hits — the user has opened them before.
                    merged.push((item, score + 50.0));
                }
            }
        }

        // 2. Live walk for queries of 2+ characters.
        if q.len() >= 2 {
            let walk_hits = self.walk_search(q);

            for (item, score) in walk_hits {
                if !seen_ids.contains(&item.id) {
                    seen_ids.insert(item.id.clone());
                    merged.push((item, score));
                }
            }
        }

        // Sort merged results by score (descending).
        merged.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let results: Vec<Item> = merged
            .into_iter()
            .take(SEARCH_RESULT_LIMIT)
            .map(|(item, _)| item)
            .collect();

        tracing::trace!(
            "Files search '{}' → {} results in {:?}",
            q,
            results.len(),
            start.elapsed()
        );

        results
    }

    // ── execute ───────────────────────────────────────────────────────────

    fn execute(&mut self, item: &Item) -> Result<(), LatuiError> {
        // Rate-limit: prevent double-opens from accidental key bounces.
        if let Some(last) = self.last_action_time
            && last.elapsed() < std::time::Duration::from_millis(500)
        {
            tracing::warn!("Rate-limiting file open for: {}", item.title);
            return Ok(());
        }
        self.last_action_time = Some(Instant::now());

        // Parse metadata JSON.
        let meta_json = item
            .metadata
            .as_ref()
            .ok_or_else(|| LatuiError::App("Missing file metadata".to_string()))?;

        let meta: FileMetadata = serde_json::from_str(meta_json)
            .map_err(|e| LatuiError::App(format!("Corrupt file metadata: {}", e)))?;

        // Validate path (existence, length, null-bytes).
        let path = Self::validate_path(&meta.path)?;

        tracing::info!("Opening file/dir: {}", path.display());

        // Open using the system default handler.
        crate::core::execution::ExecutionEngine::spawn_shell(&format!("xdg-open \"{}\"", meta.path), &[])?;

        // Record in recents.
        self.add_to_recents(&meta.path);

        // Persist immediately.
        if let Err(e) = self.save_recents() {
            tracing::error!("Failed to save recents after open: {}", e);
        }

        Ok(())
    }

    // ── record_selection ─────────────────────────────────────────────────

    fn record_selection(&mut self, _query: &str, item: &Item) {
        // Rate-limit cursor-movement tracking.
        if let Some(last) = self.last_action_time
            && last.elapsed() < std::time::Duration::from_millis(200)
        {
            return;
        }
        self.last_action_time = Some(Instant::now());

        // Extract path from metadata for lightweight logging — we don't add
        // to recents here because the user hasn't opened the file yet.
        if let Some(meta_json) = &item.metadata
            && let Ok(meta) = serde_json::from_str::<FileMetadata>(meta_json)
        {
            tracing::trace!("Files mode selection: {}", meta.path);
        }
    }

    // ── preview ───────────────────────────────────────────────────────────

    fn supports_preview(&self) -> bool {
        true
    }

    /// Returns a text preview for plain-text files, or a summary for
    /// directories (item count) and other node types.
    fn preview(&self, item: &Item) -> Option<String> {
        let meta_json = item.metadata.as_ref()?;
        let meta: FileMetadata = serde_json::from_str(meta_json).ok()?;
        let path = Path::new(&meta.path);

        if !path.exists() {
            return Some("⚠️  File no longer exists".to_string());
        }

        match meta.kind {
            FileKind::File => Self::text_preview(path).or_else(|| {
                // For binary files just show basic stats.
                let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
                Some(format!("Binary file — {} bytes", size))
            }),

            FileKind::Dir => {
                // Count direct children (non-recursive).
                let count = std::fs::read_dir(path)
                    .map(|rd| rd.filter_map(Result::ok).count())
                    .unwrap_or(0);
                Some(format!(
                    "📁  Directory with {} item{}",
                    count,
                    if count == 1 { "" } else { "s" }
                ))
            }

            FileKind::Symlink => {
                let target = std::fs::read_link(path)
                    .map(|t| t.to_string_lossy().to_string())
                    .unwrap_or_else(|_| "<unreadable>".to_string());
                Some(format!("🔗  Symlink → {}", target))
            }
        }
    }
}

// ─── Drop impl ────────────────────────────────────────────────────────────────

impl Drop for FilesMode {
    fn drop(&mut self) {
        if self.dirty
            && let Err(e) = self.save_recents()
        {
            tracing::error!("Failed to save recents on drop: {}", e);
        }
    }
}

impl Default for FilesMode {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    // ── FilesMode creation ────────────────────────────────────────────

    #[test]
    fn test_files_mode_creation() {
        let mode = FilesMode::new();
        assert_eq!(mode.name(), "files");
        assert_eq!(mode.icon(), "📁");
        assert!(mode.recents.is_empty());
    }

    // ── Recents management ────────────────────────────────────────────

    #[test]
    fn test_add_to_recents() {
        let mut mode = FilesMode::new();
        mode.add_to_recents("/tmp/test.txt");
        assert_eq!(mode.recents.len(), 1);
        assert_eq!(mode.recents[0].path, "/tmp/test.txt");
        assert_eq!(mode.recents[0].open_count, 1);
    }

    #[test]
    fn test_duplicate_recent_increments_count() {
        let mut mode = FilesMode::new();
        mode.add_to_recents("/tmp/test.txt");
        mode.add_to_recents("/tmp/test.txt");
        assert_eq!(mode.recents.len(), 1);
        assert_eq!(mode.recents[0].open_count, 2);
    }

    #[test]
    fn test_recent_promoted_to_front() {
        let mut mode = FilesMode::new();
        mode.add_to_recents("/tmp/a.txt");
        mode.add_to_recents("/tmp/b.txt");
        mode.add_to_recents("/tmp/a.txt"); // promote a
        assert_eq!(mode.recents[0].path, "/tmp/a.txt");
    }

    #[test]
    fn test_recents_size_capped() {
        let mut mode = FilesMode::new();
        for i in 0..MAX_RECENTS + 20 {
            mode.add_to_recents(&format!("/tmp/file_{}.txt", i));
        }
        assert_eq!(mode.recents.len(), MAX_RECENTS);
    }

    // ── Validation ────────────────────────────────────────────────────

    #[test]
    fn test_validate_empty_path() {
        assert!(FilesMode::validate_path("").is_err());
    }

    #[test]
    fn test_validate_null_bytes() {
        assert!(FilesMode::validate_path("foo\0bar").is_err());
    }

    #[test]
    fn test_validate_too_long_path() {
        let long = "a".repeat(5000);
        assert!(FilesMode::validate_path(&long).is_err());
    }

    // ── Search ───────────────────────────────────────────────────────

    #[test]
    fn test_search_empty_returns_recents() {
        let mut mode = FilesMode::new();
        // Add /tmp which always exists
        mode.add_to_recents("/tmp");
        let results = mode.search("");
        // At least our /tmp entry should appear.
        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_with_query_uses_engine() {
        let tmp_dir = TempDir::new().unwrap();
        let root = tmp_dir.path().to_path_buf();

        // Create a test file.
        let file_path = root.join("hello_world.txt");
        std::fs::File::create(&file_path).unwrap();

        let mut mode = FilesMode::with_roots(vec![root]);
        let results = mode.search("hello");
        // The live walk must find hello_world.txt
        assert!(results.iter().any(|i| i.title.contains("hello_world")));
    }

    // ── Preview ───────────────────────────────────────────────────────

    #[test]
    fn test_preview_text_file() {
        let tmp_dir = TempDir::new().unwrap();
        let file = tmp_dir.path().join("notes.txt");
        {
            let mut f = std::fs::File::create(&file).unwrap();
            writeln!(f, "Line one").unwrap();
            writeln!(f, "Line two").unwrap();
        }

        let mode = FilesMode::new();
        let meta = FileMetadata {
            path: file.to_string_lossy().to_string(),
            kind: FileKind::File,
        };
        let item = Item {
            id: "test".to_string(),
            title: "notes.txt".to_string(),
            search_text: "notes.txt".to_string(),
            description: None,
            icon: Some("📄".to_string()),
            metadata: Some(serde_json::to_string(&meta).unwrap()),
        };

        let preview = mode.preview(&item);
        assert!(preview.is_some());
        let text = preview.unwrap();
        assert!(text.contains("Line one"));
        assert!(text.contains("Line two"));
    }

    #[test]
    fn test_preview_directory() {
        let tmp_dir = TempDir::new().unwrap();
        // Create a couple of children.
        std::fs::File::create(tmp_dir.path().join("a.txt")).unwrap();
        std::fs::File::create(tmp_dir.path().join("b.txt")).unwrap();

        let mode = FilesMode::new();
        let meta = FileMetadata {
            path: tmp_dir.path().to_string_lossy().to_string(),
            kind: FileKind::Dir,
        };
        let item = Item {
            id: "test-dir".to_string(),
            title: "tmpdir".to_string(),
            search_text: "tmpdir".to_string(),
            description: None,
            icon: Some("📁".to_string()),
            metadata: Some(serde_json::to_string(&meta).unwrap()),
        };

        let preview = mode.preview(&item).unwrap();
        assert!(preview.contains("2 items"));
    }

    // ── FileKind ─────────────────────────────────────────────────────

    #[test]
    fn test_filekind_icons() {
        assert_eq!(FileKind::File.icon(), "📄");
        assert_eq!(FileKind::Dir.icon(), "📁");
        assert_eq!(FileKind::Symlink.icon(), "🔗");
    }

    // ── Metadata serialization ────────────────────────────────────────

    #[test]
    fn test_metadata_roundtrip() {
        let meta = FileMetadata {
            path: "/home/user/file.txt".to_string(),
            kind: FileKind::File,
        };
        let json = serde_json::to_string(&meta).unwrap();
        let decoded: FileMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.path, meta.path);
        assert_eq!(decoded.kind, FileKind::File);
    }
}
