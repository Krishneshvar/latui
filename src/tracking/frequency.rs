use crate::tracking::database::{Database, DatabaseError};
use std::path::Path;
use crate::core::utils::current_timestamp;

/// Tracks application launch frequency and calculates boosts
#[derive(Debug)]
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
                ((f64::from(stats.launch_count)).ln_1p() * 20.0).min(100.0)
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
                    0..=1 => 50.0,     // Used in last hour
                    2..=6 => 40.0,     // Used in last 6 hours
                    7..=24 => 30.0,    // Used today
                    25..=72 => 20.0,   // Used in last 3 days
                    73..=168 => 15.0,  // Used this week
                    169..=720 => 10.0, // Used this month
                    _ => 0.0,          // Older than a month
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
                let total_selections: u32 = stats.iter().map(|(_, count): &(String, u32)| *count).sum();
                if total_selections == 0 {
                    return 0.0;
                }

                // Find count for this specific app
                let app_selections = stats
                    .iter()
                    .find(|(id, _)| id == app_id)
                    .map_or(0, |(_, count)| *count);

                if app_selections == 0 {
                    return 0.0;
                }

                // Calculate percentage-based boost
                // If user always selects this app for this query, give max boost
                let percentage = (f64::from(app_selections) / f64::from(total_selections)) * 100.0;

                // Scale: 0-100% → 0-50 points
                (percentage / 2.0).min(50.0)
            }
            _ => 0.0,
        }
    }

    /// Cleanup old data
    pub fn cleanup(&mut self, days_old: u64) -> Result<(), DatabaseError> {
        self.db.cleanup_old_selections(days_old)
    }
}


