//! SQLite-backed usage tracking and frequency metrics.
//!
//! Modes notify the `FrequencyTracker` upon execution to persistently log
//! usage counts and timestamps, boosting popular items in future searches.
//!
//! - `database`: Low-level SQLite `rusqlite` interactions.
//! - `frequency`: High-level tracking API used by modes.

pub mod database;
pub mod frequency;
