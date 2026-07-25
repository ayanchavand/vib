//! File browser pane and entry detail rendering widgets.

use super::preview::render_text_preview;
use crate::app::AppState;
use crate::fs::{EntryKind, FileEntry};
use crate::theme;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
};

/// Renders the main file explorer tab (dual pane or single pane + details/preview split).
pub fn render_files_tab(frame: &mut Frame, app: &mut AppState, area: Rect) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    if app.panes.len() == 2 {
        let pane_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_chunks[0]);

        render_pane(frame, app, pane_chunks[0], 0);
        render_pane(frame, app, pane_chunks[1], 1);
    } else {
        render_pane(frame, app, main_chunks[0], 0);
    }

    let selected_entry = app.entries.get(app.selected);
    let is_text_file = selected_entry
        .map(|e| e.kind == EntryKind::Text || e.kind == EntryKind::Unknown)
        .unwrap_or(false);

    if is_text_file {
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(main_chunks[1]);

        render_selected_details(frame, app, right_chunks[0], selected_entry);
        render_text_preview(frame, app, right_chunks[1]);
    } else {
        render_selected_details(frame, app, main_chunks[1], selected_entry);
    }
}

/// Renders a single browsing pane file list with item icons and selection indicators.
pub fn render_pane(frame: &mut Frame, app: &AppState, area: Rect, pane_idx: usize) {
    let pane = match app.panes.get(pane_idx) {
        Some(p) => p,
        None => return,
    };

    let is_active = app.active_pane == pane_idx;
    let has_any_tagged = !app.tagged_files.is_empty();

    let items: Vec<ListItem> = pane
        .entries
        .iter()
        .map(|entry| {
            let is_tagged = app.tagged_files.contains(&entry.path);

            let (kind_color, icon) = match entry.kind {
                EntryKind::Directory => (theme::BLUE, " "),
                EntryKind::Text => (theme::TEAL, "󰈙 "),
                EntryKind::Image => (theme::MAUVE, " "),
                EntryKind::Audio => (theme::PINK, " "),
                EntryKind::Video => (theme::PEACH, " "),
                EntryKind::Binary => (theme::YELLOW, "󰅪 "),
                EntryKind::Unknown => (theme::SUBTEXT0, " "),
            };

            let check_span = if is_tagged {
                Some(Span::styled(
                    "󰄲 ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ))
            } else if has_any_tagged {
                Some(Span::styled("󰄱 ", Style::default().fg(theme::OVERLAY0)))
            } else {
                None
            };

            let icon_span = Span::styled(icon, Style::default().fg(kind_color));

            let name_style = if is_tagged {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else if entry.kind == EntryKind::Directory {
                Style::default()
                    .fg(theme::BLUE)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(kind_color)
            };

            let name_span = Span::styled(&entry.name, name_style);

            let mut line_spans = Vec::new();
            if let Some(cs) = check_span {
                line_spans.push(cs);
            }
            line_spans.push(icon_span);
            line_spans.push(name_span);

            ListItem::new(Line::from(line_spans))
        })
        .collect();

    let border_color = if is_active {
        theme::MAUVE
    } else {
        theme::SURFACE2
    };

    let title_prefix = if is_active {
        format!(
            " 󰉋 [Pane {}*] {} ",
            pane_idx + 1,
            pane.current_path.display()
        )
    } else {
        format!(
            " 󰉋 [Pane {}] {} ",
            pane_idx + 1,
            pane.current_path.display()
        )
    };

    let title_style = if is_active {
        Style::default()
            .fg(theme::MAUVE)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme::SUBTEXT0)
    };

    let list_block = Block::default()
        .borders(Borders::ALL)
        .border_type(if is_active {
            BorderType::Double
        } else {
            BorderType::Rounded
        })
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(title_prefix, title_style))
        .style(Style::default().bg(theme::BASE));

    let highlight_style = if is_active {
        Style::default()
            .bg(theme::SURFACE1)
            .fg(theme::YELLOW)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().bg(theme::SURFACE0).fg(theme::TEXT)
    };

    let list = List::new(items)
        .block(list_block)
        .highlight_style(highlight_style)
        .highlight_symbol(if is_active { "❯ " } else { "  " });

    let mut state = if is_active {
        app.list_state.clone()
    } else {
        pane.list_state.clone()
    };

    frame.render_stateful_widget(list, area, &mut state);
}

/// Renders the selected entry detail panel (file name, type, size, modified time, tagged status).
pub fn render_selected_details(
    frame: &mut Frame,
    app: &AppState,
    area: Rect,
    selected_entry: Option<&FileEntry>,
) {
    let preview_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::SURFACE2))
        .title(Span::styled(
            " 󰋽 Selected Details ",
            Style::default()
                .fg(theme::LAVENDER)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme::BASE));

    let detail_text = if let Some(entry) = selected_entry {
        let size_str = entry
            .size
            .map(crate::utils::format_size)
            .unwrap_or_else(|| "-".to_string());
        let mod_str = entry
            .modified
            .and_then(crate::utils::format_modified_time)
            .unwrap_or_else(|| "-".to_string());

        let (kind_str, kind_color) = match entry.kind {
            EntryKind::Directory => ("Directory", theme::BLUE),
            EntryKind::Text => ("Text Document", theme::TEAL),
            EntryKind::Image => ("Image", theme::MAUVE),
            EntryKind::Audio => ("Audio File", theme::PINK),
            EntryKind::Video => ("Video File", theme::PEACH),
            EntryKind::Binary => ("Binary / Executable", theme::YELLOW),
            EntryKind::Unknown => ("Unknown / Other", theme::SUBTEXT0),
        };

        let is_tagged = if app.tagged_files.contains(&entry.path) {
            "Yes"
        } else {
            "No"
        };

        vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(theme::SAPPHIRE)),
                Span::styled(
                    &entry.name,
                    Style::default().fg(kind_color).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Type: ", Style::default().fg(theme::SAPPHIRE)),
                Span::styled(kind_str, Style::default().fg(kind_color)),
            ]),
            Line::from(vec![
                Span::styled("Size: ", Style::default().fg(theme::SAPPHIRE)),
                Span::styled(size_str, Style::default().fg(theme::TEXT)),
            ]),
            Line::from(vec![
                Span::styled("Modified: ", Style::default().fg(theme::SAPPHIRE)),
                Span::styled(mod_str, Style::default().fg(theme::TEXT)),
            ]),
            Line::from(vec![
                Span::styled("Selected: ", Style::default().fg(theme::SAPPHIRE)),
                Span::styled(
                    is_tagged,
                    Style::default().fg(if is_tagged == "Yes" {
                        Color::Green
                    } else {
                        theme::SUBTEXT0
                    }),
                ),
            ]),
        ]
    } else {
        vec![Line::from(Span::styled(
            "No items in directory",
            Style::default().fg(theme::SUBTEXT0),
        ))]
    };

    let paragraph = Paragraph::new(detail_text).block(preview_block);
    frame.render_widget(paragraph, area);
}
