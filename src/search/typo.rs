

/// Advanced typo tolerance using multiple edit distance algorithms
/// Handles common typing mistakes: transpositions, insertions, deletions, substitutions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypoTolerance {
    /// Maximum edit distance to consider (default: 2)
    pub max_distance: usize,
    /// Minimum query length to apply typo tolerance (default: 4)
    pub min_query_length: usize,
    /// Whether to use Damerau-Levenshtein (includes transpositions)
    pub use_damerau: bool,
}

impl TypoTolerance {
    pub const fn new() -> Self {
        Self {
            max_distance: 2,
            min_query_length: 4,
            use_damerau: true,
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
                let cost = usize::from(s1_chars[i - 1] != s2_chars[j - 1]);

                curr_row[j] = (prev_row[j] + 1) // deletion
                    .min(curr_row[j - 1] + 1) // insertion
                    .min(prev_row[j - 1] + cost); // substitution
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

        for (i, row) in matrix.iter_mut().enumerate().take(len1 + 1) {
            row[0] = i;
        }
        for (j, cell) in matrix[0].iter_mut().enumerate().take(len2 + 1) {
            *cell = j;
        }

        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = usize::from(s1_chars[i - 1] != s2_chars[j - 1]);

                matrix[i][j] = (matrix[i - 1][j] + 1) // deletion
                    .min(matrix[i][j - 1] + 1) // insertion
                    .min(matrix[i - 1][j - 1] + cost); // substitution

                // Transposition
                if i > 1
                    && j > 1
                    && s1_chars[i - 1] == s2_chars[j - 2]
                    && s1_chars[i - 2] == s2_chars[j - 1]
                {
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
        if self.use_damerau {
            self.damerau_levenshtein_distance(s1, s2)
        } else {
            self.levenshtein_distance(s1, s2)
        }
    }
    
    // ...
    /// Returns None if query is too short or distance is too large
    pub fn score(&self, query: &str, target: &str) -> Option<f64> {
        // Skip if query is too short
        if query.len() < self.min_query_length {
            return None;
        }

        // Skip if target is much longer (unlikely to be a typo)
        let len_diff = query.len().abs_diff(target.len());
        if len_diff > self.max_distance {
            return None;
        }

        let distance = self.distance(query, target);

        if distance <= self.max_distance {
            Some(match distance {
                0 => 1000.0, // Exact match
                1 => 150.0,  // One typo
                2 => 100.0,  // Two typos
                _ => 0.0,
            })
        } else {
            None
        }
    }
}

impl Default for TypoTolerance {
    fn default() -> Self {
        Self::new()
    }
}
