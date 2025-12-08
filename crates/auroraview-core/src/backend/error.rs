//! WebView error types
//!
//! Unified error handling for WebView operations, inspired by Qt WebView's
//! detailed error state mapping.

use std::fmt;

/// Result type alias for WebView operations
pub type WebViewResult<T> = Result<T, WebViewError>;

/// Unified error type for WebView operations
#[derive(Debug, Clone)]
pub enum WebViewError {
    /// Backend initialization failed
    Initialization(String),

    /// Navigation failed
    Navigation(String),

    /// JavaScript execution failed
    JavaScript(String),

    /// Cookie operation failed
    Cookie(String),

    /// Settings operation failed
    Settings(String),

    /// Backend not supported on current platform
    UnsupportedPlatform(String),

    /// Backend type not available
    UnsupportedBackend(String),

    /// WebView is already closed
    Closed,

    /// Resource not found
    NotFound(String),

    /// Permission denied
    PermissionDenied(String),

    /// Network error
    Network(String),

    /// Timeout error
    Timeout(String),

    /// Invalid argument
    InvalidArgument(String),

    /// Internal error
    Internal(String),

    /// Operation not supported by this backend
    Unsupported(String),

    /// Icon loading/conversion error
    Icon(String),
}

impl fmt::Display for WebViewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Initialization(msg) => write!(f, "Initialization error: {}", msg),
            Self::Navigation(msg) => write!(f, "Navigation error: {}", msg),
            Self::JavaScript(msg) => write!(f, "JavaScript error: {}", msg),
            Self::Cookie(msg) => write!(f, "Cookie error: {}", msg),
            Self::Settings(msg) => write!(f, "Settings error: {}", msg),
            Self::UnsupportedPlatform(msg) => write!(f, "Unsupported platform: {}", msg),
            Self::UnsupportedBackend(msg) => write!(f, "Unsupported backend: {}", msg),
            Self::Closed => write!(f, "WebView is closed"),
            Self::NotFound(msg) => write!(f, "Not found: {}", msg),
            Self::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            Self::Network(msg) => write!(f, "Network error: {}", msg),
            Self::Timeout(msg) => write!(f, "Timeout: {}", msg),
            Self::InvalidArgument(msg) => write!(f, "Invalid argument: {}", msg),
            Self::Internal(msg) => write!(f, "Internal error: {}", msg),
            Self::Unsupported(msg) => write!(f, "Unsupported operation: {}", msg),
            Self::Icon(msg) => write!(f, "Icon error: {}", msg),
        }
    }
}

impl std::error::Error for WebViewError {}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = WebViewError::Navigation("failed to load".into());
        assert_eq!(err.to_string(), "Navigation error: failed to load");
    }

    #[test]
    fn test_error_helpers() {
        let err = WebViewError::init("failed");
        assert!(matches!(err, WebViewError::Initialization(_)));

        let err = WebViewError::javascript("syntax error");
        assert!(matches!(err, WebViewError::JavaScript(_)));
    }
}
