use nucleo::Matcher;

pub struct FuzzyMatcher {
    matcher: Matcher,
}

impl FuzzyMatcher {
    pub fn new() -> Self {
        Self {
            matcher: Matcher::new(),
        }
    }

    pub fn score(&self, query: &str, text: &str) -> Option<i64> {
        self.matcher.score(text, query)
    }
}
