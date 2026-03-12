/// Typo tolerance using Levenshtein distance
pub struct TypoTolerance {
    max_distance: usize,
    min_query_length: usize,
}

impl TypoTolerance {
    pub fn new() -> Self {
        Self {
            max_distance: 2,
            min_query_length: 4,
        }
    }

    /// Calculate Levenshtein distance between two strings
    pub fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let len1 = s1.len();
        let len2 = s2.len();

        if len1 == 0 {
            return len2;
        }
        if len2 == 0 {
            return len1;
        }

        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();

        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }

        matrix[len1][len2]
    }

    /// Score based on typo tolerance
    pub fn score(&self, query: &str, target: &str) -> Option<f64> {
        if query.len() < self.min_query_length {
            return None;
        }

        let distance = self.levenshtein_distance(query, target);

        if distance <= self.max_distance {
            Some(match distance {
                0 => 1000.0,
                1 => 150.0,
                2 => 100.0,
                _ => 0.0,
            })
        } else {
            None
        }
    }
}
