//! Internal seam for future generated low-level modules.
//!
//! This module is intentionally crate-private. The stable public SDK surface continues to live in
//! `src/client.rs`, `src/api/*.rs`, and `src/types/*.rs`.

pub(crate) mod operations;
pub(crate) mod schemas;
pub(crate) mod support;
