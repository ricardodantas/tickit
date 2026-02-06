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
use std::time::Duration;

use crate::config::Config;
use crate::db::Database;
use crate::sync::{RecordType, SyncClient, SyncRecord};

/// Messages from background tasks
enum BackgroundMsg {
    UpdateAvailable(String),
    SyncComplete(Result<chrono::DateTime<chrono::Utc>, String>),
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
    let mut state = AppState::new(config, db)?;

    // Spawn background update check
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let check = crate::check_for_updates_crates_io_timeout(std::time::Duration::from_secs(5));
        if let crate::VersionCheck::UpdateAvailable { latest, .. } = check {
            let _ = tx.send(BackgroundMsg::UpdateAvailable(latest));
        }
    });

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

    loop {
        // Check for background messages (non-blocking)
        if let Ok(msg) = bg_rx.try_recv() {
            match msg {
                BackgroundMsg::UpdateAvailable(version) => {
                    state.set_update_available(version);
                }
                BackgroundMsg::SyncComplete(_) => {} // Handled by sync_rx
            }
        }

        // Check for sync completion
        if let Ok(msg) = sync_rx.try_recv()
            && let BackgroundMsg::SyncComplete(result) = msg
        {
            sync_in_progress = false;
            match result {
                Ok(server_time) => {
                    state.set_last_sync(server_time);
                    state.set_status("Sync complete");
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

        // Check if sync was requested (via Ctrl+S)
        if state.sync_status.syncing && !sync_in_progress && state.is_sync_enabled() {
            sync_in_progress = true;
            let config = state.config.sync.clone();
            let last_sync = state.db.get_last_sync().ok().flatten();

            // Gather local changes
            let changes = gather_local_changes(&state.db, last_sync);

            let tx = sync_tx.clone();
            std::thread::spawn(move || {
                let mut client = SyncClient::new(config);
                let result = client.sync(changes, last_sync);
                let msg = match result {
                    Ok(response) => BackgroundMsg::SyncComplete(Ok(response.server_time)),
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
