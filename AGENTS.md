# AGENTS.md - Tickit Project

## Overview

**Tickit** ✓ is a terminal-based task manager with both CLI and TUI interfaces. It provides beautiful task management with lists, tags, priorities, and multiple export formats.

## Architecture

```
tickit/
├── src/
│   ├── main.rs          # CLI entry point, subcommands
│   ├── lib.rs           # Library root, public API
│   ├── app/             # TUI application logic
│   │   ├── mod.rs       # App initialization
│   │   ├── state.rs     # Application state
│   │   ├── ui.rs        # UI rendering
│   │   └── events.rs    # Key event handling
│   ├── config.rs        # Configuration (theme, settings)
│   ├── db.rs            # SQLite database operations
│   ├── export.rs        # Export to JSON/todo.txt/Markdown/CSV
│   ├── models.rs        # Data models (Task, List, Tag, Priority)
│   └── theme.rs         # 15 color themes
├── screenshots/         # TUI screenshots for docs
├── scripts/             # Helper scripts
├── Cargo.toml           # Dependencies & metadata
├── README.md            # User documentation
├── CHANGELOG.md         # Version history
└── LICENSE              # MIT license
```

## Key Features

### TUI Mode (`tickit`)
- **Tasks View**: Create, edit, complete, delete tasks
- **Lists View**: Organize tasks into lists with icons
- **Tags View**: Create colorful tags for categorization
- **15 themes**: Dracula, Nord, Tokyo Night, Catppuccin, etc.
- **Vim-style keybindings**: j/k navigation, familiar shortcuts
- **Task URLs**: Attach links and open with `o`

### CLI Mode
- `tickit add` — Add new tasks with options
- `tickit list` — List tasks with filters
- `tickit done` / `tickit undo` — Toggle completion
- `tickit delete` — Delete tasks
- `tickit lists` — Manage lists
- `tickit tags` — Manage tags
- `tickit export` — Export to multiple formats

### Data Model
- **Task**: title, description, URL, priority, completion, list, tags, due date
- **List**: name, icon, color, description (default: Inbox)
- **Tag**: name, color
- **Priority**: Low, Medium, High, Urgent

## Key Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| ratatui | 0.30 | TUI framework |
| crossterm | 0.29 | Terminal backend |
| tokio | 1.49 | Async runtime |
| rusqlite | 0.33 | SQLite database (bundled) |
| serde | 1.0 | Serialization |
| serde_json | 1.0 | JSON export |
| toml | 0.9 | Config format |
| clap | 4.5 | CLI parsing |
| chrono | 0.4 | Date/time handling |
| uuid | 1.16 | Unique IDs |
| open | 5.3 | Open URLs in browser |
| anyhow | 1.0 | Error handling |
| thiserror | 2.0 | Custom errors |
| tracing | 0.1 | Logging |

## Development Commands

```bash
# Run TUI in dev mode
cargo run

# Run CLI commands
cargo run -- add "Task title" --priority high
cargo run -- list --all
cargo run -- done "Task title"
cargo run -- export --format markdown

# Build release binary
cargo build --release

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy
```

## Configuration

Config file: `~/.config/tickit/config.toml`

```toml
theme = "dracula"
```

Database: `~/.config/tickit/tickit.sqlite`

## Export Formats

| Format | Extension | Description |
|--------|-----------|-------------|
| JSON | .json | Full data with all fields |
| todo.txt | .txt | Compatible with todo.txt spec |
| Markdown | .md | Human-readable with checkboxes |
| CSV | .csv | Spreadsheet-compatible |

## Current Status

✅ **Working:**
- Full TUI with 15 beautiful themes
- CLI commands for all operations
- SQLite storage with migrations
- Lists and tags management
- Priority levels
- Export to all formats
- Task URLs with browser open

📋 **Potential Improvements:**
- Due date reminders
- Recurring tasks
- Search/filter in TUI
- Sync with external services
- Homebrew/crates.io publishing

## Themes

Press `t` in the TUI to open theme picker (15 themes):
- Dracula (default), One Dark Pro, Nord
- Catppuccin Mocha/Latte, Gruvbox Dark/Light
- Tokyo Night, Solarized Dark/Light
- Monokai Pro, Rosé Pine, Kanagawa
- Everforest, Cyberpunk

## Keybindings

### Global
| Key | Action |
|-----|--------|
| Tab / Shift+Tab | Switch views |
| 1-3 | Jump to view (Tasks, Lists, Tags) |
| t | Theme picker |
| ? / F1 | Help |
| A | About |
| q | Quit |

### Tasks View
| Key | Action |
|-----|--------|
| j/k or ↑/↓ | Navigate |
| Enter/Space | Toggle complete |
| n | New task |
| e | Edit task |
| d | Delete task |
| p | Cycle priority |
| o | Open URL |
| c | Toggle show completed |

## Website

Not yet deployed.

## Related Projects

- **Hazelnut** — Terminal file organizer (same author, shared theme style)
- **Feedo** — Terminal RSS reader (same author, shared theme style)

## Notes

- Themes are currently local (not using ratatui-themes crate yet)
- Consider migrating to ratatui-themes for consistency with other projects
- SQLite uses bundled feature (no external dependency)
