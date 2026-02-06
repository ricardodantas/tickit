# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of Tickit
- TUI mode with vim-style keybindings
- CLI mode for scripting and quick actions
- Task management with title, description, URL, priority
- Lists for organizing tasks (with default Inbox)
- Tags with custom colors for categorization
- 15 beautiful themes (Dracula, Nord, Catppuccin, etc.)
- Export to JSON, todo.txt, Markdown, CSV
- SQLite storage at `~/.config/tickit/`
- About dialog with ASCII logo
- Help dialog with keybinding reference

### Keyboard Shortcuts
- `t` - Theme picker
- `A` - About dialog
- `n` - Add new item
- `e` - Edit item
- `d` - Delete item
- `g/G` - Go to top/bottom
- `j/k` - Navigate up/down
- `h/l` - Focus sidebar/main
- `1/2/3` - Switch views (Tasks/Lists/Tags)
- `Space/x` - Toggle task completion
- `?` - Help
- `q` - Quit
