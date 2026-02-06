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

/// Messages from background tasks
enum BackgroundMsg {
    UpdateAvailable(String),
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
    loop {
        // Check for background messages (non-blocking)
        if let Ok(msg) = bg_rx.try_recv() {
            match msg {
                BackgroundMsg::UpdateAvailable(version) => {
                    state.set_update_available(version);
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
