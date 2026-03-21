//! Core data structures, traits, and utilities used across all modes.
//!
//! This module houses the fundamental building blocks of LaTUI.
//!
//! - `registry`: Handles dynamic trait-boxing for the Strategy Pattern.
//! - `mode`: Defines the `Mode` trait that all feature plugins must implement.
//! - `item`: The generic `SearchResult` structure UI components consume.
//! - `utils`: Cross-cutting concerns like XDG base directory lookups.

pub mod item;
pub mod mode;
pub mod registry;
pub mod searchable_item;
pub mod utils;
pub mod icons;
pub mod execution;
