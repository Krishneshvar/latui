use std::collections::{HashMap, HashSet};
use crate::core::searchable_item::SearchableItem;

#[derive(Default)]
pub struct TrieNode {
    pub children: HashMap<char, TrieNode>,
    pub items: Vec<usize>,
}

pub struct Trie {
    root: TrieNode,
}

impl Trie {
    pub fn new() -> Self {
        Self {
            root: TrieNode::default(),
        }
    }

    pub fn insert(&mut self, word: &str, index: usize) {
        let mut node = &mut self.root;

        for ch in word.chars() {
            node = node.children.entry(ch).or_default();
            node.items.push(index);
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

        node.items.clone()
    }
}

/// Multi-token trie for efficient prefix filtering
pub struct MultiTokenTrie {
    trie: Trie,
}

impl MultiTokenTrie {
    pub fn new() -> Self {
        Self {
            trie: Trie::new(),
        }
    }

    /// Build trie from searchable items
    pub fn build(items: &[SearchableItem]) -> Self {
        let mut trie = Self::new();
        
        for (idx, item) in items.iter().enumerate() {
            trie.insert_item(item, idx);
        }
        
        trie
    }

    /// Insert all tokens from an item into the trie
    pub fn insert_item(&mut self, item: &SearchableItem, index: usize) {
        // Insert name tokens
        for token in &item.name_tokens {
            self.trie.insert(token, index);
        }
        
        // Insert keyword tokens
        for token in &item.keyword_tokens {
            self.trie.insert(token, index);
        }
        
        // Insert category tokens
        for token in &item.category_tokens {
            self.trie.insert(token, index);
        }
        
        // Insert generic name tokens
        for token in &item.generic_name_tokens {
            self.trie.insert(token, index);
        }
        
        // Insert description tokens
        for token in &item.description_tokens {
            self.trie.insert(token, index);
        }
        
        // Insert executable tokens
        for token in &item.executable_tokens {
            self.trie.insert(token, index);
        }
        
        // Insert acronyms
        for acronym in &item.acronyms {
            self.trie.insert(acronym, index);
        }
        
        // Also insert the full name and executable (lowercased)
        self.trie.insert(&item.name.to_lowercase(), index);
        self.trie.insert(&item.executable.to_lowercase(), index);
        
        // Insert keywords and categories as-is
        for keyword in &item.keywords {
            self.trie.insert(&keyword.to_lowercase(), index);
        }
        
        for category in &item.categories {
            self.trie.insert(&category.to_lowercase(), index);
        }
        
        // Insert generic name if present
        if let Some(ref generic) = item.generic_name {
            self.trie.insert(&generic.to_lowercase(), index);
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
        let mut seen = HashSet::new();
        results.into_iter()
            .filter(|idx| seen.insert(*idx))
            .collect()
    }

    /// Get candidates for multi-token query
    /// Returns indices of items that match ALL query tokens
    pub fn get_multi_token_candidates(&self, tokens: &[String]) -> Vec<usize> {
        if tokens.is_empty() {
            return Vec::new();
        }
        
        // Get candidates for first token
        let mut candidates: HashSet<usize> = self.get_candidates(&tokens[0])
            .into_iter()
            .collect();
        
        // Intersect with candidates from other tokens
        for token in &tokens[1..] {
            let token_candidates: HashSet<usize> = self.get_candidates(token)
                .into_iter()
                .collect();
            
            candidates = candidates.intersection(&token_candidates)
                .copied()
                .collect();
            
            // Early exit if no candidates remain
            if candidates.is_empty() {
                return Vec::new();
            }
        }
        
        candidates.into_iter().collect()
    }
    
    /// Get candidates using OR logic (any token matches)
    /// Returns indices of items that match ANY query token
    pub fn get_any_token_candidates(&self, tokens: &[String]) -> Vec<usize> {
        if tokens.is_empty() {
            return Vec::new();
        }
        
        let mut candidates = HashSet::new();
        
        for token in tokens {
            let token_candidates = self.get_candidates(token);
            candidates.extend(token_candidates);
        }
        
        candidates.into_iter().collect()
    }
}
