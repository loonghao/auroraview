//! Download error types

use thiserror::Error;

/// Download errors
#[derive(Debug, Error)]
pub enum DownloadError {
    /// Download not found
    #[error("Download not found: {0}")]
    NotFound(String),

    /// Download already exists
    #[error("Download already exists: {0}")]
    AlreadyExists(String),

    /// Invalid state transition
    #[error("Invalid state transition: {0}")]
    InvalidState(String),

    /// Storage error
    #[error("Storage error: {0}")]
    Storage(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Result type for download operations
pub type Result<T> = std::result::Result<T, DownloadError>;
