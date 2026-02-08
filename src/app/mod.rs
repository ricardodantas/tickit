//! TUI Application module

mod events;
mod state;
mod ui;

pub use state::AppState;

use anyhow::Result;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::*;
use std::io::stdout;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::config::Config;
use crate::db::Database;
use crate::notifications;
use crate::sync::{RecordType, SyncClient, SyncRecord, SyncResponse};

/// Messages from background tasks
enum BackgroundMsg {
    UpdateAvailable(String),
    SyncComplete(Result<SyncResponse, String>),
}

/// Run the TUI application
pub fn run() -> Result<()> {
    // Load config
    let config = Config::load()?;

    // Open database
    let db = Database::open()?;

    // Initialize terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // Create app state
    let mut state = AppState::new(config.clone(), db)?;

    // Spawn background update check
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let check = crate::check_for_updates_crates_io_timeout(std::time::Duration::from_secs(5));
        if let crate::VersionCheck::UpdateAvailable { latest, .. } = check {
            let _ = tx.send(BackgroundMsg::UpdateAvailable(latest));
        }
    });

    // Check for due tasks and send notifications (in background)
    if config.notifications {
        let db_for_notifications = Database::open().ok();
        std::thread::spawn(move || {
            if let Some(db) = db_for_notifications {
                let _ = check_and_notify_due_tasks(&db);
            }
        });
    }

    // Main loop
    let result = run_app(&mut terminal, &mut state, rx);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    state: &mut AppState,
    bg_rx: mpsc::Receiver<BackgroundMsg>,
) -> Result<()> {
    // Track if sync is in progress (to prevent multiple syncs)
    let mut sync_in_progress = false;
    // Channel for sync results
    let (sync_tx, sync_rx) = mpsc::channel::<BackgroundMsg>();
    // Track last sync time for auto-sync interval
    let mut last_sync_attempt = Instant::now();
    // Initial sync on startup if enabled
    let mut needs_initial_sync = state.is_sync_enabled();

    loop {
        // Check for background messages (non-blocking)
        if let Ok(msg) = bg_rx.try_recv() {
            match msg {
                BackgroundMsg::UpdateAvailable(version) => {
                    state.set_update_available(version);
                }
                BackgroundMsg::SyncComplete(_) => {
                    // Handled by sync_rx
                }
            }
        }

        // Check for sync completion
        if let Ok(msg) = sync_rx.try_recv()
            && let BackgroundMsg::SyncComplete(result) = msg
        {
            sync_in_progress = false;
            match result {
                Ok(response) => {
                    // Apply incoming changes from server
                    let applied = apply_incoming_changes(&state.db, &response);

                    // Update last sync time in DB
                    let _ = state.db.set_last_sync(response.server_time);
                    state.set_last_sync(response.server_time);

                    if applied > 0 {
                        state.set_status(format!("Synced ({} changes applied)", applied));
                    } else {
                        state.set_status("Synced");
                    }

                    // Refresh data after sync
                    let _ = state.refresh_data();
                }
                Err(e) => {
                    state.set_sync_error(Some(e.clone()));
                    state.set_status(format!("Sync failed: {}", e));
                }
            }
        }

        // Draw UI
        terminal.draw(|frame| ui::render(frame, state))?;

        // Process pending update after draw (so "Updating..." is visible)
        if state.pending_update {
            events::process_pending_update(state);
            // Redraw immediately after update completes
            terminal.draw(|frame| ui::render(frame, state))?;
        }

        // Auto-sync on interval (if enabled and configured)
        let sync_interval = state.config.sync.interval_secs;
        let should_auto_sync = state.is_sync_enabled()
            && sync_interval > 0
            && !sync_in_progress
            && (needs_initial_sync || last_sync_attempt.elapsed().as_secs() >= sync_interval);

        // Check if sync was requested (via Ctrl+S) or triggered by action or auto-sync
        let should_sync = (state.sync_status.syncing || state.sync_pending || should_auto_sync)
            && !sync_in_progress
            && state.is_sync_enabled();

        if should_sync {
            sync_in_progress = true;
            needs_initial_sync = false;
            last_sync_attempt = Instant::now();
            state.set_syncing(true);
            state.sync_pending = false;

            let config = state.config.sync.clone();
            let last_sync = state.db.get_last_sync().ok().flatten();

            // Gather local changes
            let changes = gather_local_changes(&state.db, last_sync);

            let tx = sync_tx.clone();
            std::thread::spawn(move || {
                let mut client = SyncClient::new(config);
                let result = client.sync(changes, last_sync);
                let msg = match result {
                    Ok(response) => BackgroundMsg::SyncComplete(Ok(response)),
                    Err(e) => BackgroundMsg::SyncComplete(Err(e.to_string())),
                };
                let _ = tx.send(msg);
            });
        }

        // Handle events
        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            events::handle_key(state, key);
        }

        // Tick for animations
        state.tick();

        if state.should_quit {
            break;
        }
    }

    // Save config on exit
    state.config.save()?;

    Ok(())
}

/// Gather local changes since last sync
fn gather_local_changes(
    db: &Database,
    last_sync: Option<chrono::DateTime<chrono::Utc>>,
) -> Vec<SyncRecord> {
    let mut changes = Vec::new();

    // Get modified tasks
    if let Some(since) = last_sync {
        if let Ok(tasks) = db.get_tasks_since(since) {
            for task in tasks {
                changes.push(SyncRecord::Task(task));
            }
        }
        if let Ok(lists) = db.get_lists_since(since) {
            for list in lists {
                changes.push(SyncRecord::List(list));
            }
        }
        if let Ok(tags) = db.get_tags_since(since) {
            for tag in tags {
                changes.push(SyncRecord::Tag(tag));
            }
        }
        // Get tombstones
        if let Ok(tombstones) = db.get_tombstones_since(since) {
            for (id, record_type_str, deleted_at) in tombstones {
                let record_type = match record_type_str.as_str() {
                    "task" => RecordType::Task,
                    "list" => RecordType::List,
                    "tag" => RecordType::Tag,
                    _ => RecordType::Task,
                };
                changes.push(SyncRecord::Deleted {
                    id,
                    record_type,
                    deleted_at,
                });
            }
        }
    } else {
        // Full sync: get all data
        if let Ok(tasks) = db.get_all_tasks() {
            for task in tasks {
                changes.push(SyncRecord::Task(task));
            }
        }
        if let Ok(lists) = db.get_lists() {
            for list in lists {
                changes.push(SyncRecord::List(list));
            }
        }
        if let Ok(tags) = db.get_tags() {
            for tag in tags {
                changes.push(SyncRecord::Tag(tag));
            }
        }
    }

    changes
}

/// Apply incoming changes from the server to the local database
fn apply_incoming_changes(db: &Database, response: &SyncResponse) -> usize {
    let mut applied = 0;

    // Sort changes: lists first, then tags, then tasks (to satisfy FK constraints)
    let mut lists = Vec::new();
    let mut tags = Vec::new();
    let mut tasks = Vec::new();
    let mut task_tags = Vec::new();
    let mut deletes = Vec::new();

    for record in &response.changes {
        match record {
            SyncRecord::List(_) => lists.push(record),
            SyncRecord::Tag(_) => tags.push(record),
            SyncRecord::Task(_) => tasks.push(record),
            SyncRecord::TaskTag(_) => task_tags.push(record),
            SyncRecord::Deleted { .. } => deletes.push(record),
        }
    }

    // Disable FK constraints during sync
    let _ = db.execute_raw("PRAGMA foreign_keys = OFF");

    // Apply in order: lists, tags, tasks, task_tags, deletes
    for record in lists
        .iter()
        .chain(tags.iter())
        .chain(tasks.iter())
        .chain(task_tags.iter())
        .chain(deletes.iter())
    {
        let result = match record {
            SyncRecord::Task(task) => db.upsert_task(task),
            SyncRecord::List(list) => db.upsert_list(list),
            SyncRecord::Tag(tag) => db.upsert_tag(tag),
            SyncRecord::TaskTag(link) => db.upsert_task_tag(link),
            SyncRecord::Deleted {
                id, record_type, ..
            } => {
                match record_type {
                    RecordType::Task => db.delete_task_by_id(*id),
                    RecordType::List => db.delete_list_by_id(*id),
                    RecordType::Tag => db.delete_tag_by_id(*id),
                    RecordType::TaskTag => Ok(()), // Handled by task update
                }
            }
        };

        if result.is_ok() {
            applied += 1;
        }
    }

    // Re-enable FK constraints
    let _ = db.execute_raw("PRAGMA foreign_keys = ON");

    applied
}

/// Check for tasks due today/tomorrow and send notifications
fn check_and_notify_due_tasks(db: &Database) -> usize {
    use crate::models::Priority;
    use chrono::Local;

    let today = Local::now().date_naive();
    let tomorrow = today.succ_opt().unwrap_or(today);

    let mut notified = 0;

    // Get all incomplete tasks with due dates
    if let Ok(tasks) = db.get_all_tasks() {
        for task in tasks {
            // Skip completed tasks
            if task.completed {
                continue;
            }

            // Check if task has a due date
            if let Some(due_datetime) = &task.due_date {
                let due_date = due_datetime.date_naive();

                if due_date == today {
                    // Task is due today
                    if notifications::notify_task_due_today(&task).is_ok() {
                        notified += 1;
                    }
                } else if due_date == tomorrow
                    && (task.priority == Priority::High || task.priority == Priority::Urgent)
                {
                    // High/urgent task due tomorrow - advance warning
                    if notifications::notify_task_due_tomorrow(&task).is_ok() {
                        notified += 1;
                    }
                } else if due_date < today {
                    // Task is overdue
                    if notifications::notify_task_overdue(&task).is_ok() {
                        notified += 1;
                    }
                }
            }
        }
    }

    notified
}
