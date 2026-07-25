//! TUI rendering engine coordinator built with Ratatui.

pub mod files;
pub mod helpers;
pub mod modals;
pub mod preview;
pub mod status_bar;

use crate::app::{AppState, LocalSendModalState};
use crate::theme;
use files::render_files_tab;
use modals::{
    render_bookmark_overlay, render_fail_banner_modal, render_localsend_overlay,
    render_new_folder_modal, render_rename_modal, render_send_modal, render_success_banner_modal,
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::Style,
    widgets::Block,
};
use status_bar::render_status_bar;

/// Master rendering entry point called on every redraw frame.
pub fn render(frame: &mut Frame, app: &mut AppState) {
    let background_block = Block::default().style(Style::default().bg(theme::BASE));
    frame.render_widget(background_block, frame.area());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Main Explorer File View
            Constraint::Length(1), // Bottom Segmented status bar
        ])
        .split(frame.area());

    render_files_tab(frame, app, chunks[0]);
    render_status_bar(frame, app, chunks[1]);

    if let Some((ref msg, _)) = app.fail_banner {
        render_fail_banner_modal(frame, msg);
    } else if let Some((ref msg, _)) = app.success_banner {
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
