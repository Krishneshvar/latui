use crate::core::item::Item;
use ratatui::widgets::ListState;
use crate::index::trie::Trie;

pub struct AppState {
    pub query: String,
    pub all_items: Vec<Item>,
    pub filtered_items: Vec<Item>,
    pub list_state: ListState,
    pub trie: Trie,
}

impl AppState {
    pub fn new(items: Vec<Item>) -> Self {
        let mut trie = Trie::new();

        for (i, item) in items.iter().enumerate() {
            trie.insert(&item.search_text, i);
        }

        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            query: String::new(),
            filtered_items: items.clone(),
            all_items: items,
            list_state,
            trie,
        }
    }

    pub fn next(&mut self) {
        if self.filtered_items.is_empty() {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.filtered_items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };

        self.list_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.filtered_items.is_empty() {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };

        self.list_state.select(Some(i));
    }

    pub fn reset_selection(&mut self) {
        self.list_state.select(Some(0));
    }
}
