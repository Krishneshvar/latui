//! Clipboard mode — searchable clipboard history manager for LaTUI.
//!
//! # What it does
//! Maintains a rolling history of clipboard entries the user has *manually
//! copied* outside LaTUI, as well as entries that flow through `execute()`.
//! Users can fuzzy-search past clips and paste any of them back to the
//! system clipboard with a single keystroke.
//!
//! # Metadata format
//! Each `Item.metadata` is the raw clipboard content (plain text).
//! No JSON wrapping — the content is the payload, keeping `execute()` trivial.
//!
//! # Backend detection (runtime)
//! 1. **Wayland** — `wl-copy` / `wl-paste`  (`$WAYLAND_DISPLAY` set)
//! 2. **X11**     — `xclip -selection clipboard` (`$DISPLAY` set, Wayland absent)
//! 3. **Fallback** — log a warning, copy silently skipped.
//!
//! # Persistence
//! History is written to `~/.local/share/latui/clipboard_history.json` (XDG
//! data dir).  Sensitive content is never logged; only entry counts and
//! lengths appear in trace-level messages.

use crate::core::{item::Item, mode::Mode, searchable_item::SearchableItem};
use crate::error::LatuiError;
use crate::search::engine::SearchEngine;

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

// ─── Constants ────────────────────────────────────────────────────────────────

/// Maximum number of clipboard entries kept in history.
const MAX_HISTORY: usize = 500;

/// Number of entries shown when the query is empty.
const RECENT_DISPLAY_LIMIT: usize = 30;

/// Characters shown in the item title (to keep the UI compact).
const TITLE_MAX_CHARS: usize = 80;

/// Maximum content length (bytes) we accept into history.
/// Prevents multi-megabyte binary pastes from bloating the store.
const MAX_CONTENT_BYTES: usize = 64 * 1024; // 64 KiB

/// Maximum size of the JSON file we will attempt to load.
const MAX_HISTORY_FILE_BYTES: u64 = 8 * 1024 * 1024; // 8 MiB

// ─── Supporting types ─────────────────────────────────────────────────────────

/// A single clipboard history entry.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct ClipEntry {
    /// Full clipboard content.
    content: String,
    /// Unix timestamp of when this entry was first recorded.
    first_seen: u64,
    /// Unix timestamp of the most recent copy.
    last_used: u64,
    /// How many times this exact content has been copied / re-pasted.
    use_count: u32,
}

/// Which clipboard backend is available on this system.
#[derive(Debug, Clone, PartialEq, Eq)]
enum ClipBackend {
    Wayland,  // wl-copy / wl-paste
    X11,      // xclip -selection clipboard
    None,     // No known tool available
}

impl ClipBackend {
    /// Detect the backend at runtime from environment variables and PATH.
    fn detect() -> Self {
        let wayland = std::env::var("WAYLAND_DISPLAY")
            .map(|v| !v.is_empty())
            .unwrap_or(false);

        if wayland && command_exists("wl-copy") {
            return ClipBackend::Wayland;
        }

        let display = std::env::var("DISPLAY")
            .map(|v| !v.is_empty())
            .unwrap_or(false);

        if display && command_exists("xclip") {
            return ClipBackend::X11;
        }

        ClipBackend::None
    }

    /// Name for logging.
    fn name(&self) -> &'static str {
        match self {
            ClipBackend::Wayland => "wl-copy (Wayland)",
            ClipBackend::X11 => "xclip (X11)",
            ClipBackend::None => "none",
        }
    }
}

// ─── ClipboardMode ────────────────────────────────────────────────────────────

/// Clipboard mode — fuzzy-searchable clipboard history with paste support.
pub struct ClipboardMode {
    /// Clipboard history (most recently used first).
    history: VecDeque<ClipEntry>,

    /// Searchable index rebuilt whenever history changes.
    searchable: Vec<SearchableItem>,

    /// Shared fuzzy search engine.
    search_engine: SearchEngine,

    /// Runtime clipboard backend (Wayland / X11 / None).
    backend: ClipBackend,

    /// Path to the JSON persistence file.
    history_path: Option<PathBuf>,

    /// Rate-limiter gate shared between `execute` and `record_selection`.
    last_action_time: Option<Instant>,

    /// True when history has been modified but not yet persisted.
    dirty: bool,
}

impl ClipboardMode {
    // ── Constructor ──────────────────────────────────────────────────────────

    pub fn new() -> Self {
        let backend = ClipBackend::detect();
        tracing::debug!("Clipboard backend detected: {}", backend.name());

        Self {
            history: VecDeque::new(),
            searchable: Vec::new(),
            search_engine: SearchEngine::new(),
            backend,
            history_path: None,
            last_action_time: None,
            dirty: false,
        }
    }

    // ── Persistence ──────────────────────────────────────────────────────────

    /// Load history from the XDG data directory.
    fn load_history(&mut self) -> Result<(), LatuiError> {
        use xdg::BaseDirectories;

        let xdg = BaseDirectories::with_prefix("latui");
        let path = xdg
            .place_data_file("clipboard_history.json")
            .map_err(|e| LatuiError::Io(std::io::Error::other(e)))?;

        self.history_path = Some(path.clone());

        // Secure the parent directory on Unix.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
                let _ = std::fs::set_permissions(
                    parent,
                    std::fs::Permissions::from_mode(0o700),
                );
            }
        }

        if !path.exists() {
            return Ok(());
        }

        // Size sanity check before reading.
        if let Ok(meta) = std::fs::metadata(&path)
            && meta.len() > MAX_HISTORY_FILE_BYTES {
                tracing::warn!("Clipboard history file too large — discarding");
                return Ok(());
            }

        match std::fs::read_to_string(&path) {
            Ok(data) => match serde_json::from_str::<Vec<ClipEntry>>(&data) {
                Ok(mut entries) => {
                    // Drop oversized or empty entries that may have snuck in.
                    entries.retain(|e| {
                        !e.content.is_empty()
                            && e.content.len() <= MAX_CONTENT_BYTES
                    });
                    entries.truncate(MAX_HISTORY);
                    self.history = entries.into();
                    tracing::info!(
                        "Loaded {} clipboard entries from disk",
                        self.history.len()
                    );
                }
                Err(e) => {
                    tracing::warn!("Failed to parse clipboard history: {}", e);
                }
            },
            Err(e) => {
                tracing::debug!("No existing clipboard history file: {}", e);
            }
        }

        Ok(())
    }

    /// Persist history to disk (no-op when clean).
    fn save_history(&mut self) -> Result<(), LatuiError> {
        if !self.dirty {
            return Ok(());
        }

        let path = match &self.history_path {
            Some(p) => p.clone(),
            None => return Ok(()),
        };

        let entries: Vec<ClipEntry> = self.history.iter().cloned().collect();
        let json = serde_json::to_string_pretty(&entries)
            .map_err(|e| LatuiError::Io(std::io::Error::other(e)))?;

        // Atomic write via temp file
        let mut tmp_path = path.clone();
        tmp_path.set_extension("tmp");
        std::fs::write(&tmp_path, json)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(
                &tmp_path,
                std::fs::Permissions::from_mode(0o600),
            );
        }

        std::fs::rename(&tmp_path, &path)?;

        self.dirty = false;
        tracing::debug!(
            "Saved {} clipboard entries to disk",
            self.history.len()
        );
        Ok(())
    }

    // ── History management ───────────────────────────────────────────────────

    /// Add or promote a clipboard entry and rebuild the search index.
    ///
    /// If the same content already exists in history it is promoted to the
    /// front and its `use_count` / `last_used` are updated.  Otherwise a new
    /// entry is prepended.  The deque is capped at `MAX_HISTORY`.
    fn record_clip(&mut self, content: &str) {
        // Reject empty or oversized content.
        if content.is_empty() || content.len() > MAX_CONTENT_BYTES {
            return;
        }

        let now = current_timestamp();

        if let Some(pos) = self.history.iter().position(|e| e.content == content) {
            // Promote + update.
            let mut entry = self.history.remove(pos).unwrap();
            entry.use_count += 1;
            entry.last_used = now;
            self.history.push_front(entry);
        } else {
            self.history.push_front(ClipEntry {
                content: content.to_string(),
                first_seen: now,
                last_used: now,
                use_count: 1,
            });

            if self.history.len() > MAX_HISTORY {
                self.history.pop_back();
            }
        }

        self.dirty = true;
        self.rebuild_searchable();
    }

    /// Rebuild the `SearchableItem` index used by the fuzzy engine.
    fn rebuild_searchable(&mut self) {
        self.searchable = self
            .history
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                let title = make_title(&entry.content);
                let char_count = entry.content.chars().count();

                let description = format!(
                    "{} char{} · used {} time{}",
                    char_count,
                    if char_count == 1 { "" } else { "s" },
                    entry.use_count,
                    if entry.use_count == 1 { "" } else { "s" },
                );

                let item = Item {
                    // Stable ID: position in history + content hash-ish via len.
                    // We use the content itself as a canonical key so that
                    // promoted entries keep their id stable.
                    id: format!("clip:{}", idx),
                    title: title.clone(),
                    search_text: entry.content.to_lowercase(),
                    description: Some(description),
                    // Metadata IS the content — simple and direct.
                    metadata: Some(entry.content.clone()),
                };

                SearchableItem::new(item)
                    // Primary field: full content text (lower weight because
                    // it can be huge).
                    .with_field("content", &entry.content, 5.0)
                    // Secondary field: the display title (first line / truncated).
                    .with_field("title", &title, 8.0)
            })
            .collect();
    }

    /// Return recent entries for an empty query, scored by recency + frequency.
    fn get_recent_items(&self) -> Vec<Item> {
        let limit = self.history.len().min(RECENT_DISPLAY_LIMIT);

        let mut scored: Vec<(Item, f64)> = self
            .history
            .iter()
            .take(limit)
            .enumerate()
            .map(|(idx, entry)| {
                let title = make_title(&entry.content);
                let char_count = entry.content.chars().count();

                let description = format!(
                    "{} char{} · used {} time{}",
                    char_count,
                    if char_count == 1 { "" } else { "s" },
                    entry.use_count,
                    if entry.use_count == 1 { "" } else { "s" },
                );

                let item = Item {
                    id: format!("clip:{}", idx),
                    title,
                    search_text: entry.content.to_lowercase(),
                    description: Some(description),
                    metadata: Some(entry.content.clone()),
                };

                // Score = recency weight + log-frequency bonus.
                let recency = (limit - idx) as f64 * 10.0;
                let freq = (entry.use_count as f64 + 1.0).ln() * 15.0;
                (item, recency + freq)
            })
            .collect();

        scored.sort_by(|a, b| {
            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
        });

        scored.into_iter().map(|(item, _)| item).collect()
    }

    // ── Clipboard I/O ────────────────────────────────────────────────────────

    /// Write `content` to the system clipboard using the detected backend.
    fn write_clipboard(&self, content: &str) -> Result<(), LatuiError> {
        match &self.backend {
            ClipBackend::Wayland => {
                let mut child = Command::new("wl-copy")
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                    .map_err(LatuiError::Io)?;

                if let Some(stdin) = child.stdin.as_mut() {
                    use std::io::Write;
                    stdin.write_all(content.as_bytes()).map_err(LatuiError::Io)?;
                }

                child.wait().map_err(LatuiError::Io)?;
                tracing::debug!(
                    "Wrote {} bytes to Wayland clipboard",
                    content.len()
                );
            }

            ClipBackend::X11 => {
                let mut child = Command::new("xclip")
                    .args(["-selection", "clipboard"])
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                    .map_err(LatuiError::Io)?;

                if let Some(stdin) = child.stdin.as_mut() {
                    use std::io::Write;
                    stdin.write_all(content.as_bytes()).map_err(LatuiError::Io)?;
                }

                child.wait().map_err(LatuiError::Io)?;
                tracing::debug!(
                    "Wrote {} bytes to X11 clipboard",
                    content.len()
                );
            }

            ClipBackend::None => {
                tracing::warn!(
                    "No clipboard backend available; cannot write to clipboard"
                );
                return Err(LatuiError::App(
                    "No clipboard tool found (install wl-copy or xclip)".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Read the current system clipboard content.
    /// Returns `None` if the backend is unavailable or the command fails.
    #[allow(dead_code)]
    fn read_clipboard(&self) -> Option<String> {
        let output = match &self.backend {
            ClipBackend::Wayland => Command::new("wl-paste")
                .arg("--no-newline")
                .output()
                .ok()?,
            ClipBackend::X11 => Command::new("xclip")
                .args(["-selection", "clipboard", "-o"])
                .output()
                .ok()?,
            ClipBackend::None => return None,
        };

        if output.status.success() {
            String::from_utf8(output.stdout).ok()
        } else {
            None
        }
    }

    // ── Validation ───────────────────────────────────────────────────────────

    /// Validate clipboard content before attempting to write it.
    fn validate_content(content: &str) -> Result<(), LatuiError> {
        if content.is_empty() {
            return Err(LatuiError::App(
                "Clipboard content is empty".to_string(),
            ));
        }
        if content.len() > MAX_CONTENT_BYTES {
            return Err(LatuiError::App(format!(
                "Content too large ({} bytes, max {})",
                content.len(),
                MAX_CONTENT_BYTES
            )));
        }
        Ok(())
    }
}

impl Default for ClipboardMode {
    fn default() -> Self {
        Self::new()
    }
}


// ─── Mode trait implementation ────────────────────────────────────────────────

impl Mode for ClipboardMode {
    fn name(&self) -> &str {
        "clipboard"
    }

    fn icon(&self) -> &str {
        "📋"
    }

    fn description(&self) -> &str {
        "Clipboard History"
    }

    fn stays_open(&self) -> bool { true }

    // ── load ──────────────────────────────────────────────────────────────

    fn load(&mut self) -> Result<(), LatuiError> {
        tracing::debug!(
            "Loading clipboard mode (backend: {})",
            self.backend.name()
        );

        self.load_history()?;
        self.rebuild_searchable();

        tracing::info!(
            "Clipboard mode loaded with {} entries",
            self.history.len()
        );
        Ok(())
    }

    // ── search ────────────────────────────────────────────────────────────

    /// Search strategy:
    /// - **Empty query** → recent entries sorted by recency + use-count.
    /// - **Non-empty query** → fuzzy match over the full content and title
    ///   using the shared `SearchEngine`.
    fn search(&mut self, query: &str) -> Vec<Item> {
        let start = Instant::now();

        if query.is_empty() {
            let results = self.get_recent_items();
            tracing::trace!(
                "Clipboard empty query → {} items in {:?}",
                results.len(),
                start.elapsed()
            );
            return results;
        }

        let q = query.trim();
        let mut results = self.search_engine.search_scored(q, &self.searchable);

        // Re-sort (SearchEngine already sorts, but we make it explicit).
        results.sort_by(|a, b| {
            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
        });

        let items: Vec<Item> = results
            .into_iter()
            .filter(|(_, score)| *score > 0.0)
            .map(|(item, _)| item)
            .collect();

        tracing::trace!(
            "Clipboard search '{}' → {} results in {:?}",
            q,
            items.len(),
            start.elapsed()
        );

        items
    }

    // ── execute ───────────────────────────────────────────────────────────

    /// Paste the selected clipboard entry back to the system clipboard.
    ///
    /// The content is taken directly from `item.metadata` (the raw clip text).
    fn execute(&mut self, item: &Item) -> Result<(), LatuiError> {
        // Rate-limit: prevent double-pastes from key bounce.
        if let Some(last) = self.last_action_time
            && last.elapsed() < std::time::Duration::from_millis(500) {
                tracing::warn!("Rate-limiting clipboard paste");
                return Ok(());
            }
        self.last_action_time = Some(Instant::now());

        let content = item
            .metadata
            .as_ref()
            .ok_or_else(|| LatuiError::App("Missing clipboard content".to_string()))?;

        Self::validate_content(content)?;

        // Write to system clipboard.
        self.write_clipboard(content)?;

        // Promote this entry in history (re-paste counts as a use).
        self.record_clip(content);

        // Persist.
        if let Err(e) = self.save_history() {
            tracing::error!("Failed to save clipboard history: {}", e);
        }

        Ok(())
    }

    // ── record_selection ─────────────────────────────────────────────────

    fn record_selection(&mut self, _query: &str, item: &Item) {
        // Rate-limit.
        if let Some(last) = self.last_action_time
            && last.elapsed() < std::time::Duration::from_millis(200) {
                return;
            }
        self.last_action_time = Some(Instant::now());

        // Log at trace level only — never log actual clipboard content.
        tracing::trace!(
            "Clipboard selection: item='{}', len={}",
            item.title,
            item.metadata.as_ref().map(|m| m.len()).unwrap_or(0)
        );
    }

    // ── preview ───────────────────────────────────────────────────────────

    fn supports_preview(&self) -> bool {
        true
    }

    /// Returns the full clip content for multi-line previews.
    ///
    /// Long content is truncated at `PREVIEW_MAX_LINES` with a trailing
    /// ellipsis so the preview panel doesn't overflow.
    fn preview(&self, item: &Item) -> Option<String> {
        let content = item.metadata.as_ref()?;

        const PREVIEW_MAX_LINES: usize = 20;
        let lines: Vec<&str> = content.lines().collect();

        if lines.len() <= PREVIEW_MAX_LINES {
            // Return as-is for short clips.
            return Some(content.clone());
        }

        // Truncate and indicate how many lines were omitted.
        let shown: Vec<&str> = lines[..PREVIEW_MAX_LINES].to_vec();
        let remaining = lines.len() - PREVIEW_MAX_LINES;
        let mut preview = shown.join("\n");
        preview.push_str(&format!(
            "\n\n… {} more line{}",
            remaining,
            if remaining == 1 { "" } else { "s" }
        ));
        Some(preview)
    }
}

// ─── Drop impl ────────────────────────────────────────────────────────────────

impl Drop for ClipboardMode {
    fn drop(&mut self) {
        if self.dirty
            && let Err(e) = self.save_history() {
                tracing::error!("Failed to save clipboard history on drop: {}", e);
            }
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Produce a compact display title from clipboard content.
///
/// Uses the first non-empty line, truncated to `TITLE_MAX_CHARS`.
/// Multi-line content is flagged with a `⏎` indicator.
fn make_title(content: &str) -> String {
    let is_multiline = content.contains('\n');

    let first_line = content
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or(content)
        .trim();

    let truncated: String = first_line.chars().take(TITLE_MAX_CHARS).collect();
    let ellipsis = if first_line.chars().count() > TITLE_MAX_CHARS {
        "…"
    } else {
        ""
    };

    if is_multiline {
        format!("{}{} ⏎", truncated, ellipsis)
    } else {
        format!("{}{}", truncated, ellipsis)
    }
}

/// Return the current Unix timestamp in seconds.
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Check whether a command exists in `PATH`.
fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn mode() -> ClipboardMode {
        ClipboardMode::new()
    }

    // ── Construction ──────────────────────────────────────────────────────

    #[test]
    fn test_creation() {
        let m = mode();
        assert_eq!(m.name(), "clipboard");
        assert_eq!(m.icon(), "📋");
        assert!(m.history.is_empty());
    }

    // ── record_clip ───────────────────────────────────────────────────────

    #[test]
    fn test_add_single_entry() {
        let mut m = mode();
        m.record_clip("hello world");
        assert_eq!(m.history.len(), 1);
        assert_eq!(m.history[0].content, "hello world");
        assert_eq!(m.history[0].use_count, 1);
    }

    #[test]
    fn test_duplicate_promotes_and_increments() {
        let mut m = mode();
        m.record_clip("first");
        m.record_clip("second");
        m.record_clip("first"); // promote
        assert_eq!(m.history.len(), 2);
        assert_eq!(m.history[0].content, "first");
        assert_eq!(m.history[0].use_count, 2);
    }

    #[test]
    fn test_history_capped_at_max() {
        let mut m = mode();
        for i in 0..MAX_HISTORY + 50 {
            m.record_clip(&format!("clip-{}", i));
        }
        assert_eq!(m.history.len(), MAX_HISTORY);
    }

    #[test]
    fn test_empty_content_ignored() {
        let mut m = mode();
        m.record_clip("");
        assert!(m.history.is_empty());
    }

    #[test]
    fn test_oversized_content_ignored() {
        let mut m = mode();
        let huge = "x".repeat(MAX_CONTENT_BYTES + 1);
        m.record_clip(&huge);
        assert!(m.history.is_empty());
    }

    // ── make_title ────────────────────────────────────────────────────────

    #[test]
    fn test_title_short_single_line() {
        let t = make_title("Hello, world!");
        assert_eq!(t, "Hello, world!");
    }

    #[test]
    fn test_title_multiline_shows_indicator() {
        let t = make_title("Line one\nLine two");
        assert!(t.contains('⏎'));
        assert!(t.starts_with("Line one"));
    }

    #[test]
    fn test_title_truncated_at_max_chars() {
        let long = "a".repeat(TITLE_MAX_CHARS + 10);
        let t = make_title(&long);
        // Should end with ellipsis
        assert!(t.ends_with('…'));
        // Char count should be TITLE_MAX_CHARS + 1 (the ellipsis)
        assert_eq!(t.chars().count(), TITLE_MAX_CHARS + 1);
    }

    #[test]
    fn test_title_skips_leading_empty_lines() {
        let t = make_title("\n\n  actual content");
        assert!(t.starts_with("actual content"));
    }

    // ── validate_content ──────────────────────────────────────────────────

    #[test]
    fn test_validate_empty_fails() {
        assert!(ClipboardMode::validate_content("").is_err());
    }

    #[test]
    fn test_validate_oversized_fails() {
        let big = "x".repeat(MAX_CONTENT_BYTES + 1);
        assert!(ClipboardMode::validate_content(&big).is_err());
    }

    #[test]
    fn test_validate_normal_passes() {
        assert!(ClipboardMode::validate_content("some text").is_ok());
    }

    // ── search ────────────────────────────────────────────────────────────

    #[test]
    fn test_search_empty_returns_recent() {
        let mut m = mode();
        m.record_clip("alpha");
        m.record_clip("beta");
        let results = m.search("");
        // Most recent ("beta") should be first.
        assert!(!results.is_empty());
        assert_eq!(results[0].metadata.as_deref(), Some("beta"));
    }

    #[test]
    fn test_search_finds_substring() {
        let mut m = mode();
        m.record_clip("the quick brown fox");
        m.record_clip("hello world");
        let results = m.search("quick");
        assert!(!results.is_empty());
        assert!(results[0]
            .metadata
            .as_deref()
            .unwrap()
            .contains("quick"));
    }

    #[test]
    fn test_search_no_match_returns_empty() {
        let mut m = mode();
        m.record_clip("hello");
        let results = m.search("zzzznotexisting");
        assert!(results.is_empty());
    }

    // ── preview ───────────────────────────────────────────────────────────

    #[test]
    fn test_preview_short_content() {
        let m = mode();
        let item = Item {
            id: "clip:0".into(),
            title: "hello".into(),
            search_text: "hello".into(),
            description: None,
            metadata: Some("hello world".into()),
        };
        let preview = m.preview(&item).unwrap();
        assert_eq!(preview, "hello world");
    }

    #[test]
    fn test_preview_truncates_long_content() {
        let m = mode();
        let mut lines = Vec::new();
        for i in 0..30 {
            lines.push(format!("Line {}", i));
        }
        let content = lines.join("\n");
        let item = Item {
            id: "clip:0".into(),
            title: "multi".into(),
            search_text: "multi".into(),
            description: None,
            metadata: Some(content),
        };
        let preview = m.preview(&item).unwrap();
        assert!(preview.contains("more line"));
    }

    #[test]
    fn test_preview_none_when_no_metadata() {
        let m = mode();
        let item = Item {
            id: "clip:0".into(),
            title: "empty".into(),
            search_text: "empty".into(),
            description: None,
            metadata: None,
        };
        assert!(m.preview(&item).is_none());
    }

    // ── ClipBackend ───────────────────────────────────────────────────────

    #[test]
    fn test_backend_name() {
        assert_eq!(ClipBackend::Wayland.name(), "wl-copy (Wayland)");
        assert_eq!(ClipBackend::X11.name(), "xclip (X11)");
        assert_eq!(ClipBackend::None.name(), "none");
    }

    // ── Dirty flag ────────────────────────────────────────────────────────

    #[test]
    fn test_dirty_set_after_record() {
        let mut m = mode();
        assert!(!m.dirty);
        m.record_clip("some content");
        assert!(m.dirty);
    }
}
