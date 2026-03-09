# architecture.md

## System Architecture

latui follows a **modular event-driven architecture**.

Core components:

1. Input System
2. Mode Router
3. Fuzzy Matching Engine
4. Plugin System
5. UI Renderer
6. Action Executor

---

## High-Level Architecture

```
User Input
    │
    ▼
Input Handler
    │
    ▼
Mode Router
    │
    ▼
Mode Plugin
    │
    ▼
Fuzzy Matcher
    │
    ▼
UI Renderer
    │
    ▼
Action Executor
```

---

## Core Modules

### Input System

Responsible for:

* Keyboard input
* Navigation
* Keybindings
* Mode switching

Built using:

* crossterm

---

### Mode Router

Determines which mode is active and routes queries to the correct plugin.

Responsibilities:

* Activate modes
* Handle mode switching
* Forward search queries

Example:

```
> firefox
```

Router sends query to:

```
apps mode
```

---

### Plugin System

Modes implement a shared trait.

Example interface:

```
pub trait Mode {
    fn name(&self) -> &str;

    fn load(&mut self);

    fn search(&self, query: &str) -> Vec<Item>;

    fn run(&self, item: &Item);
}
```

---

### Item Model

Every mode returns searchable items.

Example structure:

```
pub struct Item {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub score: i64,
    pub action: Action,
}
```

---

### Fuzzy Matching Engine

latui uses **nucleo**, a modern Rust fuzzy matcher.

Reasons:

* Extremely fast
* Designed for interactive filtering
* Scales to large datasets

Responsibilities:

* scoring
* ranking
* match highlighting

---

### UI Renderer

Built using **ratatui**.

Layout example:

```
+--------------------------------+
| prompt                         |
|--------------------------------|
| > firefox                      |
|                                |
| firefox                        |
| firefox developer edition      |
| firefox nightly                |
|                                |
+--------------------------------+
```

Components:

* Prompt
* Results list
* Highlighted selection

---

### Action Executor

Executes the selected item.

Examples:

* launch application
* focus window
* run command
* copy clipboard

---

## Wayland Integration Layer

latui provides an abstraction layer for compositor communication.

Structure:

```
wayland/

├── hyprland.rs
├── sway.rs
├── river.rs
└── generic.rs
```

Each module implements a shared interface.

Example:

```
pub trait WindowProvider {
    fn list_windows() -> Vec<Window>;
    fn focus_window(id: &str);
}
```

---

## Data Flow

1. User types a query
2. Query goes to active mode
3. Mode returns candidate items
4. Fuzzy matcher ranks them
5. UI displays results
6. User selects item
7. Action executor runs it

---

## Thread Model

latui separates work into components:

Main thread

* UI rendering
* input handling

Worker threads

* indexing
* plugin loading
* file searching

This ensures the UI stays responsive.

---

## Caching Strategy

To reduce latency, latui caches:

* desktop entries
* file indexes
* emoji datasets

Cache location:

```
~/.cache/latui/
```

---

## Error Handling

latui follows Rust best practices:

* `Result` based APIs
* structured logging
* graceful plugin failures

---

## Security Considerations

* scripts run in user context
* plugin sandboxing planned
* command execution validated

---

## Extensibility

Future plugin types:

* network search
* git repositories
* docker containers
* system services

Plugins may eventually be distributed via a registry.

---

## Long-Term Vision

latui evolves into a **universal command palette for Linux**, combining:

* launcher
* search
* system control
* automation hub
