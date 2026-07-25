//! Text file preview rendering pane.

use crate::app::AppState;
use crate::theme;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

/// Renders the text file preview box showing lines buffer with scroll offset.
pub fn render_text_preview(frame: &mut Frame, app: &AppState, area: Rect) {
    let scroll = app.text_preview_scroll;
    let total_lines = app.text_preview_lines.len();

    let title = format!(" 󰈙 Text Preview [Lines: {total_lines} | Scroll: Shift+W/S] ");
    let preview_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::SURFACE2))
        .title(Span::styled(
            title,
            Style::default()
                .fg(theme::TEAL)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme::BASE));

    let visible_lines: Vec<Line> = app
        .text_preview_lines
        .iter()
        .skip(scroll)
        .map(|line| Line::from(Span::styled(line, Style::default().fg(theme::TEXT))))
        .collect();

    let paragraph = Paragraph::new(visible_lines)
        .block(preview_block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
