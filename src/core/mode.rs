use crate::core::item::Item;
use crate::error::LatuiError;

pub trait Mode: std::fmt::Debug {
    /// Unique identifier for this mode (e.g., "apps").
    fn name(&self) -> &str;
    /// Unicode icon shown in the mode selector tab.
    fn icon(&self) -> &str;
    /// Human-readable label for the mode switcher.
    fn description(&self) -> &str;

    /// Refreshes the internal index for this mode.
    fn load(&mut self) -> Result<(), LatuiError>;

    /// Searches the mode's index for items matching the given query.
    ///
    /// **Note:** This takes `&mut self` because the underlying fuzzy search
    /// engines (like `nucleo_matcher`) utilize internal mutable buffers to 
    /// provide high-performance scoring without per-frame allocations.
    fn search(&mut self, query: &str) -> Vec<Item>;

    /// Executes the primary action for the given item.
    fn execute(&mut self, item: &Item) -> Result<(), LatuiError>;

    /// Records that an item was selected, used for usage-based ranking.
    fn record_selection(&mut self, query: &str, item: &Item);

    /// Whether the launcher should stay open after a successful `execute()`.
    ///
    /// - `false` (default): exit immediately after launch (Apps, Run, Files).
    /// - `true`: stay open so the user can pick again (Clipboard, Emojis).
    fn stays_open(&self) -> bool {
        false
    }

    // Support for interactive previews
    fn supports_preview(&self) -> bool {
        false
    }
    fn preview(&self, _item: &Item) -> Option<String> {
        None
    }
}
