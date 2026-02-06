# Tickit ✓

```
  ████████╗██╗ ██████╗██╗  ██╗██╗████████╗
  ╚══██╔══╝██║██╔════╝██║ ██╔╝██║╚══██╔══╝
     ██║   ██║██║     █████╔╝ ██║   ██║   
     ██║   ██║██║     ██╔═██╗ ██║   ██║   
     ██║   ██║╚██████╗██║  ██╗██║   ██║   
     ╚═╝   ╚═╝ ╚═════╝╚═╝  ╚═╝╚═╝   ╚═╝   
                    ✓ get stuff done
```

A stunning terminal-based task manager with CLI and TUI modes.

![Rust](https://img.shields.io/badge/rust-1.93+-orange.svg)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Features

- 🎨 **Beautiful TUI** with 15+ themes (same as Hazelnut/Feedo)
- ⌨️ **Full CLI** for scripting and quick actions
- 📋 **Lists** to organize your tasks
- 🏷️ **Tags** for flexible categorization
- 🔗 **URLs** that open in your browser
- ⚡ **Priority levels** (Low, Medium, High, Urgent)
- 📤 **Export** to JSON, todo.txt, Markdown, CSV
- 💾 **SQLite storage** in `~/.config/tickit/`

## Installation

### From source

```bash
cargo install --path .
```

### From crates.io (coming soon)

```bash
cargo install tickit
```

## Usage

### TUI Mode (default)

```bash
tickit
```

### CLI Commands

```bash
# Add a task
tickit add "Buy groceries" --priority high --list Shopping
tickit add "Review PR" --url "https://github.com/..." --tags work,urgent

# List tasks
tickit list                    # Show incomplete tasks
tickit list --all              # Include completed
tickit list --list Work        # Filter by list
tickit list --tag urgent       # Filter by tag
tickit list --json             # Output as JSON

# Complete/uncomplete
tickit done "Buy groceries"
tickit undo "Buy groceries"

# Delete
tickit delete "Old task"
tickit rm "Old task" --force   # Skip confirmation

# Manage lists
tickit lists                   # List all lists
tickit lists add "Shopping" --icon "🛒"
tickit lists delete "Old List"

# Manage tags
tickit tags                    # List all tags
tickit tags add "urgent" --color "#ff0000"
tickit tags delete "old-tag"

# Export
tickit export --format json --output tasks.json
tickit export --format todotxt
tickit export --format markdown --list Work
tickit export --format csv
```

## Keyboard Shortcuts (TUI)

### Navigation

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `g` | Go to top |
| `G` | Go to bottom |
| `h` / `←` | Focus sidebar |
| `l` / `→` | Focus main |
| `Tab` | Next view |
| `Shift+Tab` | Previous view |
| `1` | Tasks view |
| `2` | Lists view |
| `3` | Tags view |

### Tasks

| Key | Action |
|-----|--------|
| `n` | Add new task |
| `e` | Edit task |
| `d` | Delete task |
| `Space` / `x` | Toggle complete |
| `Enter` | Select list / Toggle task |
| `p` | Cycle priority |
| `o` | Open URL |
| `c` | Toggle show completed |
| `r` | Refresh |

### General

| Key | Action |
|-----|--------|
| `t` | Theme picker |
| `A` | About dialog |
| `?` / `F1` | Help |
| `q` | Quit |

## Themes

Tickit includes 15 beautiful themes:

- Dracula (default)
- One Dark Pro
- Nord
- Catppuccin Mocha / Latte
- Gruvbox Dark / Light
- Tokyo Night
- Solarized Dark / Light
- Monokai Pro
- Rosé Pine
- Kanagawa
- Everforest
- Cyberpunk

Press `t` in the TUI to switch themes.

## Data Storage

Tasks are stored in an SQLite database at:

```
~/.config/tickit/tickit.sqlite
```

Configuration is stored at:

```
~/.config/tickit/config.toml
```

## Export Formats

### JSON

Full data export with all fields.

### todo.txt

Compatible with [todo.txt](http://todotxt.org/) format:
```
(A) 2024-01-01 Task title +Project @context due:2024-01-15
```

### Markdown

Human-readable format with checkboxes:
```markdown
## 📥 Inbox
- [ ] 🔴 Urgent task
- [x] Completed task
```

### CSV

Spreadsheet-compatible format.

## Author

**Ricardo Dantas** - [GitHub](https://github.com/ricardodantas)

## License

MIT
