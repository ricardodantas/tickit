# CLAUDE.md - Tickit

Quick reference for Claude/AI assistants working on Tickit.

## What is Tickit?

A terminal task manager with lists, tags, priorities, and optional sync.

## Quick Commands

```bash
cargo run              # Run the TUI
cargo check            # Fast compile check
cargo clippy -- -D warnings  # Must pass for CI
cargo fmt              # Format code
```

## File Locations

| What | Where |
|------|-------|
| UI rendering | `src/app/ui.rs` |
| Key handling | `src/app/events.rs` |
| App state | `src/app/state.rs` |
| Data models | `src/models.rs` |
| Database | `src/db.rs` |
| Sync client | `src/sync/mod.rs` |
| Auto-update | `src/lib.rs` (bottom) |

## Common Tasks

### Add a new keybinding
1. Edit `src/app/events.rs`
2. Find the relevant view handler (`handle_tasks_view`, etc.)
3. Add match arm for the key
4. Update help in `src/app/ui.rs` → `render_help_popup`

### Add a new field to Task
1. Add field to `Task` struct in `src/models.rs`
2. Update database schema in `src/db.rs`
3. Add to task editor in `src/app/state.rs` (EditorField enum)
4. Handle in `src/app/events.rs` task editor
5. Render in `src/app/ui.rs` task editor

### Add a new dialog/mode
1. Add variant to `Mode` enum in `src/app/state.rs`
2. Add render function in `src/app/ui.rs`
3. Call from `render()` based on mode
4. Add key handler in `src/app/events.rs`

## Data Model

```
Task
├── id: UUID
├── title: String
├── description: Option<String>
├── priority: Priority (Urgent/High/Medium/Low)
├── due_date: Option<DateTime>
├── completed: bool
├── list_id: UUID
└── tag_ids: Vec<UUID>

List
├── id: UUID
├── name: String
└── is_inbox: bool

Tag
├── id: UUID
├── name: String
└── color: Option<String>
```

## Sync Notes

- All records have `updated_at` for sync
- Deletions tracked in `tombstones` table
- Sync uses JSON over HTTP to self-hosted server
- Configure in `~/.config/tickit/config.toml`

## CI Gotchas

- Format with `cargo fmt` before committing
- Clippy must pass with `-D warnings`
