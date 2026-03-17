/// Phase 1 Integration Tests (Post-Refactor)
/// Tests for ModeRegistry and mode switching functionality

use latui::core::registry::ModeRegistry;
use latui::core::mode::Mode;
use latui::core::item::Item;
use latui::error::LatuiError;

struct MockMode {
    pub name: String,
}

impl Mode for MockMode {
    fn name(&self) -> &str { &self.name }
    fn icon(&self) -> &str { "M" }
    fn description(&self) -> &str { "Mock" }
    fn load(&mut self) -> Result<(), LatuiError> { Ok(()) }
    fn search(&mut self, _query: &str) -> Vec<Item> { vec![] }
    fn execute(&mut self, _item: &Item) -> Result<(), LatuiError> { Ok(()) }
    fn record_selection(&mut self, _query: &str, _item: &Item) {}
}

#[test]
fn test_mode_registry_initialization() {
    let mut registry = ModeRegistry::new();
    registry.register("apps", Box::new(MockMode { name: "apps".to_string() }));
    registry.register("run", Box::new(MockMode { name: "run".to_string() }));
    
    // Default mode isn't set automatically in core registry, main.rs does it usually.
    // But let's check basic structure.
    assert_eq!(registry.modes.len(), 2);
    
    registry.switch_mode("apps").unwrap();
    assert_eq!(registry.active_mode, "apps");
    
    let mode_order = registry.get_mode_order();
    assert_eq!(mode_order.len(), 2);
    assert_eq!(mode_order[0], "apps");
}

#[test]
fn test_mode_switching() {
    let mut registry = ModeRegistry::new();
    registry.register("apps", Box::new(MockMode { name: "apps".to_string() }));
    registry.register("run", Box::new(MockMode { name: "run".to_string() }));
    
    registry.switch_mode("apps").unwrap();
    assert_eq!(registry.active_mode, "apps");
    
    registry.switch_mode("run").expect("Failed to switch to run mode");
    assert_eq!(registry.active_mode, "run");
    
    let result = registry.switch_mode("nonexistent");
    assert!(result.is_err());
    assert_eq!(registry.active_mode, "run");
}

#[test]
fn test_next_mode_navigation() {
    let mut registry = ModeRegistry::new();
    registry.register("apps", Box::new(MockMode { name: "apps".to_string() }));
    registry.register("run", Box::new(MockMode { name: "run".to_string() }));
    registry.switch_mode("apps").unwrap();
    
    registry.next_mode();
    assert_eq!(registry.active_mode, "run");
    
    registry.next_mode();
    assert_eq!(registry.active_mode, "apps"); // Wrap around
}
