//! Tickit - A stunning terminal-based task manager
//!
//! Features:
//! - Beautiful TUI with multiple themes
//! - CLI for quick task management
//! - Lists and tags for organization
//! - SQLite storage
//! - Export to multiple formats

pub mod app;
pub mod config;
pub mod db;
pub mod export;
pub mod models;
pub mod theme;

pub use config::Config;
pub use db::Database;
pub use models::{ExportFormat, List, Priority, Tag, Task};
pub use theme::Theme;
