//! Tab error types

use thiserror::Error;

/// Tab errors
#[derive(Debug, Error)]
pub enum TabError {
    /// Tab not found
    #[error("Tab not found: {0}")]
    NotFound(String),

    /// Tab group not found
    #[error("Tab group not found: {0}")]
    GroupNotFound(String),

    /// Invalid operation
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// Session error
    #[error("Session error: {0}")]
    Session(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Result type for tab operations
pub type Result<T> = std::result::Result<T, TabError>;
