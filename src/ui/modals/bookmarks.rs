//! Bookmarks list modal overlay rendering widget.

use crate::app::AppState;
use crate::theme;
use crate::ui::helpers::centered_rect;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
};

/// Renders the bookmarks list modal dialogue.
pub fn render_bookmark_overlay(frame: &mut Frame, app: &mut AppState) {
    let area = centered_rect(70, 70, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::PINK))
        .title(Span::styled(
            format!(
                " 󰃀 Bookmarks Overlay ({}) [Enter: Jump | d: Delete | Esc: Close] ",
                app.bookmarks.len()
            ),
            Style::default()
                .fg(theme::PINK)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme::CRUST));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    if app.bookmarks.is_empty() {
        let empty_lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                " 󰃀  No Bookmarks Saved Yet! ",
                Style::default()
                    .fg(theme::PEACH)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press [b] in the file browser to bookmark any directory or file.",
                Style::default().fg(theme::TEXT),
            )),
            Line::from(Span::styled(
                "Bookmarked paths will be saved persistently across sessions.",
                Style::default().fg(theme::SUBTEXT0),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "[Esc / Shift+B] Close",
                Style::default().fg(theme::SURFACE2),
            )),
        ];
        let p = Paragraph::new(empty_lines).alignment(Alignment::Center);
        frame.render_widget(p, inner_area);
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(inner_area);

        let items: Vec<ListItem> = app
            .bookmarks
            .iter()
            .map(|path| {
                let exists = path.exists();
                let is_dir = path.is_dir();

                let (icon, kind_color) = if !exists {
                    ("󰀦 ", theme::RED)
                } else if is_dir {
                    (" ", theme::BLUE)
                } else {
                    (" ", theme::TEAL)
                };

                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.display().to_string());

                let display_line = vec![
                    Span::styled(format!("{icon} "), Style::default().fg(kind_color)),
                    Span::styled(
                        name,
                        Style::default()
                            .fg(if exists { theme::TEXT } else { theme::RED })
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" ({})", path.display()),
                        Style::default().fg(theme::SUBTEXT0),
                    ),
                ];

                ListItem::new(Line::from(display_line))
            })
            .collect();

        let list = List::new(items)
            .highlight_style(
                Style::default()
                    .bg(theme::SURFACE1)
                    .fg(theme::YELLOW)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("❯ ");

        frame.render_stateful_widget(list, chunks[0], &mut app.bookmark_list_state);

        let selected_path = app
            .bookmarks
            .get(app.bookmark_selected)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "-".to_string());

        let footer_lines = vec![
            Line::from(Span::styled(
                "──────────── Help & Quick Shortcuts ────────────",
                Style::default().fg(theme::SURFACE2),
            )),
            Line::from(vec![
                Span::styled(" Selected: ", Style::default().fg(theme::SAPPHIRE)),
                Span::styled(
                    selected_path,
                    Style::default()
                        .fg(theme::TEXT)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "  │  [Enter] Jump  [d] Delete  [Esc] Close",
                    Style::default().fg(theme::TEAL),
                ),
            ]),
        ];
        let footer_p = Paragraph::new(footer_lines).alignment(Alignment::Center);
        frame.render_widget(footer_p, chunks[1]);
    }
}
