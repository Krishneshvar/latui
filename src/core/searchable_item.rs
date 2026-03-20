use crate::core::item::Item;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// A searchable item with multiple indexed fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchableItem {
    /// The original item
    pub item: Item,

    /// Indexed fields for searching
    pub fields: Vec<IndexedField>,

    /// Extracted acronyms
    pub acronyms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedField {
    pub name: String,
    pub text: String,
    pub tokens: Vec<String>,
    pub weight: f64,
}

impl SearchableItem {
    /// Create a new searchable item
    pub const fn new(item: Item) -> Self {
        Self {
            item,
            fields: Vec::new(),
            acronyms: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_field(mut self, name: &str, text: &str, weight: f64) -> Self {
        use crate::search::tokenizer::Tokenizer;
        let tokenizer = Tokenizer::new();

        let sanitized = sanitize_text(text);
        let tokens = if weight >= 8.0 {
            tokenizer.tokenize_comprehensive(&sanitized)
        } else {
            tokenizer.tokenize(&sanitized)
        };

        if weight >= 9.0 {
            self.acronyms
                .extend(tokenizer.extract_all_acronyms(&sanitized));
            self.acronyms.sort();
            self.acronyms.dedup();
        }

        self.fields.push(IndexedField {
            name: name.to_string(),
            text: sanitized,
            tokens,
            weight,
        });

        self
    }

    /// Get all searchable text fields with their weights
    pub fn get_weighted_fields(&self) -> Vec<SearchField<'_>> {
        self.fields
            .iter()
            .map(|f| SearchField {
                text: Cow::Borrowed(&f.text),
                tokens: Cow::Borrowed(&f.tokens),
                weight: f.weight,
            })
            .collect()
    }
}

fn sanitize_text(text: &str) -> String {
    text.chars().filter(|c| !c.is_control()).collect()
}

/// A searchable field view for the search engine
#[derive(Clone, Debug)]
pub struct SearchField<'a> {
    pub text: std::borrow::Cow<'a, str>,
    pub tokens: std::borrow::Cow<'a, [String]>,
    pub weight: f64,
}
