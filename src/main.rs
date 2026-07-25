//! `vib` - Terminal file manager with built-in LocalSend P2P file transfers.

use color_eyre::Result;
use crossterm::event::EventStream;
use futures_util::StreamExt;
use std::env;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

mod app;
mod error;
mod events;
mod fs;
mod input;
mod localsend;
mod theme;
mod ui;
mod utils;

use app::AppState;
use events::AppEvent;
use localsend::client::LocalSendClient;
use localsend::discovery::DiscoveryEngine;
use localsend::protocol::LOCALSEND_DEFAULT_PORT;
use localsend::server::start_server;
use localsend::tls::generate_self_signed_cert;
use ratatui::DefaultTerminal;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let _ = rustls::crypto::ring::default_provider().install_default();

    let terminal = ratatui::init();

    let result = run_app(terminal).await;
    ratatui::restore();

    result
}

async fn run_app(mut terminal: DefaultTerminal) -> Result<()> {
    let cwd = env::current_dir()?.canonicalize()?;

    let hostname_str = hostname::get()
        .map(|h| h.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "vib-client".to_string());
    let alias = format!("vib ({hostname_str})");

    let download_dir = Arc::new(Mutex::new(cwd.clone()));

    let port = LOCALSEND_DEFAULT_PORT;
    let tls_config = generate_self_signed_cert(&alias)
        .map_err(|e| crate::error::AppError::LocalSend(e.to_string()))?;

    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<AppEvent>();

    let discovery_engine = Arc::new(DiscoveryEngine::new(
        alias.clone(),
        tls_config.fingerprint.clone(),
        port,
        event_tx.clone(),
    ));

    let discovery_task_engine = discovery_engine.clone();
    tokio::spawn(async move {
        if let Err(e) = discovery_task_engine.start().await {
            eprintln!("Discovery engine stopped: {e}");
        }
    });

    let server_tx = event_tx.clone();
    let server_alias = alias.clone();
    let server_fp = tls_config.fingerprint.clone();
    let server_config = tls_config.server_config.clone();
    let server_download_dir = download_dir.clone();

    tokio::spawn(async move {
        if let Err(e) = start_server(
            server_alias,
            server_fp,
            port,
            server_config,
            server_download_dir,
            server_tx,
        )
        .await
        {
            eprintln!("LocalSend HTTPS Server error: {e}");
        }
    });

    let client = Arc::new(LocalSendClient::new(
        alias.clone(),
        tls_config.fingerprint.clone(),
        port,
    ));

    let mut app = AppState::new(cwd, alias, tls_config.fingerprint, port, download_dir);
    app.load()?;

    let mut reader = EventStream::new();
    let mut tick_interval = tokio::time::interval(std::time::Duration::from_millis(250));

    loop {
        app.update();
        terminal.draw(|frame| ui::render(frame, &mut app))?;

        tokio::select! {
            _ = tick_interval.tick() => {}
            maybe_event = reader.next() => {
                if let Some(Ok(crossterm::event::Event::Key(key))) = maybe_event
                    && key.kind == crossterm::event::KeyEventKind::Press
                {
                    if app.show_new_folder_modal || app.show_rename_modal {
                        if let Err(err) = app.handle_input_key(key) {
                            app.set_error(err);
                        }
                    } else {
                        let action = input::map_key(key);
                        let is_send_action = (matches!(action, input::Action::Enter | input::Action::OpenSendModal))
                            && (app.show_send_modal || app.localsend_modal == crate::app::LocalSendModalState::SendMode)
                            && app.active_receive_progress.is_none();

                        if is_send_action {
                            if let Some(peer) = app.peer_list.get(app.peer_selected).cloned() {
                                let tagged_paths: Vec<_> = app.tagged_files.iter().cloned().collect();
                                if !tagged_paths.is_empty() {
                                    let total_size: u64 = tagged_paths
                                        .iter()
                                        .filter_map(|p| std::fs::metadata(p).ok())
                                        .map(|m| m.len())
                                        .sum();
                                    app.active_receive_progress = Some((
                                        0,
                                        total_size,
                                        format!("Preparing upload to {}...", peer.alias),
                                    ));
                                    app.tagged_files.clear();

                                    let client_clone = client.clone();
                                    let event_tx_clone = event_tx.clone();

                                    tokio::spawn(async move {
                                        client_clone.send_files(peer, tagged_paths, event_tx_clone).await;
                                    });
                                } else {
                                    app.set_status("No files selected! Select files using [Space] first.".to_string());
                                }
                            }
                        } else {
                            if matches!(action, input::Action::ScanPeers) {
                                let _ = event_tx.send(AppEvent::TriggerScan);
                            }
                            match app.handle_action(action) {
                                Ok(_) => {}
                                Err(err) => {
                                    if err.to_string().contains("Quit") {
                                        break;
                                    }
                                    app.set_error(err);
                                }
                            }
                        }
                    }
                }
            }
            Some(app_evt) = event_rx.recv() => {
                if matches!(app_evt, AppEvent::TriggerScan) {
                    let discovery_clone = discovery_engine.clone();
                    tokio::spawn(async move {
                        if let Ok(socket) = tokio::net::UdpSocket::bind("0.0.0.0:0").await {
                            let _ = socket.set_broadcast(true);
                            let _ = discovery_clone.announce(&socket).await;
                        }
                    });
                }
                app.handle_event(app_evt);
            }
        }
    }

    Ok(())
}
