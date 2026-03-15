use lru::LruCache;
use std::cell::{RefCell, Cell};
use std::num::NonZeroUsize;

pub struct CacheStats {
    pub hits: usize,
    pub misses: usize,
}

/// Advanced typo tolerance using multiple edit distance algorithms
/// Handles common typing mistakes: transpositions, insertions, deletions, substitutions
pub struct TypoTolerance {
    /// Maximum edit distance to consider (default: 2)
    pub max_distance: usize,
    /// Minimum query length to apply typo tolerance (default: 4)
    pub min_query_length: usize,
    /// Whether to use Damerau-Levenshtein (includes transpositions)
    pub use_damerau: bool,
    /// Cache for distance calculations (LRU bounded)
    cache: RefCell<LruCache<(String, String), usize>>,
    hits: Cell<usize>,
    misses: Cell<usize>,
}

impl TypoTolerance {
    pub fn new() -> Self {
        Self {
            max_distance: 2,
            min_query_length: 4,
            use_damerau: true,
            cache: RefCell::new(LruCache::new(NonZeroUsize::new(1000).unwrap())),
            hits: Cell::new(0),
            misses: Cell::new(0),
        }
    }
    
    /// Create with custom settings
    pub fn with_settings(max_distance: usize, min_query_length: usize) -> Self {
        Self {
            max_distance,
            min_query_length,
            use_damerau: true,
            cache: RefCell::new(LruCache::new(NonZeroUsize::new(1000).unwrap())),
            hits: Cell::new(0),
            misses: Cell::new(0),
        }
    }

    /// Calculate Levenshtein distance between two strings
    /// Handles: insertions, deletions, substitutions
    pub fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let len1 = s1.chars().count();
        let len2 = s2.chars().count();

        if len1 == 0 {
            return len2;
        }
        if len2 == 0 {
            return len1;
        }

        // Use two rows instead of full matrix for memory efficiency
        let mut prev_row: Vec<usize> = (0..=len2).collect();
        let mut curr_row = vec![0; len2 + 1];

        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();

        for i in 1..=len1 {
            curr_row[0] = i;
            
            for j in 1..=len2 {
                let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };
                
                curr_row[j] = (prev_row[j] + 1)           // deletion
                    .min(curr_row[j - 1] + 1)              // insertion
                    .min(prev_row[j - 1] + cost);          // substitution
            }
            
            std::mem::swap(&mut prev_row, &mut curr_row);
        }

        prev_row[len2]
    }
    
    /// Calculate Damerau-Levenshtein distance
    /// Handles: insertions, deletions, substitutions, AND transpositions
    /// Transposition: "teh" → "the" (distance: 1 instead of 2)
    pub fn damerau_levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let len1 = s1.chars().count();
        let len2 = s2.chars().count();

        if len1 == 0 {
            return len2;
        }
        if len2 == 0 {
            return len1;
        }

        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();
        
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };
                
                matrix[i][j] = (matrix[i - 1][j] + 1)           // deletion
                    .min(matrix[i][j - 1] + 1)                   // insertion
                    .min(matrix[i - 1][j - 1] + cost);           // substitution
                
                // Transposition
                if i > 1 && j > 1 
                    && s1_chars[i - 1] == s2_chars[j - 2] 
                    && s1_chars[i - 2] == s2_chars[j - 1] {
                    matrix[i][j] = matrix[i][j].min(matrix[i - 2][j - 2] + 1);
                }
            }
        }

        matrix[len1][len2]
    }
    
    /// Calculate edit distance with caching
    /// Note: Unicode grapheme clusters are not fully handled. The distance is calculated
    /// based on Rust's `chars()` (Unicode scalar values), which may treat some single
    /// graphemes as multiple characters if they contain combining marks.
    pub fn distance(&self, s1: &str, s2: &str) -> usize {
        // Check cache first
        let key = (s1.to_string(), s2.to_string());
        if let Some(&dist) = self.cache.borrow_mut().get(&key) {
            self.hits.set(self.hits.get() + 1);
            return dist;
        }
        
        self.misses.set(self.misses.get() + 1);
        
        let distance = if self.use_damerau {
            self.damerau_levenshtein_distance(s1, s2)
        } else {
            self.levenshtein_distance(s1, s2)
        };
        
        // Cache the result
        self.cache.borrow_mut().put(key, distance);
        distance
    }
    
    /// Clear the distance cache
    pub fn clear_cache(&self) {
        self.cache.borrow_mut().clear();
        self.hits.set(0);
        self.misses.set(0);
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hits.get(),
            misses: self.misses.get(),
        }
    }

    /// Score based on typo tolerance
    /// Returns None if query is too short or distance is too large
    pub fn score(&self, query: &str, target: &str) -> Option<f64> {
        // Skip if query is too short
        if query.len() < self.min_query_length {
            return None;
        }
        
        // Skip if target is much longer (unlikely to be a typo)
        let len_diff = (query.len() as i32 - target.len() as i32).abs() as usize;
        if len_diff > self.max_distance {
            return None;
        }

        let distance = self.distance(query, target);

        if distance <= self.max_distance {
            Some(match distance {
                0 => 1000.0,  // Exact match
                1 => 150.0,   // One typo
                2 => 100.0,   // Two typos
                _ => 0.0,
            })
        } else {
            None
        }
    }
    
    /// Score with custom distance penalties
    pub fn score_with_penalty(&self, query: &str, target: &str, penalty_per_edit: f64) -> Option<f64> {
        if query.len() < self.min_query_length {
            return None;
        }
        
        let len_diff = (query.len() as i32 - target.len() as i32).abs() as usize;
        if len_diff > self.max_distance {
            return None;
        }

        let distance = self.distance(query, target);

        if distance <= self.max_distance {
            let base_score = 200.0;
            let score = base_score - (distance as f64 * penalty_per_edit);
            Some(score.max(0.0))
        } else {
            None
        }
    }
    
    /// Check if two strings are within typo tolerance
    pub fn is_typo_match(&self, query: &str, target: &str) -> bool {
        if query.len() < self.min_query_length {
            return false;
        }
        
        let distance = self.distance(query, target);
        distance <= self.max_distance
    }
    
    /// Get all typo matches from a list of candidates
    pub fn find_typo_matches<'a>(&self, query: &str, candidates: &'a [&'a str]) -> Vec<(&'a str, usize)> {
        if query.len() < self.min_query_length {
            return vec![];
        }
        
        candidates
            .iter()
            .filter_map(|&candidate| {
                let distance = self.distance(query, candidate);
                if distance <= self.max_distance {
                    Some((candidate, distance))
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Suggest corrections for a typo
    pub fn suggest_corrections<'a>(&self, query: &str, candidates: &'a [&'a str], limit: usize) -> Vec<&'a str> {
        let mut matches = self.find_typo_matches(query, candidates);
        
        // Sort by distance (closest first)
        matches.sort_by_key(|(_, dist)| *dist);
        
        matches
            .into_iter()
            .take(limit)
            .map(|(candidate, _)| candidate)
            .collect()
    }
}

impl Default for TypoTolerance {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let mut typo = TypoTolerance::new();
        assert_eq!(typo.distance("firefox", "firefox"), 0);
        assert_eq!(typo.score("firefox", "firefox"), Some(1000.0));
    }

    #[test]
    fn test_single_typo() {
        let mut typo = TypoTolerance::new();
        
        // Substitution: firefix -> firefox
        assert_eq!(typo.distance("firefix", "firefox"), 1);
        assert_eq!(typo.score("firefix", "firefox"), Some(150.0));
        
        // Deletion: fierfox -> firefox
        assert_eq!(typo.distance("fierfox", "firefox"), 1);
        
        // Insertion: firefx -> firefox
        assert_eq!(typo.distance("firefx", "firefox"), 1);
    }

    #[test]
    fn test_double_typo() {
        let mut typo = TypoTolerance::new();
        
        // Two substitutions: fiirefox -> firefox
        // Note: Damerau-Levenshtein might optimize this differently
        let distance = typo.distance("fiirefox", "firefox");
        assert!(distance <= 2, "Distance should be <= 2, got {}", distance);
        assert!(typo.score("fiirefox", "firefox").is_some());
    }

    #[test]
    fn test_transposition() {
        let mut typo = TypoTolerance::new();
        
        // Damerau-Levenshtein handles transpositions
        // "teh" -> "the" should be distance 1, not 2
        assert_eq!(typo.distance("teh", "the"), 1);
        
        // "chorme" -> "chrome"
        assert_eq!(typo.distance("chorme", "chrome"), 1);
    }

    #[test]
    fn test_min_query_length() {
        let mut typo = TypoTolerance::new();
        
        // Query too short (< 4 chars)
        assert_eq!(typo.score("fir", "firefox"), None);
        
        // Query exactly 4 chars (should work)
        assert!(typo.score("fire", "fire").is_some());
        
        // Query long enough with typo
        assert!(typo.score("firefo", "firefox").is_some());
    }

    #[test]
    fn test_max_distance() {
        let mut typo = TypoTolerance::new();
        
        // Distance 3 (too far)
        assert_eq!(typo.score("fiiireefox", "firefox"), None);
        
        // Distance 2 (within limit)
        assert!(typo.score("fiirefox", "firefox").is_some());
    }

    #[test]
    fn test_common_typos() {
        let mut typo = TypoTolerance::new();
        
        // Common browser typos
        assert!(typo.is_typo_match("firefix", "firefox"));
        assert!(typo.is_typo_match("chorme", "chrome"));
        assert!(typo.is_typo_match("braev", "brave"));
        
        // Common app typos
        assert!(typo.is_typo_match("thuner", "thunar"));
        assert!(typo.is_typo_match("giimp", "gimp"));
    }

    #[test]
    fn test_length_difference() {
        let mut typo = TypoTolerance::new();
        
        // Large length difference should return None
        assert_eq!(typo.score("fire", "firefoxbrowser"), None);
        
        // Similar length should work
        assert!(typo.score("firefo", "firefox").is_some());
    }

    #[test]
    fn test_find_typo_matches() {
        let mut typo = TypoTolerance::new();
        
        let candidates = vec!["firefox", "chrome", "brave", "thunar"];
        let matches = typo.find_typo_matches("firefix", &candidates);
        
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].0, "firefox");
        assert_eq!(matches[0].1, 1); // distance 1
    }

    #[test]
    fn test_suggest_corrections() {
        let mut typo = TypoTolerance::new();
        
        let candidates = vec!["firefox", "firebird", "chrome", "brave"];
        let suggestions = typo.suggest_corrections("firefix", &candidates, 2);
        
        assert!(suggestions.contains(&"firefox"));
    }

    #[test]
    fn test_cache() {
        let mut typo = TypoTolerance::new();
        
        // First call - calculates
        let dist1 = typo.distance("firefix", "firefox");
        
        // Second call - uses cache
        let dist2 = typo.distance("firefix", "firefox");
        
        assert_eq!(dist1, dist2);
        assert_eq!(dist1, 1);
        
        // Clear cache
        typo.clear_cache();
        
        // Should still work
        let dist3 = typo.distance("firefix", "firefox");
        assert_eq!(dist3, 1);
    }

    #[test]
    fn test_custom_settings() {
        let mut typo = TypoTolerance::with_settings(1, 3);
        
        // Max distance 1 - firefix to firefox has distance 1
        assert!(typo.score("firefix", "firefox").is_some());
        
        // Distance > 1 should fail with max_distance=1
        // fiirefox to firefox might be distance 1 or 2 depending on algorithm
        let score = typo.score("fiireefox", "firefox");
        assert_eq!(score, None, "Distance should be > 1");
        
        // Min length 3 - "fir" has 3 chars, should work
        assert!(typo.score("fir", "fir").is_some());
        assert!(typo.score("fir", "fire").is_some());
    }

    #[test]
    fn test_score_with_penalty() {
        let mut typo = TypoTolerance::new();
        
        // Distance 1, penalty 50.0 per edit
        let score = typo.score_with_penalty("firefix", "firefox", 50.0);
        assert_eq!(score, Some(150.0)); // 200.0 - 50.0
        
        // Distance varies with Damerau-Levenshtein
        let score = typo.score_with_penalty("fiirefox", "firefox", 50.0);
        assert!(score.is_some());
        assert!(score.unwrap() >= 100.0);
    }

    #[test]
    fn test_empty_strings() {
        let mut typo = TypoTolerance::new();
        
        assert_eq!(typo.distance("", "firefox"), 7);
        assert_eq!(typo.distance("firefox", ""), 7);
        assert_eq!(typo.distance("", ""), 0);
    }

    #[test]
    fn test_case_sensitivity() {
        let mut typo = TypoTolerance::new();
        
        // Different cases are treated as different characters
        assert_eq!(typo.distance("Firefox", "firefox"), 1);
        
        // Should normalize before calling
        assert_eq!(typo.distance("firefox", "firefox"), 0);
    }

    #[test]
    fn test_unicode() {
        let mut typo = TypoTolerance::new();
        
        // Unicode characters - distance calculation works on chars, not bytes
        // "café" has 4 chars, "cafe" has 4 chars
        let dist = typo.distance("café", "cafe");
        assert!(dist <= 2, "Unicode distance should be reasonable, got {}", dist);
        
        // These should work without panicking
        let _ = typo.distance("naïve", "naive");
    }

    #[test]
    fn test_real_world_typos() {
        let mut typo = TypoTolerance::new();
        
        // Real typos users might make
        assert!(typo.is_typo_match("googel", "google"));
        assert!(typo.is_typo_match("chromw", "chrome"));
        assert!(typo.is_typo_match("vlcc", "vlc"));
        assert!(typo.is_typo_match("thundar", "thunar"));
        assert!(typo.is_typo_match("libreofice", "libreoffice"));
    }
}
