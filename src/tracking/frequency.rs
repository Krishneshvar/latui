use crate::tracking::database::{Database, DatabaseError, UsageStats};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Tracks application launch frequency and calculates boosts
pub struct FrequencyTracker {
    db: Database,
}

impl FrequencyTracker {
    /// Create a new frequency tracker with database
    pub fn new(db_path: &Path) -> Result<Self, DatabaseError> {
        let db = Database::new(db_path)?;
        Ok(Self { db })
    }

    /// Record an app launch
    pub fn record_launch(&mut self, app_id: &str) -> Result<(), DatabaseError> {
        self.db.record_launch(app_id)
    }

    /// Record a query → app selection
    pub fn record_selection(&mut self, query: &str, app_id: &str) -> Result<(), DatabaseError> {
        self.db.record_selection(query, app_id)
    }

    /// Get usage stats for an app
    pub fn get_stats(&self, app_id: &str) -> Result<Option<UsageStats>, DatabaseError> {
        self.db.get_usage_stats(app_id)
    }

    /// Calculate frequency boost for an app
    /// Uses logarithmic scale for diminishing returns
    pub fn get_frequency_boost(&self, app_id: &str) -> f64 {
        match self.db.get_usage_stats(app_id) {
            Ok(Some(stats)) => {
                // Logarithmic boost: ln(count + 1) * 20
                // Examples:
                // - 0 launches: 0.0
                // - 1 launch: 13.86
                // - 5 launches: 35.84
                // - 10 launches: 47.96
                // - 50 launches: 76.35
                // - 100 launches: 92.10
                ((stats.launch_count as f64 + 1.0).ln() * 20.0).min(100.0)
            }
            _ => 0.0,
        }
    }

    /// Calculate recency boost for an app
    /// Recent usage gets higher boost with time decay
    pub fn get_recency_boost(&self, app_id: &str) -> f64 {
        match self.db.get_usage_stats(app_id) {
            Ok(Some(stats)) => {
                if stats.last_used == 0 {
                    return 0.0;
                }

                let now = current_timestamp();
                let seconds_since_use = now.saturating_sub(stats.last_used);
                let hours_since_use = seconds_since_use / 3600;

                // Time-based boost with decay
                match hours_since_use {
                    0..=1 => 50.0,      // Used in last hour
                    2..=6 => 40.0,      // Used in last 6 hours
                    7..=24 => 30.0,     // Used today
                    25..=72 => 20.0,    // Used in last 3 days
                    73..=168 => 15.0,   // Used this week
                    169..=720 => 10.0,  // Used this month
                    _ => 0.0,           // Older than a month
                }
            }
            _ => 0.0,
        }
    }

    /// Get combined boost (frequency + recency)
    pub fn get_total_boost(&self, app_id: &str) -> f64 {
        self.get_frequency_boost(app_id) + self.get_recency_boost(app_id)
    }

    /// Get query-specific boost based on past selections
    pub fn get_query_boost(&self, query: &str, app_id: &str) -> f64 {
        match self.db.get_query_stats(query) {
            Ok(stats) => {
                // Find this app in the stats
                let total_selections: u32 = stats.iter().map(|(_, count)| count).sum();
                if total_selections == 0 {
                    return 0.0;
                }

                // Find count for this specific app
                let app_selections = stats
                    .iter()
                    .find(|(id, _)| id == app_id)
                    .map(|(_, count)| *count)
                    .unwrap_or(0);

                if app_selections == 0 {
                    return 0.0;
                }

                // Calculate percentage-based boost
                // If user always selects this app for this query, give max boost
                let percentage = (app_selections as f64 / total_selections as f64) * 100.0;
                
                // Scale: 0-100% → 0-50 points
                (percentage / 2.0).min(50.0)
            }
            _ => 0.0,
        }
    }

    /// Get top apps by launch count
    pub fn get_top_apps(&self, limit: usize) -> Vec<(String, u32)> {
        self.db.get_top_apps(limit).unwrap_or_default()
    }

    /// Get recently used apps
    pub fn get_recent_apps(&self, limit: usize) -> Vec<(String, u64)> {
        self.db.get_recent_apps(limit).unwrap_or_default()
    }

    /// Cleanup old data
    pub fn cleanup(&mut self, days_old: u64) -> Result<(), DatabaseError> {
        self.db.cleanup_old_selections(days_old)
    }
}

/// Get current Unix timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_db_path() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("latui_freq_test_{}_{}.db", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        path
    }

    #[test]
    fn test_frequency_boost() {
        let path = temp_db_path();
        let mut tracker = FrequencyTracker::new(&path).unwrap();

        // No launches = no boost
        assert_eq!(tracker.get_frequency_boost("firefox"), 0.0);

        // Record some launches
        for _ in 0..10 {
            tracker.record_launch("firefox").unwrap();
        }

        let boost = tracker.get_frequency_boost("firefox");
        assert!(boost > 0.0);
        assert!(boost < 100.0);

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_recency_boost() {
        let path = temp_db_path();
        let mut tracker = FrequencyTracker::new(&path).unwrap();

        // Record a launch (will have current timestamp)
        tracker.record_launch("firefox").unwrap();

        // Should get high recency boost
        let boost = tracker.get_recency_boost("firefox");
        assert!(boost >= 40.0); // Should be in 0-1 hour or 2-6 hour range

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_query_boost() {
        let path = temp_db_path();
        let mut tracker = FrequencyTracker::new(&path).unwrap();

        // Record selections
        tracker.record_selection("br", "brave").unwrap();
        tracker.record_selection("br", "brave").unwrap();
        tracker.record_selection("br", "brave").unwrap();
        tracker.record_selection("br", "chromium").unwrap();

        // Brave should get higher boost (75% of selections)
        let brave_boost = tracker.get_query_boost("br", "brave");
        let chromium_boost = tracker.get_query_boost("br", "chromium");

        assert!(brave_boost > chromium_boost);
        assert!(brave_boost > 30.0); // 75% → 37.5 points

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_total_boost() {
        let path = temp_db_path();
        let mut tracker = FrequencyTracker::new(&path).unwrap();

        tracker.record_launch("firefox").unwrap();

        let total = tracker.get_total_boost("firefox");
        let freq = tracker.get_frequency_boost("firefox");
        let rec = tracker.get_recency_boost("firefox");

        assert_eq!(total, freq + rec);

        let _ = fs::remove_file(&path);
    }
}
