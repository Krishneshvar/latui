/// Tokenizes text into searchable tokens
pub struct Tokenizer;

impl Tokenizer {
    pub fn new() -> Self {
        Self
    }

    /// Extract all tokens from text
    pub fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .map(String::from)
            .collect()
    }

    /// Extract acronym from multi-word text
    pub fn extract_acronym(&self, text: &str) -> Option<String> {
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.len() < 2 {
            return None;
        }

        let acronym: String = words
            .iter()
            .filter_map(|w| w.chars().next())
            .collect();

        if acronym.is_empty() {
            None
        } else {
            Some(acronym.to_lowercase())
        }
    }

    /// Normalize a token
    pub fn normalize(&self, token: &str) -> String {
        token.to_lowercase()
    }
}
