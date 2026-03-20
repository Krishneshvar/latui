#![warn(clippy::pedantic, clippy::nursery, missing_debug_implementations)]
#![allow(clippy::module_name_repetitions, clippy::must_use_candidate, clippy::similar_names, clippy::missing_errors_doc, clippy::too_many_lines, clippy::cast_possible_truncation, clippy::doc_markdown, clippy::cast_possible_wrap, clippy::cast_sign_loss, clippy::cast_lossless)]

//! LaTUI — A premium, high-performance, and modular TUI launcher.
//!
//! This crate provides the core logic, state management, and search modes
//! for the LaTUI application. It is designed with production-grade 
//! robustness, parallel search capabilities, and rich aesthetics.

/// Application state and business logic orchestration.
pub mod app;

/// Persistent cache subsystem for indexed items.
pub mod cache;

/// Configuration loading and theme management.
pub mod config;

/// Core traits and shared data structures.
pub mod core;

/// Centralized error handling types.
pub mod error;

/// Low-level indexing structures (e.g. Tries).
pub mod index;

/// Generic fuzzy matching and scoring logic.
pub mod matcher;

/// Built-in search modes (Apps, Run, Files, etc.).
pub mod modes;

/// Parallel search engine and tokenization.
pub mod search;

/// Usage tracking and frequency-based boosting.
pub mod tracking;

/// Terminal UI rendering and styling.
pub mod ui;
