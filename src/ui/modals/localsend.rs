//! LocalSend overlay dialogue and peer device picker modals.

use crate::app::{AppState, LocalSendModalState};
use crate::theme;
use crate::ui::helpers::centered_rect;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Gauge, List, ListItem, Paragraph},
};

/// Renders the main LocalSend overlay window (Menu, Send, and Receive modes).
pub fn render_localsend_overlay(frame: &mut Frame, app: &AppState) {
    let area = centered_rect(70, 75, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Green))
        .title(Span::styled(
            " 󰏭 LocalSend Overlay [Press Esc/t to Close] ",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme::CRUST));

    let ascii_art = r#"
 _                    _  ____                 _ 
| |    ___   ___ __ _| |/ ___|  ___ _ __   __| |
| |   / _ \ / __/ _` | |\___ \ / _ \ '_ \ / _` |
| |__| (_) | (_| (_| | | ___) |  __/ | | | (_| |
|_____\___/ \___\__,_|_||____/ \___|_| |_|\__,_|
"#;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(7), Constraint::Min(0)])
        .split(block.inner(area));

    frame.render_widget(block, area);

    let ascii_paragraph = Paragraph::new(ascii_art)
        .style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);

    frame.render_widget(ascii_paragraph, chunks[0]);

    if app.localsend_modal == LocalSendModalState::Menu {
        let send_selected = app.localsend_modal_selected == 0;
        let recv_selected = app.localsend_modal_selected == 1;
        let has_tagged = !app.tagged_files.is_empty();

        let send_text = if !has_tagged {
            "  [Disabled] Send Files (0 selected)  ".to_string()
        } else if send_selected {
            format!("> Send Files ({} selected) <", app.tagged_files.len())
        } else {
            format!("  Send Files ({} selected)  ", app.tagged_files.len())
        };

        let recv_text = if recv_selected {
            "> Receive Files (Wait for Connection) <".to_string()
        } else {
            "  Receive Files (Wait for Connection)  ".to_string()
        };

        let send_style = if !has_tagged {
            Style::default().fg(theme::OVERLAY0)
        } else if send_selected {
            Style::default()
                .fg(theme::YELLOW)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::TEXT)
        };

        let recv_style = if recv_selected {
            Style::default()
                .fg(theme::YELLOW)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::TEXT)
        };

        let menu_lines = vec![
            Line::from(""),
            Line::from(Span::styled(send_text, send_style)),
            Line::from(""),
            Line::from(Span::styled(recv_text, recv_style)),
            Line::from(""),
            Line::from(Span::styled(
                "─────────────────────────────────────────────────────────────",
                Style::default().fg(theme::SURFACE2),
            )),
            Line::from(Span::styled(
                "Navigation: [Up/Down] Select  |  [Enter] Confirm  |  [t/Esc] Close",
                Style::default().fg(theme::SUBTEXT0),
            )),
        ];

        let menu_paragraph = Paragraph::new(menu_lines).alignment(Alignment::Center);
        frame.render_widget(menu_paragraph, chunks[1]);
    } else if app.localsend_modal == LocalSendModalState::SendMode {
        if let Some((bytes, total, ref name)) = app.active_receive_progress {
            let ratio = if total > 0 {
                (bytes as f64 / total as f64).min(1.0)
            } else {
                0.0
            };

            let gauge_label = format!(
                "{:.1}% ({} / {})",
                ratio * 100.0,
                crate::utils::format_size(bytes),
                crate::utils::format_size(total)
            );
            let gauge = Gauge::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(theme::YELLOW))
                        .title(Span::styled(
                            " 󰏭 Sending Files Stream... ",
                            Style::default()
                                .fg(theme::YELLOW)
                                .add_modifier(Modifier::BOLD),
                        )),
                )
                .gauge_style(
                    Style::default()
                        .fg(theme::YELLOW)
                        .bg(theme::SURFACE0)
                        .add_modifier(Modifier::BOLD),
                )
                .ratio(ratio)
                .label(gauge_label);

            let send_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(4), Constraint::Min(0)])
                .split(chunks[1]);

            frame.render_widget(gauge, send_chunks[0]);

            let lines = vec![
                Line::from(""),
                Line::from(Span::styled(
                    name.clone(),
                    Style::default()
                        .fg(theme::TEXT)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(
                    "Uploading payload over encrypted TLS socket...",
                    Style::default().fg(theme::SUBTEXT0),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "[Esc] Cancel Transfer",
                    Style::default().fg(theme::RED).add_modifier(Modifier::BOLD),
                )),
            ];

            let p = Paragraph::new(lines).alignment(Alignment::Center);
            frame.render_widget(p, send_chunks[1]);
        } else {
            let send_subchunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(4),
                    Constraint::Min(0),
                    Constraint::Length(2),
                ])
                .split(chunks[1]);

            let total_bytes: u64 = app
                .tagged_files
                .iter()
                .filter_map(|p| std::fs::metadata(p).ok())
                .map(|m| m.len())
                .sum();

            let summary_text = vec![
                Line::from(vec![
                    Span::styled(
                        " 󰈔 Tagged Payload: ",
                        Style::default()
                            .fg(theme::MAUVE)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(
                            "{} files ({})",
                            app.tagged_files.len(),
                            crate::utils::format_size(total_bytes)
                        ),
                        Style::default()
                            .fg(theme::TEXT)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(Span::styled(
                    app.tagged_files
                        .iter()
                        .take(3)
                        .map(|p| p.file_name().unwrap_or_default().to_string_lossy())
                        .collect::<Vec<_>>()
                        .join(", ")
                        + if app.tagged_files.len() > 3 {
                            "..."
                        } else {
                            ""
                        },
                    Style::default().fg(theme::SUBTEXT0),
                )),
            ];

            let summary_p = Paragraph::new(summary_text)
                .block(
                    Block::default()
                        .borders(Borders::BOTTOM)
                        .border_style(Style::default().fg(theme::SURFACE2)),
                )
                .alignment(Alignment::Center);
            frame.render_widget(summary_p, send_subchunks[0]);

            if app.peer_list.is_empty() {
                let empty_lines = vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        "󰑐 Searching for nearby LocalSend devices on network...",
                        Style::default()
                            .fg(theme::YELLOW)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "Make sure LocalSend is open on the target phone, PC, or tablet.",
                        Style::default().fg(theme::SUBTEXT0),
                    )),
                    Line::from(Span::styled(
                        "Press [r / F5] to scan network again.",
                        Style::default().fg(theme::TEAL),
                    )),
                ];
                let empty_p = Paragraph::new(empty_lines).alignment(Alignment::Center);
                frame.render_widget(empty_p, send_subchunks[1]);
            } else {
                let items: Vec<ListItem> = app
                    .peer_list
                    .iter()
                    .enumerate()
                    .map(|(idx, peer)| {
                        let icon = match peer.device_type {
                            Some(crate::localsend::protocol::DeviceType::Mobile) => "📱",
                            Some(crate::localsend::protocol::DeviceType::Desktop) => "💻",
                            Some(crate::localsend::protocol::DeviceType::Web) => "🌐",
                            Some(crate::localsend::protocol::DeviceType::Headless)
                            | Some(crate::localsend::protocol::DeviceType::Server) => "🖥️",
                            _ => "󰖩",
                        };
                        let model = peer.device_model.as_deref().unwrap_or(&peer.version);
                        let text = format!("{} {} ({}) [{}]", icon, peer.alias, peer.ip, model);
                        let style = if idx == app.peer_selected {
                            Style::default()
                                .fg(theme::YELLOW)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(theme::TEXT)
                        };
                        ListItem::new(text).style(style)
                    })
                    .collect();

                let list_block = Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme::SURFACE2))
                    .title(Span::styled(
                        " Available Receiver Devices ",
                        Style::default()
                            .fg(theme::GREEN)
                            .add_modifier(Modifier::BOLD),
                    ));

                let mut state = app.peer_list_state.clone();
                let list = List::new(items)
                    .block(list_block)
                    .highlight_style(
                        Style::default()
                            .bg(theme::SURFACE1)
                            .fg(theme::YELLOW)
                            .add_modifier(Modifier::BOLD),
                    )
                    .highlight_symbol("❯ ");

                frame.render_stateful_widget(list, send_subchunks[1], &mut state);
            }

            let nav_hint = Paragraph::new(Line::from(Span::styled(
                "Navigation: [Up/Down] Select Device  |  [Enter/s] Send  |  [r] Refresh  |  [Esc/t] Back",
                Style::default().fg(theme::SUBTEXT0),
            )))
            .alignment(Alignment::Center);
            frame.render_widget(nav_hint, send_subchunks[2]);
        }
    } else if app.localsend_modal == LocalSendModalState::ReceiveMode {
        if let Some((bytes, total, ref _name)) = app.active_receive_progress {
            let ratio = if total > 0 {
                (bytes as f64 / total as f64).min(1.0)
            } else {
                0.0
            };

            let gauge_label = format!(
                "{:.1}% ({} / {})",
                ratio * 100.0,
                crate::utils::format_size(bytes),
                crate::utils::format_size(total)
            );
            let gauge = Gauge::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Green))
                        .title(Span::styled(
                            " 󰏭 Receiving File Stream... ",
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD),
                        )),
                )
                .gauge_style(
                    Style::default()
                        .fg(Color::Green)
                        .bg(theme::SURFACE0)
                        .add_modifier(Modifier::BOLD),
                )
                .ratio(ratio)
                .label(gauge_label);

            let recv_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(4), Constraint::Min(0)])
                .split(chunks[1]);

            frame.render_widget(gauge, recv_chunks[0]);

            let lines = vec![
                Line::from(""),
                Line::from(Span::styled(
                    format!("Save Location: {}", app.current_path.display()),
                    Style::default().fg(theme::TEXT),
                )),
                Line::from(Span::styled(
                    "Streaming binary data from peer...",
                    Style::default().fg(theme::SUBTEXT0),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "[Esc] Cancel Transfer",
                    Style::default().fg(theme::RED).add_modifier(Modifier::BOLD),
                )),
            ];

            let p = Paragraph::new(lines).alignment(Alignment::Center);
            frame.render_widget(p, recv_chunks[1]);
        } else if let Some(req) = app.incoming_requests.first() {
            let total_size: u64 = req.files.iter().map(|f| f.size).sum();
            let mut lines = vec![
                Line::from(Span::styled(
                    " 󰏭  INCOMING TRANSFER REQUEST ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(
                    format!(" Sender: {}", req.peer.alias),
                    Style::default().fg(theme::TEXT),
                )),
                Line::from(Span::styled(
                    format!(
                        " Total Files: {} | Total Size: {}",
                        req.files.len(),
                        crate::utils::format_size(total_size)
                    ),
                    Style::default().fg(theme::SUBTEXT0),
                )),
                Line::from(Span::styled(
                    " Files to receive:",
                    Style::default().fg(theme::SAPPHIRE),
                )),
            ];
            for file in &req.files {
                lines.push(Line::from(Span::styled(
                    format!(
                        "   - {} ({})",
                        file.file_name,
                        crate::utils::format_size(file.size)
                    ),
                    Style::default().fg(theme::TEXT),
                )));
            }
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled(
                    " [y] or [Enter] ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Accept & Download to Current Folder",
                    Style::default().fg(theme::TEXT),
                ),
            ]));
            lines.push(Line::from(vec![
                Span::styled(
                    " [n]            ",
                    Style::default().fg(theme::RED).add_modifier(Modifier::BOLD),
                ),
                Span::styled("Decline Request", Style::default().fg(theme::TEXT)),
            ]));

            let receive_paragraph = Paragraph::new(lines).alignment(Alignment::Center);
            frame.render_widget(receive_paragraph, chunks[1]);
        } else {
            let anim_frames = ["       @       ", "     * @ *     ", "  *  * @ *  *  "];
            let current_anim = anim_frames[app.anim_frame % 3];

            let lines = vec![
                Line::from(""),
                Line::from(Span::styled(
                    current_anim,
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "󰑐 Listening for incoming LocalSend requests...",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(
                    format!("Save Location: {}", app.current_path.display()),
                    Style::default().fg(theme::TEXT),
                )),
                Line::from(Span::styled(
                    format!("Listening on UDP/TCP port {} (LocalSend v2)...", app.port),
                    Style::default().fg(theme::SUBTEXT0),
                )),
                Line::from(Span::styled(
                    format!("Alias: {} | Fingerprint: {}", app.alias, app.fingerprint),
                    Style::default().fg(theme::TEAL),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Waiting for nearby phone or PC to send files...",
                    Style::default().fg(theme::OVERLAY0),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "[t/Esc] Close LocalSend Window",
                    Style::default().fg(theme::SUBTEXT0),
                )),
            ];

            let receive_paragraph = Paragraph::new(lines).alignment(Alignment::Center);
            frame.render_widget(receive_paragraph, chunks[1]);
        }
    }
}

/// Renders the standalone target peer device selection modal.
pub fn render_send_modal(frame: &mut Frame, app: &mut AppState) {
    let area = centered_rect(60, 50, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(theme::MAUVE))
        .title(Span::styled(
            format!(
                " Send {} Selected Files - Select Target Peer ",
                app.tagged_files.len()
            ),
            Style::default()
                .fg(theme::MAUVE)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme::CRUST));

    let items: Vec<ListItem> = app
        .peer_list
        .iter()
        .map(|peer| {
            let icon = match peer.device_type {
                Some(crate::localsend::protocol::DeviceType::Mobile) => "📱",
                Some(crate::localsend::protocol::DeviceType::Desktop) => "💻",
                Some(crate::localsend::protocol::DeviceType::Web) => "🌐",
                Some(crate::localsend::protocol::DeviceType::Headless)
                | Some(crate::localsend::protocol::DeviceType::Server) => "🖥️",
                _ => "󰖩",
            };
            let model = peer.device_model.as_deref().unwrap_or(&peer.version);
            let text = format!("{} {} ({}) [{}]", icon, peer.alias, peer.ip, model);
            ListItem::new(text).style(Style::default().fg(theme::SKY))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(theme::SURFACE1)
                .fg(theme::YELLOW)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("❯ ");

    frame.render_stateful_widget(list, area, &mut app.peer_list_state);
}
