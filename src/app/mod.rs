//! Central application state and UI lifecycle management.
//!
//! The `app` module orchestrates the main event loop, handles global
//! state (e.g., active modes, search input), and interacts with the Ratatui backend.
//!
//! - `state`: Defines `AppState` containing configurations and mode registries.
//! - `controller`: Provides the main polling loop for terminal events.

pub mod controller;
pub mod state;
