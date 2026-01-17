//! Error types for auroraview-testing

use thiserror::Error;

/// Inspector error types
#[derive(Debug, Error)]
pub enum InspectorError {
    /// CDP connection error
    #[error("CDP connection error: {0}")]
    Connection(String),

    /// CDP command error
    #[error("CDP command failed: {0}")]
    Command(String),

    /// WebSocket error
    #[error("WebSocket error: {0}")]
    WebSocket(String),

    /// Timeout error
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Element not found
    #[error("Element not found: {0}")]
    ElementNotFound(String),

    /// Invalid ref ID
    #[error("Invalid ref ID: {0}")]
    InvalidRef(String),

    /// JavaScript evaluation error
    #[error("JavaScript error: {0}")]
    JavaScript(String),

    /// Navigation error
    #[error("Navigation error: {0}")]
    Navigation(String),

    /// Screenshot error
    #[error("Screenshot error: {0}")]
    Screenshot(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Parse error
    #[error("Parse error: {0}")]
    Parse(String),

    /// Session error
    #[error("Session error: {0}")]
    Session(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<serde_json::Error> for InspectorError {
    fn from(err: serde_json::Error) -> Self {
        InspectorError::Serialization(err.to_string())
    }
}

impl From<url::ParseError> for InspectorError {
    fn from(err: url::ParseError) -> Self {
        InspectorError::Parse(err.to_string())
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for InspectorError {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        InspectorError::WebSocket(err.to_string())
    }
}

/// Result type alias
pub type Result<T> = std::result::Result<T, InspectorError>;
