use crate::core::item::Item;
use crate::core::registry::ModeRegistry;
use crate::ui::theme::Theme;
use ratatui::widgets::ListState;

pub struct AppState {
    pub query: String,
    pub filtered_items: Vec<Item>,
    pub list_state: ListState,
    pub mode_registry: ModeRegistry,
    pub active_tab: usize,
    pub show_preview: bool,
    pub theme: Theme,
}

impl AppState {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            query: String::new(),
            filtered_items: Vec::new(),
            list_state,
            mode_registry: ModeRegistry::new(),
            active_tab: 0,
            show_preview: false,
            theme: Theme::default(),
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

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
