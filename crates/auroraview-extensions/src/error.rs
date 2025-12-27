//! Extension error types

use thiserror::Error;

/// Extension-related errors
#[derive(Error, Debug)]
pub enum ExtensionError {
    /// Manifest parsing error
    #[error("Failed to parse manifest: {0}")]
    ManifestParse(String),

    /// Manifest validation error
    #[error("Invalid manifest: {0}")]
    ManifestInvalid(String),

    /// Extension not found
    #[error("Extension not found: {0}")]
    NotFound(String),

    /// Extension already loaded
    #[error("Extension already loaded: {0}")]
    AlreadyLoaded(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Storage error
    #[error("Storage error: {0}")]
    Storage(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// API not supported
    #[error("API not supported: {0}")]
    ApiNotSupported(String),

    /// Invalid argument
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// Invalid parameters
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),

    /// Unknown method
    #[error("Unknown method: {0}")]
    UnknownMethod(String),

    /// Runtime error
    #[error("Runtime error: {0}")]
    Runtime(String),
}

/// Result type for extension operations
pub type ExtensionResult<T> = Result<T, ExtensionError>;
