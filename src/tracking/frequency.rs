use std::collections::HashMap;

/// Tracks application launch frequency
pub struct FrequencyTracker {
    stats: HashMap<String, UsageStats>,
}

#[derive(Clone)]
pub struct UsageStats {
    pub launch_count: u32,
    pub last_used: u64,
}

impl FrequencyTracker {
    pub fn new() -> Self {
        Self {
            stats: HashMap::new(),
        }
    }

    /// Record an app launch
    pub fn record_launch(&mut self, app_id: &str) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.stats
            .entry(app_id.to_string())
            .and_modify(|s| {
                s.launch_count += 1;
                s.last_used = now;
            })
            .or_insert(UsageStats {
                launch_count: 1,
                last_used: now,
            });
    }

    /// Get usage stats for an app
    pub fn get_stats(&self, app_id: &str) -> Option<&UsageStats> {
        self.stats.get(app_id)
    }
}
