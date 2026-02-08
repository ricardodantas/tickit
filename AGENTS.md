# AGENTS.md - Tickit

Guidelines for AI agents working on this codebase.

## Project Overview

Tickit is a stunning terminal-based task manager with optional sync support.

**Stack:** Rust, Ratatui (TUI), SQLite, optional self-hosted sync server

## Architecture

```
src/
├── app/           # TUI application
│   ├── mod.rs     # Main app loop, sync handling
│   ├── state.rs   # AppState, Mode, View, Focus
│   ├── events.rs  # Key event handling
│   └── ui.rs      # Ratatui rendering
├── models.rs      # Task, List, Tag, Priority
├── db.rs          # SQLite database with sync support
├── config.rs      # Configuration + sync settings
├── sync/          # Sync client for self-hosted server
├── export.rs      # Export to JSON, CSV, Markdown
├── notifications.rs # Desktop notifications for due tasks
├── theme.rs       # Theme support (15 themes)
├── lib.rs         # Library + auto-update functions
└── main.rs        # CLI interface
```

## Key Patterns

### Views & Modes
- `View`: Tasks, Lists, Tags
- `Mode`: Normal, AddTask, EditTask, AddList, EditList, AddTag, EditTag, Confirm, Help, ThemePicker, About, Export, UpdateConfirm, Updating
- `Focus`: Sidebar, Main

### Task Editor
- Multi-field editor: Title, Description, DueDate, Priority, List, Tags
- Tab/Shift+Tab to navigate fields
- Inline tag creation within task editor

### Sync System
- Optional self-hosted sync server
- Automatic sync on data changes
- Tombstones for deletion tracking
- Last-write-wins conflict resolution

### Update System
- Background check on startup via `check_for_updates_crates_io()`
- Yellow banner at top when update available
- Press `[U]` from anywhere to update
- Detects Homebrew vs Cargo installation

## CLI Commands

```bash
tickit                   # Launch TUI
tickit add "task"        # Quick add task
tickit list              # List tasks
tickit done <id>         # Mark complete
tickit export json       # Export data
```

## Development

```bash
cargo run                # Run TUI
cargo test               # Run tests
cargo clippy             # Lint
cargo fmt                # Format
```

## CI Requirements

All PRs must pass:
- `cargo check`
- `cargo test`
- `cargo fmt -- --check`
- `cargo clippy -- -D warnings`
- Build on Linux, macOS, Windows

## Code Style

- Use `anyhow` for error handling
- UUIDs for all record IDs (sync-friendly)
- Timestamps in UTC
- Keep UI responsive - long operations in background threads
