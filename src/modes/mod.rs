//! The operational Modes (Strategies) available in LaTUI.
//!
//! Every feature in LaTUI is a "Mode" implementing the `Mode` trait.
//! They encapsulate loading data, searching, and execution.
//!
//! - `apps`: The core `.desktop` entry application launcher.
//! - `clipboard`: X11/Wayland clipboard history manager.
//! - `custom`: User-defined shell script menus.
//! - `emojis`: Fast keyword-based emoji picker.
//! - `files`: High-performance filesystem navigator.
//! - `run`: Shell command history executor.

pub mod apps;
pub mod clipboard;
pub mod custom;
pub mod emojis;
pub mod files;
pub mod run;
