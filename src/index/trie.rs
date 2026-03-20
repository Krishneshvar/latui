use crate::core::searchable_item::SearchableItem;
use rustc_hash::{FxHashMap, FxHashSet};
use std::time::Instant;
use tracing::{debug, info};

pub const MAX_TRIE_WORD_LENGTH: usize = 32;

#[derive(Debug, Default)]
pub struct TrieNode {
    pub children: FxHashMap<char, TrieNode>,
    pub items: FxHashSet<usize>,
}

#[derive(Debug)]
pub struct Trie {
    root: TrieNode,
}

impl Default for Trie {
    fn default() -> Self {
        Self::new()
    }
}

impl Trie {
    pub fn new() -> Self {
        Self {
            root: TrieNode::default(),
        }
    }

    pub fn insert(&mut self, word: &str, index: usize) {
        if word.len() > MAX_TRIE_WORD_LENGTH {
            // Drop strings exceeding our sensible dimension limits to bound memory usage securely
            return;
        }

        let mut node = &mut self.root;

        for ch in word.chars() {
            node = node.children.entry(ch).or_default();
            node.items.insert(index);
        }
    }

    pub fn search(&self, prefix: &str) -> Vec<usize> {
        let mut node = &self.root;

        for ch in prefix.chars() {
            match node.children.get(&ch) {
                Some(n) => node = n,
                None => return Vec::new(),
            }
        }

        node.items.iter().copied().collect()
    }
}

#[derive(Debug)]
/// Multi-token trie for efficient prefix filtering
pub struct MultiTokenTrie {
    trie: Trie,
}

impl Default for MultiTokenTrie {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiTokenTrie {
    pub fn new() -> Self {
        Self { trie: Trie::new() }
    }

    /// Build trie from searchable items
    pub fn build(items: &[SearchableItem]) -> Self {
        debug!(
            "Commencing MultiTokenTrie index building for {} items...",
            items.len()
        );
        let start_time = Instant::now();
        let mut trie = Self::new();

        for (idx, item) in items.iter().enumerate() {
            trie.insert_item(item, idx);
        }

        info!(
            "Built MultiTokenTrie search index with {} items in {:?}",
            items.len(),
            start_time.elapsed()
        );

        trie
    }

    /// Insert all tokens from an item into the trie
    pub fn insert_item(&mut self, item: &SearchableItem, index: usize) {
        // Insert tokens from all fields
        for field in &item.fields {
            for token in &field.tokens {
                self.trie.insert(token, index);
            }
            // Also insert the full text lowercased (for exact/prefix matches)
            self.trie.insert(&field.text.to_lowercase(), index);
        }

        // Insert acronyms
        for acronym in &item.acronyms {
            self.trie.insert(acronym, index);
        }
    }

    /// Get candidate item indices for a query
    /// Returns indices of items that have at least one token starting with the query
    pub fn get_candidates(&self, query: &str) -> Vec<usize> {
        if query.is_empty() {
            return Vec::new();
        }

        let results = self.trie.search(query);

        // Remove duplicates while preserving order
        let mut seen = FxHashSet::default();
        results
            .into_iter()
            .filter(|idx| seen.insert(*idx))
            .collect()
    }

    /// Get candidates for multi-token query
    /// Returns indices of items that match ALL query tokens
    pub fn get_multi_token_candidates(&self, tokens: &[String]) -> Vec<usize> {
        if tokens.is_empty() {
            return Vec::new();
        }

        // Optimize: Sort tokens by length (descending) assuming longer tokens are rarer
        // This helps the intersection process reduce search space faster
        let mut sorted_tokens: Vec<&String> = tokens.iter().collect();
        sorted_tokens.sort_by_key(|a| std::cmp::Reverse(a.len()));

        // Get candidates for first token
        let mut candidates: FxHashSet<usize> =
            self.get_candidates(sorted_tokens[0]).into_iter().collect();

        // Intersect with candidates from other tokens
        for token in &sorted_tokens[1..] {
            let token_candidates: FxHashSet<usize> =
                self.get_candidates(token).into_iter().collect();

            candidates = candidates
                .intersection(&token_candidates)
                .copied()
                .collect();

            // Early exit if no candidates remain
            if candidates.is_empty() {
                return Vec::new();
            }
        }

        let mut result: Vec<usize> = candidates.into_iter().collect();
        result.sort_unstable(); // Preserve stable visual tracking post-intersection
        result
    }
}
