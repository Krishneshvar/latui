use crate::core::item::Item;

pub struct AppState {
    pub query: String,
    pub items: Vec<Item>,
    pub selected: usize,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            items: Vec::new(),
            selected: 0,
        }
    }
}
