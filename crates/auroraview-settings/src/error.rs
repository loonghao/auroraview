//! Error types for settings operations.

use thiserror::Error;

/// Result type for settings operations.
pub type Result<T> = std::result::Result<T, SettingsError>;

/// Errors that can occur during settings operations.
#[derive(Debug, Error)]
pub enum SettingsError {
    /// Setting key not found.
    #[error("Setting not found: {0}")]
    NotFound(String),

    /// Type mismatch when getting a setting value.
    #[error("Type mismatch for setting '{key}': expected {expected}, got {actual}")]
    TypeMismatch {
        key: String,
        expected: String,
        actual: String,
    },

    /// Validation failed for a setting value.
    #[error("Validation failed for setting '{key}': {reason}")]
    ValidationFailed { key: String, reason: String },

    /// Invalid key format.
    #[error("Invalid setting key: {0}")]
    InvalidKey(String),

    /// Schema not found for a setting.
    #[error("Schema not found for setting: {0}")]
    SchemaNotFound(String),

    /// IO error during persistence.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
