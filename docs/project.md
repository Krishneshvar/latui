# latui

Terminal Application Launcher for Wayland

---

# project.md

## Overview

**latui** is a fast, modular, terminal-based launcher designed for Wayland systems. It acts as a TUI replacement for tools like rofi, wofi, and fuzzel while remaining compositor‑agnostic.

latui focuses on:

* Speed
* Extensibility
* Scriptability
* Wayland compatibility
* Minimal resource usage
* Terminal-first workflows

The project is intended to become a universal launcher capable of handling application launching, window switching, file searching, command execution, clipboard access, and custom workflows through plugins.

---

## Goals

### Primary Goals

1. Provide a **fast TUI launcher** that works on any Wayland compositor.
2. Replace common launcher utilities such as:

   * rofi
   * wofi
   * fuzzel
3. Offer a **plugin-based architecture** so users can add new modes.
4. Provide **extensive configuration and theming**.
5. Maintain **very low startup latency (<50ms)**.

### Secondary Goals

* Universal search across apps, files, windows, and clipboard
* Script integration
* Custom workflows
* Extensible plugin ecosystem

---

## Key Features

### 1. Multiple Modes

latui supports multiple modes similar to rofi.

Examples:

* `apps` → launch desktop applications
* `run` → execute shell commands
* `files` → fuzzy search files
* `windows` → switch windows
* `clipboard` → clipboard history
* `emoji` → emoji search
* `calc` → quick calculations

Each mode is implemented as a **plugin module**.

---

### 2. Fuzzy Search

latui uses a high‑performance fuzzy matching engine to rank results quickly and accurately.

Requirements:

* Typo tolerance
* Fast scoring
* Highlighted matches
* Large dataset support

---

### 3. Wayland Compatibility

latui aims to work with **all Wayland compositors**.

Strategies include:

* Using **xdg-desktop-portal** when possible
* Supporting compositor-specific integrations
* Providing a **fallback abstraction layer**

Example compositor integrations:

* Hyprland
* Sway
* river
* KDE KWin
* GNOME Mutter

---

### 4. Plugin System

latui modes are implemented as plugins.

Benefits:

* Users can extend functionality
* Developers can build integrations
* Custom workflows become possible

Plugins can provide:

* searchable items
* actions
* preview content

---

### 5. Script Integration

Users can register scripts that act as dynamic data sources.

Example:

```
~/.config/latui/scripts/
    wifi
    bluetooth
    docker
    systemctl
```

Scripts expose two commands:

```
script --list
script --run <item>
```

---

### 6. Theming

latui supports color theming for terminal environments.

Example:

```
[theme]
bg = "#1e1e2e"
fg = "#cdd6f4"
accent = "#89b4fa"
selection = "#313244"
```

---

### 7. Configuration

latui uses **TOML configuration files**.

Reasons for choosing TOML:

* Human-readable
* Strong Rust ecosystem support
* Hierarchical structure
* Easy to validate

Configuration location:

```
~/.config/latui/config.toml
```

---

## Technology Stack

Language:

* Rust

Core Libraries:

* ratatui (TUI rendering)
* crossterm (terminal backend)
* nucleo (fuzzy matching)
* serde + toml (configuration)
* zbus (DBus communication)

---

## Performance Targets

| Metric         | Target  |
| -------------- | ------- |
| Startup time   | < 50 ms |
| Search latency | < 10 ms |
| Memory usage   | < 30 MB |

---

## Repository Structure

```
latui/

├── src/
│   ├── app
│   ├── config
│   ├── core
│   ├── modes
│   ├── plugins
│   ├── ui
│   └── util
│
├── assets/
├── docs/
│   ├── architecture.md
│   └── project.md
│
├── examples/
└── Cargo.toml
```

---

## Future Features

Potential roadmap items:

* Preview panels
* Multi-select actions
* Persistent launcher mode
* Remote plugin registry
* AI command palette

---
