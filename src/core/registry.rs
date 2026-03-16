use std::collections::HashMap;
use crate::core::mode::Mode;
use crate::error::LatuiError;
use crate::modes::apps::AppsMode;
use crate::modes::run::RunMode;
use crate::modes::files::FilesMode;
use crate::modes::clipboard::ClipboardMode;
use crate::modes::emojis::EmojisMode;

pub struct ModeRegistry {
    pub modes: HashMap<String, Box<dyn Mode>>,
    pub active_mode: String,
    pub default_mode: String,
}

impl ModeRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            modes: HashMap::new(),
            active_mode: "apps".to_string(),
            default_mode: "apps".to_string(),
        };
        
        // Register built-in modes
        registry.register("apps", Box::new(AppsMode::new()));
        registry.register("run", Box::new(RunMode::new()));
        registry.register("files", Box::new(FilesMode::new()));
        registry.register("clipboard", Box::new(ClipboardMode::new()));
        registry.register("emojis", Box::new(EmojisMode::new()));
        
        registry
    }
    
    pub fn register(&mut self, name: &str, mode: Box<dyn Mode>) {
        self.modes.insert(name.to_string(), mode);
    }
    
    pub fn switch_mode(&mut self, mode_name: &str) -> Result<(), LatuiError> {
        if self.modes.contains_key(mode_name) {
            self.active_mode = mode_name.to_string();
            Ok(())
        } else {
            Err(LatuiError::App(format!("Mode '{}' not found", mode_name)))
        }
    }

    pub fn get_active_mode(&self) -> Option<&dyn Mode> {
        let mode = self.modes.get(&self.active_mode)?;
        Some(mode.as_ref())
    }

    pub fn get_active_mode_mut(&mut self) -> Option<&mut dyn Mode> {
        let mode = self.modes.get_mut(&self.active_mode)?;
        Some(mode.as_mut())
    }
}
