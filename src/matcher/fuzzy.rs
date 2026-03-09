use nucleo_matcher::{
    Matcher,
    Config,
    pattern::{Pattern, CaseMatching, Normalization},
    Utf32Str,
};

pub struct FuzzyMatcher {
    matcher: Matcher,
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
