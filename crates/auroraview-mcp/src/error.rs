//! Errors for the `AuroraView` MCP server.

use thiserror::Error;

/// Errors for the `AuroraView` MCP server.
///
/// These errors are returned by `McpServer` and `McpRunner` methods.
#[derive(Debug, Error)]
pub enum McpError {
    /// `WebView` instance with the given ID was not found.
    #[error("WebView not found: {0}")]
    WebViewNotFound(String),

    /// Tool execution failed with an error message.
    #[error("Tool execution failed: {0}")]
    ToolExecution(String),

    /// MCP server is not running.
    #[error("Server not running")]
    ServerNotRunning,

    /// MCP server is already running on the given port.
    #[error("Server already running on port {0}")]
    AlreadyRunning(u16),

    /// Invalid configuration provided.
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// mDNS broadcast operation failed.
    #[error("mDNS broadcast error: {0}")]
    MdnsBroadcast(String),

    /// `WebView` registry has reached its capacity limit.
    #[error("WebView registry capacity exceeded: max {0} instances")]
    CapacityExceeded(usize),

    /// Invalid URL scheme provided (must be http, https, or file).
    #[error("Invalid URL scheme '{0}': must be http, https, or file")]
    InvalidUrl(String),

    /// `eval_js` was called with an empty script.
    #[error("eval_js script must not be empty")]
    EmptyScript,

    /// JSON serialization or deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// I/O error during file or network operation.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Catch-all error for other error types.
    #[error("Anyhow error: {0}")]
    Other(#[from] anyhow::Error),
}

/// Result type alias for `AuroraView` MCP server operations.
pub type Result<T> = std::result::Result<T, McpError>;
