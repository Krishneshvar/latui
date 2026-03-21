# Architecture

LaTUI is designed with a strict emphasis on performance, low latency (<50ms startup), and modularity. It is built using Rust and the [Ratatui](https://github.com/ratatui-org/ratatui) terminal UI library. 

This document outlines the high-level architecture, the module responsibilities, and the core design patterns that make LaTUI extensible.

## 1. High-Level Overview

LaTUI functions as a state machine where UI rendering, event handling, and data searching are decoupled. The core pipeline is:
1. **Input/Event Loop**: Capture keystrokes (Crossterm).
2. **State Mutation**: Update queries or UI selection indexes.
3. **Execution/Search**: Run non-blocking search queries against the active dataset.
4. **Rendering**: Draw the updated state to the terminal buffer via Ratatui.

### Directory Structure

The `src/` directory is logically divided as follows:
- **`app/`**: Contains the central application state (`AppState`) and the main event loop / controller.
- **`config/`**: Handles loading, parsing (`TOML`), and applying user configurations and themes from `~/.config/latui/config.toml`.
- **`core/`**: General-purpose utilities, error handling traits, and XDG base directory integrations.
- **`ui/`**: Reusable Ratatui rendering components (e.g., Tab bars, search inputs, list views, and image renderers).
- **`modes/`**: The distinct operational modes of LaTUI (Apps, Files, Run, Clipboard, Emojis).
- **`search/` & `matcher/`**: The hybrid search engine.
- **`tracking/`**: SQLite-based telemetry for tracking application usage frequency.

## 2. Core Design Patterns

### The Strategy Pattern (Modes)

At the heart of LaTUI's extensibility is the **Mode Trait** (Strategy Pattern). Instead of hardcoding logic for applications, files, or clipboard entries into the main loop,, all features implement a unified `Mode` interface.

```rust
pub trait Mode {
    fn name(&self) -> &str;
    fn load(&mut self) -> Result<()>;
    fn search(&mut self, query: &str) -> Vec<SearchResult>;
    fn execute(&mut self, item: &SearchResult) -> Result<()>;
    // ... rendering/preview hooks
}
```

The Application State (`AppState`) holds a **Mode Registry** that dynamically dispatches events to the active mode. Adding a new feature to LaTUI simply requires:
1. Creating a new struct in `src/modes/`.
2. Implementing the `Mode` trait.
3. Registering it in `main.rs`.

### The Hybrid Search Engine (`search/` and `matcher/`)

To achieve <10ms search latency, LaTUI abstracts search into two tiers:
1. **Prefix Filtering (Trie)**: An $O(m)$ exact prefix matcher that rapidly filters out irrelevant items on the first keystroke.
2. **Fuzzy Scoring (Levenshtein/Nucleo)**: For remaining items, LaTUI uses a typo-tolerant fuzzy matcher (backed by `nucleo-matcher` or standard edit-distance algorithms) to rank results.

### Frequency Tracking (`tracking/`)

LaTUI leans heavily on a SQLite database (`usage.db`) to record execution frequencies. 
- When an item is executed (e.g., launching Firefox), the `tracking` module increments its execution count and updates the `last_used` timestamp.
- The `search` engine queries this SQLite DB at runtime to artificially boost the relevance scores of frequently used items.

## 3. The UI Rendering Pipeline (`ui/`)

LaTUI uses immediate-mode rendering via Ratatui. The `controller.rs` orchestrates the draw cycle:
1. The **Theme** struct provides the current color palette.
2. The UI is split into Chunks (Title/Tabs, Search Input, Main List, Preview Pane) via standard Ratatui Layouts.
3. Items are drawn based on the current active Mode's data. If the terminal supports graphics protocols (Kitty/Sixel), LaTUI utilizes the `ratatui-image` crate within the renderer to overlay actual image buffers.

## 4. Concurrency

While the main UI and event loop operate on a single thread to guarantee deterministic rendering, expensive I/O operations (like deep filesystem traversal in `FilesMode` using the `ignore` crate) are parallelized via `rayon`.
