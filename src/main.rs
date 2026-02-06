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
    }

    Ok(())
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
