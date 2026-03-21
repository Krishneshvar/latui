# LaTUI: The Ultra-Fast Multi-Mode Launcher

<p align="center">
  <img src="assets/latui.png" alt="LaTUI Logo" width="300" />
</p>

<p align="center">
  <b>A blazing-fast, modular, and extensible productivity hub built with Rust and Ratatui.</b>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Language-Rust-orange?style=for-the-badge&logo=rust" alt="Rust" />
  <img src="https://img.shields.io/badge/Framework-Ratatui-red?style=for-the-badge&logo=rust" alt="Ratatui" />
  <img src="https://img.shields.io/badge/License-GPL--3.0-blue?style=for-the-badge" alt="GPL-3.0" />
</p>

---

## Overview

**LaTUI** (Launcher TUI) is more than just a simple application launcher. It’s an ultra-high performance terminal interface designed for sub-50ms latency, offering a unified, keyboard-centric workflow for everything from launching software to managing your clipboard and searching your filesystem.

Built with a **Strategy-Pattern** architecture, LaTUI is compositor-agnostic and uses a hybrid search engine combining **O(m) Trie-based prefix filtering** with **Levenshtein typo tolerance** and **SQLite-backed frequency tracking**.

---

## Key Features

- **Extreme Performance**: Built for speed with Rust, achieving <50ms startup and <10ms search latency.
- **Intelligent Ranking**: Learns your habits using SQLite to track usage frequency and recency.
- **Modular Architecture**: Every feature is a "Mode"—add or remove functionality with zero core logic changes.
- **Hybrid Search Engine**: Smart prefix matching + Damerau-Levenshtein typo tolerance.
- **Modern Interface**: Premium, responsive TUI with tabs, previews, and smooth layouts.
- **Config-First**: Fully customizable via `~/.config/latui/config.toml`.

---

## Available Modes

LaTUI is organized into specialized **Modes**, each optimized for specific tasks. Use `Tab` to switch between them instantly.

### Apps Mode (Launcher)
The core launcher experience. Index your `.desktop` files with keyword-aware search (e.g., search "browser" for Firefox). Features all-time frequency and recency boosting.

Apps mode also supports real icon-image rendering in terminals that expose image protocols (Kitty graphics / Sixel). In practice this means terminals like `kitty` and `foot` (with sixel enabled) can render app icon images; other terminals automatically fall back to text icons.

### Files Mode (Navigator)
Search and navigate your filesystem at lightning speeds. Uses the high-performance `ignore` crate for traversal and includes **text file previews**.

### Run Mode (Shell)
A persistent shell command executor with history deduplication and fuzzy searching. No more searching through shell history; find it instantly in LaTUI.

### Clipboard Mode (Manager)
Manage your clipboard history across Wayland (`wl-clipboard`) and X11 (`xclip`). Features persistent storage and privacy-aware `0600` file permissions.

### Emojis Mode (Picker)
A fast, keyword-searchable emoji picker. Copy any of the 240+ embedded emojis to your clipboard instantly with frequency-based sorting.

---

## Architecture

LaTUI follows a **loosely coupled Strategy Pattern** where the interaction between the UI and functionality is mediated by a common trait. This allows for massive extensibility and isolation of concerns.

For a deep dive into LaTUI's structure, performance considerations, and internal modules, please see our [Architecture Documentation](./ARCHITECTURE.md) (or refer to the project's GitHub Wiki).

## Configuration

LaTUI is thoroughly customizable via `~/.config/latui/config.toml`. You can configure colors, layouts, layout variants, modes, and keybindings.

For full configuration instructions, layout modes, and theming features, please consult the [Configuration Guide](./CONFIGURATION.md) (or the relevant page on the GitHub Wiki).

---

## Roadmap & Progress

| Phase | Description | Status |
| :--- | :--- | :---: |
| **Phase 1** | Core Search & Multi-field Indexing | ✅ |
| **Phase 2** | Typo Tolerance & Basic UI | ✅ |
| **Phase 3** | **Run Mode** Implementation | ✅ |
| **Phase 4** | **Emojis Mode** & Keyword Search | ✅ |
| **Phase 5** | **Files Mode** & Live Previews | ✅ |
| **Phase 6** | **Clipboard Mode** & Wayland/X11 | ✅ |
| **Phase 7** | Themes & Advanced Configuration | 🏗️ |

---

## Installation

### Arch Linux (AUR)
```bash
yay -S latui
```

### From Source
```bash
git clone https://github.com/Krishneshvar/latui
cd latui
cargo build --release
```

---

## License & Contributing

Licensed under the **GPL-3.0**. Contributions are welcome! Please ensure all new logic follows the Strategy Pattern and includes appropriate unit tests.

---

<p align="center">Made with ❤️ for the Linux Community</p>
