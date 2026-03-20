use crate::core::item::Item;
use crate::core::registry::ModeRegistry;
use crate::config::theme::AppConfig;
use ratatui::widgets::ListState;
use ratatui_image::picker::{Picker, ProtocolType};
use ratatui_image::protocol::StatefulProtocol;
use std::collections::HashSet;
use std::path::PathBuf;
use lru::LruCache;
use std::num::NonZeroUsize;

pub struct ImageSupport {
    pub picker: Picker,
    pub protocol: ProtocolType,
}

pub struct AppState {
    pub query: String,
    pub filtered_items: Vec<Item>,
    pub list_state: ListState,
    pub mode_registry: ModeRegistry,
    pub active_tab: usize,
    pub show_preview: bool,
    pub config: AppConfig,
    pub image_support: Option<ImageSupport>,
    pub icon_preview_protocols: LruCache<String, StatefulProtocol>,
    pub desktop_icon_path_cache: LruCache<String, Option<PathBuf>>,
    pub failed_icon_paths: HashSet<String>,
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
            config: AppConfig::default(),
            image_support: None,
            icon_preview_protocols: LruCache::new(NonZeroUsize::new(100).unwrap()),
            desktop_icon_path_cache: LruCache::new(NonZeroUsize::new(500).unwrap()),
            failed_icon_paths: HashSet::new(),
        }
    }

    pub fn detect_image_support(&mut self) {
        let picker = Picker::from_query_stdio().unwrap_or_else(|e| {
            tracing::debug!("Image picker initialization failed: {}; using halfblocks", e);
            Picker::halfblocks()
        });
        let protocol = picker.protocol_type();
        match protocol {
            ProtocolType::Kitty | ProtocolType::Sixel => {
                self.image_support = Some(ImageSupport { picker, protocol });
                tracing::info!("Terminal image protocol enabled: {:?}", protocol);
            }
            _ => {
                self.image_support = None;
                tracing::debug!(
                    "Terminal image protocol unavailable (detected {:?}); using text icons",
                    protocol
                );
            }
        }
    }

    pub fn clear_icon_render_cache(&mut self) {
        self.icon_preview_protocols.clear();
        self.desktop_icon_path_cache.clear();
        self.failed_icon_paths.clear();
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
