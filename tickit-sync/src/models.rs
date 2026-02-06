//! Sync data models (shared types between client and server)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Priority level for tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    #[default]
    Medium,
    High,
    Urgent,
}

/// A task/todo item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub url: Option<String>,
    pub priority: Priority,
    pub completed: bool,
    pub list_id: Uuid,
    pub tag_ids: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
}

/// A list/project that contains tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct List {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub icon: String,
    pub color: Option<String>,
    pub is_inbox: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub sort_order: i32,
}

/// A tag that can be attached to tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub color: String,
    pub created_at: DateTime<Utc>,
}

/// Link between task and tag
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

/// A record that can be synced
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SyncRecord {
    Task(Task),
    List(List),
    Tag(Tag),
    TaskTag(TaskTagLink),
    Deleted {
        id: Uuid,
        record_type: RecordType,
        deleted_at: DateTime<Utc>,
    },
}

/// Request to sync changes with server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRequest {
    /// Device identifier
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
