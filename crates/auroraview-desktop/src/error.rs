//! Desktop-specific errors

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DesktopError {
    #[error("Window creation failed: {0}")]
    WindowCreation(String),

    #[error("WebView creation failed: {0}")]
    WebViewCreation(String),

    #[error("Window not found: {0}")]
    WindowNotFound(String),

    #[error("Event loop error: {0}")]
    EventLoop(String),

    #[error("Tray error: {0}")]
    Tray(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, DesktopError>;
