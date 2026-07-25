use crate::localsend::protocol::{FileDto, Peer};
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

#[derive(Debug, Clone)]
pub struct IncomingTransferRequest {
    pub peer: Peer,
    pub files: Vec<FileDto>,
    pub response_tx: Arc<Mutex<Option<oneshot::Sender<bool>>>>,
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    PeerDiscovered(Peer),
    IncomingTransfer(IncomingTransferRequest),
    TransferProgress {
        session_id: String,
        file_id: String,
        bytes_transferred: u64,
        total_bytes: u64,
        is_upload: bool,
    },
    TransferCompleted {
        session_id: String,
        message: String,
    },
    TransferFailed {
        session_id: String,
        error: String,
    },
    TriggerScan,
    StatusMessage(String),
}
