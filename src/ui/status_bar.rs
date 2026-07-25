//! Bottom segmented `btop`-style status bar rendering widget.

use crate::app::AppState;
use crate::theme;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

/// Renders the bottom segmented status bar containing hotkey shortcuts, error messages, and progress indicators.
pub fn render_status_bar(frame: &mut Frame, app: &AppState, area: Rect) {
    if let Some((ref msg, _)) = app.fail_banner {
        let p = Paragraph::new(format!(" CANCELLED: {msg}")).style(
            Style::default()
                .bg(theme::RED)
                .fg(theme::CRUST)
                .add_modifier(Modifier::BOLD),
        );
        frame.render_widget(p, area);
        return;
    }

    if let Some((ref err, _)) = app.error {
        let p = Paragraph::new(format!(" ERROR: {err}")).style(
            Style::default()
                .bg(theme::RED)
                .fg(theme::CRUST)
                .add_modifier(Modifier::BOLD),
        );
        frame.render_widget(p, area);
        return;
    }

    if let Some((ref msg, _)) = app.status_message {
        let p = Paragraph::new(format!(" INFO: {msg}")).style(
            Style::default()
                .bg(theme::YELLOW)
                .fg(theme::CRUST)
                .add_modifier(Modifier::BOLD),
        );
        frame.render_widget(p, area);
        return;
    }

    let incoming_count = app.incoming_requests.len();
    let tagged_count = app.tagged_files.len();

    let spans = if tagged_count > 0 {
        vec![
            Span::styled(
                " [c] ",
                Style::default()
                    .bg(theme::SURFACE1)
                    .fg(theme::GREEN)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " Copy ",
                Style::default().bg(theme::SURFACE0).fg(theme::TEXT),
            ),
            Span::styled(
                "│",
                Style::default().bg(theme::SURFACE0).fg(theme::SURFACE2),
            ),
            Span::styled(
                " [x] ",
                Style::default()
                    .bg(theme::SURFACE1)
                    .fg(theme::PEACH)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " Cut ",
                Style::default().bg(theme::SURFACE0).fg(theme::TEXT),
            ),
            Span::styled(
                "│",
                Style::default().bg(theme::SURFACE0).fg(theme::SURFACE2),
            ),
            Span::styled(
                " [t] ",
                Style::default()
                    .bg(theme::SURFACE1)
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " LocalSend ",
                Style::default().bg(theme::SURFACE0).fg(Color::Green),
            ),
            Span::styled(
                "│",
                Style::default().bg(theme::SURFACE0).fg(theme::SURFACE2),
            ),
            Span::styled(
                " [p] ",
                Style::default()
                    .bg(theme::SURFACE1)
                    .fg(theme::SAPPHIRE)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " Paste ",
                Style::default().bg(theme::SURFACE0).fg(theme::TEXT),
            ),
            Span::styled(
                "│",
                Style::default().bg(theme::SURFACE0).fg(theme::SURFACE2),
            ),
            Span::styled(
                format!(" ({tagged_count} Selected) "),
                Style::default()
                    .bg(theme::SURFACE0)
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]
    } else {
        let mut default_spans = vec![
            Span::styled(
                " [Space] ",
                Style::default()
                    .bg(theme::SURFACE1)
                    .fg(theme::PEACH)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " Select ",
                Style::default().bg(theme::SURFACE0).fg(theme::TEXT),
            ),
            Span::styled(
                " (0) ",
                Style::default()
                    .bg(theme::SURFACE0)
                    .fg(theme::SUBTEXT0)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "│",
                Style::default().bg(theme::SURFACE0).fg(theme::SURFACE2),
            ),
            Span::styled(
                " [v] ",
                Style::default()
                    .bg(theme::SURFACE1)
                    .fg(theme::LAVENDER)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " Select All ",
                Style::default().bg(theme::SURFACE0).fg(theme::TEXT),
            ),
            Span::styled(
                "│",
                Style::default().bg(theme::SURFACE0).fg(theme::SURFACE2),
            ),
            Span::styled(
                " [Enter] ",
                Style::default()
                    .bg(theme::SURFACE1)
                    .fg(theme::SAPPHIRE)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " Open ",
                Style::default().bg(theme::SURFACE0).fg(theme::TEXT),
            ),
            Span::styled(
                "│",
                Style::default().bg(theme::SURFACE0).fg(theme::SURFACE2),
            ),
            Span::styled(
                " [b] ",
                Style::default()
                    .bg(theme::SURFACE1)
                    .fg(theme::PINK)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " Bookmark ",
                Style::default().bg(theme::SURFACE0).fg(theme::TEXT),
            ),
            Span::styled(
                "│",
                Style::default().bg(theme::SURFACE0).fg(theme::SURFACE2),
            ),
            Span::styled(
                " [Shift+B] ",
                Style::default()
                    .bg(theme::SURFACE1)
                    .fg(theme::MAUVE)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " Bookmarks ",
                Style::default().bg(theme::SURFACE0).fg(theme::TEXT),
            ),
            Span::styled(
                "│",
                Style::default().bg(theme::SURFACE0).fg(theme::SURFACE2),
            ),
            Span::styled(
                " [Shift+W/S] ",
                Style::default()
                    .bg(theme::SURFACE1)
                    .fg(theme::TEAL)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " Scroll Preview ",
                Style::default().bg(theme::SURFACE0).fg(theme::TEXT),
            ),
            Span::styled(
                "│",
                Style::default().bg(theme::SURFACE0).fg(theme::SURFACE2),
            ),
            Span::styled(
                " [n] ",
                Style::default()
                    .bg(theme::SURFACE1)
                    .fg(theme::MAUVE)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " New Folder ",
                Style::default().bg(theme::SURFACE0).fg(theme::TEXT),
            ),
            Span::styled(
                "│",
                Style::default().bg(theme::SURFACE0).fg(theme::SURFACE2),
            ),
            Span::styled(
                " [r] ",
                Style::default()
                    .bg(theme::SURFACE1)
                    .fg(theme::TEAL)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " Rename ",
                Style::default().bg(theme::SURFACE0).fg(theme::TEXT),
            ),
            Span::styled(
                "│",
                Style::default().bg(theme::SURFACE0).fg(theme::SURFACE2),
            ),
            Span::styled(
                " [1/2] ",
                Style::default()
                    .bg(theme::SURFACE1)
                    .fg(theme::YELLOW)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " Pane 1/2 ",
                Style::default().bg(theme::SURFACE0).fg(theme::TEXT),
            ),
            Span::styled(
                "│",
                Style::default().bg(theme::SURFACE0).fg(theme::SURFACE2),
            ),
            Span::styled(
                " [Tab] ",
                Style::default()
                    .bg(theme::SURFACE1)
                    .fg(theme::SAPPHIRE)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " Switch Pane ",
                Style::default().bg(theme::SURFACE0).fg(theme::TEXT),
            ),
            Span::styled(
                "│",
                Style::default().bg(theme::SURFACE0).fg(theme::SURFACE2),
            ),
        ];

        if incoming_count > 0 {
            default_spans.push(Span::styled(
                format!(" 󰏭 INCOMING REQUEST ({incoming_count}) [t] "),
                Style::default()
                    .bg(Color::Green)
                    .fg(theme::CRUST)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            default_spans.push(Span::styled(
                " [t] ",
                Style::default()
                    .bg(theme::SURFACE1)
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ));
            default_spans.push(Span::styled(
                " LocalSend Window ",
                Style::default().bg(theme::SURFACE0).fg(Color::Green),
            ));
        }
        default_spans
    };

    let p = Paragraph::new(Line::from(spans)).style(Style::default().bg(theme::SURFACE0));
    frame.render_widget(p, area);
}
