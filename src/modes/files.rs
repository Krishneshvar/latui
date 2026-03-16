use crate::core::{item::Item, mode::Mode};
use crate::error::LatuiError;

pub struct FilesMode {
}

impl FilesMode {
    pub fn new() -> Self {
        Self {}
    }
}

impl Mode for FilesMode {
    fn name(&self) -> &str { "files" }
    fn icon(&self) -> &str { "📁" }
    fn description(&self) -> &str { "Filesystem Search" }
    
    fn load(&mut self) -> Result<(), LatuiError> {
        Ok(())
    }
    
    fn search(&mut self, _query: &str) -> Vec<Item> {
        Vec::new()
    }
    
    fn execute(&mut self, _item: &Item) -> Result<(), LatuiError> {
        Ok(())
    }
    
    fn record_selection(&mut self, _query: &str, _item: &Item) {
    }
}
