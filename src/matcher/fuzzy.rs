use nucleo_matcher::{
    Config, Matcher, Utf32Str,
    pattern::{CaseMatching, Normalization, Pattern},
};

pub struct FuzzyMatcher {
    matcher: Matcher,
}

impl std::fmt::Debug for FuzzyMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FuzzyMatcher").finish_non_exhaustive()
    }
}

impl Default for FuzzyMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl FuzzyMatcher {
    pub fn new() -> Self {
        Self {
            matcher: Matcher::new(Config::DEFAULT),
        }
    }

    pub fn filter(&mut self, query: &str, items: &[&str]) -> Vec<(usize, i64)> {
        let mut results: Vec<(usize, i64)> = Vec::new();

        if query.is_empty() {
            for (i, _) in items.iter().enumerate() {
                results.push((i, 0));
            }
            return results;
        }

        let pattern = Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart);

        // buffer reused for UTF32 conversion
        let mut buf = Vec::new();

        for (i, text) in items.iter().enumerate() {
            let haystack = Utf32Str::new(text, &mut buf);

            if let Some(score) = pattern.score(haystack, &mut self.matcher) {
                results.push((i, score as i64));
            }

            buf.clear();
        }

        results.sort_by(|a, b| b.1.cmp(&a.1));

        results
    }
}
