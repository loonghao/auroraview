//! Plugin error types
//!
//! Common error handling for the plugin system.

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Plugin error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginErrorCode {
    /// Plugin not found
    PluginNotFound,
    /// Command not found
    CommandNotFound,
    /// Invalid arguments
    InvalidArgs,
    /// Permission denied
    PermissionDenied,
    /// Path not allowed by scope
    ScopeViolation,
    /// File not found
    FileNotFound,
    /// IO error
    IoError,
    /// Encoding error
    EncodingError,
    /// Clipboard error
    ClipboardError,
    /// Shell/process error
    ShellError,
    /// Dialog cancelled
    DialogCancelled,
    /// Unknown error
    Unknown,
}

impl PluginErrorCode {
    /// Get the error code string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PluginNotFound => "PLUGIN_NOT_FOUND",
            Self::CommandNotFound => "COMMAND_NOT_FOUND",
            Self::InvalidArgs => "INVALID_ARGS",
            Self::PermissionDenied => "PERMISSION_DENIED",
            Self::ScopeViolation => "SCOPE_VIOLATION",
            Self::FileNotFound => "FILE_NOT_FOUND",
            Self::IoError => "IO_ERROR",
            Self::EncodingError => "ENCODING_ERROR",
            Self::ClipboardError => "CLIPBOARD_ERROR",
            Self::ShellError => "SHELL_ERROR",
            Self::DialogCancelled => "DIALOG_CANCELLED",
            Self::Unknown => "UNKNOWN",
        }
    }
}

impl fmt::Display for PluginErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Plugin error type
#[derive(Debug, Error)]
pub struct PluginError {
    /// Error code
    code: PluginErrorCode,
    /// Error message
    message: String,
}

impl PluginError {
    /// Create a new plugin error
    pub fn new(code: PluginErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    /// Get the error code
    pub fn code(&self) -> String {
        self.code.as_str().to_string()
    }

    /// Get the error message
    pub fn message(&self) -> String {
        self.message.clone()
    }

    /// Get the error code enum
    pub fn error_code(&self) -> PluginErrorCode {
        self.code
    }

    /// Create a command not found error
    pub fn command_not_found(cmd: &str) -> Self {
        Self::new(
            PluginErrorCode::CommandNotFound,
            format!("Command '{}' not found", cmd),
        )
    }

    /// Create an invalid args error
    pub fn invalid_args(msg: impl Into<String>) -> Self {
        Self::new(PluginErrorCode::InvalidArgs, msg)
    }

    /// Create a scope violation error
    pub fn scope_violation(path: &str) -> Self {
        Self::new(
            PluginErrorCode::ScopeViolation,
            format!("Path '{}' is not allowed by scope configuration", path),
        )
    }

    /// Create a file not found error
    pub fn file_not_found(path: &str) -> Self {
        Self::new(
            PluginErrorCode::FileNotFound,
            format!("File not found: {}", path),
        )
    }

    /// Create an IO error
    pub fn io_error(err: std::io::Error) -> Self {
        Self::new(PluginErrorCode::IoError, err.to_string())
    }

    /// Create a clipboard error
    pub fn clipboard_error(msg: impl Into<String>) -> Self {
        Self::new(PluginErrorCode::ClipboardError, msg)
    }

    /// Create a shell error
    pub fn shell_error(msg: impl Into<String>) -> Self {
        Self::new(PluginErrorCode::ShellError, msg)
    }

    /// Create a dialog cancelled error
    pub fn dialog_cancelled() -> Self {
        Self::new(PluginErrorCode::DialogCancelled, "Dialog was cancelled")
    }

    /// Create a generic plugin error (for plugin-specific errors)
    pub fn from_plugin(_plugin_name: &str, msg: impl Into<String>) -> Self {
        Self::new(PluginErrorCode::Unknown, msg)
    }
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

/// Plugin result type
pub type PluginResult<T> = Result<T, PluginError>;
