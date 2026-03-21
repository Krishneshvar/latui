//! The hybrid search engine combining exact prefix and fuzzy matching.
//!
//! To achieve sub-10ms response times, the search module tokenizes inputs
//! and delegates to the fastest path available (`nucleo-matcher` for fuzzy).
//!
//! - `engine`: The primary parallel search pipeline.
//! - `tokenizer`: Input splitting and normalization.
//! - `typo`: Fast Levenshtein-distance fallbacks.

pub mod engine;
pub mod tokenizer;
pub mod typo;
