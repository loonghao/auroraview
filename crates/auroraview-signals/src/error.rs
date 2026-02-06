//! Error types for Aurora Signals

use thiserror::Error;

/// Errors that can occur in the signal system
#[derive(Error, Debug)]
pub enum SignalError {
    /// Signal not found in registry
    #[error("Signal not found: {0}")]
    SignalNotFound(String),

    /// Bridge not found
    #[error("Bridge not found: {0}")]
    BridgeNotFound(String),

    /// Failed to emit event
    #[error("Failed to emit event: {0}")]
    EmitFailed(String),

    /// Middleware rejected the event
    #[error("Event rejected by middleware: {0}")]
    MiddlewareRejected(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Bridge error
    #[error("Bridge error: {0}")]
    BridgeError(String),

    /// Invalid pattern
    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),
}

impl From<serde_json::Error> for SignalError {
    fn from(err: serde_json::Error) -> Self {
        SignalError::SerializationError(err.to_string())
    }
}

impl From<regex::Error> for SignalError {
    fn from(err: regex::Error) -> Self {
        SignalError::InvalidPattern(err.to_string())
    }
}
