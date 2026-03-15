/// Ranks search results with frequency and recency boosts
pub struct Ranker;

impl Ranker {
    pub fn new() -> Self {
        Self
    }

    /// Calculate frequency boost
    pub fn frequency_boost(&self, launch_count: u32) -> f64 {
        ((launch_count as f64) + 1.0).ln() * 20.0
    }

    /// Calculate recency boost
    pub fn recency_boost(&self, last_used_timestamp: u64) -> f64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let hours_since = (now - last_used_timestamp) / 3600;

        match hours_since {
            0..=1 => 50.0,
            2..=24 => 30.0,
            25..=168 => 15.0,
            _ => 0.0,
        }
    }

    /// Calculate final score with boosts
    pub fn rank(&self, base_score: f64, frequency: u32, last_used: u64) -> f64 {
        base_score + self.frequency_boost(frequency) + self.recency_boost(last_used)
    }
}
