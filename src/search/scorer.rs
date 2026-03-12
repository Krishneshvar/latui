use crate::matcher::fuzzy::FuzzyMatcher;

/// Hybrid scoring system combining multiple matching algorithms
pub struct HybridScorer {
    fuzzy_matcher: FuzzyMatcher,
}

impl HybridScorer {
    pub fn new() -> Self {
        Self {
            fuzzy_matcher: FuzzyMatcher::new(),
        }
    }

    /// Score a query against text
    pub fn score(&mut self, query: &str, text: &str) -> f64 {
        // TODO: Implement hybrid scoring
        // For now, just use fuzzy matching
        let results = self.fuzzy_matcher.filter(query, &[text]);
        results.first().map(|(_, score)| *score as f64).unwrap_or(0.0)
    }

    /// Check for exact match
    pub fn exact_match(&self, query: &str, text: &str) -> bool {
        query.to_lowercase() == text.to_lowercase()
    }

    /// Check for prefix match
    pub fn prefix_match(&self, query: &str, text: &str) -> bool {
        text.to_lowercase().starts_with(&query.to_lowercase())
    }

    /// Check for word boundary match
    pub fn word_boundary_match(&self, query: &str, text: &str) -> bool {
        text.to_lowercase()
            .split_whitespace()
            .any(|word| word.starts_with(&query.to_lowercase()))
    }
}
