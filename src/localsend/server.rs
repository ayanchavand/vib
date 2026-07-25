use crate::events::{AppEvent, IncomingTransferRequest};
use crate::localsend::protocol::{
    DeviceType, FileDto, InfoDto, LOCALSEND_DEFAULT_PORT, PROTOCOL_VERSION, Peer,
    PrepareUploadRespDto, RegisterDto,
};
use axum::{
    Json, Router,
    body::Body,
    extract::{Extension, Query, State},
    http::{StatusCode, header},
    middleware::{self, Next},
    response::IntoResponse,
    routing::{get, post},
};
use futures_util::StreamExt;
use hyper_util::service::TowerToHyperService;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

#[derive(Clone)]
pub struct ServerState {
    pub alias: String,
    pub fingerprint: String,
    pub event_tx: tokio::sync::mpsc::UnboundedSender<AppEvent>,
    pub download_dir: Arc<Mutex<PathBuf>>,
    pub active_sessions: Arc<Mutex<HashMap<String, ActiveSession>>>,
}

#[derive(Clone)]
pub struct ActiveSession {
    pub files: HashMap<String, (String, u64)>, // file_id -> (file_name, size)
    pub tokens: HashMap<String, String>,       // file_id -> token
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadQuery {
    pub session_id: String,
    pub file_id: String,
    pub token: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadQuery {
    pub session_id: String,
    pub file_id: String,
    pub token: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelQuery {
    pub session_id: String,
}

pub async fn start_server(
    alias: String,
    fingerprint: String,
    port: u16,
    server_config: Arc<rustls::ServerConfig>,
    download_dir: Arc<Mutex<PathBuf>>,
    event_tx: tokio::sync::mpsc::UnboundedSender<AppEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = ServerState {
        alias,
        fingerprint,
        event_tx,
        download_dir,
        active_sessions: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/api/localsend/v2/info", get(handle_info))
        .route("/api/localsend/v2/register", post(handle_register))
        .route(
            "/api/localsend/v2/prepare-upload",
            post(handle_prepare_upload),
        )
        .route("/api/localsend/v2/upload", post(handle_upload))
        .route("/api/localsend/v2/download", get(handle_download))
        .route("/api/localsend/v2/cancel", post(handle_cancel))
        .with_state(state);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    let tls_acceptor = TlsAcceptor::from(server_config);

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let acceptor = tls_acceptor.clone();
        let app_clone = app.clone();
        let client_ip = peer_addr.ip().to_string();

        tokio::spawn(async move {
            if let Ok(tls_stream) = acceptor.accept(stream).await {
                let io = hyper_util::rt::TokioIo::new(tls_stream);
                let ip_extension = client_ip.clone();

                let app_with_ip = app_clone.layer(middleware::from_fn(
                    move |mut req: axum::http::Request<Body>, next: Next| {
                        let ip = ip_extension.clone();
                        async move {
                            req.extensions_mut().insert(ip);
                            next.run(req).await
                        }
                    },
                ));

                let hyper_service = TowerToHyperService::new(app_with_ip);

                let _ = hyper_util::server::conn::auto::Builder::new(
                    hyper_util::rt::TokioExecutor::new(),
                )
                .serve_connection(io, hyper_service)
                .await;
            }
        });
    }
}

async fn handle_info(State(state): State<ServerState>) -> Json<InfoDto> {
    Json(InfoDto {
        alias: state.alias,
        version: PROTOCOL_VERSION.to_string(),
        device_model: Some("vib TUI Client".to_string()),
        device_type: Some(DeviceType::Desktop),
        fingerprint: state.fingerprint,
        download: true,
    })
}

async fn handle_register(
    State(state): State<ServerState>,
    Extension(peer_ip): Extension<String>,
    body: String,
) -> impl IntoResponse {
    let payload: RegisterDto = match serde_json::from_str(&body) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to parse HTTP register payload: {}", e);
            return (
                StatusCode::OK,
                Json(InfoDto {
                    alias: state.alias.clone(),
                    version: PROTOCOL_VERSION.to_string(),
                    device_model: Some("vib TUI Client".to_string()),
                    device_type: Some(DeviceType::Desktop),
                    fingerprint: state.fingerprint.clone(),
                    download: true,
                }),
            )
                .into_response();
        }
    };

    let peer_port = payload.port.unwrap_or(LOCALSEND_DEFAULT_PORT);
    let peer_protocol = payload.protocol.unwrap_or_else(|| "https".to_string());
    let alias = if payload.alias.is_empty() {
        format!("Device ({})", peer_ip)
    } else {
        payload.alias
    };
    let fingerprint = if payload.fingerprint.is_empty() {
        format!("{}:{}", peer_ip, peer_port)
    } else {
        payload.fingerprint
    };

    let peer = Peer {
        alias,
        version: payload.version,
        device_model: payload.device_model,
        device_type: payload.device_type,
        fingerprint,
        ip: peer_ip,
        port: peer_port,
        protocol: peer_protocol,
    };
    let _ = state.event_tx.send(AppEvent::PeerDiscovered(peer));

    (
        StatusCode::OK,
        Json(InfoDto {
            alias: state.alias,
            version: PROTOCOL_VERSION.to_string(),
            device_model: Some("vib TUI Client".to_string()),
            device_type: Some(DeviceType::Desktop),
            fingerprint: state.fingerprint,
            download: true,
        }),
    )
        .into_response()
}

async fn handle_prepare_upload(
    State(state): State<ServerState>,
    Extension(peer_ip): Extension<String>,
    body: String,
) -> impl IntoResponse {
    let json_val: serde_json::Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to parse prepare-upload body: {}", e);
            return (StatusCode::BAD_REQUEST, "Invalid prepare-upload payload").into_response();
        }
    };

    let session_id = Uuid::new_v4().to_string();
    let mut response_files = HashMap::new();
    let mut session_files = HashMap::new();
    let mut file_dtos = Vec::new();

    let info_obj = json_val.get("info");
    let peer_alias = info_obj
        .and_then(|i| i.get("alias"))
        .and_then(|a| a.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("LocalSend Device")
        .to_string();
    let peer_version = info_obj
        .and_then(|i| i.get("version"))
        .and_then(|v| v.as_str())
        .unwrap_or("2.0")
        .to_string();
    let peer_model = info_obj
        .and_then(|i| i.get("deviceModel"))
        .and_then(|m| m.as_str())
        .map(|s| s.to_string());
    let peer_port = info_obj
        .and_then(|i| i.get("port"))
        .and_then(|p| p.as_u64())
        .map(|p| p as u16)
        .unwrap_or(LOCALSEND_DEFAULT_PORT);
    let peer_protocol = info_obj
        .and_then(|i| i.get("protocol"))
        .and_then(|p| p.as_str())
        .unwrap_or("https")
        .to_string();
    let peer_fingerprint = info_obj
        .and_then(|i| i.get("fingerprint"))
        .and_then(|f| f.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{}:{}", peer_ip, peer_port));

    let peer = Peer {
        alias: peer_alias,
        version: peer_version,
        device_model: peer_model,
        device_type: Some(DeviceType::Mobile),
        fingerprint: peer_fingerprint,
        ip: peer_ip,
        port: peer_port,
        protocol: peer_protocol,
    };

    if let Some(files_val) = json_val.get("files") {
        if let Some(files_map) = files_val.as_object() {
            for (key, fval) in files_map {
                let fid = fval
                    .get("id")
                    .and_then(|i| i.as_str())
                    .unwrap_or(key)
                    .to_string();
                let fname = fval
                    .get("fileName")
                    .or_else(|| fval.get("file_name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("received_file")
                    .to_string();
                let fsize = fval.get("size").and_then(|s| s.as_u64()).unwrap_or(0);
                let ftype = fval
                    .get("fileType")
                    .and_then(|t| t.as_str())
                    .map(|s| s.to_string());
                let token = Uuid::new_v4().to_string();

                response_files.insert(key.clone(), token.clone());
                response_files.insert(fid.clone(), token.clone());

                session_files.insert(key.clone(), (fname.clone(), fsize));
                session_files.insert(fid.clone(), (fname.clone(), fsize));

                file_dtos.push(FileDto {
                    id: fid,
                    file_name: fname,
                    size: fsize,
                    file_type: ftype,
                    sha256: None,
                    preview: None,
                    metadata: None,
                });
            }
        } else if let Some(files_arr) = files_val.as_array() {
            for (idx, fval) in files_arr.iter().enumerate() {
                let fid = fval
                    .get("id")
                    .and_then(|i| i.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("{}", idx));
                let fname = fval
                    .get("fileName")
                    .or_else(|| fval.get("file_name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("received_file")
                    .to_string();
                let fsize = fval.get("size").and_then(|s| s.as_u64()).unwrap_or(0);
                let ftype = fval
                    .get("fileType")
                    .and_then(|t| t.as_str())
                    .map(|s| s.to_string());
                let token = Uuid::new_v4().to_string();

                response_files.insert(fid.clone(), token.clone());
                session_files.insert(fid.clone(), (fname.clone(), fsize));

                file_dtos.push(FileDto {
                    id: fid,
                    file_name: fname,
                    size: fsize,
                    file_type: ftype,
                    sha256: None,
                    preview: None,
                    metadata: None,
                });
            }
        }
    }

    let session = ActiveSession {
        files: session_files,
        tokens: response_files.clone(),
    };

    state
        .active_sessions
        .lock()
        .unwrap()
        .insert(session_id.clone(), session);

    let (response_tx, response_rx) = tokio::sync::oneshot::channel::<bool>();

    let _ = state
        .event_tx
        .send(AppEvent::IncomingTransfer(IncomingTransferRequest {
            peer,
            files: file_dtos,
            response_tx: Arc::new(Mutex::new(Some(response_tx))),
        }));

    match response_rx.await {
        Ok(true) => (
            StatusCode::OK,
            Json(PrepareUploadRespDto {
                session_id,
                files: response_files,
            }),
        )
            .into_response(),
        _ => StatusCode::FORBIDDEN.into_response(),
    }
}

async fn handle_upload(
    State(state): State<ServerState>,
    Query(query): Query<UploadQuery>,
    body: Body,
) -> StatusCode {
    let file_info = {
        let sessions = state.active_sessions.lock().unwrap();
        if let Some(session) = sessions.get(&query.session_id) {
            if session.tokens.get(&query.file_id) != Some(&query.token) {
                return StatusCode::FORBIDDEN;
            }
            session.files.get(&query.file_id).cloned()
        } else {
            return StatusCode::NOT_FOUND;
        }
    };

    let (file_name, total_size) = match file_info {
        Some(info) => info,
        None => return StatusCode::NOT_FOUND,
    };

    let active_dir = state.download_dir.lock().unwrap().clone();
    let save_path = active_dir.join(&file_name);
    if let Some(parent) = save_path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }

    let mut file = match File::create(&save_path).await {
        Ok(f) => f,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    let mut stream = body.into_data_stream();
    let mut bytes_transferred = 0u64;

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                if file.write_all(&chunk).await.is_err() {
                    return StatusCode::INTERNAL_SERVER_ERROR;
                }
                bytes_transferred += chunk.len() as u64;

                let _ = state.event_tx.send(AppEvent::TransferProgress {
                    session_id: query.session_id.clone(),
                    file_id: query.file_id.clone(),
                    bytes_transferred,
                    total_bytes: total_size,
                    is_upload: false,
                });
            }
            Err(_) => return StatusCode::BAD_REQUEST,
        }
    }

    let _ = state.event_tx.send(AppEvent::TransferCompleted {
        session_id: query.session_id,
        message: format!("Received {}", file_name),
    });

    StatusCode::OK
}

async fn handle_download(
    State(state): State<ServerState>,
    Query(query): Query<DownloadQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let file_info = {
        let sessions = state.active_sessions.lock().unwrap();
        if let Some(session) = sessions.get(&query.session_id) {
            if let Some(ref req_token) = query.token
                && session.tokens.get(&query.file_id) != Some(req_token)
            {
                return Err(StatusCode::FORBIDDEN);
            }
            session.files.get(&query.file_id).cloned()
        } else {
            return Err(StatusCode::NOT_FOUND);
        }
    };

    let (file_name, _total_size) = match file_info {
        Some(info) => info,
        None => return Err(StatusCode::NOT_FOUND),
    };

    let active_dir = state.download_dir.lock().unwrap().clone();
    let file_path = active_dir.join(&file_name);
    let file = match File::open(&file_path).await {
        Ok(f) => f,
        Err(_) => return Err(StatusCode::NOT_FOUND),
    };

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let headers = [
        (header::CONTENT_TYPE, "application/octet-stream".to_string()),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", file_name),
        ),
    ];

    Ok((headers, body))
}

async fn handle_cancel(
    State(state): State<ServerState>,
    Query(query): Query<CancelQuery>,
) -> StatusCode {
    state
        .active_sessions
        .lock()
        .unwrap()
        .remove(&query.session_id);
    let _ = state.event_tx.send(AppEvent::TransferFailed {
        session_id: query.session_id,
        error: "Transfer cancelled by sender".to_string(),
    });
    StatusCode::OK
}
