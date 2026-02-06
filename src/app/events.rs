//! Event handling for the TUI

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::state::{AppState, EditorField, Focus, Mode, View};
use crate::theme::Theme;

/// Handle a key event
pub fn handle_key(state: &mut AppState, key: KeyEvent) {
    // Handle mode-specific input first
    match state.mode {
        Mode::ThemePicker => {
            handle_theme_picker(state, key);
            return;
        }
        Mode::Help => {
            if matches!(key.code, KeyCode::Esc | KeyCode::Char('?') | KeyCode::Enter) {
                state.mode = Mode::Normal;
                state.show_help = false;
            }
            return;
        }
        Mode::About => {
            handle_about(state, key);
            return;
        }
        Mode::AddTask | Mode::EditTask => {
            handle_task_editor(state, key);
            return;
        }
        Mode::AddList | Mode::EditList => {
            handle_list_editor(state, key);
            return;
        }
        Mode::AddTag | Mode::EditTag => {
            handle_tag_editor(state, key);
            return;
        }
        Mode::Confirm => {
            handle_confirm(state, key);
            return;
        }
        Mode::Export => {
            handle_export(state, key);
            return;
        }
        Mode::Normal => {}
    }

    // Global keybindings (like Hazelnut)
    match (key.modifiers, key.code) {
        // Quit
        (KeyModifiers::CONTROL, KeyCode::Char('c'))
        | (KeyModifiers::CONTROL, KeyCode::Char('q')) => {
            state.should_quit = true;
            return;
        }
        (_, KeyCode::Char('q')) => {
            state.should_quit = true;
            return;
        }
        // Help
        (_, KeyCode::Char('?')) | (_, KeyCode::F(1)) => {
            state.mode = Mode::Help;
            state.show_help = true;
            return;
        }
        // Tab between views
        (_, KeyCode::Tab) => {
            state.view = match state.view {
                View::Tasks => View::Lists,
                View::Lists => View::Tags,
                View::Tags => View::Tasks,
            };
            state.focus = Focus::Main;
            return;
        }
        (KeyModifiers::SHIFT, KeyCode::BackTab) => {
            state.view = match state.view {
                View::Tasks => View::Tags,
                View::Lists => View::Tasks,
                View::Tags => View::Lists,
            };
            state.focus = Focus::Main;
            return;
        }
        // Number keys for quick navigation (like Hazelnut)
        (_, KeyCode::Char('1')) => {
            state.view = View::Tasks;
            state.focus = Focus::Main;
            return;
        }
        (_, KeyCode::Char('2')) => {
            state.view = View::Lists;
            state.focus = Focus::Main;
            return;
        }
        (_, KeyCode::Char('3')) => {
            state.view = View::Tags;
            state.focus = Focus::Main;
            return;
        }
        // Theme picker (just 't', like Hazelnut/Feedo)
        (_, KeyCode::Char('t')) => {
            state.theme_index = Theme::all()
                .iter()
                .position(|t| *t == state.theme.inner())
                .unwrap_or(0);
            state.mode = Mode::ThemePicker;
            return;
        }
        // About dialog (A like Hazelnut)
        (_, KeyCode::Char('A')) => {
            state.mode = Mode::About;
            return;
        }
        _ => {}
    }

    // View-specific keybindings
    match state.view {
        View::Tasks => handle_tasks_view(state, key),
        View::Lists => handle_lists_view(state, key),
        View::Tags => handle_tags_view(state, key),
    }
}

/// Handle tasks view keybindings
fn handle_tasks_view(state: &mut AppState, key: KeyEvent) {
    match key.code {
        // Focus switching (sidebar/main) with h/l
        KeyCode::Char('h') | KeyCode::Left => {
            state.focus = Focus::Sidebar;
        }
        KeyCode::Char('l') | KeyCode::Right => {
            state.focus = Focus::Main;
        }

        // Navigation
        KeyCode::Char('j') | KeyCode::Down => match state.focus {
            Focus::Sidebar => {
                if state.list_index < state.lists.len() {
                    state.list_index += 1;
                }
            }
            Focus::Main => {
                if !state.tasks.is_empty() && state.task_index < state.tasks.len() - 1 {
                    state.task_index += 1;
                }
            }
        },
        KeyCode::Char('k') | KeyCode::Up => match state.focus {
            Focus::Sidebar => {
                if state.list_index > 0 {
                    state.list_index -= 1;
                }
            }
            Focus::Main => {
                if state.task_index > 0 {
                    state.task_index -= 1;
                }
            }
        },
        KeyCode::Char('g') | KeyCode::Home => match state.focus {
            Focus::Sidebar => state.list_index = 0,
            Focus::Main => state.task_index = 0,
        },
        KeyCode::Char('G') | KeyCode::End => match state.focus {
            Focus::Sidebar => state.list_index = state.lists.len(),
            Focus::Main => {
                if !state.tasks.is_empty() {
                    state.task_index = state.tasks.len() - 1;
                }
            }
        },

        // Enter - select list or toggle task
        KeyCode::Enter => match state.focus {
            Focus::Sidebar => {
                if state.list_index == 0 {
                    state.selected_list_id = None;
                } else if let Some(list) = state.lists.get(state.list_index - 1) {
                    state.selected_list_id = Some(list.id);
                }
                let _ = state.refresh_tasks();
                state.task_index = 0;
                state.focus = Focus::Main;
            }
            Focus::Main => {
                let _ = state.toggle_task();
            }
        },

        // Space or x - toggle task completion
        KeyCode::Char(' ') | KeyCode::Char('x') if state.focus == Focus::Main => {
            let _ = state.toggle_task();
        }

        // Add new task (n like Hazelnut)
        KeyCode::Char('n') => {
            state.start_add_task();
        }

        // Edit task (e like Hazelnut)
        KeyCode::Char('e') if state.focus == Focus::Main => {
            state.start_edit_task();
        }

        // Delete task (d like Hazelnut)
        KeyCode::Char('d') | KeyCode::Delete if state.focus == Focus::Main => {
            state.confirm_delete_task();
        }

        // Toggle show completed (c)
        KeyCode::Char('c') => {
            state.toggle_show_completed();
        }

        // Cycle priority (p)
        KeyCode::Char('p') if state.focus == Focus::Main => {
            let _ = state.cycle_task_priority();
        }

        // Open URL (o)
        KeyCode::Char('o') if state.focus == Focus::Main => {
            state.open_task_url();
        }

        // Refresh (r)
        KeyCode::Char('r') => {
            let _ = state.refresh_data();
            state.set_status("Refreshed");
        }

        _ => {}
    }
}

/// Handle lists view keybindings
fn handle_lists_view(state: &mut AppState, key: KeyEvent) {
    let len = state.lists.len();

    match key.code {
        // Add new list (n like Hazelnut)
        KeyCode::Char('n') => {
            state.start_add_list();
            return;
        }
        _ => {}
    }

    if len == 0 {
        return;
    }

    match key.code {
        // Navigation
        KeyCode::Char('j') | KeyCode::Down => {
            if state.list_index < len - 1 {
                state.list_index += 1;
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if state.list_index > 0 {
                state.list_index -= 1;
            }
        }
        KeyCode::Char('g') | KeyCode::Home => {
            state.list_index = 0;
        }
        KeyCode::Char('G') | KeyCode::End => {
            state.list_index = len - 1;
        }

        // Edit (e like Hazelnut)
        KeyCode::Char('e') => {
            state.start_edit_list();
        }

        // Delete (d like Hazelnut)
        KeyCode::Char('d') | KeyCode::Delete => {
            state.confirm_delete_list();
        }

        _ => {}
    }
}

/// Handle tags view keybindings
fn handle_tags_view(state: &mut AppState, key: KeyEvent) {
    let len = state.tags.len();

    match key.code {
        // Add new tag (n like Hazelnut)
        KeyCode::Char('n') => {
            state.start_add_tag();
            return;
        }
        _ => {}
    }

    if len == 0 {
        return;
    }

    match key.code {
        // Navigation
        KeyCode::Char('j') | KeyCode::Down => {
            if state.tag_index < len - 1 {
                state.tag_index += 1;
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if state.tag_index > 0 {
                state.tag_index -= 1;
            }
        }
        KeyCode::Char('g') | KeyCode::Home => {
            state.tag_index = 0;
        }
        KeyCode::Char('G') | KeyCode::End => {
            state.tag_index = len - 1;
        }

        // Edit (e like Hazelnut)
        KeyCode::Char('e') => {
            state.start_edit_tag();
        }

        // Delete (d like Hazelnut)
        KeyCode::Char('d') | KeyCode::Delete => {
            state.confirm_delete_tag();
        }

        _ => {}
    }
}

/// Handle theme picker (like Hazelnut)
fn handle_theme_picker(state: &mut AppState, key: KeyEvent) {
    let themes = Theme::all();
    let len = themes.len();

    match key.code {
        KeyCode::Esc => {
            state.mode = Mode::Normal;
        }
        KeyCode::Enter => {
            state.set_status(format!("Theme: {}", state.theme.name()));
            state.mode = Mode::Normal;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.theme_index = (state.theme_index + 1) % len;
            state.set_theme(Theme::from(themes[state.theme_index]));
        }
        KeyCode::Up | KeyCode::Char('k') => {
            state.theme_index = state.theme_index.checked_sub(1).unwrap_or(len - 1);
            state.set_theme(Theme::from(themes[state.theme_index]));
        }
        KeyCode::Home | KeyCode::Char('g') => {
            state.theme_index = 0;
            state.set_theme(Theme::from(themes[state.theme_index]));
        }
        KeyCode::End | KeyCode::Char('G') => {
            state.theme_index = len - 1;
            state.set_theme(Theme::from(themes[state.theme_index]));
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

    // Check if we're in a text input field
    let is_text_field = matches!(
        state.editor_field,
        EditorField::Title | EditorField::Description | EditorField::DueDate
    );

    match key.code {
        KeyCode::Esc => {
            state.mode = Mode::Normal;
            state.editing_task = None;
        }
        KeyCode::Enter if !key.modifiers.contains(KeyModifiers::SHIFT) => {
            // If on Tags field and cursor is on "Add new", start adding
            if state.editor_field == EditorField::Tags
                && state.editor_tag_cursor == state.tags.len()
            {
                state.start_inline_add_tag();
            } else {
                let _ = state.save_task();
            }
        }
        KeyCode::Tab => state.next_editor_field(),
        KeyCode::BackTab => state.prev_editor_field(),

        // Text input for title and description fields
        KeyCode::Char(c) if is_text_field => {
            state.input_buffer.insert(state.cursor_pos, c);
            state.cursor_pos += 1;
        }
        KeyCode::Backspace if is_text_field => {
            if state.cursor_pos > 0 {
                state.cursor_pos -= 1;
                state.input_buffer.remove(state.cursor_pos);
            }
        }
        KeyCode::Delete if is_text_field => {
            if state.cursor_pos < state.input_buffer.len() {
                state.input_buffer.remove(state.cursor_pos);
            }
        }
        KeyCode::Left if is_text_field => {
            if state.cursor_pos > 0 {
                state.cursor_pos -= 1;
            }
        }
        KeyCode::Right if is_text_field => {
            if state.cursor_pos < state.input_buffer.len() {
                state.cursor_pos += 1;
            }
        }
        KeyCode::Home if is_text_field => {
            state.cursor_pos = 0;
        }
        KeyCode::End if is_text_field => {
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
    if key.code == KeyCode::Esc {
        state.mode = Mode::Normal;
    }
}

/// Handle about dialog
fn handle_about(state: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
            state.mode = Mode::Normal;
        }
        KeyCode::Char('G') | KeyCode::Char('g') => {
            // Open GitHub repo
            let _ = open::that("https://github.com/ricardodantas/tickit");
        }
        // Handle update from about dialog
        KeyCode::Char('u') => {
            if state.update_available.is_some() {
                state.start_update();
            }
        }
        _ => {}
    }
}

/// Process pending update (called from main loop)
pub fn process_pending_update(state: &mut AppState) {
    if !state.pending_update {
        return;
    }

    let pm = crate::detect_package_manager();
    match crate::run_update(&pm) {
        Ok(()) => {
            if let Some(version) = &state.update_available {
                state.update_result = Some(format!("Updated to v{}! Restart to use.", version));
            } else {
                state.update_result = Some("Update complete! Restart to use.".to_string());
            }
            state.update_available = None;
        }
        Err(e) => {
            state.update_result = Some(format!("Update failed: {}", e));
        }
    }

    state.pending_update = false;
    if let Some(msg) = &state.update_result {
        state.set_status(msg.clone());
    }
}
