//! Dialogue modals for new folder creation and item renaming.

use crate::app::AppState;
use crate::theme;
use crate::ui::helpers::centered_rect;
use ratatui::{
    Frame,
    layout::Alignment,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

/// Renders the new folder input modal dialogue.
pub fn render_new_folder_modal(frame: &mut Frame, app: &AppState) {
    let area = centered_rect(50, 25, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::MAUVE))
        .title(Span::styled(
            "  Create New Directory [Enter: Create | Esc: Cancel] ",
            Style::default()
                .fg(theme::MAUVE)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme::CRUST));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Enter folder name:",
            Style::default().fg(theme::SAPPHIRE),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "> ",
                Style::default()
                    .fg(theme::YELLOW)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                &app.new_folder_input,
                Style::default()
                    .fg(theme::TEXT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("█", Style::default().fg(theme::MAUVE)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "─────────────────────────────────────────",
            Style::default().fg(theme::SURFACE2),
        )),
        Line::from(Span::styled(
            "[Enter] Confirm & Create  │  [Esc] Cancel",
            Style::default().fg(theme::SUBTEXT0),
        )),
    ];

    let p = Paragraph::new(text).alignment(Alignment::Center);
    frame.render_widget(p, inner_area);
}

/// Renders the item rename input modal dialogue.
pub fn render_rename_modal(frame: &mut Frame, app: &AppState) {
    let area = centered_rect(50, 25, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::TEAL))
        .title(Span::styled(
            " 󰏫 Rename Item [Enter: Confirm | Esc: Cancel] ",
            Style::default()
                .fg(theme::TEAL)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme::CRUST));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Enter new name:",
            Style::default().fg(theme::SAPPHIRE),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "> ",
                Style::default()
                    .fg(theme::YELLOW)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                &app.rename_input,
                Style::default()
                    .fg(theme::TEXT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("█", Style::default().fg(theme::TEAL)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "─────────────────────────────────────────",
            Style::default().fg(theme::SURFACE2),
        )),
        Line::from(Span::styled(
            "[Enter] Confirm Rename  │  [Esc] Cancel",
            Style::default().fg(theme::SUBTEXT0),
        )),
    ];

    let p = Paragraph::new(text).alignment(Alignment::Center);
    frame.render_widget(p, inner_area);
}
