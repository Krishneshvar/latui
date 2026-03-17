# LaTUI: Multi-Mode Implementation Guide

This document provides a detailed technical roadmap and implementation guide for expanding LaTUI from a single-mode application launcher into a versatile multi-mode productivity tool.

---

## 🏗️ Project Status & Assessment

### Current Progress
| Component | Status | Details |
| :--- | :--- | :--- |
| **Core Infrastructure** | ✅ 100% | `Mode` trait, `ModeRegistry`, and library structure are complete. |
| **Apps Mode** | ✅ 100% | Fully functional with trie-based search, indexing, and frequency tracking. |
| **Run Mode** | 🚧 10% | Stub implementation created; logic pending. |
| **Files Mode** | 🚧 10% | Stub implementation created; logic pending. |
| **Clipboard Mode** | 🚧 10% | Stub implementation created; logic pending. |
| **Emojis Mode** | 🚧 10% | Stub implementation created; logic pending. |
| **UI Multi-Mode Support** | ❌ 0% | UI currently only supports a single list view. |
| **Theme System** | 🚧 10% | Stub implementation created. |
| **Supporting Infrastructure** | ✅ 100% | Error handling, logging, and database migrations are ready. |

### File Structure Analysis
The current directory structure is optimized for modularity:
- `src/core/`: Contains the `Mode` trait and `Registry`.
- `src/modes/`: Individual implementations for each functionality.
- `src/app/`: Application state management.
- `src/ui/`: TUI rendering logic.

---

## 🚀 Implementation Phases

### Phase 1: Core Action Types & Registry Integration
Expand the shared vocabulary of the application to support diverse actions across different modes.

#### 1.1 Expand Action Types
Update `src/core/action.rs` to include actions for all planned modes:

```rust
use serde::{Serialize, Deserialize};
use std::path::PathBuf;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Action {
    // Apps mode
    Launch(String),
    
    // Run mode
    Command(String),
    
    // Files mode
    OpenFile(PathBuf),
    OpenFolder(PathBuf),
    
    // Clipboard mode
    CopyToClipboard(String),
    PasteFromClipboard,
    
    // Emojis mode
    InsertEmoji(String),
    
    // Custom modes
    Custom(String, Vec<String>),  // (command, args)
}
```

#### 1.2 Integrate Registry into Main Loop
Update `src/main.rs` to initialize the `ModeRegistry` and handle mode switching:

```rust
fn main() -> anyhow::Result<()> {
    // ... initial setup ...

    let mut registry = ModeRegistry::new();
    
    // Initialize all modes
    for (_, mode) in registry.modes.iter_mut() {
        if let Err(e) = mode.load() {
            tracing::warn!("Failed to load mode: {}", e);
        }
    }
    
    let mut app = AppState::new(Vec::new());
    app.mode_registry = Some(registry);
    
    // ... main loop ...
}

// Handling Mode Switching (Tab/BackTab)
match key.code {
    KeyCode::Tab => {
        if let Some(ref mut registry) = app.mode_registry {
            let modes: Vec<_> = registry.modes.keys().cloned().collect();
            let current_idx = modes.iter().position(|m| m == &registry.active_mode).unwrap_or(0);
            let next_idx = (current_idx + 1) % modes.len();
            let _ = registry.switch_mode(&modes[next_idx]);
            update_results(&mut app);
        }
    }
    _ => {}
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
            // Show recent commands
            return self.get_recent_history(10);
        }
        
        // 1. Direct command Execution
        results.push(Item {
            id: "direct".into(),
            title: format!("Run: {}", query),
            action: Action::Command(query.to_string()),
            // ...
        });
        
        // 2. History matches
        // ... search history ...
        
        results
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
fn search_directory(&self, dir: &PathBuf, query: &str, max_results: usize) -> Vec<PathBuf> {
    use walkdir::WalkDir;
    
    WalkDir::new(dir)
        .max_depth(3) // Performance constraint
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_name().to_string_lossy().to_lowercase().contains(query))
        .take(max_results)
        .map(|e| e.path().to_path_buf())
        .collect()
}
```

### 3. Clipboard Mode
A searchable manager for OS clipboard history.

**Key Features:**
- Persistent clip storage.
- Quick copy-back functionality via `wl-copy` or `xclip`.
- Full-text preview for long snippets.

### 4. Emojis Mode
A lightweight emoji picker with keyword and category search.

**Key Features:**
- Embedded database of common emojis.
- Searchable by name, category, or keywords.
- Single-click copy to clipboard.

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

1.  **Phase 1:** Registry and Action type expansion.
2.  **Phase 2:** Run Mode (History & Execution).
3.  **Phase 3:** Emojis Mode (Simple static data).
4.  **Phase 4:** UI Update (Tabs & Layout).
5.  **Phase 5:** Files Mode (Recursive search & Preview).
6.  **Phase 6:** Clipboard Mode (OS integration).
7.  **Phase 7:** Themes & Configuration.

---

*This implementation guide is designed to be self-contained. Each phase can be tackled independently once the core registry for Phase 1 is in place.*
