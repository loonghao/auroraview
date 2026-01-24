//! DCC-specific errors

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DccError {
    #[error("WebView creation failed: {0}")]
    WebViewCreation(String),

    #[error("Invalid parent HWND")]
    InvalidParent,

    #[error("Window not found: {0}")]
    WindowNotFound(String),

    #[error("DCC not supported: {0}")]
    UnsupportedDcc(String),

    #[error("Threading error: {0}")]
    Threading(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("WebView2 COM error: {0}")]
    #[cfg(target_os = "windows")]
    Com(String),
}

pub type Result<T> = std::result::Result<T, DccError>;
