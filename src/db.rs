//! Database module for SQLite storage

use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use std::path::PathBuf;
use uuid::Uuid;

use crate::models::{List, Priority, Tag, Task};

/// Database connection wrapper
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open or create the database at the default location
    pub fn open() -> Result<Self> {
        let path = Self::default_path()?;
        Self::open_path(&path)
    }

    /// Open or create the database at a specific path
    pub fn open_path(path: &PathBuf) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create config directory")?;
        }

        let conn = Connection::open(path).context("Failed to open database")?;

        let db = Self { conn };
        db.init()?;

        Ok(db)
    }

    /// Get the default database path
    pub fn default_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Could not determine config directory")?
            .join("tickit");
        Ok(config_dir.join("tickit.sqlite"))
    }

    /// Initialize the database schema
    fn init(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            -- Lists table
            CREATE TABLE IF NOT EXISTS lists (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                icon TEXT NOT NULL DEFAULT '📋',
                color TEXT,
                is_inbox INTEGER NOT NULL DEFAULT 0,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            -- Tags table
            CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                color TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            -- Tasks table
            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT,
                url TEXT,
                priority TEXT NOT NULL DEFAULT 'medium',
                completed INTEGER NOT NULL DEFAULT 0,
                list_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                completed_at TEXT,
                due_date TEXT,
                FOREIGN KEY (list_id) REFERENCES lists(id) ON DELETE CASCADE
            );

            -- Task-Tag junction table
            CREATE TABLE IF NOT EXISTS task_tags (
                task_id TEXT NOT NULL,
                tag_id TEXT NOT NULL,
                PRIMARY KEY (task_id, tag_id),
                FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            );

            -- Indexes for common queries
            CREATE INDEX IF NOT EXISTS idx_tasks_list ON tasks(list_id);
            CREATE INDEX IF NOT EXISTS idx_tasks_completed ON tasks(completed);
            CREATE INDEX IF NOT EXISTS idx_tasks_priority ON tasks(priority);
            CREATE INDEX IF NOT EXISTS idx_task_tags_task ON task_tags(task_id);
            CREATE INDEX IF NOT EXISTS idx_task_tags_tag ON task_tags(tag_id);
            "#,
        )?;

        // Ensure inbox list exists
        self.ensure_inbox()?;

        Ok(())
    }

    /// Ensure the inbox list exists
    fn ensure_inbox(&self) -> Result<()> {
        let count: i32 =
            self.conn
                .query_row("SELECT COUNT(*) FROM lists WHERE is_inbox = 1", [], |row| {
                    row.get(0)
                })?;

        if count == 0 {
            let inbox = List::inbox();
            self.insert_list(&inbox)?;
        }

        Ok(())
    }

    // ==================== Lists ====================

    /// Insert a new list
    pub fn insert_list(&self, list: &List) -> Result<()> {
        self.conn.execute(
            r#"INSERT INTO lists (id, name, description, icon, color, is_inbox, sort_order, created_at, updated_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#,
            params![
                list.id.to_string(),
                list.name,
                list.description,
                list.icon,
                list.color,
                list.is_inbox as i32,
                list.sort_order,
                list.created_at.to_rfc3339(),
                list.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// Get all lists
    pub fn get_lists(&self) -> Result<Vec<List>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, icon, color, is_inbox, sort_order, created_at, updated_at 
             FROM lists ORDER BY sort_order, name"
        )?;

        let lists = stmt.query_map([], |row| {
            Ok(List {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                name: row.get(1)?,
                description: row.get(2)?,
                icon: row.get(3)?,
                color: row.get(4)?,
                is_inbox: row.get::<_, i32>(5)? != 0,
                sort_order: row.get(6)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
            })
        })?;

        lists.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Get the inbox list
    pub fn get_inbox(&self) -> Result<List> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, icon, color, is_inbox, sort_order, created_at, updated_at 
             FROM lists WHERE is_inbox = 1"
        )?;

        stmt.query_row([], |row| {
            Ok(List {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                name: row.get(1)?,
                description: row.get(2)?,
                icon: row.get(3)?,
                color: row.get(4)?,
                is_inbox: row.get::<_, i32>(5)? != 0,
                sort_order: row.get(6)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
            })
        })
        .map_err(Into::into)
    }

    /// Update a list
    pub fn update_list(&self, list: &List) -> Result<()> {
        self.conn.execute(
            r#"UPDATE lists SET name = ?2, description = ?3, icon = ?4, color = ?5, 
               sort_order = ?6, updated_at = ?7 WHERE id = ?1"#,
            params![
                list.id.to_string(),
                list.name,
                list.description,
                list.icon,
                list.color,
                list.sort_order,
                chrono::Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// Delete a list (moves tasks to inbox)
    pub fn delete_list(&self, list_id: Uuid) -> Result<()> {
        let inbox = self.get_inbox()?;

        // Move tasks to inbox
        self.conn.execute(
            "UPDATE tasks SET list_id = ?1 WHERE list_id = ?2",
            params![inbox.id.to_string(), list_id.to_string()],
        )?;

        // Delete the list
        self.conn.execute(
            "DELETE FROM lists WHERE id = ?1 AND is_inbox = 0",
            params![list_id.to_string()],
        )?;

        Ok(())
    }

    // ==================== Tags ====================

    /// Insert a new tag
    pub fn insert_tag(&self, tag: &Tag) -> Result<()> {
        self.conn.execute(
            "INSERT INTO tags (id, name, color, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![
                tag.id.to_string(),
                tag.name,
                tag.color,
                tag.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// Get all tags
    pub fn get_tags(&self) -> Result<Vec<Tag>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, color, created_at FROM tags ORDER BY name")?;

        let tags = stmt.query_map([], |row| {
            Ok(Tag {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
            })
        })?;

        tags.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Update a tag
    pub fn update_tag(&self, tag: &Tag) -> Result<()> {
        self.conn.execute(
            "UPDATE tags SET name = ?2, color = ?3 WHERE id = ?1",
            params![tag.id.to_string(), tag.name, tag.color],
        )?;
        Ok(())
    }

    /// Delete a tag
    pub fn delete_tag(&self, tag_id: Uuid) -> Result<()> {
        self.conn.execute(
            "DELETE FROM tags WHERE id = ?1",
            params![tag_id.to_string()],
        )?;
        Ok(())
    }

    // ==================== Tasks ====================

    /// Insert a new task
    pub fn insert_task(&self, task: &Task) -> Result<()> {
        self.conn.execute(
            r#"INSERT INTO tasks (id, title, description, url, priority, completed, list_id, 
               created_at, updated_at, completed_at, due_date)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"#,
            params![
                task.id.to_string(),
                task.title,
                task.description,
                task.url,
                format!("{:?}", task.priority).to_lowercase(),
                task.completed as i32,
                task.list_id.to_string(),
                task.created_at.to_rfc3339(),
                task.updated_at.to_rfc3339(),
                task.completed_at.map(|dt| dt.to_rfc3339()),
                task.due_date.map(|dt| dt.to_rfc3339()),
            ],
        )?;

        // Insert tag associations
        for tag_id in &task.tag_ids {
            self.conn.execute(
                "INSERT OR IGNORE INTO task_tags (task_id, tag_id) VALUES (?1, ?2)",
                params![task.id.to_string(), tag_id.to_string()],
            )?;
        }

        Ok(())
    }

    /// Get all tasks for a list
    pub fn get_tasks_for_list(&self, list_id: Uuid) -> Result<Vec<Task>> {
        self.get_tasks_with_filter(Some(list_id), None, None)
    }

    /// Get all tasks
    pub fn get_all_tasks(&self) -> Result<Vec<Task>> {
        self.get_tasks_with_filter(None, None, None)
    }

    /// Get tasks with optional filters
    pub fn get_tasks_with_filter(
        &self,
        list_id: Option<Uuid>,
        completed: Option<bool>,
        tag_id: Option<Uuid>,
    ) -> Result<Vec<Task>> {
        let mut sql = String::from(
            "SELECT DISTINCT t.id, t.title, t.description, t.url, t.priority, t.completed, 
             t.list_id, t.created_at, t.updated_at, t.completed_at, t.due_date
             FROM tasks t",
        );

        let mut conditions = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if tag_id.is_some() {
            sql.push_str(" JOIN task_tags tt ON t.id = tt.task_id");
        }

        if let Some(lid) = list_id {
            conditions.push("t.list_id = ?");
            params_vec.push(Box::new(lid.to_string()));
        }

        if let Some(c) = completed {
            conditions.push("t.completed = ?");
            params_vec.push(Box::new(c as i32));
        }

        if let Some(tid) = tag_id {
            conditions.push("tt.tag_id = ?");
            params_vec.push(Box::new(tid.to_string()));
        }

        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }

        sql.push_str(" ORDER BY t.completed, t.priority DESC, t.created_at DESC");

        let mut stmt = self.conn.prepare(&sql)?;

        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|p| p.as_ref()).collect();

        // First collect just the task IDs
        let task_ids: Vec<String> = stmt
            .query_map(params_refs.as_slice(), |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

        let mut result = Vec::new();
        for task_id in task_ids {
            // Get fresh row for this task
            let mut task_stmt = self.conn.prepare(
                "SELECT id, title, description, url, priority, completed, list_id, 
                 created_at, updated_at, completed_at, due_date FROM tasks WHERE id = ?1",
            )?;

            let task = task_stmt.query_row(params![task_id], |row| {
                let priority_str: String = row.get(4)?;
                let priority = match priority_str.as_str() {
                    "low" => Priority::Low,
                    "high" => Priority::High,
                    "urgent" => Priority::Urgent,
                    _ => Priority::Medium,
                };

                Ok(Task {
                    id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                    title: row.get(1)?,
                    description: row.get(2)?,
                    url: row.get(3)?,
                    priority,
                    completed: row.get::<_, i32>(5)? != 0,
                    list_id: Uuid::parse_str(&row.get::<_, String>(6)?).unwrap(),
                    tag_ids: Vec::new(), // Filled below
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                        .unwrap()
                        .with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                        .unwrap()
                        .with_timezone(&chrono::Utc),
                    completed_at: row
                        .get::<_, Option<String>>(9)?
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&chrono::Utc)),
                    due_date: row
                        .get::<_, Option<String>>(10)?
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&chrono::Utc)),
                })
            })?;

            // Get tags for this task
            let mut task = task;
            task.tag_ids = self.get_task_tags(task.id)?;
            result.push(task);
        }

        Ok(result)
    }

    /// Get tag IDs for a task
    fn get_task_tags(&self, task_id: Uuid) -> Result<Vec<Uuid>> {
        let mut stmt = self
            .conn
            .prepare("SELECT tag_id FROM task_tags WHERE task_id = ?1")?;

        let tags = stmt.query_map(params![task_id.to_string()], |row| {
            Ok(Uuid::parse_str(&row.get::<_, String>(0)?).unwrap())
        })?;

        tags.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Update a task
    pub fn update_task(&self, task: &Task) -> Result<()> {
        self.conn.execute(
            r#"UPDATE tasks SET title = ?2, description = ?3, url = ?4, priority = ?5, 
               completed = ?6, list_id = ?7, updated_at = ?8, completed_at = ?9, due_date = ?10 
               WHERE id = ?1"#,
            params![
                task.id.to_string(),
                task.title,
                task.description,
                task.url,
                format!("{:?}", task.priority).to_lowercase(),
                task.completed as i32,
                task.list_id.to_string(),
                chrono::Utc::now().to_rfc3339(),
                task.completed_at.map(|dt| dt.to_rfc3339()),
                task.due_date.map(|dt| dt.to_rfc3339()),
            ],
        )?;

        // Update tag associations
        self.conn.execute(
            "DELETE FROM task_tags WHERE task_id = ?1",
            params![task.id.to_string()],
        )?;

        for tag_id in &task.tag_ids {
            self.conn.execute(
                "INSERT INTO task_tags (task_id, tag_id) VALUES (?1, ?2)",
                params![task.id.to_string(), tag_id.to_string()],
            )?;
        }

        Ok(())
    }

    /// Delete a task
    pub fn delete_task(&self, task_id: Uuid) -> Result<()> {
        self.conn.execute(
            "DELETE FROM tasks WHERE id = ?1",
            params![task_id.to_string()],
        )?;
        Ok(())
    }

    /// Get task count for a list
    pub fn get_task_count(&self, list_id: Uuid, include_completed: bool) -> Result<i32> {
        let sql = if include_completed {
            "SELECT COUNT(*) FROM tasks WHERE list_id = ?1"
        } else {
            "SELECT COUNT(*) FROM tasks WHERE list_id = ?1 AND completed = 0"
        };

        self.conn
            .query_row(sql, params![list_id.to_string()], |row| row.get(0))
            .map_err(Into::into)
    }

    /// Get total task count
    pub fn get_total_task_count(&self, include_completed: bool) -> Result<i32> {
        let sql = if include_completed {
            "SELECT COUNT(*) FROM tasks"
        } else {
            "SELECT COUNT(*) FROM tasks WHERE completed = 0"
        };

        self.conn
            .query_row(sql, [], |row| row.get(0))
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_database_init() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.sqlite");
        let db = Database::open_path(&path).unwrap();

        // Should have inbox list
        let lists = db.get_lists().unwrap();
        assert_eq!(lists.len(), 1);
        assert!(lists[0].is_inbox);
    }

    #[test]
    fn test_task_crud() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.sqlite");
        let db = Database::open_path(&path).unwrap();

        let inbox = db.get_inbox().unwrap();

        // Create task
        let task = Task::new("Test task", inbox.id);
        db.insert_task(&task).unwrap();

        // Read tasks
        let tasks = db.get_tasks_for_list(inbox.id).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "Test task");

        // Update task
        let mut updated = tasks[0].clone();
        updated.title = "Updated task".to_string();
        db.update_task(&updated).unwrap();

        let tasks = db.get_tasks_for_list(inbox.id).unwrap();
        assert_eq!(tasks[0].title, "Updated task");

        // Delete task
        db.delete_task(tasks[0].id).unwrap();
        let tasks = db.get_tasks_for_list(inbox.id).unwrap();
        assert!(tasks.is_empty());
    }
}
