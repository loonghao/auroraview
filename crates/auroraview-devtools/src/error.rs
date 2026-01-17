//! DevTools error types

use thiserror::Error;

/// DevTools errors
#[derive(Debug, Error)]
pub enum DevToolsError {
    /// DevTools not enabled
    #[error("DevTools not enabled")]
    NotEnabled,

    /// DevTools already open
    #[error("DevTools already open")]
    AlreadyOpen,

    /// CDP connection error
    #[error("CDP connection error: {0}")]
    CdpConnection(String),

    /// CDP command error
    #[error("CDP command error: {0}")]
    CdpCommand(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Result type for DevTools operations
pub type Result<T> = std::result::Result<T, DevToolsError>;
