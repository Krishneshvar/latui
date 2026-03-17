use crate::core::item::Item;
use crate::core::searchable_item::{SearchField, SearchableItem};
use crate::matcher::fuzzy::FuzzyMatcher;
use crate::search::tokenizer::Tokenizer;
use crate::search::typo::TypoTolerance;

pub struct SearchEngine {
    tokenizer: Tokenizer,
    typo_tolerance: TypoTolerance,
    fuzzy_matcher: FuzzyMatcher,
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchEngine {
    pub fn new() -> Self {
        Self {
            tokenizer: Tokenizer::new(),
            typo_tolerance: TypoTolerance::new(),
            fuzzy_matcher: FuzzyMatcher::new(),
        }
    }

    pub fn search(&mut self, query: &str, items: &[SearchableItem]) -> Vec<Item> {
        self.search_scored(query, items)
            .into_iter()
            .map(|(item, _)| item)
            .collect()
    }

    pub fn search_scored(&mut self, query: &str, items: &[SearchableItem]) -> Vec<(Item, f64)> {
        if query.is_empty() {
            return items.iter().map(|s| (s.item.clone(), 0.0)).collect();
        }

        let q = query.to_lowercase();
        let query_tokens = self.tokenizer.tokenize(&q);

        let mut scored_items: Vec<(usize, f64)> = Vec::new();

        for (idx, searchable) in items.iter().enumerate() {
            let mut best_score: f64 = 0.0;

            // Check acronym match
            for acronym in &searchable.acronyms {
                if acronym == &q {
                    best_score = best_score.max(2500.0);
                } else if acronym.starts_with(&q) {
                    best_score = best_score.max(2000.0);
                }
            }

            // Score each field
            let fields = searchable.get_weighted_fields();
            for field in fields {
                let field_score = self.score_field(&q, &query_tokens, &field);
                best_score = best_score.max(field_score * field.weight);
            }

            if best_score > 0.0 {
                scored_items.push((idx, best_score));
            }
        }

        scored_items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scored_items
            .into_iter()
            .map(|(idx, score)| (items[idx].item.clone(), score))
            .collect()
    }

    fn score_field(&mut self, query: &str, query_tokens: &[String], field: &SearchField) -> f64 {
        let field_text = field.text.to_lowercase();

        // Exact match
        if field_text == query {
            return 1000.0;
        }

        // Prefix match
        if field_text.starts_with(query) {
            return 500.0;
        }

        // Token-based matching
        let mut score = 0.0;

        // Check if query matches any token exactly
        if field.tokens.iter().any(|t| t == query) {
            score = 400.0;
        }
        // Check if query is prefix of any token
        else if field.tokens.iter().any(|t| t.starts_with(query)) {
            score = 350.0;
        }
        // Word boundary match
        else if field_text
            .split_whitespace()
            .any(|word| word.starts_with(query))
        {
            score = 300.0;
        }
        // Multi-token match
        else if !query_tokens.is_empty() {
            let all_match = query_tokens
                .iter()
                .all(|qt| field.tokens.iter().any(|ft| ft.contains(qt)));
            if all_match {
                score = 250.0;
            }
        }

        // Typo tolerance
        if score == 0.0 {
            if let Some(typo_score) = self.typo_tolerance.score(query, &field_text) {
                score = typo_score;
            } else {
                for token in field.tokens.iter() {
                    if let Some(typo_score) = self.typo_tolerance.score(query, token) {
                        score = score.max(typo_score);
                    }
                }
            }
        }

        // Substring match
        if score == 0.0 && field_text.contains(query) {
            score = 100.0;
        }

        // Fuzzy match
        if score == 0.0 {
            let results = self.fuzzy_matcher.filter(query, &[&field_text]);
            if let Some((_, f_score)) = results.first() {
                score = (*f_score as f64).min(200.0);
            }
        }

        score
    }
}
