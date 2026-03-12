use std::collections::HashMap;

/// Learns user preferences from query -> selection patterns
pub struct LearningSystem {
    patterns: HashMap<String, HashMap<String, u32>>,
}

impl LearningSystem {
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
        }
    }

    /// Record a selection for a query
    pub fn record_selection(&mut self, query: &str, app_id: &str) {
        self.patterns
            .entry(query.to_string())
            .or_insert_with(HashMap::new)
            .entry(app_id.to_string())
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }

    /// Get learned boost for an app given a query
    pub fn get_learned_boost(&self, query: &str, app_id: &str) -> f64 {
        if let Some(selections) = self.patterns.get(query) {
            if let Some(count) = selections.get(app_id) {
                return (*count as f64) * 5.0;
            }
        }
        0.0
    }
}
