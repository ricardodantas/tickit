//! Event handling for the TUI

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::state::{AppState, EditorField, Focus, Mode, View};
use crate::theme::Theme;

/// Handle a key event
pub fn handle_key(state: &mut AppState, key: KeyEvent) {
    // Global shortcuts
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('c') | KeyCode::Char('q') => {
                state.should_quit = true;
                return;
            }
            _ => {}
        }
    }

    match state.mode {
        Mode::Normal => handle_normal_mode(state, key),
        Mode::ThemePicker => handle_theme_picker(state, key),
        Mode::Help => handle_help(state, key),
        Mode::AddTask | Mode::EditTask => handle_task_editor(state, key),
        Mode::AddList | Mode::EditList => handle_list_editor(state, key),
        Mode::AddTag | Mode::EditTag => handle_tag_editor(state, key),
        Mode::Confirm => handle_confirm(state, key),
        Mode::Export => handle_export(state, key),
    }
}

/// Handle key events in normal mode
fn handle_normal_mode(state: &mut AppState, key: KeyEvent) {
    match key.code {
        // Quit
        KeyCode::Char('q') => state.should_quit = true,

        // Help
        KeyCode::Char('?') => {
            state.show_help = true;
            state.mode = Mode::Help;
        }

        // Theme picker
        KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.mode = Mode::ThemePicker;
        }

        // View navigation (tabs)
        KeyCode::Char('1') => {
            state.view = View::Tasks;
            state.focus = Focus::Main;
        }
        KeyCode::Char('2') => {
            state.view = View::Lists;
            state.focus = Focus::Main;
        }
        KeyCode::Char('3') => {
            state.view = View::Tags;
            state.focus = Focus::Main;
        }

        // Tab between views
        KeyCode::Tab => {
            state.view = match state.view {
                View::Tasks => View::Lists,
                View::Lists => View::Tags,
                View::Tags => View::Tasks,
            };
        }
        KeyCode::BackTab => {
            state.view = match state.view {
                View::Tasks => View::Tags,
                View::Lists => View::Tasks,
                View::Tags => View::Lists,
            };
        }

        // Focus switching (sidebar/main)
        KeyCode::Char('h') | KeyCode::Left if state.view == View::Tasks => {
            state.focus = Focus::Sidebar;
        }
        KeyCode::Char('l') | KeyCode::Right if state.view == View::Tasks => {
            state.focus = Focus::Main;
        }

        // Navigation
        KeyCode::Char('j') | KeyCode::Down => handle_down(state),
        KeyCode::Char('k') | KeyCode::Up => handle_up(state),
        KeyCode::Char('g') => {
            // Go to top
            match (state.view, state.focus) {
                (View::Tasks, Focus::Sidebar) => state.list_index = 0,
                (View::Tasks, Focus::Main) => state.task_index = 0,
                (View::Lists, _) => state.list_index = 0,
                (View::Tags, _) => state.tag_index = 0,
            }
        }
        KeyCode::Char('G') => {
            // Go to bottom
            match (state.view, state.focus) {
                (View::Tasks, Focus::Sidebar) => {
                    state.list_index = state.lists.len(); // +1 for "All"
                }
                (View::Tasks, Focus::Main) => {
                    if !state.tasks.is_empty() {
                        state.task_index = state.tasks.len() - 1;
                    }
                }
                (View::Lists, _) => {
                    if !state.lists.is_empty() {
                        state.list_index = state.lists.len();
                    }
                }
                (View::Tags, _) => {
                    if !state.tags.is_empty() {
                        state.tag_index = state.tags.len() - 1;
                    }
                }
            }
        }

        // Selection (list selection in sidebar)
        KeyCode::Enter => handle_enter(state),

        // Add new item
        KeyCode::Char('a') | KeyCode::Char('n') => {
            match state.view {
                View::Tasks => state.start_add_task(),
                View::Lists => state.start_add_list(),
                View::Tags => state.start_add_tag(),
            }
        }

        // Edit selected item
        KeyCode::Char('e') => {
            match state.view {
                View::Tasks if state.focus == Focus::Main => state.start_edit_task(),
                _ => {}
            }
        }

        // Delete selected item
        KeyCode::Char('d') | KeyCode::Delete => {
            match state.view {
                View::Tasks if state.focus == Focus::Main => state.confirm_delete_task(),
                View::Lists => state.confirm_delete_list(),
                View::Tags => state.confirm_delete_tag(),
                _ => {}
            }
        }

        // Toggle task completion
        KeyCode::Char(' ') | KeyCode::Char('x') if state.view == View::Tasks && state.focus == Focus::Main => {
            let _ = state.toggle_task();
        }

        // Toggle show completed
        KeyCode::Char('c') => state.toggle_show_completed(),

        // Cycle priority
        KeyCode::Char('p') if state.view == View::Tasks && state.focus == Focus::Main => {
            let _ = state.cycle_task_priority();
        }

        // Open URL
        KeyCode::Char('o') if state.view == View::Tasks && state.focus == Focus::Main => {
            state.open_task_url();
        }

        // Refresh
        KeyCode::Char('r') => {
            let _ = state.refresh_data();
            state.set_status("Refreshed");
        }

        _ => {}
    }
}

fn handle_down(state: &mut AppState) {
    match (state.view, state.focus) {
        (View::Tasks, Focus::Sidebar) => {
            if state.list_index < state.lists.len() {
                state.list_index += 1;
            }
        }
        (View::Tasks, Focus::Main) => {
            if !state.tasks.is_empty() && state.task_index < state.tasks.len() - 1 {
                state.task_index += 1;
            }
        }
        (View::Lists, _) => {
            if state.list_index < state.lists.len() {
                state.list_index += 1;
            }
        }
        (View::Tags, _) => {
            if !state.tags.is_empty() && state.tag_index < state.tags.len() - 1 {
                state.tag_index += 1;
            }
        }
    }
}

fn handle_up(state: &mut AppState) {
    match (state.view, state.focus) {
        (View::Tasks, Focus::Sidebar) | (View::Lists, _) => {
            if state.list_index > 0 {
                state.list_index -= 1;
            }
        }
        (View::Tasks, Focus::Main) => {
            if state.task_index > 0 {
                state.task_index -= 1;
            }
        }
        (View::Tags, _) => {
            if state.tag_index > 0 {
                state.tag_index -= 1;
            }
        }
    }
}

fn handle_enter(state: &mut AppState) {
    match (state.view, state.focus) {
        (View::Tasks, Focus::Sidebar) => {
            // Select list and filter tasks
            if state.list_index == 0 {
                state.selected_list_id = None;
            } else if let Some(list) = state.lists.get(state.list_index - 1) {
                state.selected_list_id = Some(list.id);
            }
            let _ = state.refresh_tasks();
            state.task_index = 0;
            state.focus = Focus::Main;
        }
        (View::Tasks, Focus::Main) => {
            // Toggle task or open editor
            let _ = state.toggle_task();
        }
        (View::Lists, _) => {
            // Could open list editor
        }
        (View::Tags, _) => {
            // Could open tag editor
        }
    }
}

/// Handle theme picker
fn handle_theme_picker(state: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            state.mode = Mode::Normal;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            let themes = Theme::all();
            if state.theme_index < themes.len() - 1 {
                state.theme_index += 1;
                state.set_theme(themes[state.theme_index]);
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if state.theme_index > 0 {
                state.theme_index -= 1;
                state.set_theme(Theme::all()[state.theme_index]);
            }
        }
        KeyCode::Enter => {
            state.set_status(format!("Theme: {}", state.theme.name()));
            state.mode = Mode::Normal;
        }
        _ => {}
    }
}

/// Handle help dialog
fn handle_help(state: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
            state.show_help = false;
            state.mode = Mode::Normal;
        }
        _ => {}
    }
}

/// Handle task editor
fn handle_task_editor(state: &mut AppState, key: KeyEvent) {
    // Handle inline tag creation mode
    if state.editor_adding_tag {
        match key.code {
            KeyCode::Esc => {
                state.cancel_inline_tag();
            }
            KeyCode::Enter => {
                let _ = state.save_inline_tag();
            }
            KeyCode::Char(c) => {
                state.editor_new_tag_buffer.push(c);
            }
            KeyCode::Backspace => {
                state.editor_new_tag_buffer.pop();
            }
            _ => {}
        }
        return;
    }

    match key.code {
        KeyCode::Esc => {
            state.mode = Mode::Normal;
            state.editing_task = None;
        }
        KeyCode::Enter if !key.modifiers.contains(KeyModifiers::SHIFT) => {
            // If on Tags field and cursor is on "Add new", start adding
            if state.editor_field == EditorField::Tags && state.editor_tag_cursor == state.tags.len() {
                state.start_inline_add_tag();
            } else {
                let _ = state.save_task();
            }
        }
        KeyCode::Tab => state.next_editor_field(),
        KeyCode::BackTab => state.prev_editor_field(),

        // Text input for title field
        KeyCode::Char(c) if state.editor_field == EditorField::Title => {
            state.input_buffer.insert(state.cursor_pos, c);
            state.cursor_pos += 1;
        }
        KeyCode::Backspace if state.editor_field == EditorField::Title => {
            if state.cursor_pos > 0 {
                state.cursor_pos -= 1;
                state.input_buffer.remove(state.cursor_pos);
            }
        }
        KeyCode::Delete if state.editor_field == EditorField::Title => {
            if state.cursor_pos < state.input_buffer.len() {
                state.input_buffer.remove(state.cursor_pos);
            }
        }
        KeyCode::Left if state.editor_field == EditorField::Title => {
            if state.cursor_pos > 0 {
                state.cursor_pos -= 1;
            }
        }
        KeyCode::Right if state.editor_field == EditorField::Title => {
            if state.cursor_pos < state.input_buffer.len() {
                state.cursor_pos += 1;
            }
        }
        KeyCode::Home if state.editor_field == EditorField::Title => {
            state.cursor_pos = 0;
        }
        KeyCode::End if state.editor_field == EditorField::Title => {
            state.cursor_pos = state.input_buffer.len();
        }

        // Priority field
        KeyCode::Char('j') | KeyCode::Down if state.editor_field == EditorField::Priority => {
            state.editor_priority = state.editor_priority.next();
        }
        KeyCode::Char('k') | KeyCode::Up if state.editor_field == EditorField::Priority => {
            state.editor_priority = state.editor_priority.prev();
        }

        // List field
        KeyCode::Char('j') | KeyCode::Down if state.editor_field == EditorField::List => {
            if state.editor_list_index < state.lists.len() - 1 {
                state.editor_list_index += 1;
            }
        }
        KeyCode::Char('k') | KeyCode::Up if state.editor_field == EditorField::List => {
            if state.editor_list_index > 0 {
                state.editor_list_index -= 1;
            }
        }

        // Tags field - navigate and toggle
        KeyCode::Char('j') | KeyCode::Down if state.editor_field == EditorField::Tags => {
            state.editor_tag_cursor_down();
        }
        KeyCode::Char('k') | KeyCode::Up if state.editor_field == EditorField::Tags => {
            state.editor_tag_cursor_up();
        }
        KeyCode::Char(' ') if state.editor_field == EditorField::Tags => {
            // If cursor is on "Add new tag" option
            if state.editor_tag_cursor == state.tags.len() {
                state.start_inline_add_tag();
            } else {
                state.toggle_editor_tag();
            }
        }
        KeyCode::Char('n') if state.editor_field == EditorField::Tags => {
            // Quick shortcut to add new tag
            state.start_inline_add_tag();
        }

        _ => {}
    }
}

/// Handle list editor
fn handle_list_editor(state: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            state.mode = Mode::Normal;
            state.editing_list = None;
        }
        KeyCode::Enter => {
            let _ = state.save_list();
        }
        KeyCode::Char(c) => {
            state.input_buffer.insert(state.cursor_pos, c);
            state.cursor_pos += 1;
        }
        KeyCode::Backspace => {
            if state.cursor_pos > 0 {
                state.cursor_pos -= 1;
                state.input_buffer.remove(state.cursor_pos);
            }
        }
        KeyCode::Delete => {
            if state.cursor_pos < state.input_buffer.len() {
                state.input_buffer.remove(state.cursor_pos);
            }
        }
        KeyCode::Left => {
            if state.cursor_pos > 0 {
                state.cursor_pos -= 1;
            }
        }
        KeyCode::Right => {
            if state.cursor_pos < state.input_buffer.len() {
                state.cursor_pos += 1;
            }
        }
        _ => {}
    }
}

/// Handle tag editor
fn handle_tag_editor(state: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            state.mode = Mode::Normal;
            state.editing_tag = None;
        }
        KeyCode::Enter => {
            let _ = state.save_tag();
        }
        KeyCode::Char(c) => {
            state.input_buffer.insert(state.cursor_pos, c);
            state.cursor_pos += 1;
        }
        KeyCode::Backspace => {
            if state.cursor_pos > 0 {
                state.cursor_pos -= 1;
                state.input_buffer.remove(state.cursor_pos);
            }
        }
        KeyCode::Delete => {
            if state.cursor_pos < state.input_buffer.len() {
                state.input_buffer.remove(state.cursor_pos);
            }
        }
        KeyCode::Left => {
            if state.cursor_pos > 0 {
                state.cursor_pos -= 1;
            }
        }
        KeyCode::Right => {
            if state.cursor_pos < state.input_buffer.len() {
                state.cursor_pos += 1;
            }
        }
        _ => {}
    }
}

/// Handle confirmation dialog
fn handle_confirm(state: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Char('y') | KeyCode::Enter => {
            let _ = state.execute_confirm();
        }
        KeyCode::Char('n') | KeyCode::Esc => {
            state.cancel_confirm();
        }
        _ => {}
    }
}

/// Handle export dialog
fn handle_export(state: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            state.mode = Mode::Normal;
        }
        _ => {}
    }
}
