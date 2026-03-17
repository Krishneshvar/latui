# LaTUI: Multi-Mode Implementation Guide

This document provides a detailed technical roadmap and implementation guide for LaTUI's multi-mode productivity toolset.

---

## 🏗️ Project Status & Assessment

### Current Progress
| Component | Status | Details |
| :--- | :--- | :--- |
| **Core Infrastructure** | ✅ 100% | `Mode` trait, `ModeRegistry`, and library structure are complete. |
| **Loose Coupling Architecture** | ✅ 100% | Strategy Pattern with metadata-based execution is implemented. |
| **Apps Mode** | ✅ 100% | Fully functional with trie-based search, indexing, frequency tracking, and keyword mappings. |
| **Run Mode** | ✅ 100% | Comprehensive shell command executor with persistent history and fuzzy search. |
| **Files Mode** | ✅ 100% | High-performance filesystem navigator with recent files tracking and text previews. |
| **Clipboard Mode** | ✅ 100% | System clipboard history manager with wayland/x11 backends and persistent storage. |
| **Emojis Mode** | ✅ 100% | Embedded emoji picker with keyword indexing, recency tracking, and copy-on-select. |
| **UI Multi-Mode Support** | ✅ 100% | Tabs, mode switching, and multi-mode UI complete. |
| **Theme System** | 🚧 10% | Basic theme structure exists; further rendering support planned. |
| **Supporting Infrastructure** | ✅ 100% | Error handling, logging, SQLite-based frequency tracking, and XDG persistence are ready. |

### Architecture Design
The implementation uses a **loosely coupled Strategy Pattern**:
- **Metadata-based execution**: Each mode interprets `Item.metadata` independently (e.g., RunMode stores raw commands, FilesMode stores JSON paths/kinds).
- **Loose Coupling**: New modes can be added with zero changes to the core app logic.
- **Dependency Injection**: Modes receive shared components like `FrequencyTracker` or `KeywordMapper` during registration.

---

## 🎯 Design Philosophy

### Loose Coupling vs. Central Enums
LaTUI explicitly avoids a central `Action` enum in favor of polymorphic `execute()` calls on the `Mode` trait.

**Benefits:**
1. **Extensibility**: Add modes without modifying core types.
2. **Encapsulation**: Execution logic lives exactly where the data is defined.
3. **Simplicity**: No complex dispatch logic or large match statements in the controller.

---

## 🚀 Completed Modes

### 1. Apps Mode (Launcher)
- **Engine**: Trie-based prefix filtering + Fuzzy scoring via `SearchEngine`.
- **Ranking**: Frequency (all-time), Recency (last used), and Query-specific (learning) boosts.
- **Keywords**: Semantic mappings (e.g., "browser" → Firefox).

### 2. Run Mode (Shell)
- **Features**: Persistent `run_history.json`, deduplication, and max-capacity capping (1000 entries).
- **Execution**: Direct shell execution via `sh -c`.

### 3. Files Mode (Filesystem)
- **Strategy**: Hybrid search. Fuzzy-search in recents + live walk using the `ignore` crate for speed.
- **Preview**: Native text file previewing with size capping.
- **Security**: Path validation and rate-limiting on execution.

### 4. Clipboard Mode (History)
- **Backends**: Automatic runtime detection of `wl-copy`/`wl-paste` (Wayland) or `xclip` (X11).
- **Storage**: Persistent JSON history with 0600 file permissions for privacy.
- **Interaction**: `stays_open = true` allows picking multiple items without relaunching.

### 5. Emojis Mode (Picker)
- **Database**: 240+ static embedded emojis with keyword/category indexing.
- **Flow**: Fast fuzzy search → copy to clipboard → frequency tracking.

---

## 📅 Roadmap Summary

1.  **Phase 1:** ✅ Registry and Mode Architecture
2.  **Phase 2:** ✅ Run Mode Implementation
3.  **Phase 3:** ✅ Emojis Mode Implementation
4.  **Phase 4:** ✅ UI Update (Tabs & Layout)
5.  **Phase 5:** ✅ Files Mode Implementation
6.  **Phase 6:** ✅ Clipboard Mode Implementation
7.  **Phase 7:** 🚧 Themes & Configuration Refinement

---

*This implementation guide reflects the current state of LaTUI. All primary modes are built to production-grade standards with comprehensive test coverage.*
