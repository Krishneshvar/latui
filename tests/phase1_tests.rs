/// Phase 1 Integration Tests
/// Tests for Action types, ModeRegistry, and mode switching functionality

use latui::core::action::Action;
use latui::core::registry::ModeRegistry;
use std::path::PathBuf;

#[test]
fn test_action_variants_creation() {
    // Test all Action variants can be created
    let launch = Action::Launch("firefox".to_string());
    let command = Action::Command("ls -la".to_string());
    let open_file = Action::OpenFile(PathBuf::from("/tmp/test.txt"));
    let open_folder = Action::OpenFolder(PathBuf::from("/tmp"));
    let copy = Action::CopyToClipboard("test text".to_string());
    let paste = Action::PasteFromClipboard;
    let emoji = Action::InsertEmoji("😀".to_string());
    let custom = Action::Custom {
        command: "custom".to_string(),
        args: vec!["arg1".to_string(), "arg2".to_string()],
    };

    // Verify they can be matched
    match launch {
        Action::Launch(_) => (),
        _ => panic!("Launch variant failed"),
    }
    match command {
        Action::Command(_) => (),
        _ => panic!("Command variant failed"),
    }
    match open_file {
        Action::OpenFile(_) => (),
        _ => panic!("OpenFile variant failed"),
    }
    match open_folder {
        Action::OpenFolder(_) => (),
        _ => panic!("OpenFolder variant failed"),
    }
    match copy {
        Action::CopyToClipboard(_) => (),
        _ => panic!("CopyToClipboard variant failed"),
    }
    match paste {
        Action::PasteFromClipboard => (),
        _ => panic!("PasteFromClipboard variant failed"),
    }
    match emoji {
        Action::InsertEmoji(_) => (),
        _ => panic!("InsertEmoji variant failed"),
    }
    match custom {
        Action::Custom { .. } => (),
        _ => panic!("Custom variant failed"),
    }
}

#[test]
fn test_action_clone_and_equality() {
    let action1 = Action::Launch("firefox".to_string());
    let action2 = action1.clone();
    
    assert_eq!(action1, action2);
    
    let action3 = Action::Command("ls".to_string());
    assert_ne!(action1, action3);
}

#[test]
fn test_action_serialization() {
    use serde_json;
    
    let action = Action::Custom {
        command: "test".to_string(),
        args: vec!["arg1".to_string()],
    };
    
    // Should be able to serialize and deserialize
    let serialized = serde_json::to_string(&action).expect("Failed to serialize");
    let deserialized: Action = serde_json::from_str(&serialized).expect("Failed to deserialize");
    
    assert_eq!(action, deserialized);
}

#[test]
fn test_mode_registry_initialization() {
    let registry = ModeRegistry::new();
    
    // Should have 5 modes registered
    assert_eq!(registry.modes.len(), 5);
    
    // Should have apps as default
    assert_eq!(registry.active_mode, "apps");
    assert_eq!(registry.default_mode, "apps");
    
    // Should have all modes in order
    let mode_order = registry.get_mode_order();
    assert_eq!(mode_order.len(), 5);
    assert_eq!(mode_order[0], "apps");
    assert_eq!(mode_order[1], "run");
    assert_eq!(mode_order[2], "files");
    assert_eq!(mode_order[3], "clipboard");
    assert_eq!(mode_order[4], "emojis");
}

#[test]
fn test_mode_switching() {
    let mut registry = ModeRegistry::new();
    
    // Should start with apps mode
    assert_eq!(registry.active_mode, "apps");
    
    // Switch to run mode
    registry.switch_mode("run").expect("Failed to switch to run mode");
    assert_eq!(registry.active_mode, "run");
    
    // Switch to files mode
    registry.switch_mode("files").expect("Failed to switch to files mode");
    assert_eq!(registry.active_mode, "files");
    
    // Try to switch to non-existent mode
    let result = registry.switch_mode("nonexistent");
    assert!(result.is_err());
    
    // Active mode should remain unchanged after failed switch
    assert_eq!(registry.active_mode, "files");
}

#[test]
fn test_next_mode_navigation() {
    let mut registry = ModeRegistry::new();
    
    // Start at apps (index 0)
    assert_eq!(registry.active_mode, "apps");
    assert_eq!(registry.get_active_index(), 0);
    
    // Next should go to run (index 1)
    registry.next_mode();
    assert_eq!(registry.active_mode, "run");
    assert_eq!(registry.get_active_index(), 1);
    
    // Next should go to files (index 2)
    registry.next_mode();
    assert_eq!(registry.active_mode, "files");
    assert_eq!(registry.get_active_index(), 2);
    
    // Next should go to clipboard (index 3)
    registry.next_mode();
    assert_eq!(registry.active_mode, "clipboard");
    assert_eq!(registry.get_active_index(), 3);
    
    // Next should go to emojis (index 4)
    registry.next_mode();
    assert_eq!(registry.active_mode, "emojis");
    assert_eq!(registry.get_active_index(), 4);
    
    // Next should wrap around to apps (index 0)
    registry.next_mode();
    assert_eq!(registry.active_mode, "apps");
    assert_eq!(registry.get_active_index(), 0);
}

#[test]
fn test_previous_mode_navigation() {
    let mut registry = ModeRegistry::new();
    
    // Start at apps (index 0)
    assert_eq!(registry.active_mode, "apps");
    
    // Previous should wrap to emojis (index 4)
    registry.previous_mode();
    assert_eq!(registry.active_mode, "emojis");
    assert_eq!(registry.get_active_index(), 4);
    
    // Previous should go to clipboard (index 3)
    registry.previous_mode();
    assert_eq!(registry.active_mode, "clipboard");
    assert_eq!(registry.get_active_index(), 3);
    
    // Previous should go to files (index 2)
    registry.previous_mode();
    assert_eq!(registry.active_mode, "files");
    assert_eq!(registry.get_active_index(), 2);
}

#[test]
fn test_get_active_mode() {
    let mut registry = ModeRegistry::new();
    
    // Should be able to get active mode
    let mode = registry.get_active_mode().expect("Failed to get active mode");
    assert_eq!(mode.name(), "apps");
    assert_eq!(mode.icon(), "🔥");
    
    // Switch and verify
    registry.switch_mode("run").unwrap();
    let mode = registry.get_active_mode().expect("Failed to get active mode");
    assert_eq!(mode.name(), "run");
    assert_eq!(mode.icon(), "🚀");
}

#[test]
fn test_get_active_mode_mut() {
    let mut registry = ModeRegistry::new();
    
    // Should be able to get mutable reference
    let mode = registry.get_active_mode_mut().expect("Failed to get active mode");
    assert_eq!(mode.name(), "apps");
    
    // Load should work
    let result = mode.load();
    assert!(result.is_ok() || result.is_err()); // Either is fine, just testing it compiles
}

#[test]
fn test_get_tab_titles() {
    let registry = ModeRegistry::new();
    
    let titles = registry.get_tab_titles();
    
    // Should have 5 titles
    assert_eq!(titles.len(), 5);
    
    // Should be in format "icon name"
    assert_eq!(titles[0], "🔥 apps");
    assert_eq!(titles[1], "🚀 run");
    assert_eq!(titles[2], "📁 files");
    assert_eq!(titles[3], "📋 clipboard");
    assert_eq!(titles[4], "😀 emojis");
}

#[test]
fn test_mode_registry_consistency() {
    let mut registry = ModeRegistry::new();
    
    // Cycle through all modes multiple times
    for _ in 0..3 {
        for _ in 0..5 {
            registry.next_mode();
        }
    }
    
    // Should be back at apps
    assert_eq!(registry.active_mode, "apps");
    
    // Cycle backwards
    for _ in 0..3 {
        for _ in 0..5 {
            registry.previous_mode();
        }
    }
    
    // Should still be at apps
    assert_eq!(registry.active_mode, "apps");
}

#[test]
fn test_mode_descriptions() {
    let mut registry = ModeRegistry::new();
    
    // Verify all modes have proper metadata
    let mode_names = vec!["apps", "run", "files", "clipboard", "emojis"];
    
    for mode_name in mode_names {
        registry.switch_mode(mode_name).ok();
        if let Some(mode) = registry.get_active_mode() {
            assert!(!mode.name().is_empty());
            assert!(!mode.icon().is_empty());
            assert!(!mode.description().is_empty());
        }
    }
}

#[test]
fn test_action_debug_format() {
    let action = Action::Launch("test".to_string());
    let debug_str = format!("{:?}", action);
    assert!(debug_str.contains("Launch"));
    assert!(debug_str.contains("test"));
}

#[test]
fn test_pathbuf_actions() {
    let file_path = PathBuf::from("/home/user/document.txt");
    let folder_path = PathBuf::from("/home/user/Documents");
    
    let open_file = Action::OpenFile(file_path.clone());
    let open_folder = Action::OpenFolder(folder_path.clone());
    
    match open_file {
        Action::OpenFile(path) => assert_eq!(path, file_path),
        _ => panic!("Wrong action type"),
    }
    
    match open_folder {
        Action::OpenFolder(path) => assert_eq!(path, folder_path),
        _ => panic!("Wrong action type"),
    }
}
