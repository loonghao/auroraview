//! WebView error types
//!
//! Unified error handling for WebView operations, inspired by Qt WebView's
//! detailed error state mapping.

use thiserror::Error;

/// Result type alias for WebView operations
pub type WebViewResult<T> = Result<T, WebViewError>;

/// Unified error type for WebView operations
#[derive(Debug, Clone, Error)]
pub enum WebViewError {
    /// Backend initialization failed
    #[error("Initialization error: {0}")]
    Initialization(String),

    /// Navigation failed
    #[error("Navigation error: {0}")]
    Navigation(String),

    /// JavaScript execution failed
    #[error("JavaScript error: {0}")]
    JavaScript(String),

    /// Cookie operation failed
    #[error("Cookie error: {0}")]
    Cookie(String),

    /// Settings operation failed
    #[error("Settings error: {0}")]
    Settings(String),

    /// Backend not supported on current platform
    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),

    /// Backend type not available
    #[error("Unsupported backend: {0}")]
    UnsupportedBackend(String),

    /// WebView is already closed
    #[error("WebView is closed")]
    Closed,

    /// Resource not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// Timeout error
    #[error("Timeout: {0}")]
    Timeout(String),

    /// Invalid argument
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Operation not supported by this backend
    #[error("Unsupported operation: {0}")]
    Unsupported(String),

    /// Icon loading/conversion error
    #[error("Icon error: {0}")]
    Icon(String),
}

impl WebViewError {
    /// Create an initialization error
    pub fn init(msg: impl Into<String>) -> Self {
        Self::Initialization(msg.into())
    }

    /// Create a navigation error
    pub fn navigation(msg: impl Into<String>) -> Self {
        Self::Navigation(msg.into())
    }

    /// Create a JavaScript error
    pub fn javascript(msg: impl Into<String>) -> Self {
        Self::JavaScript(msg.into())
    }

    /// Create an invalid argument error
    pub fn invalid_arg(msg: impl Into<String>) -> Self {
        Self::InvalidArgument(msg.into())
    }

    /// Create an internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    /// Create an icon error
    pub fn icon(msg: impl Into<String>) -> Self {
        Self::Icon(msg.into())
    }
}
