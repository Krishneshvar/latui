use std::collections::HashMap;
use crate::core::mode::Mode;
use crate::error::LatuiError;

/// Central registry for managing all available modes in the application.
/// Handles mode registration, switching, and access to active modes.
pub struct ModeRegistry {
    /// Map of mode names to their implementations
    pub modes: HashMap<String, Box<dyn Mode>>,
    /// Currently active mode name
    pub active_mode: String,
    /// Default mode to use on startup
    pub default_mode: String,
    /// Ordered list of mode names for tab navigation
    mode_order: Vec<String>,
}

impl ModeRegistry {
    /// Creates a new empty ModeRegistry.
    pub fn new() -> Self {
        Self {
            modes: HashMap::new(),
            active_mode: String::new(),
            default_mode: String::new(),
            mode_order: Vec::new(),
        }
    }
    
    /// Registers a new mode with the given name.
    /// Modes are added to the navigation order in registration sequence.
    pub fn register(&mut self, name: &str, mode: Box<dyn Mode>) {
        if self.modes.is_empty() {
            self.active_mode = name.to_string();
            self.default_mode = name.to_string();
        }
        self.modes.insert(name.to_string(), mode);
        self.mode_order.push(name.to_string());
    }
    
    /// Switches to the specified mode by name.
    /// Returns an error if the mode doesn't exist.
    pub fn switch_mode(&mut self, mode_name: &str) -> Result<(), LatuiError> {
        if self.modes.contains_key(mode_name) {
            tracing::info!("Switching from '{}' to '{}' mode", self.active_mode, mode_name);
            self.active_mode = mode_name.to_string();
            Ok(())
        } else {
            Err(LatuiError::App(format!("Mode '{}' not found", mode_name)))
        }
    }
    
    /// Switches to the next mode in the registration order (circular).
    pub fn next_mode(&mut self) {
        if let Some(current_idx) = self.mode_order.iter().position(|m| m == &self.active_mode) {
            let next_idx = (current_idx + 1) % self.mode_order.len();
            let next_mode = self.mode_order[next_idx].clone();
            let _ = self.switch_mode(&next_mode);
        }
    }
    
    /// Switches to the previous mode in the registration order (circular).
    pub fn previous_mode(&mut self) {
        if let Some(current_idx) = self.mode_order.iter().position(|m| m == &self.active_mode) {
            let prev_idx = if current_idx == 0 {
                self.mode_order.len() - 1
            } else {
                current_idx - 1
            };
            let prev_mode = self.mode_order[prev_idx].clone();
            let _ = self.switch_mode(&prev_mode);
        }
    }

    /// Returns an immutable reference to the currently active mode.
    pub fn get_active_mode(&self) -> Option<&dyn Mode> {
        let mode = self.modes.get(&self.active_mode)?;
        Some(mode.as_ref())
    }

    /// Returns a mutable reference to the currently active mode.
    pub fn get_active_mode_mut(&mut self) -> Option<&mut dyn Mode> {
        let mode = self.modes.get_mut(&self.active_mode)?;
        Some(mode.as_mut())
    }
    
    /// Returns the index of the currently active mode in the mode order.
    /// Used for UI tab highlighting.
    pub fn get_active_index(&self) -> usize {
        self.mode_order
            .iter()
            .position(|m| m == &self.active_mode)
            .unwrap_or(0)
    }
    
    /// Returns a list of tab titles for UI rendering.
    /// Format: "icon name" (e.g., "🔥 apps")
    pub fn get_tab_titles(&self) -> Vec<String> {
        self.mode_order
            .iter()
            .filter_map(|name| {
                self.modes.get(name).map(|mode| {
                    format!("{} {}", mode.icon(), mode.name())
                })
            })
            .collect()
    }
    
    /// Returns the ordered list of mode names.
    pub fn get_mode_order(&self) -> &[String] {
        &self.mode_order
    }
}

impl Default for ModeRegistry {
    fn default() -> Self {
        Self::new()
    }
}
