//! Tickit CLI Application
//!
//! Terminal-based task manager with beautiful TUI and CLI modes.

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use tickit::{Database, ExportFormat, List, Priority, Tag, Task};

#[derive(Parser, Debug)]
#[command(name = "tickit")]
#[command(author, version, about = "A stunning terminal-based task manager")]
struct Cli {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start the TUI (default)
    Ui,

    /// Add a new task
    Add {
        /// Task title
        title: String,

        /// Task description
        #[arg(short, long)]
        description: Option<String>,

        /// URL to attach
        #[arg(short, long)]
        url: Option<String>,

        /// Priority (low, medium, high, urgent)
        #[arg(short, long, default_value = "medium")]
        priority: String,

        /// List name to add task to
        #[arg(short, long)]
        list: Option<String>,

        /// Tags to attach (comma-separated)
        #[arg(short, long)]
        tags: Option<String>,

        /// Due date (YYYY-MM-DD format)
        #[arg(long)]
        due: Option<String>,
    },

    /// List tasks
    #[command(alias = "ls")]
    List {
        /// Filter by list name
        #[arg(short, long)]
        list: Option<String>,

        /// Show completed tasks
        #[arg(short, long)]
        all: bool,

        /// Filter by tag
        #[arg(short, long)]
        tag: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Mark task as complete
    Done {
        /// Task ID or title (partial match)
        task: String,
    },

    /// Mark task as not complete
    Undo {
        /// Task ID or title (partial match)
        task: String,
    },

    /// Delete a task
    #[command(alias = "rm")]
    Delete {
        /// Task ID or title (partial match)
        task: String,

        /// Skip confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// Manage lists
    Lists {
        #[command(subcommand)]
        command: Option<ListCommands>,
    },

    /// Manage tags
    Tags {
        #[command(subcommand)]
        command: Option<TagCommands>,
    },

    /// Export tasks
    Export {
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Format (json, todotxt, markdown, csv)
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Filter by list
        #[arg(short, long)]
        list: Option<String>,
    },

    /// Check for updates and install if available
    Update,

    /// Manually trigger a sync with the server
    Sync {
        /// Show sync status instead of syncing
        #[arg(long)]
        status: bool,

        /// Force full sync (ignore last_sync timestamp)
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand, Debug)]
enum ListCommands {
    /// List all lists
    #[command(alias = "ls")]
    List,

    /// Add a new list
    Add {
        /// List name
        name: String,

        /// Icon/emoji
        #[arg(short, long, default_value = "📋")]
        icon: String,
    },

    /// Delete a list
    #[command(alias = "rm")]
    Delete {
        /// List name
        name: String,
    },
}

#[derive(Subcommand, Debug)]
enum TagCommands {
    /// List all tags
    #[command(alias = "ls")]
    List,

    /// Add a new tag
    Add {
        /// Tag name
        name: String,

        /// Color (hex)
        #[arg(short, long)]
        color: Option<String>,
    },

    /// Delete a tag
    #[command(alias = "rm")]
    Delete {
        /// Tag name
        name: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_env_filter("tickit=debug")
            .init();
    }

    match cli.command {
        None | Some(Commands::Ui) => {
            // Start TUI
            tickit::app::run()?;
        }

        Some(Commands::Add {
            title,
            description,
            url,
            priority,
            list,
            tags,
            due,
        }) => {
            let db = Database::open()?;

            // Find list
            let list_id = if let Some(list_name) = list {
                let lists = db.get_lists()?;
                lists
                    .iter()
                    .find(|l| l.name.to_lowercase() == list_name.to_lowercase())
                    .map(|l| l.id)
                    .unwrap_or_else(|| db.get_inbox().unwrap().id)
            } else {
                db.get_inbox()?.id
            };

            // Parse priority
            let priority = match priority.to_lowercase().as_str() {
                "low" | "l" => Priority::Low,
                "high" | "h" => Priority::High,
                "urgent" | "u" => Priority::Urgent,
                _ => Priority::Medium,
            };

            // Parse due date
            let due_date = due.and_then(|s| {
                chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d")
                    .ok()
                    .map(|date| date.and_hms_opt(23, 59, 59).unwrap().and_utc())
            });

            // Create task
            let mut task = Task::new(&title, list_id);
            task.priority = priority;
            task.description = description;
            task.url = url;
            task.due_date = due_date;

            // Add tags
            if let Some(tag_str) = tags {
                let db_tags = db.get_tags()?;
                for tag_name in tag_str.split(',').map(|s| s.trim()) {
                    if let Some(tag) = db_tags
                        .iter()
                        .find(|t| t.name.to_lowercase() == tag_name.to_lowercase())
                    {
                        task.tag_ids.push(tag.id);
                    }
                }
            }

            db.insert_task(&task)?;
            println!("✓ Added: {}", title);
        }

        Some(Commands::List {
            list,
            all,
            tag,
            json,
        }) => {
            let db = Database::open()?;
            let lists = db.get_lists()?;
            let tags = db.get_tags()?;

            // Find list filter
            let list_id = list.and_then(|name| {
                lists
                    .iter()
                    .find(|l| l.name.to_lowercase() == name.to_lowercase())
                    .map(|l| l.id)
            });

            // Find tag filter
            let tag_id = tag.and_then(|name| {
                tags.iter()
                    .find(|t| t.name.to_lowercase() == name.to_lowercase())
                    .map(|t| t.id)
            });

            let completed = if all { None } else { Some(false) };
            let tasks = db.get_tasks_with_filter(list_id, completed, tag_id)?;

            if json {
                let output = serde_json::to_string_pretty(&tasks)?;
                println!("{}", output);
            } else if tasks.is_empty() {
                println!("No tasks found.");
            } else {
                for task in tasks {
                    let checkbox = if task.completed { "☑" } else { "☐" };
                    let priority = task.priority.icon();
                    let list_name = lists
                        .iter()
                        .find(|l| l.id == task.list_id)
                        .map(|l| l.name.as_str())
                        .unwrap_or("?");

                    println!("{} {} {} [{}]", checkbox, priority, task.title, list_name);
                }
            }
        }

        Some(Commands::Done { task }) => {
            let db = Database::open()?;
            let tasks = db.get_all_tasks()?;

            if let Some(mut t) = find_task(&tasks, &task) {
                t.complete();
                db.update_task(&t)?;
                println!("✓ Completed: {}", t.title);
            } else {
                println!("Task not found: {}", task);
            }
        }

        Some(Commands::Undo { task }) => {
            let db = Database::open()?;
            let tasks = db.get_all_tasks()?;

            if let Some(mut t) = find_task(&tasks, &task) {
                t.uncomplete();
                db.update_task(&t)?;
                println!("↺ Reopened: {}", t.title);
            } else {
                println!("Task not found: {}", task);
            }
        }

        Some(Commands::Delete { task, force }) => {
            let db = Database::open()?;
            let tasks = db.get_all_tasks()?;

            if let Some(t) = find_task(&tasks, &task) {
                if !force {
                    print!("Delete \"{}\"? [y/N] ", t.title);
                    use std::io::{self, Write};
                    io::stdout().flush()?;
                    let mut input = String::new();
                    io::stdin().read_line(&mut input)?;
                    if !input.trim().eq_ignore_ascii_case("y") {
                        println!("Cancelled.");
                        return Ok(());
                    }
                }
                db.delete_task(t.id)?;
                println!("✗ Deleted: {}", t.title);
            } else {
                println!("Task not found: {}", task);
            }
        }

        Some(Commands::Lists { command }) => {
            let db = Database::open()?;

            match command {
                None | Some(ListCommands::List) => {
                    let lists = db.get_lists()?;
                    for list in lists {
                        let inbox = if list.is_inbox { " (default)" } else { "" };
                        let count = db.get_task_count(list.id, false)?;
                        println!("{} {} ({} tasks){}", list.icon, list.name, count, inbox);
                    }
                }
                Some(ListCommands::Add { name, icon }) => {
                    let list = List::new(&name).with_icon(&icon);
                    db.insert_list(&list)?;
                    println!("✓ Created list: {} {}", icon, name);
                }
                Some(ListCommands::Delete { name }) => {
                    let lists = db.get_lists()?;
                    if let Some(list) = lists
                        .iter()
                        .find(|l| l.name.to_lowercase() == name.to_lowercase())
                    {
                        if list.is_inbox {
                            println!("Cannot delete inbox.");
                        } else {
                            db.delete_list(list.id)?;
                            println!("✗ Deleted list: {}", name);
                        }
                    } else {
                        println!("List not found: {}", name);
                    }
                }
            }
        }

        Some(Commands::Tags { command }) => {
            let db = Database::open()?;

            match command {
                None | Some(TagCommands::List) => {
                    let tags = db.get_tags()?;
                    if tags.is_empty() {
                        println!("No tags yet.");
                    } else {
                        for tag in tags {
                            println!("● {} ({})", tag.name, tag.color);
                        }
                    }
                }
                Some(TagCommands::Add { name, color }) => {
                    let mut tag = Tag::new(&name);
                    if let Some(c) = color {
                        tag = tag.with_color(&c);
                    }
                    db.insert_tag(&tag)?;
                    println!("✓ Created tag: {}", name);
                }
                Some(TagCommands::Delete { name }) => {
                    let tags = db.get_tags()?;
                    if let Some(tag) = tags
                        .iter()
                        .find(|t| t.name.to_lowercase() == name.to_lowercase())
                    {
                        db.delete_tag(tag.id)?;
                        println!("✗ Deleted tag: {}", name);
                    } else {
                        println!("Tag not found: {}", name);
                    }
                }
            }
        }

        Some(Commands::Export {
            output,
            format,
            list,
        }) => {
            let db = Database::open()?;
            let lists = db.get_lists()?;
            let tags = db.get_tags()?;

            // Filter by list
            let list_id = list.and_then(|name| {
                lists
                    .iter()
                    .find(|l| l.name.to_lowercase() == name.to_lowercase())
                    .map(|l| l.id)
            });

            let tasks = if let Some(lid) = list_id {
                db.get_tasks_for_list(lid)?
            } else {
                db.get_all_tasks()?
            };

            // Parse format
            let fmt = match format.to_lowercase().as_str() {
                "todotxt" | "todo.txt" | "txt" => ExportFormat::TodoTxt,
                "markdown" | "md" => ExportFormat::Markdown,
                "csv" => ExportFormat::Csv,
                _ => ExportFormat::Json,
            };

            // Export
            if let Some(path) = output {
                let mut file = std::fs::File::create(&path)?;
                tickit::export::export_tasks(&mut file, &tasks, &lists, &tags, fmt)?;
                println!("Exported {} tasks to {}", tasks.len(), path.display());
            } else {
                let mut stdout = std::io::stdout();
                tickit::export::export_tasks(&mut stdout, &tasks, &lists, &tags, fmt)?;
            }
        }

        Some(Commands::Update) => {
            run_update_command();
        }

        Some(Commands::Sync { status, force }) => {
            run_sync_command(status, force)?;
        }
    }

    Ok(())
}

/// Run the sync command
fn run_sync_command(status_only: bool, force: bool) -> Result<()> {
    use tickit::{
        Config, Database,
        sync::{SyncClient, SyncRecord},
    };

    let config = Config::load()?;
    let db = Database::open()?;

    if !config.sync.enabled {
        println!("⚠ Sync is disabled in config.");
        println!("\nTo enable sync, add to ~/.config/tickit/config.toml:");
        println!();
        println!("  [sync]");
        println!("  enabled = true");
        println!("  server = \"http://your-server:3030\"");
        println!("  token = \"your-token\"");
        return Ok(());
    }

    if config.sync.server.is_none() || config.sync.token.is_none() {
        println!("⚠ Sync is enabled but not configured.");
        println!("\nMissing server and/or token in config.");
        return Ok(());
    }

    let mut client = SyncClient::new(config.sync.clone());

    if status_only {
        let last_sync = db.get_last_sync()?;
        println!("Sync Status:");
        println!(
            "  Server: {}",
            config.sync.server.as_deref().unwrap_or("not set")
        );
        println!("  Enabled: {}", config.sync.enabled);
        println!(
            "  Last sync: {}",
            last_sync
                .map(|t| t.to_string())
                .unwrap_or_else(|| "never".to_string())
        );
        return Ok(());
    }

    if force {
        println!("⟳ Force syncing (ignoring last_sync)...");
    } else {
        println!("⟳ Syncing...");
    }

    // Gather local changes - use None for force sync to get everything
    let last_sync = if force { None } else { db.get_last_sync()? };
    let mut changes: Vec<SyncRecord> = Vec::new();

    // Get all data for full sync, or changes since last sync
    let tasks = if let Some(since) = last_sync {
        db.get_tasks_since(since)?
    } else {
        db.get_all_tasks()?
    };
    for task in tasks {
        changes.push(SyncRecord::Task(task));
    }

    let lists = if let Some(since) = last_sync {
        db.get_lists_since(since)?
    } else {
        db.get_lists()?
    };
    for list in lists {
        changes.push(SyncRecord::List(list));
    }

    let tags = if let Some(since) = last_sync {
        db.get_tags_since(since)?
    } else {
        db.get_tags()?
    };
    for tag in tags {
        changes.push(SyncRecord::Tag(tag));
    }

    // Get tombstones
    if let Some(since) = last_sync {
        let tombstones = db.get_tombstones_since(since)?;
        for tomb in tombstones {
            let record_type = match tomb.1.as_str() {
                "task" => tickit::sync::RecordType::Task,
                "list" => tickit::sync::RecordType::List,
                "tag" => tickit::sync::RecordType::Tag,
                "task_tag" => tickit::sync::RecordType::TaskTag,
                _ => continue,
            };
            changes.push(SyncRecord::Deleted {
                id: tomb.0,
                record_type,
                deleted_at: tomb.2,
            });
        }
    }

    println!("  Uploading {} changes...", changes.len());

    // Sync - pass None for force sync to get all changes from server
    match client.sync(changes, if force { None } else { db.get_last_sync()? }) {
        Ok(response) => {
            println!("  Received {} changes from server", response.changes.len());

            // Sort changes: lists first, then tags, then tasks (to satisfy FK constraints)
            let mut lists = Vec::new();
            let mut tags = Vec::new();
            let mut tasks = Vec::new();
            let mut deletes = Vec::new();

            for record in response.changes {
                match &record {
                    SyncRecord::List(_) => lists.push(record),
                    SyncRecord::Tag(_) => tags.push(record),
                    SyncRecord::Task(_) => tasks.push(record),
                    SyncRecord::Deleted { .. } => deletes.push(record),
                    _ => {}
                }
            }

            // Disable FK constraints during sync
            let _ = db.execute_raw("PRAGMA foreign_keys = OFF");

            // Apply incoming changes in order
            let mut applied = 0;
            for record in lists.into_iter().chain(tags).chain(tasks).chain(deletes) {
                let result = match record {
                    SyncRecord::Task(task) => db.upsert_task(&task),
                    SyncRecord::List(list) => db.upsert_list(&list),
                    SyncRecord::Tag(tag) => db.upsert_tag(&tag),
                    SyncRecord::Deleted {
                        id, record_type, ..
                    } => {
                        match record_type {
                            tickit::sync::RecordType::Task => {
                                let _ = db.delete_task(id);
                            }
                            tickit::sync::RecordType::List => {
                                let _ = db.delete_list(id);
                            }
                            tickit::sync::RecordType::Tag => {
                                let _ = db.delete_tag(id);
                            }
                            _ => {}
                        }
                        Ok(())
                    }
                    _ => Ok(()),
                };
                if result.is_ok() {
                    applied += 1;
                }
            }

            // Re-enable FK constraints
            let _ = db.execute_raw("PRAGMA foreign_keys = ON");

            // Update last sync time
            db.set_last_sync(response.server_time)?;

            if !response.conflicts.is_empty() {
                println!("  ⚠ {} conflicts (server won)", response.conflicts.len());
            }

            println!("✓ Sync complete! Applied {} changes.", applied);
        }
        Err(e) => {
            println!("✗ Sync failed: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Run the update command
fn run_update_command() {
    use tickit::{
        VERSION, VersionCheck, check_for_updates_crates_io, detect_package_manager, run_update,
    };

    println!("✓ Checking for updates...\n");

    let pm = detect_package_manager();
    println!("  Installed via: {}", pm.name());
    println!("  Current version: {}", VERSION);

    // Use crates.io API (no rate limits, more reliable)
    let check = check_for_updates_crates_io();

    match check {
        VersionCheck::UpdateAvailable { latest, .. } => {
            println!("  Latest version: {}", latest);
            println!("\n⬆ Update available! Installing...\n");

            match run_update(&pm) {
                Ok(()) => {
                    println!("✓ Successfully updated to {}!", latest);
                    println!("\nRestart tickit to use the new version.");
                }
                Err(e) => {
                    println!("✗ Update failed: {}", e);
                    println!("\nYou can manually update with:");
                    println!("  {}", pm.update_command());
                    std::process::exit(1);
                }
            }
        }
        VersionCheck::UpToDate => {
            println!("\n✓ Already on the latest version!");
        }
        VersionCheck::CheckFailed(msg) => {
            println!("\n⚠ Could not check for updates: {}", msg);
            std::process::exit(1);
        }
    }
}

/// Find a task by ID or partial title match
fn find_task(tasks: &[Task], query: &str) -> Option<Task> {
    // Try UUID first
    if let Ok(uuid) = uuid::Uuid::parse_str(query) {
        return tasks.iter().find(|t| t.id == uuid).cloned();
    }

    // Try partial title match
    let query_lower = query.to_lowercase();
    tasks
        .iter()
        .find(|t| t.title.to_lowercase().contains(&query_lower))
        .cloned()
}
