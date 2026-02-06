//! Sync module for optional cloud synchronization
//!
//! This module provides functionality to sync tasks, lists, and tags
//! with a self-hosted tickit-sync server.

mod client;
mod types;

pub use client::SyncClient;
pub use types::*;
