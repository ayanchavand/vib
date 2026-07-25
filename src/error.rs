//! Application error types and implementations for `vib`.

use std::fmt;

/// Represents errors that can occur during application execution.
#[derive(Debug)]
pub enum AppError {
    /// Standard I/O operations error.
    Io(String),
    /// Access or permission denied.
    PermissionDenied,
    /// Targeted path or resource not found.
    NotFound,
    /// Errors originating from LocalSend client/server interactions.
    LocalSend(String),
    /// Generic or custom human-readable error message.
    Message(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(msg) => write!(f, "I/O error: {msg}"),
            AppError::PermissionDenied => write!(f, "Permission denied"),
            AppError::NotFound => write!(f, "File or directory not found"),
            AppError::LocalSend(msg) => write!(f, "LocalSend error: {msg}"),
            AppError::Message(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::PermissionDenied => AppError::PermissionDenied,
            std::io::ErrorKind::NotFound => AppError::NotFound,
            _ => AppError::Io(err.to_string()),
        }
    }
}
