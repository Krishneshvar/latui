//! Terminal User Interface (TUI) components and rendering logic.
//!
//! Built atop `ratatui`, this module converts the abstract configuration
//! and application state into visual terminal buffers. It heavily relies on
//! `crossterm` for terminal control.
//!
//! - `renderer`: The main draw loop dispatch.
//! - `theme` & `bundled_themes`: Parsed color palettes.
//! - `style_resolver`: Translates config HEX colors into Ratatui `Color`s.
//! - `components`: Reusable UI widgets (tabs, scrollbars, inputs).

pub mod renderer;
pub mod theme;
pub mod bundled_themes;
pub mod style_resolver;
pub mod components;
