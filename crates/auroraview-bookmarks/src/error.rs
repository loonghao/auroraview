//! Bookmark error types

use thiserror::Error;

/// Bookmark errors
#[derive(Debug, Error)]
pub enum BookmarkError {
    /// Bookmark not found
    #[error("Bookmark not found: {0}")]
    NotFound(String),

    /// Folder not found
    #[error("Folder not found: {0}")]
    FolderNotFound(String),

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

/// Result type for bookmark operations
pub type Result<T> = std::result::Result<T, BookmarkError>;
