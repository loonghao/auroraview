//! History error types

use thiserror::Error;

/// History errors
#[derive(Debug, Error)]
pub enum HistoryError {
    /// Entry not found
    #[error("History entry not found: {0}")]
    NotFound(String),

    /// Invalid URL
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

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

/// Result type for history operations
pub type Result<T> = std::result::Result<T, HistoryError>;
