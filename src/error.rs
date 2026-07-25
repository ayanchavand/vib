use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Io(String),
    PermissionDenied,
    NotFound,
    Message(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(msg) => write!(f, "I/O error: {}", msg),
            AppError::PermissionDenied => write!(f, "Permission denied"),
            AppError::NotFound => write!(f, "File or directory not found"),
            AppError::Message(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        use std::io::ErrorKind::*;

        match err.kind() {
            PermissionDenied => AppError::PermissionDenied,
            NotFound => AppError::NotFound,
            _ => AppError::Io(err.to_string()),
        }
    }
}
