use std::collections::HashMap;

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
