//! Big success and cancel banner popup rendering.

use crate::ui::helpers::centered_rect;

use crate::theme;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

/// Renders the big cancellation / failure popup banner.
pub fn render_fail_banner_modal(frame: &mut Frame, msg: &str) {
    let area = centered_rect(80, 55, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(theme::RED))
        .style(Style::default().bg(theme::CRUST));

    let banner_art = r#"
███████╗ █████╗ ██╗██╗     ███████╗██████╗ 
██╔════╝██╔══██╗██║██║     ██╔════╝██╔══██╗
█████╗  ███████║██║██║     █████╗  ██║  ██║
██╔══╝  ██╔══██║██║██║     ██╔══╝  ██║  ██║
██║     ██║  ██║██║███████╗███████╗██████╔╝
╚═╝     ╚═╝  ╚═╝╚═╝╚══════╝╚══════╝╚═════╝ 
"#;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(0)])
        .split(block.inner(area));

    frame.render_widget(block, area);

    let banner_paragraph = Paragraph::new(banner_art)
        .style(Style::default().fg(theme::RED).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);

    frame.render_widget(banner_paragraph, chunks[0]);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "󰅖  TRANSFER CANCELLED OR FAILED  󰅖",
            Style::default().fg(theme::RED).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            msg,
            Style::default()
                .fg(theme::TEXT)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "The file transfer was stopped. No complete files were saved.",
            Style::default().fg(theme::PEACH),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "─────────────────────────────────────────────────────────────",
            Style::default().fg(theme::SURFACE2),
        )),
        Line::from(Span::styled(
            "[Press Enter / Esc / Space to Dismiss]",
            Style::default().fg(theme::YELLOW),
        )),
    ];

    let detail_paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(detail_paragraph, chunks[1]);
}

/// Renders the big success popup banner.
pub fn render_success_banner_modal(frame: &mut Frame, msg: &str) {
    let area = centered_rect(80, 55, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Green))
        .style(Style::default().bg(theme::CRUST));

    let banner_art = r#"
███████╗██╗   ██╗ ██████╗  ██████╗ ███████╗███████╗███████╗
██╔════╝██║   ██║██╔════╝ ██╔════╝ ██╔════╝██╔════╝██╔════╝
███████╗██║   ██║██║      ██║      █████╗  ███████╗███████╗
╚════██║██║   ██║██║      ██║      ██╔══╝  ╚════██║╚════██║
███████║╚██████╔╝╚██████╗ ╚██████╗ ███████╗███████║███████║
╚══════╝ ╚═════╝  ╚═════╝  ╚═════╝ ╚══════╝╚══════╝╚══════╝
"#;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(0)])
        .split(block.inner(area));

    frame.render_widget(block, area);

    let banner_paragraph = Paragraph::new(banner_art)
        .style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);

    frame.render_widget(banner_paragraph, chunks[0]);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "󰄬  FILE RECEIVED SUCCESSFULLY!  󰄬",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            msg,
            Style::default()
                .fg(theme::TEXT)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "File saved directly to your current file browser folder.",
            Style::default().fg(theme::SAPPHIRE),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "─────────────────────────────────────────────────────────────",
            Style::default().fg(theme::SURFACE2),
        )),
        Line::from(Span::styled(
            "[Press Enter / Esc / Space to Dismiss]",
            Style::default().fg(theme::YELLOW),
        )),
    ];

    let detail_paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(detail_paragraph, chunks[1]);
}
