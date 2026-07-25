use crate::app::{AppState, LocalSendModalState};
use crate::fs::{EntryKind, FileEntry};
use crate::theme;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Gauge, List, ListItem, Paragraph, Wrap},
};

pub fn render(frame: &mut Frame, app: &mut AppState) {
    // Fill background with Catppuccin Base color
    let background_block = Block::default().style(Style::default().bg(theme::BASE));
    frame.render_widget(background_block, frame.area());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Main Explorer File View
            Constraint::Length(1), // Bottom Segmented btop Status Bar
        ])
        .split(frame.area());

    // Explorer Main Content
    render_files_tab(frame, app, chunks[0]);

    // Explorer Status Bar (btop style segmented bar)
    render_status_bar(frame, app, chunks[1]);

    // Overlays
    if let Some((ref msg, _)) = app.success_banner {
        render_success_banner_modal(frame, msg);
    } else if app.show_new_folder_modal {
        render_new_folder_modal(frame, app);
    } else if app.show_rename_modal {
        render_rename_modal(frame, app);
    } else if app.show_bookmark_modal {
        render_bookmark_overlay(frame, app);
    } else if app.localsend_modal != LocalSendModalState::Closed {
        render_localsend_overlay(frame, app);
    } else if app.show_send_modal {
        render_send_modal(frame, app);
    }
}

fn render_status_bar(frame: &mut Frame, app: &AppState, area: Rect) {
    if let Some((ref err, _)) = app.error {
        let p = Paragraph::new(format!(" ERROR: {}", err)).style(
            Style::default()
                .bg(theme::RED)
                .fg(theme::CRUST)
                .add_modifier(Modifier::BOLD),
        );
        frame.render_widget(p, area);
        return;
    }

    if let Some((ref msg, _)) = app.status_message {
        let p = Paragraph::new(format!(" INFO: {}", msg)).style(
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
                format!(" ({} Selected) ", tagged_count),
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
                format!(" 󰏭 INCOMING REQUEST ({}) [t] ", incoming_count),
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

fn render_files_tab(frame: &mut Frame, app: &mut AppState, area: Rect) {
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

    // Right Pane: Details & Optional Text Preview Split
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

fn render_pane(frame: &mut Frame, app: &AppState, area: Rect, pane_idx: usize) {
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

fn render_selected_details(
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

fn render_text_preview(frame: &mut Frame, app: &AppState, area: Rect) {
    let scroll = app.text_preview_scroll;
    let total_lines = app.text_preview_lines.len();

    let title = format!(
        " 󰈙 Text Preview [Lines: {} | Scroll: Shift+W/S] ",
        total_lines
    );
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

fn render_localsend_overlay(frame: &mut Frame, app: &AppState) {
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
        .constraints([
            Constraint::Length(7), // ASCII Art Banner Area
            Constraint::Min(0),    // Options / Receive view
        ])
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
            ];

            let p = Paragraph::new(lines).alignment(Alignment::Center);
            frame.render_widget(p, send_chunks[1]);
        } else {
            let send_subchunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(4), // Selected files summary
                    Constraint::Min(0),    // Target devices list
                    Constraint::Length(2), // Navigation hint
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
                    "Streaming binary data from mobile device...",
                    Style::default().fg(theme::SUBTEXT0),
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

fn render_success_banner_modal(frame: &mut Frame, msg: &str) {
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
        .constraints([
            Constraint::Length(8), // SUCCESS ASCII Banner
            Constraint::Min(0),    // Detail message
        ])
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

fn render_send_modal(frame: &mut Frame, app: &mut AppState) {
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

fn render_bookmark_overlay(frame: &mut Frame, app: &mut AppState) {
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
            .constraints([
                Constraint::Min(0),    // Bookmarks list
                Constraint::Length(3), // Footer / Selection Info
            ])
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
                    Span::styled(format!("{} ", icon), Style::default().fg(kind_color)),
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

fn render_new_folder_modal(frame: &mut Frame, app: &AppState) {
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

fn render_rename_modal(frame: &mut Frame, app: &AppState) {
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

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
