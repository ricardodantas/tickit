//! Data models for Tickit

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Priority level for tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    /// Low priority
    Low,
    /// Normal/Medium priority (default)
    #[default]
    Medium,
    /// High priority
    High,
    /// Urgent priority
    Urgent,
}

impl Priority {
    /// Get all priority levels
    pub const fn all() -> &'static [Self] {
        &[Self::Low, Self::Medium, Self::High, Self::Urgent]
    }

    /// Get the display name
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Urgent => "Urgent",
        }
    }

    /// Get the icon for this priority
    pub const fn icon(&self) -> &'static str {
        match self {
            Self::Low => "○",
            Self::Medium => "◐",
            Self::High => "●",
            Self::Urgent => "◉",
        }
    }

    /// Get next priority (cycles)
    pub fn next(&self) -> Self {
        match self {
            Self::Low => Self::Medium,
            Self::Medium => Self::High,
            Self::High => Self::Urgent,
            Self::Urgent => Self::Low,
        }
    }

    /// Get previous priority (cycles)
    pub fn prev(&self) -> Self {
        match self {
            Self::Low => Self::Urgent,
            Self::Medium => Self::Low,
            Self::High => Self::Medium,
            Self::Urgent => Self::High,
        }
    }
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// A task/todo item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique identifier
    pub id: Uuid,
    /// Task title
    pub title: String,
    /// Optional description
    pub description: Option<String>,
    /// Optional URL (can be opened in browser)
    pub url: Option<String>,
    /// Priority level
    pub priority: Priority,
    /// Whether the task is completed
    pub completed: bool,
    /// ID of the list this task belongs to
    pub list_id: Uuid,
    /// IDs of tags attached to this task
    pub tag_ids: Vec<Uuid>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Completion timestamp (if completed)
    pub completed_at: Option<DateTime<Utc>>,
    /// Optional due date
    pub due_date: Option<DateTime<Utc>>,
}

impl Task {
    /// Create a new task with the given title
    pub fn new(title: impl Into<String>, list_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            description: None,
            url: None,
            priority: Priority::default(),
            completed: false,
            list_id,
            tag_ids: Vec::new(),
            created_at: now,
            updated_at: now,
            completed_at: None,
            due_date: None,
        }
    }

    /// Mark the task as completed
    pub fn complete(&mut self) {
        self.completed = true;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Mark the task as not completed
    pub fn uncomplete(&mut self) {
        self.completed = false;
        self.completed_at = None;
        self.updated_at = Utc::now();
    }

    /// Toggle completion status
    pub fn toggle(&mut self) {
        if self.completed {
            self.uncomplete();
        } else {
            self.complete();
        }
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the URL
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Set the priority
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, tag_id: Uuid) -> Self {
        if !self.tag_ids.contains(&tag_id) {
            self.tag_ids.push(tag_id);
        }
        self
    }

    /// Set the due date
    pub fn with_due_date(mut self, due_date: DateTime<Utc>) -> Self {
        self.due_date = Some(due_date);
        self
    }
}

/// A list/project that contains tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct List {
    /// Unique identifier
    pub id: Uuid,
    /// List name
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// Icon/emoji for the list
    pub icon: String,
    /// Color for the list (hex code or name)
    pub color: Option<String>,
    /// Whether this is the default inbox list
    pub is_inbox: bool,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Sort order
    pub sort_order: i32,
}

impl List {
    /// Create a new list with the given name
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: None,
            icon: "📋".to_string(),
            color: None,
            is_inbox: false,
            created_at: now,
            updated_at: now,
            sort_order: 0,
        }
    }

    /// Create the default Inbox list
    pub fn inbox() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: "Inbox".to_string(),
            description: Some("Default list for new tasks".to_string()),
            icon: "📥".to_string(),
            color: None,
            is_inbox: true,
            created_at: now,
            updated_at: now,
            sort_order: -1, // Always first
        }
    }

    /// Set the icon
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = icon.into();
        self
    }

    /// Set the color
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// A tag that can be attached to tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    /// Unique identifier
    pub id: Uuid,
    /// Tag name
    pub name: String,
    /// Color for the tag (hex code)
    pub color: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

impl Tag {
    /// Create a new tag with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            color: Self::random_color(),
            created_at: Utc::now(),
        }
    }

    /// Set the color
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = color.into();
        self
    }

    /// Generate a random pleasant color
    fn random_color() -> String {
        const COLORS: &[&str] = &[
            "#f38ba8", // Red
            "#fab387", // Peach
            "#f9e2af", // Yellow
            "#a6e3a1", // Green
            "#94e2d5", // Teal
            "#89b4fa", // Blue
            "#cba6f7", // Mauve
            "#f5c2e7", // Pink
            "#eba0ac", // Maroon
            "#89dceb", // Sky
        ];
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as usize;
        COLORS[seed % COLORS.len()].to_string()
    }
}

/// Export format for tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    /// JSON format
    Json,
    /// todo.txt format
    TodoTxt,
    /// Markdown format
    Markdown,
    /// CSV format
    Csv,
}

impl ExportFormat {
    /// Get all export formats
    pub const fn all() -> &'static [Self] {
        &[Self::Json, Self::TodoTxt, Self::Markdown, Self::Csv]
    }

    /// Get the display name
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Json => "JSON",
            Self::TodoTxt => "todo.txt",
            Self::Markdown => "Markdown",
            Self::Csv => "CSV",
        }
    }

    /// Get the file extension
    pub const fn extension(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::TodoTxt => "txt",
            Self::Markdown => "md",
            Self::Csv => "csv",
        }
    }
}
