//! Transfer state definitions for LocalSend file transfer tracking.

/// Progress information tracking active or recent LocalSend transfer sessions.
#[derive(Debug, Clone)]
pub struct TransferProgressInfo {
    /// Total bytes transferred so far in current session.
    pub bytes_transferred: u64,
    /// Total byte size of session.
    pub total_bytes: u64,
    /// Human-readable status description.
    pub status: String,
}
