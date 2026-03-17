use crate::core::item::Item;
use crate::error::LatuiError;

pub trait Mode {
    fn name(&self) -> &str;
    fn icon(&self) -> &str;
    fn description(&self) -> &str;
    
    fn load(&mut self) -> Result<(), LatuiError>;
    fn search(&mut self, query: &str) -> Vec<Item>;
    fn execute(&mut self, item: &Item) -> Result<(), LatuiError>;
    fn record_selection(&mut self, query: &str, item: &Item);
    
    // Support for interactive previews
    fn supports_preview(&self) -> bool { false }
    fn preview(&self, _item: &Item) -> Option<String> { None }
}
