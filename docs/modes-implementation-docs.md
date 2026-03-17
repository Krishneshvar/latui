# LaTUI: Multi-Mode Implementation Guide

This document provides a detailed technical roadmap and implementation guide for expanding LaTUI from a single-mode application launcher into a versatile multi-mode productivity tool.

---

## 🏗️ Project Status & Assessment

### Current Progress
| Component | Status | Details |
| :--- | :--- | :--- |
| **Core Infrastructure** | ✅ 100% | `Mode` trait, `ModeRegistry`, and library structure are complete. |
| **Loose Coupling Architecture** | ✅ 100% | Strategy Pattern with metadata-based execution is implemented. |
| **Apps Mode** | ✅ 100% | Fully functional with trie-based search, indexing, and frequency tracking. |
| **Run Mode** | 🚧 10% | Stub implementation created; logic pending. |
| **Files Mode** | 🚧 10% | Stub implementation created; logic pending. |
| **Clipboard Mode** | 🚧 10% | Stub implementation created; logic pending. |
| **Emojis Mode** | 🚧 10% | Stub implementation created; logic pending. |
| **UI Multi-Mode Support** | ✅ 100% | Tabs, mode switching, and multi-mode UI complete. |
| **Theme System** | 🚧 10% | Stub implementation created. |
| **Supporting Infrastructure** | ✅ 100% | Error handling, logging, and database migrations are ready. |

### Architecture Design
The current implementation uses a **loosely coupled Strategy Pattern**:
- **No central Action enum**: Each mode interprets `Item.metadata` independently
- **Better extensibility**: New modes don't require core type changes
- **Clean separation**: Modes are self-contained and independently testable

### File Structure Analysis
The current directory structure is optimized for modularity:
- `src/core/`: Contains the `Mode` trait, `Registry`, and `Item` struct.
- `src/modes/`: Individual mode implementations (each handles its own execution).
- `src/app/`: Application state management and controller.
- `src/ui/`: TUI rendering logic with tabs and mode switching.

---

## 🎯 Design Philosophy

### Why Loose Coupling?

The original plan included a central `Action` enum, but the current architecture uses **loose coupling** instead:

**Benefits:**
1. **Extensibility**: Add new modes without modifying core types
2. **Independence**: Each mode evolves independently
3. **Simplicity**: No complex action dispatching logic needed
4. **Type Safety**: Modes validate their own metadata

**Trade-offs:**
- Metadata is stringly-typed (but validated by each mode)
- No compile-time guarantees about metadata format
- Each mode must handle its own parsing

**Verdict**: The loose coupling approach is **superior** for a plugin-based architecture.

---

## 🚀 Implementation Phases

### Phase 1: Core Registry Integration & Mode Architecture ✅ COMPLETE
Establish the foundational multi-mode architecture using loose coupling and the Strategy Pattern.

#### 1.1 Architecture Overview
The current implementation uses a **loosely coupled** design where:
- Each Mode is responsible for its own execution logic
- Items store mode-specific data in `metadata: Option<String>`
- No central Action enum is needed (better extensibility)
- Follows the Strategy Pattern for clean separation of concerns

**Current Item Structure:**
```rust
pub struct Item {
    pub id: String,
    pub title: String,
    pub search_text: String,
    pub description: Option<String>,
    pub metadata: Option<String>,  // Mode-specific execution data
}
```

**Mode Trait (Strategy Pattern):**
```rust
pub trait Mode {
    fn name(&self) -> &str;
    fn icon(&self) -> &str;
    fn description(&self) -> &str;
    
    fn load(&mut self) -> Result<(), LatuiError>;
    fn search(&mut self, query: &str) -> Vec<Item>;
    fn execute(&mut self, item: &Item) -> Result<(), LatuiError>;  // Each mode interprets Item
    fn record_selection(&mut self, query: &str, item: &Item);
}
```

#### 1.2 Registry Integration (COMPLETE)
The `ModeRegistry` is fully integrated in `src/main.rs`:

```rust
// Mode registration
app.mode_registry.register("apps", Box::new(AppsMode::new(frequency_tracker, keyword_mapper)));
app.mode_registry.register("run", Box::new(RunMode::new()));
app.mode_registry.register("files", Box::new(FilesMode::new()));
app.mode_registry.register("clipboard", Box::new(ClipboardMode::new()));
app.mode_registry.register("emojis", Box::new(EmojisMode::new()));

// Mode switching in controller (Tab/Shift+Tab)
(KeyCode::Tab, KeyModifiers::NONE) => {
    app.mode_registry.next_mode();
    app.query.clear();
    self.update_results(app);
}
```

#### 1.3 Execution Flow
```
User presses Enter
  → Controller calls mode.execute(item)
    → Mode interprets item.metadata
      → Mode performs action (launch app, run command, open file, etc.)
```

**Example - AppsMode:**
```rust
fn execute(&mut self, item: &Item) -> Result<(), LatuiError> {
    if let Some(cmd) = &item.metadata {  // metadata contains the command
        Command::new("sh").arg("-c").arg(cmd).spawn()?;
    }
    Ok(())
}
```

---

## 🛠️ Mode Implementations

### 1. Run Mode (Command Executor)
Allows executing arbitrary shell commands with history tracking.

**Key Features:**
- Shell integration (`$SHELL` or `/bin/sh`).
- Persistent command history (`run_history.json`).
- Direct execution and history-based search.

```rust
// src/modes/run.rs implementation highlights
impl Mode for RunMode {
    fn name(&self) -> &str { "run" }
    fn icon(&self) -> &str { "🚀" }
    
    fn search(&mut self, query: &str) -> Vec<Item> {
        let mut results = Vec::new();
        
        if query.is_empty() {
            return self.get_recent_history(10);
        }
        
        // Direct command execution
        results.push(Item {
            id: "direct".into(),
            title: format!("Run: {}", query),
            search_text: query.to_lowercase(),
            description: Some("Execute command".into()),
            metadata: Some(query.to_string()),  // Store command in metadata
        });
        
        // Add history matches...
        
        results
    }
    
    fn execute(&mut self, item: &Item) -> Result<(), LatuiError> {
        if let Some(cmd) = &item.metadata {
            // Execute the command stored in metadata
            Command::new("sh").arg("-c").arg(cmd).spawn()?;
            self.add_to_history(cmd);
        }
        Ok(())
    }
}
```

### 2. Files Mode (Filesystem Search)
High-performance file and folder discovery.

**Key Features:**
- Recent files tracking.
- Home directory depth-limited search (via `walkdir`).
- Preview support for text files.

```rust
// src/modes/files.rs implementation highlights
impl Mode for FilesMode {
    fn search(&mut self, query: &str) -> Vec<Item> {
        use walkdir::WalkDir;
        
        WalkDir::new(&self.search_dir)
            .max_depth(3)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_name().to_string_lossy().to_lowercase().contains(query))
            .take(50)
            .map(|e| {
                let path = e.path();
                Item {
                    id: path.to_string_lossy().to_string(),
                    title: path.file_name().unwrap().to_string_lossy().to_string(),
                    search_text: path.to_string_lossy().to_lowercase(),
                    description: Some(path.parent().unwrap().to_string_lossy().to_string()),
                    metadata: Some(path.to_string_lossy().to_string()),  // Store path
                }
            })
            .collect()
    }
    
    fn execute(&mut self, item: &Item) -> Result<(), LatuiError> {
        if let Some(path) = &item.metadata {
            // Open file with default application
            Command::new("xdg-open").arg(path).spawn()?;
        }
        Ok(())
    }
}
```

### 3. Clipboard Mode
A searchable manager for OS clipboard history.

**Key Features:**
- Persistent clip storage.
- Quick copy-back functionality via `wl-copy` or `xclip`.
- Full-text preview for long snippets.

```rust
// src/modes/clipboard.rs implementation
impl Mode for ClipboardMode {
    fn search(&mut self, query: &str) -> Vec<Item> {
        self.history
            .iter()
            .filter(|clip| clip.to_lowercase().contains(&query.to_lowercase()))
            .map(|clip| Item {
                id: format!("clip-{}", clip.len()),
                title: clip.chars().take(50).collect(),
                search_text: clip.to_lowercase(),
                description: Some(format!("{} chars", clip.len())),
                metadata: Some(clip.clone()),  // Store clipboard content
            })
            .collect()
    }
    
    fn execute(&mut self, item: &Item) -> Result<(), LatuiError> {
        if let Some(content) = &item.metadata {
            // Copy to clipboard using wl-copy or xclip
            Command::new("wl-copy").arg(content).spawn()?;
        }
        Ok(())
    }
}
```

### 4. Emojis Mode
A lightweight emoji picker with keyword and category search.

**Key Features:**
- Embedded database of common emojis.
- Searchable by name, category, or keywords.
- Single-click copy to clipboard.

```rust
// src/modes/emojis.rs implementation
impl Mode for EmojisMode {
    fn search(&mut self, query: &str) -> Vec<Item> {
        self.emoji_db
            .iter()
            .filter(|(name, _)| name.contains(&query.to_lowercase()))
            .map(|(name, emoji)| Item {
                id: format!("emoji-{}", name),
                title: format!("{} {}", emoji, name),
                search_text: name.clone(),
                description: Some("Emoji".into()),
                metadata: Some(emoji.clone()),  // Store emoji character
            })
            .collect()
    }
    
    fn execute(&mut self, item: &Item) -> Result<(), LatuiError> {
        if let Some(emoji) = &item.metadata {
            // Copy emoji to clipboard
            Command::new("wl-copy").arg(emoji).spawn()?;
        }
        Ok(())
    }
}
```

---

## 🎨 UI & Theme System

### Multi-Mode UI Extensions
Update `src/ui/renderer.rs` to support mode tabs and a preview panel.

```rust
pub fn draw(f: &mut Frame, app: &mut AppState) {
    let chunks = Layout::default()
        .constraints([
            Constraint::Length(3),  // Mode tabs
            Constraint::Length(3),  // Search input
            Constraint::Min(0),     // Results
            Constraint::Length(5),  // Preview (optional)
        ])
        .split(f.area());

    // Render Mode Tabs
    if let Some(ref registry) = app.mode_registry {
        let tabs = Tabs::new(registry.get_tab_names())
            .block(Block::default().borders(Borders::ALL).title("Modes"))
            .select(registry.get_active_index());
        f.render_widget(tabs, chunks[0]);
    }
    
    // ... render search and results ...
}
```

### Dynamic Theme System
Implement a TOML-based theme system to allow users to customize colors.

```toml
# ~/.config/latui/themes/dark.toml
name = "dark"

[colors]
background = { r = 30, g = 30, b = 30 }
foreground = { r = 200, g = 200, b = 200 }
accent = { r = 100, g = 150, b = 255 }
tab_active = { r = 255, g = 200, b = 0 }
```

---

## ⚙️ Configuration
The configuration system ties all modes together via `~/.config/latui/config.toml`.

```toml
[general]
default_mode = "apps"
theme = "dark"

[keybindings]
switch_mode_next = "Tab"
toggle_preview = "Ctrl+p"

[modes.run]
enabled = true
max_history = 100

[modes.files]
search_paths = ["~", "~/Documents"]
```

---

## 📅 Roadmap Summary

1.  **Phase 1:** ✅ Registry and Mode Architecture (COMPLETE)
2.  **Phase 2:** 🚧 Run Mode (History & Execution)
3.  **Phase 3:** 🚧 Emojis Mode (Simple static data)
4.  **Phase 4:** ✅ UI Update (Tabs & Layout) (COMPLETE)
5.  **Phase 5:** 🚧 Files Mode (Recursive search & Preview)
6.  **Phase 6:** 🚧 Clipboard Mode (OS integration)
7.  **Phase 7:** 🚧 Themes & Configuration

---

## 📝 Implementation Notes

### For New Mode Developers

When creating a new mode:

1. **Define your metadata format** (document it in comments)
2. **Validate metadata** in your `execute()` method
3. **Handle errors gracefully** (return `LatuiError` on failure)
4. **Use `Item.metadata`** to store execution data

**Example Template:**
```rust
pub struct MyMode {
    // Your state here
}

impl Mode for MyMode {
    fn name(&self) -> &str { "mymode" }
    fn icon(&self) -> &str { "🎯" }
    fn description(&self) -> &str { "My Custom Mode" }
    
    fn load(&mut self) -> Result<(), LatuiError> {
        // Initialize your mode
        Ok(())
    }
    
    fn search(&mut self, query: &str) -> Vec<Item> {
        // Return items with metadata
        vec![Item {
            id: "item-1".into(),
            title: "Example".into(),
            search_text: "example".into(),
            description: Some("Description".into()),
            metadata: Some("your-execution-data".into()),
        }]
    }
    
    fn execute(&mut self, item: &Item) -> Result<(), LatuiError> {
        // Parse and validate metadata
        let data = item.metadata.as_ref()
            .ok_or_else(|| LatuiError::App("Missing metadata".into()))?;
        
        // Perform your action
        // ...
        
        Ok(())
    }
    
    fn record_selection(&mut self, _query: &str, _item: &Item) {
        // Optional: track usage
    }
}
```

---

*This implementation guide reflects the current loosely coupled architecture. Each phase can be tackled independently.*
