use unicode_segmentation::UnicodeSegmentation;

/// Advanced tokenizer for extracting searchable tokens from text
/// Handles: whitespace, punctuation, CamelCase, acronyms, unicode
pub struct Tokenizer {
    /// Whether to extract acronyms from multi-word text
    pub extract_acronyms: bool,
    /// Whether to split CamelCase words
    pub split_camel_case: bool,
    /// Minimum token length to keep
    pub min_token_length: usize,
}

impl Tokenizer {
    pub fn new() -> Self {
        Self {
            extract_acronyms: true,
            split_camel_case: true,
            min_token_length: 1,
        }
    }

    /// Extract all tokens from text with all strategies
    pub fn tokenize(&self, text: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        
        // Basic word splitting
        let words = self.split_words(text);
        
        for word in &words {
            // Add the original word (normalized)
            let normalized = self.normalize(word);
            if normalized.len() >= self.min_token_length {
                tokens.push(normalized.clone());
            }
            
            // Split CamelCase if enabled
            if self.split_camel_case {
                let camel_tokens = self.split_camel_case_word(word);
                for token in camel_tokens {
                    let normalized = self.normalize(&token);
                    if normalized.len() >= self.min_token_length && !tokens.contains(&normalized) {
                        tokens.push(normalized);
                    }
                }
            }
        }
        
        // Extract acronym from full text if enabled
        if self.extract_acronyms
            && let Some(acronym) = self.extract_acronym(text)
                && !tokens.contains(&acronym) {
                    tokens.push(acronym);
                }
        
        tokens
    }

    /// Split text into words by whitespace and common separators
    pub fn split_words(&self, text: &str) -> Vec<String> {
        text.split(|c: char| {
            c.is_whitespace() || c == '-' || c == '_' || c == '.' || c == '/'
        })
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
    }

    /// Split CamelCase words into separate tokens
    /// Examples:
    /// - "LibreOffice" -> ["Libre", "Office"]
    /// - "VLCPlayer" -> ["VLC", "Player"]
    /// - "GIMP" -> ["GIMP"] (all caps, no split)
    pub fn split_camel_case_word(&self, word: &str) -> Vec<String> {
        let chars: Vec<char> = word.chars().collect();
        if chars.len() < 2 {
            return if word.is_empty() { vec![] } else { vec![word.to_string()] };
        }

        let mut tokens = Vec::new();
        let mut current = String::new();
        
        for i in 0..chars.len() {
            let ch = chars[i];
            
            // Check if we should split here
            let should_split = if i > 0 && i < chars.len() - 1 {
                let prev = chars[i - 1];
                let next = chars[i + 1];
                
                // Split on lowercase -> uppercase transition
                // "camelCase" -> "camel" + "Case"
                (prev.is_lowercase() && ch.is_uppercase()) ||
                // Split on uppercase -> uppercase -> lowercase transition
                // "XMLParser" -> "XML" + "Parser"
                (prev.is_uppercase() && ch.is_uppercase() && next.is_lowercase())
            } else {
                false
            };
            
            if should_split && !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
            
            current.push(ch);
        }
        
        if !current.is_empty() {
            tokens.push(current);
        }
        
        // If no splits occurred and word is all uppercase, return as-is
        if tokens.len() == 1 {
            return tokens;
        }
        
        // Filter out very short tokens (single letters) unless they're meaningful
        tokens.into_iter()
            .filter(|t| t.len() > 1 || t.chars().all(|c| c.is_uppercase()))
            .collect()
    }

    /// Extract acronym from multi-word text
    /// Examples:
    /// - "Google Chrome" -> "gc"
    /// - "Visual Studio Code" -> "vsc"
    /// - "VLC Media Player" -> "vmp"
    pub fn extract_acronym(&self, text: &str) -> Option<String> {
        let words: Vec<&str> = text
            .split(|c: char| c.is_whitespace() || c == '-' || c == '_')
            .filter(|w| !w.is_empty())
            .collect();
        
        // Need at least 2 words for an acronym
        if words.len() < 2 {
            return None;
        }
        
        let acronym: String = words
            .iter()
            .filter_map(|w| {
                // Get first character of each word
                w.chars().next()
            })
            .collect();
        
        if acronym.len() >= 2 {
            Some(self.normalize(&acronym))
        } else {
            None
        }
    }

    /// Extract all possible acronyms from text
    /// For "Visual Studio Code" returns: ["vsc", "vs", "vc", "sc"]
    pub fn extract_all_acronyms(&self, text: &str) -> Vec<String> {
        let words: Vec<&str> = text
            .split(|c: char| c.is_whitespace() || c == '-' || c == '_')
            .filter(|w| !w.is_empty())
            .collect();
        
        if words.len() < 2 {
            return vec![];
        }
        
        let mut acronyms = Vec::new();
        
        // Full acronym
        if let Some(full) = self.extract_acronym(text) {
            acronyms.push(full);
        }
        
        // Partial acronyms (first 2 words, last 2 words, etc.)
        if words.len() >= 2 {
            // First two words
            let first_two: String = words[..2]
                .iter()
                .filter_map(|w| w.chars().next())
                .collect();
            if first_two.len() == 2 {
                acronyms.push(self.normalize(&first_two));
            }
            
            // Last two words
            if words.len() > 2 {
                let last_two: String = words[words.len()-2..]
                    .iter()
                    .filter_map(|w| w.chars().next())
                    .collect();
                if last_two.len() == 2 && !acronyms.contains(&self.normalize(&last_two)) {
                    acronyms.push(self.normalize(&last_two));
                }
            }
        }
        
        acronyms
    }

    pub fn normalize(&self, token: &str) -> String {
        let mut result = String::with_capacity(token.len());
        for g in token.trim().graphemes(true) {
            if let Some(base_char) = g.chars().next() {
                // Keep only the base character (first char of grapheme)
                // and convert it to lowercase directly
                result.extend(base_char.to_lowercase());
            }
        }
        result
    }

    /// Tokenize with all strategies and return unique tokens
    pub fn tokenize_comprehensive(&self, text: &str) -> Vec<String> {
        let mut all_tokens = Vec::new();
        
        // Get basic tokens
        all_tokens.extend(self.tokenize(text));
        
        // Get all acronyms
        all_tokens.extend(self.extract_all_acronyms(text));
        
        // Remove duplicates while preserving order
        let mut seen = std::collections::HashSet::new();
        all_tokens.into_iter()
            .filter(|t| seen.insert(t.clone()))
            .collect()
    }
}

impl Default for Tokenizer {
    fn default() -> Self {
        Self::new()
    }
}


