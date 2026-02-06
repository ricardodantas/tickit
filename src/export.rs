//! Export functionality for tasks

use anyhow::Result;
use chrono::Utc;
use std::io::Write;

use crate::models::{ExportFormat, List, Priority, Tag, Task};

/// Export tasks to a specific format
pub fn export_tasks<W: Write>(
    writer: &mut W,
    tasks: &[Task],
    lists: &[List],
    tags: &[Tag],
    format: ExportFormat,
) -> Result<()> {
    match format {
        ExportFormat::Json => export_json(writer, tasks, lists, tags),
        ExportFormat::TodoTxt => export_todotxt(writer, tasks, lists, tags),
        ExportFormat::Markdown => export_markdown(writer, tasks, lists, tags),
        ExportFormat::Csv => export_csv(writer, tasks, lists, tags),
    }
}

/// Export to JSON format
fn export_json<W: Write>(
    writer: &mut W,
    tasks: &[Task],
    lists: &[List],
    tags: &[Tag],
) -> Result<()> {
    let export = serde_json::json!({
        "exported_at": Utc::now().to_rfc3339(),
        "lists": lists,
        "tags": tags,
        "tasks": tasks,
    });

    serde_json::to_writer_pretty(writer, &export)?;
    Ok(())
}

/// Export to todo.txt format
/// Format: (A) 2024-01-01 Task title +project @context due:2024-01-15
fn export_todotxt<W: Write>(
    writer: &mut W,
    tasks: &[Task],
    lists: &[List],
    tags: &[Tag],
) -> Result<()> {
    for task in tasks {
        let mut line = String::new();

        // Completion status
        if task.completed {
            line.push_str("x ");
            if let Some(completed_at) = task.completed_at {
                line.push_str(&completed_at.format("%Y-%m-%d ").to_string());
            }
        }

        // Priority (only for incomplete tasks in todo.txt)
        if !task.completed {
            let priority_char = match task.priority {
                Priority::Urgent => 'A',
                Priority::High => 'B',
                Priority::Medium => 'C',
                Priority::Low => 'D',
            };
            line.push_str(&format!("({}) ", priority_char));
        }

        // Creation date
        line.push_str(&task.created_at.format("%Y-%m-%d ").to_string());

        // Title
        line.push_str(&task.title);

        // Project (list)
        if let Some(list) = lists.iter().find(|l| l.id == task.list_id) {
            let project_name = list.name.replace(' ', "_");
            line.push_str(&format!(" +{}", project_name));
        }

        // Contexts (tags)
        for tag_id in &task.tag_ids {
            if let Some(tag) = tags.iter().find(|t| t.id == *tag_id) {
                let tag_name = tag.name.replace(' ', "_");
                line.push_str(&format!(" @{}", tag_name));
            }
        }

        // Due date
        if let Some(due) = task.due_date {
            line.push_str(&format!(" due:{}", due.format("%Y-%m-%d")));
        }

        // URL
        if let Some(url) = &task.url {
            line.push_str(&format!(" url:{}", url));
        }

        writeln!(writer, "{}", line)?;
    }

    Ok(())
}

/// Export to Markdown format
fn export_markdown<W: Write>(
    writer: &mut W,
    tasks: &[Task],
    lists: &[List],
    tags: &[Tag],
) -> Result<()> {
    writeln!(writer, "# Tasks")?;
    writeln!(writer)?;
    writeln!(
        writer,
        "Exported: {}",
        Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    )?;
    writeln!(writer)?;

    // Group tasks by list
    for list in lists {
        let list_tasks: Vec<_> = tasks.iter().filter(|t| t.list_id == list.id).collect();
        if list_tasks.is_empty() {
            continue;
        }

        writeln!(writer, "## {} {}", list.icon, list.name)?;
        writeln!(writer)?;

        for task in list_tasks {
            let checkbox = if task.completed { "[x]" } else { "[ ]" };
            let priority_marker = match task.priority {
                Priority::Urgent => "🔴 ",
                Priority::High => "🟠 ",
                Priority::Medium => "",
                Priority::Low => "⚪ ",
            };

            write!(writer, "- {} {}{}", checkbox, priority_marker, task.title)?;

            // Tags inline
            if !task.tag_ids.is_empty() {
                let tag_names: Vec<_> = task
                    .tag_ids
                    .iter()
                    .filter_map(|id| tags.iter().find(|t| t.id == *id))
                    .map(|t| format!("`{}`", t.name))
                    .collect();
                if !tag_names.is_empty() {
                    write!(writer, " {}", tag_names.join(" "))?;
                }
            }

            writeln!(writer)?;

            // Description as sub-item
            if let Some(desc) = &task.description {
                writeln!(writer, "  - {}", desc)?;
            }

            // URL as sub-item
            if let Some(url) = &task.url {
                writeln!(writer, "  - 🔗 {}", url)?;
            }

            // Due date
            if let Some(due) = task.due_date {
                writeln!(writer, "  - 📅 Due: {}", due.format("%Y-%m-%d"))?;
            }
        }

        writeln!(writer)?;
    }

    Ok(())
}

/// Export to CSV format
fn export_csv<W: Write>(
    writer: &mut W,
    tasks: &[Task],
    lists: &[List],
    tags: &[Tag],
) -> Result<()> {
    // Header
    writeln!(
        writer,
        "Title,Description,URL,Priority,Completed,List,Tags,Due Date,Created At"
    )?;

    for task in tasks {
        let list_name = lists
            .iter()
            .find(|l| l.id == task.list_id)
            .map(|l| l.name.as_str())
            .unwrap_or("");

        let tag_names: Vec<_> = task
            .tag_ids
            .iter()
            .filter_map(|id| tags.iter().find(|t| t.id == *id))
            .map(|t| t.name.as_str())
            .collect();

        writeln!(
            writer,
            "{},{},{},{},{},{},{},{},{}",
            csv_escape(&task.title),
            csv_escape(task.description.as_deref().unwrap_or("")),
            csv_escape(task.url.as_deref().unwrap_or("")),
            task.priority.name(),
            task.completed,
            csv_escape(list_name),
            csv_escape(&tag_names.join("; ")),
            task.due_date
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_default(),
            task.created_at.format("%Y-%m-%d %H:%M:%S"),
        )?;
    }

    Ok(())
}

/// Escape a string for CSV
fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
