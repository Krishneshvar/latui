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

/// Supported terminal image protocols for icon rendering.
#[derive(Debug)]
pub struct ImageSupport {
    /// The image picker that handles protocol-specific rendering.
    pub picker: Picker,
    /// The high-level protocol used (e.g., Kitty, Sixel).
    pub protocol: ProtocolType,
}

/// Global application state.
///
/// **Invariants:**
/// - `filtered_items` must always be consistent with the current `query` for 
///   the active mode.
/// - `list_state.selected()` must always be either `None` (empty results) or
///   `Some(idx)` where `idx < filtered_items.len()`.
pub struct AppState {
    /// The user's current search query string.
    pub query: String,
    /// The list of items matching the current query.
    pub filtered_items: Vec<Item>,
    /// The current selection and scroll state for the results list.
    pub list_state: ListState,
    /// Registry of all available search modes.
    pub mode_registry: ModeRegistry,
    /// The zero-indexed tab currently being displayed in the UI.
    pub active_tab: usize,
    /// Whether the optional side preview pane is currently visible.
    pub show_preview: bool,
    /// The merged user configuration and theme settings.
    pub config: AppConfig,
    /// Detailed image support info, if the terminal supports it.
    pub image_support: Option<ImageSupport>,

    /// Cache of rendered icon protocols for reuse across frames.
    pub icon_preview_protocols: LruCache<String, StatefulProtocol>,
    /// Cache of resolved paths to desktop application icon files.
    pub desktop_icon_path_cache: LruCache<String, Option<PathBuf>>,
    /// Set of icon paths that previously failed to resolve or load.
    pub failed_icon_paths: HashSet<String>,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("query", &self.query)
            .field("filtered_items_count", &self.filtered_items.len())
            .field("active_tab", &self.active_tab)
            .field("show_preview", &self.show_preview)
            .finish_non_exhaustive()
    }
}

impl AppState {
    /// Create a new application state with default settings.
    ///
    /// # Panics
    ///
    /// Panics if the hardcoded LRU cache capacities (100, 500) are zero.
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
            icon_preview_protocols: LruCache::new(NonZeroUsize::new(100).expect("Infallible size")),
            desktop_icon_path_cache: LruCache::new(NonZeroUsize::new(500).expect("Infallible size")),
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

    pub const fn next(&mut self) {
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

    pub const fn previous(&mut self) {
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

    pub const fn reset_selection(&mut self) {
        self.list_state.select(Some(0));
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
