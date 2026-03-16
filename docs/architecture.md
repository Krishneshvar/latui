# LATUI Multi-Mode Launcher: Architecture Overview

LATUI is a powerful, extensible, and high-performance TUI launcher designed for modern Linux environments. This document outlines the core architecture, design principles, and implementation roadmap.

---

## 1. Core Architecture Principles

The architecture is built on two primary pillars: **Pluggability** and **Centralized Management**.

### 1.1 Plugin-Based Mode System
Each functionality (applications, file search, clipboard history, etc.) is implemented as a self-contained "Mode." All modes must implement the common `Mode` trait to ensure consistency and interoperability.

```rust
pub trait Mode: Send + Sync {
    fn name(&self) -> &str;
    fn icon(&self) -> &str;
    fn description(&self) -> &str;
    
    fn load(&mut self) -> Result<(), LatuiError>;
    fn search(&mut self, query: &str) -> Vec<Item>;
    fn execute(&mut self, item: &Item) -> Result<(), LatuiError>;
    fn record_selection(&mut self, query: &str, item: &Item);
    
    // Support for interactive previews
    fn supports_preview(&self) -> bool { false }
    fn preview(&self, item: &Item) -> Option<String> { None }
}
```

### 1.2 Mode Registry Pattern
A central registry manages the lifecycle and switching of modes. This allows for lazy loading and efficient memory management.

```rust
pub struct ModeRegistry {
    modes: HashMap<String, Box<dyn Mode>>,
    active_mode: String,
    default_mode: String,
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
        
        // Load custom user-defined modes
        registry.load_custom_modes();
        
        registry
    }
    
    pub fn switch_mode(&mut self, mode_name: &str) -> Result<(), LatuiError> {
        if self.modes.contains_key(mode_name) {
            self.active_mode = mode_name.to_string();
            Ok(())
        } else {
            Err(LatuiError::App(format!("Mode '{}' not found", mode_name)))
        }
    }
}
```

---

## 2. Standard Mode Implementations

### 2.1 Apps Mode (Core)
The primary mode for launching desktop applications.
*   **Status:** Implemented ✅
*   **Features:** Multi-field indexing, typo tolerance (Levenstein distance), frequency tracking via a usage database.
*   **Data Structure:** Trie-based prefix search.

### 2.2 Run Mode (Command Executor)
A fast interface for executing arbitrary shell commands with history support.

```rust
pub struct RunMode {
    history: Vec<String>,
    env_vars: HashMap<String, String>,
    shell: String,
}
```

### 2.3 Files Mode (Filesystem Search)
Locate files and folders using recent file history, bookmarks, and an optional recursive background indexer.

```rust
pub struct FilesMode {
    recent_files: Vec<PathBuf>,
    bookmarks: Vec<PathBuf>,
    indexer: Option<FileIndexer>,
}
```

### 2.4 Clipboard Mode
Navigate and search through recently copied items (Text, Images, or Files).

```rust
pub struct ClipboardMode {
    history: VecDeque<ClipboardEntry>,
    max_entries: usize,
}
```

### 2.5 Emojis Mode
A quick-access emoji picker with keyword-based search.

```rust
pub struct Emoji {
    symbol: String,
    name: String,
    keywords: Vec<String>,
    category: String,
}
```

### 2.6 Custom Modes (User-Defined)
Users can define their own modes via simple TOML configurations in `~/.config/latui/modes/*.toml`. This allows for creating custom "sub-launchers" for wallpapers, power profiles, or scripts.

---

## 3. Configuration System

LATUI uses TOML for all configuration, ensuring human-readability and easy automation.

### 3.1 Main Configuration (`config.toml`)
```toml
[general]
default_mode = "apps"
theme = "dark"
max_results = 10

[keybindings]
switch_mode = "Tab"
next_item = "Down"
prev_item = "Up"
execute = "Enter"
cancel = "Esc"

[modes.apps]
enabled = true
cache_ttl = 3600

[modes.files]
enabled = true
index_home = true
index_paths = ["~/Documents", "~/Projects"]
```

### 3.2 Custom Mode Definition Example
Users can create specialized modes like a wallpaper switcher:
```toml
[mode]
name = "wallpapers"
icon = "🖼️"
description = "Switch desktop wallpapers"

[[items]]
title = "Dark Mountain"
keywords = ["dark", "mountain", "night"]
action = { type = "command", value = "feh --bg-fill ~/wallpapers/dark-mountain.jpg" }
```

---

## 4. UI Architecture

### 4.1 Mockup Layout
The UI is designed to be sleek and informative, inspired by modern command palettes and TUI design patterns.

```text
┌─────────────────────────────────────────────────┐
│ [Apps] [Run] [Files] [Clipboard] [Emojis] [+]  │  ← Mode tabs
├─────────────────────────────────────────────────┤
│ > query_text_here                               │  ← Search input
├─────────────────────────────────────────────────┤
│ 🔥 Firefox Browser                         ⭐⭐⭐ │  ← Results
│ 🌐 Google Chrome                           ⭐⭐  │
│ 📁 Brave Browser                           ⭐   │
│ ...                                             │
├─────────────────────────────────────────────────┤
│ [Preview Panel]                                 │  ← Optional preview
│ Firefox is a free and open-source web browser  │
│ developed by Mozilla Foundation...              │
└─────────────────────────────────────────────────┘
```

### 4.2 State Management
The UI state is centralized in an `AppState` struct, managing the query string, result list, and active mode transition.

```rust
pub struct AppState {
    pub query: String,
    pub filtered_items: Vec<Item>,
    pub list_state: ListState,
    pub mode_registry: ModeRegistry,
    pub active_tab: usize,
    pub show_preview: bool,
    pub theme: Theme,
}
```

---

## 5. Directory Structure

Standardized paths for configuration and data storage:

```text
~/.config/latui/
├── config.toml              # Main configuration
├── keywords.toml            # Keyword mappings (Apps mode)
├── themes/                  # Color schemes
│   └── dark.toml
└── modes/                   # Custom mode definitions (.toml)

~/.local/share/latui/
├── usage.db                 # Usage tracking database (SQLite)
├── cache/                   # Persistent cache files
└── logs/                    # Application logs

~/.cache/latui/              # Ephemeral runtime cache
```

---

## 6. Implementation Roadmap

### Phase 1: Core Multi-Mode Infrastructure
-   Refactor `Mode` trait and implement `ModeRegistry`.
-   Update UI to support Mode Tabs and status transitions.
-   Implement the global configuration parser.

### Phase 2: Standard Modes Development
-   **Run Mode:** History tracking and PATH executable search.
-   **Files Mode:** Recently used files and directory indexing.
-   **Clipboard Mode:** Integration with system clipboards (Wayland/X11).
-   **Emojis Mode:** Bundled emoji database search.

### Phase 3: Customization & Extensibility
-   TOML-based Custom Mode parser.
-   Theme system (dynamic color swapping).
-   Dynamic Plugin support (`.so` library loading).

### Phase 4: Polish & Advanced Search
-   Interactive preview panel (e.g., file content, app descriptions).
-   Fuzzy mode switching (e.g., typing `:f` to jump to Files mode).
-   Smart mode detection based on query patterns.

---

## 7. Design Rationale

*   **Plugin Architecture:** Isolation of concerns. Bugs in one mode don't crash the core launcher.
*   **Mode Registry:** Centralized state management for easy keyboard navigation across modes.
*   **TOML-First Config:** Balances developer convenience with user-friendly extensibility.
*   **Static Search, Dynamic Execution:** Search is kept fast and local, while execution handles the varied complexities of launching shell commands or opening files.

---

*This architecture is designed for scalability and maintains high code quality standards while allowing for future growth into a comprehensive system utility.*
