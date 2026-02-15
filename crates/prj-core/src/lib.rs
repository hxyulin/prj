//! Core library for `prj` — a local project manager.
//!
//! Provides project detection, database persistence, statistics collection,
//! artifact cleaning, and workspace export/import.

pub mod clean;
pub mod config;
pub mod detect;
pub mod error;
pub mod manifest;
pub mod project;
pub mod stats;
