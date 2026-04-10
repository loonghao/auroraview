use thiserror::Error;

/// Errors for the AuroraView MCP server.
#[derive(Debug, Error)]
pub enum McpError {
    #[error("WebView not found: {0}")]
    WebViewNotFound(String),

    #[error("Tool execution failed: {0}")]
    ToolExecution(String),

    #[error("Server not running")]
    ServerNotRunning,

    #[error("Server already running on port {0}")]
    AlreadyRunning(u16),

    #[error("mDNS broadcast error: {0}")]
    MdnsBroadcast(String),

    #[error("WebView registry capacity exceeded: max {0} instances")]
    CapacityExceeded(usize),

    #[error("Invalid URL scheme '{0}': must be http, https, or file")]
    InvalidUrl(String),

    #[error("eval_js script must not be empty")]
    EmptyScript,

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Anyhow error: {0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, McpError>;
