//! Error types for the browser crate

use thiserror::Error;

/// Browser-specific errors
#[derive(Error, Debug)]
pub enum BrowserError {
    /// Tab not found
    #[error("Tab not found: {0}")]
    TabNotFound(String),

    /// Bookmark not found
    #[error("Bookmark not found: {0}")]
    BookmarkNotFound(String),

    /// Extension not found
    #[error("Extension not found: {0}")]
    ExtensionNotFound(String),

    /// WebView creation failed
    #[error("WebView creation failed: {0}")]
    WebViewCreation(String),

    /// Window creation failed
    #[error("Window creation failed: {0}")]
    WindowCreation(String),

    /// Navigation failed
    #[error("Navigation failed: {0}")]
    Navigation(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Invalid URL
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// Extension error
    #[error("Extension error: {0}")]
    Extension(String),
}

/// Result type alias for browser operations
pub type Result<T> = std::result::Result<T, BrowserError>;
