//! Async event definitions for application lifecycle and LocalSend tasks.

use crate::localsend::protocol::{FileDto, Peer};
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

/// Represents an incoming file transfer request from a remote peer.
#[derive(Debug, Clone)]
pub struct IncomingTransferRequest {
    /// Information about the sending device.
    pub peer: Peer,
    /// List of metadata for files offered in the transfer request.
    pub files: Vec<FileDto>,
    /// Thread-safe responder channel to accept (`true`) or decline (`false`) the transfer.
    pub response_tx: Arc<Mutex<Option<oneshot::Sender<bool>>>>,
}

/// System and LocalSend events received by the main loop.
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// A new LocalSend peer device was discovered on the network.
    PeerDiscovered(Peer),
    /// A remote peer requested to send files.
    IncomingTransfer(IncomingTransferRequest),
    /// Progress update for an ongoing file upload or download session.
    TransferProgress {
        session_id: String,
        file_id: String,
        bytes_transferred: u64,
        total_bytes: u64,
        is_upload: bool,
    },
    /// Notification that a file transfer session completed successfully.
    TransferCompleted { session_id: String, message: String },
    /// Notification that a file transfer session encountered an error or was cancelled.
    TransferFailed { session_id: String, error: String },
    /// Request to trigger an active UDP peer discovery scan.
    TriggerScan,
    /// Transient status bar notification message.
    StatusMessage(String),
}
