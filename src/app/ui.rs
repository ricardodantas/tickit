//! UI rendering for the TUI

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Tabs, Wrap},
};

use super::state::{AppState, EditorField, Focus, Mode, View};
use crate::theme::Theme;

/// ASCII art logo for Tickit (used in help screen)
#[allow(dead_code)]
const LOGO: &str = r#"
████████╗██╗ ██████╗██╗  ██╗██╗████████╗
╚══██╔══╝██║██╔════╝██║ ██╔╝██║╚══██╔══╝
   ██║   ██║██║     █████╔╝ ██║   ██║   
   ██║   ██║██║     ██╔═██╗ ██║   ██║   
   ██║   ██║╚██████╗██║  ██╗██║   ██║   
   ╚═╝   ╚═╝ ╚═════╝╚═╝  ╚═╝╚═╝   ╚═╝   
"#;

/// Tickit icon
const ICON: &str = "✓";

/// Render the entire UI
pub fn render(frame: &mut Frame, state: &AppState) {
    let colors = state.theme.colors();

    // Set background
    let area = frame.area();
    let bg_block = Block::default().style(Style::default().bg(colors.bg));
    frame.render_widget(bg_block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tabs
            Constraint::Min(0),    // Main content
            Constraint::Length(1), // Status bar
        ])
        .split(area);

    render_tabs(frame, state, chunks[0]);
    render_main(frame, state, chunks[1]);
    render_status_bar(frame, state, chunks[2]);

    // Render popups/dialogs
    if state.show_help || state.mode == Mode::Help {
        render_help_popup(frame, state);
    }

    if state.mode == Mode::ThemePicker {
        render_theme_picker(frame, state);
    }

    if state.mode == Mode::Confirm {
        render_confirm_dialog(frame, state);
    }

    if matches!(state.mode, Mode::AddTask | Mode::EditTask) {
        render_task_editor(frame, state);
    }

    if matches!(state.mode, Mode::AddList | Mode::EditList) {
        render_simple_editor(frame, state, "List");
    }

    if matches!(state.mode, Mode::AddTag | Mode::EditTag) {
        render_simple_editor(frame, state, "Tag");
    }
}

/// Render the tab bar
fn render_tabs(frame: &mut Frame, state: &AppState, area: Rect) {
    let colors = state.theme.colors();

    let titles: Vec<Line> = View::all()
        .iter()
        .enumerate()
        .map(|(i, view)| {
            let style = if *view == state.view {
                colors.tab_active()
            } else {
                colors.tab()
            };
            Line::from(vec![
                Span::styled(format!(" {} ", i + 1), colors.key_hint()),
                Span::styled(format!("{} {} ", view.icon(), view.name()), style),
            ])
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .title(format!(" {} Tickit ", ICON))
                .title_style(colors.logo_style_primary())
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(colors.block()),
        )
        .highlight_style(colors.tab_active())
        .select(View::all().iter().position(|v| *v == state.view).unwrap_or(0));

    frame.render_widget(tabs, area);
}

/// Render the main content area
fn render_main(frame: &mut Frame, state: &AppState, area: Rect) {
    match state.view {
        View::Tasks => render_tasks_view(frame, state, area),
        View::Lists => render_lists_view(frame, state, area),
        View::Tags => render_tags_view(frame, state, area),
    }
}

/// Render the tasks view with sidebar
fn render_tasks_view(frame: &mut Frame, state: &AppState, area: Rect) {
    let colors = state.theme.colors();

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(24), // Sidebar
            Constraint::Min(0),     // Task list
        ])
        .split(area);

    // Sidebar (lists)
    let sidebar_focused = state.focus == Focus::Sidebar;
    let sidebar_style = if sidebar_focused {
        colors.block_focus()
    } else {
        colors.block()
    };

    let mut list_items: Vec<ListItem> = Vec::new();

    // "All" item
    let all_selected = state.list_index == 0;
    let all_style = if all_selected {
        colors.selected()
    } else {
        colors.text()
    };
    let task_count = state.db.get_total_task_count(!state.show_completed).unwrap_or(0);
    list_items.push(ListItem::new(Line::from(vec![
        Span::styled("  📚 ", all_style),
        Span::styled("All", all_style),
        Span::styled(format!(" ({})", task_count), colors.text_muted()),
    ])));

    // Lists
    for (i, list) in state.lists.iter().enumerate() {
        let selected = state.list_index == i + 1;
        let style = if selected {
            colors.selected()
        } else {
            colors.text()
        };
        let count = state.db.get_task_count(list.id, !state.show_completed).unwrap_or(0);
        list_items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("  {} ", list.icon), style),
            Span::styled(&list.name, style),
            Span::styled(format!(" ({})", count), colors.text_muted()),
        ])));
    }

    let sidebar = List::new(list_items)
        .block(
            Block::default()
                .title(" Lists ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(sidebar_style),
        );

    frame.render_widget(sidebar, chunks[0]);

    // Task list
    let main_focused = state.focus == Focus::Main;
    let main_style = if main_focused {
        colors.block_focus()
    } else {
        colors.block()
    };

    let list_name = if state.list_index == 0 {
        "All Tasks".to_string()
    } else {
        state.lists.get(state.list_index - 1)
            .map(|l| l.name.clone())
            .unwrap_or_else(|| "Tasks".to_string())
    };

    let task_items: Vec<ListItem> = state.tasks
        .iter()
        .enumerate()
        .map(|(i, task)| {
            let selected = i == state.task_index;
            let base_style = if selected {
                colors.selected()
            } else {
                colors.text()
            };

            let checkbox = if task.completed { "☑" } else { "☐" };
            let checkbox_style = if task.completed {
                colors.text_success()
            } else {
                colors.text_muted()
            };

            let title_style = if task.completed {
                base_style.add_modifier(Modifier::CROSSED_OUT).fg(colors.fg_muted)
            } else {
                base_style
            };

            let priority_style = colors.priority_style(task.priority);
            let priority_icon = task.priority.icon();

            let mut spans = vec![
                Span::styled(format!(" {} ", checkbox), checkbox_style),
                Span::styled(format!("{} ", priority_icon), priority_style),
                Span::styled(&task.title, title_style),
            ];

            // Add URL indicator
            if task.url.is_some() {
                spans.push(Span::styled(" 🔗", colors.text_info()));
            }

            // Add tag indicators
            if !task.tag_ids.is_empty() {
                let tag_count = task.tag_ids.len();
                spans.push(Span::styled(format!(" [{}]", tag_count), colors.text_secondary()));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let show_status = if state.show_completed { "" } else { " (hiding completed)" };
    let tasks_block = List::new(task_items)
        .block(
            Block::default()
                .title(format!(" {} {} ", list_name, show_status))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(main_style),
        );

    frame.render_widget(tasks_block, chunks[1]);
}

/// Render the lists view
fn render_lists_view(frame: &mut Frame, state: &AppState, area: Rect) {
    let colors = state.theme.colors();

    let list_items: Vec<ListItem> = state.lists
        .iter()
        .enumerate()
        .map(|(i, list)| {
            let selected = i + 1 == state.list_index || (i == 0 && state.list_index == 0);
            let style = if selected {
                colors.selected()
            } else {
                colors.text()
            };

            let inbox_marker = if list.is_inbox { " (default)" } else { "" };

            ListItem::new(Line::from(vec![
                Span::styled(format!("  {} ", list.icon), style),
                Span::styled(&list.name, style),
                Span::styled(inbox_marker, colors.text_muted()),
            ]))
        })
        .collect();

    let lists = List::new(list_items)
        .block(
            Block::default()
                .title(" Lists ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(colors.block_focus()),
        );

    frame.render_widget(lists, area);
}

/// Render the tags view
fn render_tags_view(frame: &mut Frame, state: &AppState, area: Rect) {
    let colors = state.theme.colors();

    let tag_items: Vec<ListItem> = state.tags
        .iter()
        .enumerate()
        .map(|(i, tag)| {
            let selected = i == state.tag_index;
            let style = if selected {
                colors.selected()
            } else {
                colors.text()
            };

            // Parse hex color for tag
            let tag_color = parse_hex_color(&tag.color).unwrap_or(colors.accent);

            ListItem::new(Line::from(vec![
                Span::styled("  ● ", Style::default().fg(tag_color)),
                Span::styled(&tag.name, style),
            ]))
        })
        .collect();

    let content = if tag_items.is_empty() {
        List::new(vec![ListItem::new(Span::styled(
            "  No tags yet. Press 'a' to add one.",
            colors.text_muted(),
        ))])
    } else {
        List::new(tag_items)
    };

    let tags = content.block(
        Block::default()
            .title(" Tags ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(colors.block_focus()),
    );

    frame.render_widget(tags, area);
}

/// Render the status bar
fn render_status_bar(frame: &mut Frame, state: &AppState, area: Rect) {
    let colors = state.theme.colors();

    let status_text = if let Some(msg) = &state.status_message {
        msg.clone()
    } else {
        format!(
            " {} {} │ ? Help │ Ctrl+T Theme │ q Quit",
            ICON,
            state.theme.name()
        )
    };

    let status = Paragraph::new(status_text)
        .style(colors.text_muted())
        .alignment(Alignment::Left);

    frame.render_widget(status, area);
}

/// Render help popup
fn render_help_popup(frame: &mut Frame, state: &AppState) {
    let colors = state.theme.colors();
    let area = centered_rect(60, 70, frame.area());

    frame.render_widget(Clear, area);

    let help_text = vec![
        Line::from(vec![
            Span::styled("Navigation", colors.text_primary().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  j/↓  ", colors.key_hint()),
            Span::raw("Move down"),
        ]),
        Line::from(vec![
            Span::styled("  k/↑  ", colors.key_hint()),
            Span::raw("Move up"),
        ]),
        Line::from(vec![
            Span::styled("  h/←  ", colors.key_hint()),
            Span::raw("Focus sidebar"),
        ]),
        Line::from(vec![
            Span::styled("  l/→  ", colors.key_hint()),
            Span::raw("Focus main"),
        ]),
        Line::from(vec![
            Span::styled("  Tab  ", colors.key_hint()),
            Span::raw("Next view"),
        ]),
        Line::from(vec![
            Span::styled("  1-3  ", colors.key_hint()),
            Span::raw("Switch to view"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Tasks", colors.text_primary().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  a/n  ", colors.key_hint()),
            Span::raw("Add new task"),
        ]),
        Line::from(vec![
            Span::styled("  e    ", colors.key_hint()),
            Span::raw("Edit task"),
        ]),
        Line::from(vec![
            Span::styled("  d    ", colors.key_hint()),
            Span::raw("Delete task"),
        ]),
        Line::from(vec![
            Span::styled("  Space/x  ", colors.key_hint()),
            Span::raw("Toggle complete"),
        ]),
        Line::from(vec![
            Span::styled("  p    ", colors.key_hint()),
            Span::raw("Cycle priority"),
        ]),
        Line::from(vec![
            Span::styled("  o    ", colors.key_hint()),
            Span::raw("Open URL"),
        ]),
        Line::from(vec![
            Span::styled("  c    ", colors.key_hint()),
            Span::raw("Toggle show completed"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("General", colors.text_primary().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Ctrl+T  ", colors.key_hint()),
            Span::raw("Theme picker"),
        ]),
        Line::from(vec![
            Span::styled("  r    ", colors.key_hint()),
            Span::raw("Refresh"),
        ]),
        Line::from(vec![
            Span::styled("  ?    ", colors.key_hint()),
            Span::raw("Toggle help"),
        ]),
        Line::from(vec![
            Span::styled("  q    ", colors.key_hint()),
            Span::raw("Quit"),
        ]),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Help ")
                .title_style(colors.text_primary())
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(colors.block_focus()),
        )
        .style(colors.text())
        .wrap(Wrap { trim: false });

    frame.render_widget(help, area);
}

/// Render theme picker
fn render_theme_picker(frame: &mut Frame, state: &AppState) {
    let colors = state.theme.colors();
    let area = centered_rect(40, 60, frame.area());

    frame.render_widget(Clear, area);

    let theme_items: Vec<ListItem> = Theme::all()
        .iter()
        .enumerate()
        .map(|(i, theme)| {
            let selected = i == state.theme_index;
            let style = if selected {
                colors.selected()
            } else {
                colors.text()
            };
            let marker = if selected { "► " } else { "  " };
            ListItem::new(format!("{}{}", marker, theme.name())).style(style)
        })
        .collect();

    let themes = List::new(theme_items)
        .block(
            Block::default()
                .title(" Theme ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(colors.block_focus()),
        );

    frame.render_widget(themes, area);
}

/// Render confirmation dialog
fn render_confirm_dialog(frame: &mut Frame, state: &AppState) {
    let colors = state.theme.colors();
    let area = centered_rect(50, 20, frame.area());

    frame.render_widget(Clear, area);

    let text = vec![
        Line::from(""),
        Line::from(state.confirm_message.as_str()),
        Line::from(""),
        Line::from(vec![
            Span::styled("  y  ", colors.key_hint()),
            Span::raw("Yes  "),
            Span::styled("  n  ", colors.key_hint()),
            Span::raw("No"),
        ]),
    ];

    let dialog = Paragraph::new(text)
        .block(
            Block::default()
                .title(" Confirm ")
                .title_style(colors.text_warning())
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(colors.block_focus()),
        )
        .alignment(Alignment::Center);

    frame.render_widget(dialog, area);
}

/// Render task editor
fn render_task_editor(frame: &mut Frame, state: &AppState) {
    let colors = state.theme.colors();
    let area = centered_rect(60, 70, frame.area());

    frame.render_widget(Clear, area);

    let title = if state.mode == Mode::AddTask {
        " New Task "
    } else {
        " Edit Task "
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title input
            Constraint::Length(3), // Description input
            Constraint::Length(3), // Priority
            Constraint::Length(3), // List
            Constraint::Min(6),    // Tags (expanded)
            Constraint::Length(1), // Help
        ])
        .split(area);

    // Title field
    let title_focused = state.editor_field == EditorField::Title;
    let title_style = if title_focused {
        colors.block_focus()
    } else {
        colors.block()
    };
    
    // Show the correct title value depending on focus
    let title_display = if title_focused {
        state.input_buffer.as_str()
    } else {
        state.editor_title_buffer.as_str()
    };
    
    let title_input = Paragraph::new(title_display)
        .block(
            Block::default()
                .title(" Title ")
                .borders(Borders::ALL)
                .border_style(title_style),
        );
    frame.render_widget(title_input, chunks[0]);
    
    if title_focused && !state.editor_adding_tag {
        frame.set_cursor_position((
            chunks[0].x + state.cursor_pos as u16 + 1,
            chunks[0].y + 1,
        ));
    }

    // Description field
    let desc_focused = state.editor_field == EditorField::Description;
    let desc_style = if desc_focused {
        colors.block_focus()
    } else {
        colors.block()
    };
    let desc_display = if desc_focused {
        state.input_buffer.as_str()
    } else {
        state.editor_description_buffer.as_str()
    };
    let desc_input = Paragraph::new(desc_display)
        .block(
            Block::default()
                .title(" Description (optional) ")
                .borders(Borders::ALL)
                .border_style(desc_style),
        );
    frame.render_widget(desc_input, chunks[1]);
    
    if desc_focused && !state.editor_adding_tag {
        frame.set_cursor_position((
            chunks[1].x + state.cursor_pos as u16 + 1,
            chunks[1].y + 1,
        ));
    }

    // Priority field
    let priority_focused = state.editor_field == EditorField::Priority;
    let priority_style = if priority_focused {
        colors.block_focus()
    } else {
        colors.block()
    };
    let priority_text = format!(
        "{} {}",
        state.editor_priority.icon(),
        state.editor_priority.name()
    );
    let priority_input = Paragraph::new(priority_text)
        .block(
            Block::default()
                .title(" Priority (j/k to change) ")
                .borders(Borders::ALL)
                .border_style(priority_style),
        );
    frame.render_widget(priority_input, chunks[2]);

    // List field
    let list_focused = state.editor_field == EditorField::List;
    let list_style = if list_focused {
        colors.block_focus()
    } else {
        colors.block()
    };
    let list_text = state.lists
        .get(state.editor_list_index)
        .map(|l| format!("{} {}", l.icon, l.name))
        .unwrap_or_else(|| "📥 Inbox".to_string());
    let list_input = Paragraph::new(list_text)
        .block(
            Block::default()
                .title(" List (j/k to change) ")
                .borders(Borders::ALL)
                .border_style(list_style),
        );
    frame.render_widget(list_input, chunks[3]);

    // Tags field - show as selectable list
    let tags_focused = state.editor_field == EditorField::Tags;
    let tags_style = if tags_focused {
        colors.block_focus()
    } else {
        colors.block()
    };

    let mut tag_items: Vec<ListItem> = state.tags
        .iter()
        .enumerate()
        .map(|(i, tag)| {
            let is_selected = state.editor_tag_indices.contains(&i);
            let is_cursor = tags_focused && i == state.editor_tag_cursor;
            
            let checkbox = if is_selected { "☑" } else { "☐" };
            let marker = if is_cursor { "► " } else { "  " };
            
            let style = if is_cursor {
                colors.selected()
            } else {
                colors.text()
            };
            
            let tag_color = parse_hex_color(&tag.color).unwrap_or(colors.accent);
            
            ListItem::new(Line::from(vec![
                Span::styled(marker, style),
                Span::styled(format!("{} ", checkbox), if is_selected { colors.text_success() } else { colors.text_muted() }),
                Span::styled("● ", Style::default().fg(tag_color)),
                Span::styled(&tag.name, style),
            ]))
        })
        .collect();

    // Add "Add new tag" option
    let add_new_cursor = tags_focused && state.editor_tag_cursor == state.tags.len();
    let add_style = if add_new_cursor { colors.selected() } else { colors.text_muted() };
    let add_marker = if add_new_cursor { "► " } else { "  " };
    
    if state.editor_adding_tag {
        // Show input field for new tag
        tag_items.push(ListItem::new(Line::from(vec![
            Span::styled(add_marker, add_style),
            Span::styled("+ ", colors.text_success()),
            Span::styled(&state.editor_new_tag_buffer, colors.text()),
            Span::styled("_", colors.text_primary()), // cursor
        ])));
    } else {
        tag_items.push(ListItem::new(Line::from(vec![
            Span::styled(add_marker, add_style),
            Span::styled("+ Add new tag...", add_style),
        ])));
    }

    let tags_list = List::new(tag_items)
        .block(
            Block::default()
                .title(" Tags (Space: toggle, n: new) ")
                .borders(Borders::ALL)
                .border_style(tags_style),
        );
    frame.render_widget(tags_list, chunks[4]);

    // Help text
    let help_text = if state.editor_adding_tag {
        "Enter: save tag │ Esc: cancel"
    } else {
        "Tab: next field │ Enter: save │ Esc: cancel"
    };
    let help = Paragraph::new(help_text)
        .style(colors.text_muted())
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[5]);

    // Outer block
    let outer = Block::default()
        .title(title)
        .title_style(colors.text_primary())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(colors.block_focus())
        .style(Style::default().bg(colors.bg));
    frame.render_widget(outer, area);
}

/// Render simple name editor (for lists and tags)
fn render_simple_editor(frame: &mut Frame, state: &AppState, item_type: &str) {
    let colors = state.theme.colors();
    let area = centered_rect(50, 25, frame.area());

    frame.render_widget(Clear, area);

    let title = if state.mode == Mode::AddList || state.mode == Mode::AddTag {
        format!(" New {} ", item_type)
    } else {
        format!(" Edit {} ", item_type)
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Name input
            Constraint::Min(1),    // Spacer
            Constraint::Length(1), // Help
        ])
        .split(area);

    let input = Paragraph::new(state.input_buffer.as_str())
        .block(
            Block::default()
                .title(" Name ")
                .borders(Borders::ALL)
                .border_style(colors.block_focus()),
        );
    frame.render_widget(input, chunks[0]);

    frame.set_cursor_position((
        chunks[0].x + state.cursor_pos as u16 + 1,
        chunks[0].y + 1,
    ));

    let help = Paragraph::new("Enter: save │ Esc: cancel")
        .style(colors.text_muted())
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[2]);

    let outer = Block::default()
        .title(title)
        .title_style(colors.text_primary())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(colors.block_focus())
        .style(Style::default().bg(colors.bg));
    frame.render_widget(outer, area);
}

/// Create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Parse a hex color string
fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}
