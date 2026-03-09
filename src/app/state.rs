use crate::core::item::Item;

pub struct AppState {
    pub query: String,
    pub all_items: Vec<Item>,
    pub filtered_items: Vec<Item>,
    pub selected: usize,
}

impl AppState {
    pub fn new(items: Vec<Item>) -> Self {
        Self {
            query: String::new(),
            filtered_items: items.clone(),
            all_items: items,
            selected: 0,
        }
    }

    pub fn reset_selection(&mut self) {
        self.selected = 0;
    }
}
