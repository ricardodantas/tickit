//! Sync data types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{List, Tag, Task};

/// A record that can be synced
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SyncRecord {
    Task(Task),
    List(List),
    Tag(Tag),
    TaskTag(TaskTagLink),
    /// Tombstone for deleted records
    Deleted {
        id: Uuid,
        record_type: RecordType,
        deleted_at: DateTime<Utc>,
    },
}

/// Link between task and tag (for junction table sync)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTagLink {
    pub task_id: Uuid,
    pub tag_id: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Type of record (for tombstones)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordType {
    Task,
    List,
    Tag,
    TaskTag,
}

/// Request to sync changes with server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRequest {
    /// Device identifier (for conflict resolution)
    pub device_id: Uuid,
    /// Timestamp of last successful sync (None = full sync)
    pub last_sync: Option<DateTime<Utc>>,
    /// Changes from this client since last sync
    pub changes: Vec<SyncRecord>,
}

/// Response from sync server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResponse {
    /// Server timestamp for this sync
    pub server_time: DateTime<Utc>,
    /// Changes from other devices to apply locally
    pub changes: Vec<SyncRecord>,
    /// IDs of records that had conflicts (server won)
    pub conflicts: Vec<Uuid>,
}

/// Sync status for UI display
#[derive(Debug, Clone, Default)]
pub struct SyncStatus {
    /// Is sync currently in progress?
    pub syncing: bool,
    /// Last successful sync time
    pub last_sync: Option<DateTime<Utc>>,
    /// Last error message (if any)
    pub last_error: Option<String>,
    /// Number of pending local changes
    pub pending_changes: usize,
}

impl SyncStatus {
    pub fn is_configured(server: &Option<String>, token: &Option<String>) -> bool {
        server.is_some() && token.is_some()
    }
}
