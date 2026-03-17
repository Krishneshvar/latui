use crate::core::{item::Item, mode::Mode};
use crate::error::LatuiError;

pub struct RunMode {
    // history: Vec<String>,
    // env_vars: HashMap<String, String>,
    // shell: String,
}

impl RunMode {
    pub fn new() -> Self {
        Self {}
    }
}

impl Mode for RunMode {
    fn name(&self) -> &str { "run" }
    fn icon(&self) -> &str { "🚀" }
    fn description(&self) -> &str { "Command Executor" }
    
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
