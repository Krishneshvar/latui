use crate::core::{item::Item, mode::Mode, searchable_item::SearchableItem};
use crate::error::LatuiError;
use crate::search::engine::SearchEngine;
use crate::tracking::frequency::FrequencyTracker;

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

/// Maximum number of commands to keep in history
const MAX_HISTORY_SIZE: usize = 1000;

/// Maximum number of recent commands to show when query is empty
const RECENT_COMMANDS_LIMIT: usize = 20;

/// Command history entry
#[derive(Clone, Serialize, Deserialize, Debug)]
struct HistoryEntry {
    command: String,
    timestamp: u64,
    execution_count: u32,
}

/// Run mode for executing shell commands with intelligent history tracking
pub struct RunMode {
    /// Command history (most recent first)
    history: VecDeque<HistoryEntry>,

    /// Searchable items built from history
    searchable_history: Vec<SearchableItem>,

    /// Search engine for fuzzy matching
    search_engine: SearchEngine,

    /// Frequency tracker for usage-based ranking
    frequency_tracker: Option<FrequencyTracker>,

    /// Path to history file
    history_path: Option<PathBuf>,

    /// Shell to use for execution
    shell: String,

    /// Rate limiting
    last_action_time: Option<Instant>,

    /// Whether history has been modified (needs saving)
    dirty: bool,
}

impl RunMode {
    pub fn new() -> Self {
        Self::with_tracker(None)
    }

    pub fn with_tracker(frequency_tracker: Option<FrequencyTracker>) -> Self {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

        Self {
            history: VecDeque::new(),
            searchable_history: Vec::new(),
            search_engine: SearchEngine::new(),
            frequency_tracker,
            history_path: None,
            shell,
            last_action_time: None,
            dirty: false,
        }
    }
}

impl Default for RunMode {
    fn default() -> Self {
        Self::new()
    }
}

impl RunMode {
    /// Load command history from disk
    fn load_history(&mut self) -> Result<(), LatuiError> {
        use xdg::BaseDirectories;

        let xdg = BaseDirectories::with_prefix("latui");
        let history_path = xdg
            .place_data_file("run_history.json")
            .map_err(|e| LatuiError::Io(std::io::Error::other(e)))?;

        self.history_path = Some(history_path.clone());

        // Try to load existing history
        if history_path.exists() {
            match std::fs::read_to_string(&history_path) {
                Ok(data) => {
                    // Validate file size (max 1MB)
                    if data.len() > 1024 * 1024 {
                        tracing::warn!("History file too large, truncating");
                        return Ok(());
                    }

                    match serde_json::from_str::<Vec<HistoryEntry>>(&data) {
                        Ok(mut entries) => {
                            // Limit to MAX_HISTORY_SIZE
                            entries.truncate(MAX_HISTORY_SIZE);
                            self.history = entries.into();
                            tracing::info!("Loaded {} commands from history", self.history.len());
                        }
                        Err(e) => {
                            tracing::warn!("Failed to parse history file: {}", e);
                        }
                    }
                }
                Err(e) => {
                    tracing::debug!("No existing history file: {}", e);
                }
            }
        }

        // Set secure permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(parent) = history_path.parent() {
                let _ = std::fs::create_dir_all(parent);
                let _ = std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700));
            }
        }

        Ok(())
    }

    /// Save command history to disk
    fn save_history(&mut self) -> Result<(), LatuiError> {
        if !self.dirty {
            return Ok(());
        }

        let history_path = match &self.history_path {
            Some(path) => path,
            None => return Ok(()),
        };

        let entries: Vec<HistoryEntry> = self.history.iter().cloned().collect();

        let json = serde_json::to_string_pretty(&entries)
            .map_err(|e| LatuiError::Io(std::io::Error::other(e)))?;

        // Write to temporary file for atomicity
        let mut tmp_path = history_path.clone();
        tmp_path.set_extension("tmp");

        std::fs::write(&tmp_path, json)?;

        // Set secure permissions on tmp file before moving
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o600));
        }

        std::fs::rename(&tmp_path, history_path)?;

        self.dirty = false;
        tracing::debug!("Saved {} commands to history", self.history.len());

        Ok(())
    }

    /// Add command to history
    fn add_to_history(&mut self, command: &str) {
        let now = current_timestamp();

        // Check if command already exists in history
        if let Some(pos) = self.history.iter().position(|e| e.command == command) {
            // Move to front and increment count
            let mut entry = self.history.remove(pos).unwrap();
            entry.execution_count += 1;
            entry.timestamp = now;
            self.history.push_front(entry);
        } else {
            // Add new entry
            let entry = HistoryEntry {
                command: command.to_string(),
                timestamp: now,
                execution_count: 1,
            };
            self.history.push_front(entry);

            // Limit history size
            if self.history.len() > MAX_HISTORY_SIZE {
                self.history.pop_back();
            }
        }

        self.dirty = true;
        self.rebuild_searchable_history();
    }

    /// Rebuild searchable items from history
    fn rebuild_searchable_history(&mut self) {
        self.searchable_history = self
            .history
            .iter()
            .map(|entry| {
                let item = Item {
                    id: format!("cmd:{}", entry.command),
                    title: entry.command.clone(),
                    search_text: entry.command.to_lowercase(),
                    description: Some(format!(
                        "Executed {} time{}",
                        entry.execution_count,
                        if entry.execution_count == 1 { "" } else { "s" }
                    )),
                    icon: None,
                    metadata: Some(entry.command.clone()),
                };

                SearchableItem::new(item).with_field("command", &entry.command, 10.0)
            })
            .collect();
    }

    /// Get recent commands for empty query
    fn get_recent_commands(&self) -> Vec<Item> {
        let mut results: Vec<(Item, f64)> = self
            .history
            .iter()
            .take(RECENT_COMMANDS_LIMIT)
            .enumerate()
            .map(|(idx, entry)| {
                let item = Item {
                    id: format!("cmd:{}", entry.command),
                    title: entry.command.clone(),
                    search_text: entry.command.to_lowercase(),
                    description: Some(format!(
                        "Executed {} time{}",
                        entry.execution_count,
                        if entry.execution_count == 1 { "" } else { "s" }
                    )),
                    icon: None,
                    metadata: Some(entry.command.clone()),
                };

                // Score based on recency and frequency
                let recency_score = (RECENT_COMMANDS_LIMIT - idx) as f64 * 10.0;
                let frequency_score = (entry.execution_count as f64).ln() * 20.0;
                let mut score = recency_score + frequency_score;

                // Add frequency tracker boost if available
                if let Some(ref tracker) = self.frequency_tracker {
                    score += tracker.get_total_boost(&item.id);
                }

                (item, score)
            })
            .collect();

        // Sort by score
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        results.into_iter().map(|(item, _)| item).collect()
    }

    /// Validate command for security
    fn validate_command(&self, command: &str) -> Result<(), LatuiError> {
        // Check length
        if command.is_empty() {
            return Err(LatuiError::App("Command cannot be empty".to_string()));
        }

        if command.len() > 4096 {
            return Err(LatuiError::App("Command too long".to_string()));
        }

        // Check for null bytes (security)
        if command.contains('\0') {
            return Err(LatuiError::App("Command contains null bytes".to_string()));
        }

        Ok(())
    }

    /// Execute a shell command
    fn execute_command(&mut self, command: &str) -> Result<(), LatuiError> {
        self.validate_command(command)?;

        tracing::info!("Executing command: {}", command);

        // Spawn command in background
        let child = Command::new(&self.shell).arg("-c").arg(command).spawn();

        match child {
            Ok(_) => {
                self.add_to_history(command);

                // Record in frequency tracker
                if let Some(ref mut tracker) = self.frequency_tracker {
                    let id = format!("cmd:{}", command);
                    if let Err(e) = tracker.record_launch(&id) {
                        tracing::error!("Failed to record command execution: {}", e);
                    }
                }

                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to execute command '{}': {}", command, e);
                Err(LatuiError::Io(e))
            }
        }
    }
}

impl Mode for RunMode {
    fn name(&self) -> &str {
        "run"
    }

    fn icon(&self) -> &str {
        "🚀"
    }

    fn description(&self) -> &str {
        "Command Executor"
    }

    fn load(&mut self) -> Result<(), LatuiError> {
        tracing::debug!("Loading run mode with shell: {}", self.shell);

        self.load_history()?;
        self.rebuild_searchable_history();

        tracing::info!(
            "Run mode loaded with {} commands in history",
            self.history.len()
        );
        Ok(())
    }

    fn search(&mut self, query: &str) -> Vec<Item> {
        // Empty query: show recent commands
        if query.is_empty() {
            return self.get_recent_commands();
        }

        let start = Instant::now();
        let q = query.trim();

        let mut results: Vec<(Item, f64)> = Vec::new();

        // Always include direct execution option as first result
        let direct_item = Item {
            id: format!("direct:{}", q),
            title: q.to_string(),
            search_text: q.to_lowercase(),
            description: Some("Execute command".to_string()),
            icon: None,
            metadata: Some(q.to_string()),
        };
        results.push((direct_item, 10000.0)); // Highest priority

        // Search through history
        if !self.searchable_history.is_empty() {
            let mut history_results = self
                .search_engine
                .search_scored(q, &self.searchable_history);

            // Apply frequency boosts
            if let Some(ref tracker) = self.frequency_tracker {
                for (item, score) in history_results.iter_mut() {
                    *score += tracker.get_frequency_boost(&item.id);
                    *score += tracker.get_recency_boost(&item.id);
                    *score += tracker.get_query_boost(q, &item.id);
                }

                // Re-sort after applying boosts
                history_results
                    .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            }

            // Add history results (but with lower priority than direct execution)
            for (item, score) in history_results {
                if score > 0.0 {
                    results.push((item, score));
                }
            }
        }

        let final_results: Vec<Item> = results.into_iter().map(|(item, _)| item).collect();

        tracing::trace!(
            "Run mode search for '{}' completed in {:?} with {} results",
            query,
            start.elapsed(),
            final_results.len()
        );

        final_results
    }

    fn execute(&mut self, item: &Item) -> Result<(), LatuiError> {
        // Rate limiting
        if let Some(last) = self.last_action_time
            && last.elapsed() < std::time::Duration::from_millis(500)
        {
            tracing::warn!("Rate limiting execution for: {}", item.title);
            return Ok(());
        }
        self.last_action_time = Some(Instant::now());

        // Extract command from metadata
        let command = item
            .metadata
            .as_ref()
            .ok_or_else(|| LatuiError::App("Missing command metadata".to_string()))?;

        // Execute the command
        self.execute_command(command)?;

        // Save history after execution
        if let Err(e) = self.save_history() {
            tracing::error!("Failed to save command history: {}", e);
        }

        Ok(())
    }

    fn record_selection(&mut self, query: &str, item: &Item) {
        // Rate limiting
        if let Some(last) = self.last_action_time
            && last.elapsed() < std::time::Duration::from_millis(200)
        {
            return;
        }
        self.last_action_time = Some(Instant::now());

        // Record in frequency tracker
        if let Some(ref mut tracker) = self.frequency_tracker
            && let Err(e) = tracker.record_selection(query, &item.id)
        {
            tracing::error!("Failed to record selection: {}", e);
        }
    }
}

impl Drop for RunMode {
    fn drop(&mut self) {
        // Save history on drop
        if self.dirty
            && let Err(e) = self.save_history()
        {
            tracing::error!("Failed to save history on drop: {}", e);
        }
    }
}

/// Get current Unix timestamp
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_mode_creation() {
        let mode = RunMode::new();
        assert_eq!(mode.name(), "run");
        assert_eq!(mode.icon(), "🚀");
        assert!(mode.history.is_empty());
    }

    #[test]
    fn test_add_to_history() {
        let mut mode = RunMode::new();
        mode.add_to_history("ls -la");

        assert_eq!(mode.history.len(), 1);
        assert_eq!(mode.history[0].command, "ls -la");
        assert_eq!(mode.history[0].execution_count, 1);
    }

    #[test]
    fn test_duplicate_command_increments_count() {
        let mut mode = RunMode::new();
        mode.add_to_history("ls -la");
        mode.add_to_history("ls -la");

        assert_eq!(mode.history.len(), 1);
        assert_eq!(mode.history[0].execution_count, 2);
    }

    #[test]
    fn test_history_size_limit() {
        let mut mode = RunMode::new();

        for i in 0..MAX_HISTORY_SIZE + 10 {
            mode.add_to_history(&format!("command_{}", i));
        }

        assert_eq!(mode.history.len(), MAX_HISTORY_SIZE);
    }

    #[test]
    fn test_validate_command() {
        let mode = RunMode::new();

        assert!(mode.validate_command("ls -la").is_ok());
        assert!(mode.validate_command("").is_err());
        assert!(mode.validate_command(&"x".repeat(5000)).is_err());
        assert!(mode.validate_command("test\0null").is_err());
    }

    #[test]
    fn test_search_empty_query() {
        let mut mode = RunMode::new();
        mode.add_to_history("ls -la");
        mode.add_to_history("cd /tmp");
        mode.rebuild_searchable_history();

        let results = mode.search("");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_with_query() {
        let mut mode = RunMode::new();
        mode.add_to_history("ls -la");
        mode.add_to_history("cd /tmp");
        mode.rebuild_searchable_history();

        let results = mode.search("ls");
        assert!(!results.is_empty());
        assert_eq!(results[0].title, "ls"); // Direct execution first
    }
}
