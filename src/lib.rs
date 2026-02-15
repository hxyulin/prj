//! `prj` — a local project manager for the command line.
//!
//! Maintains a database of development projects and provides fuzzy search,
//! navigation, tagging, statistics, artifact cleaning, and workspace
//! export/import.

pub mod cli;
pub mod core;
pub mod shell;
pub mod tui;
