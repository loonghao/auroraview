//! Error types for AI Agent

use thiserror::Error;

/// AI Agent error types
#[derive(Error, Debug)]
pub enum AIError {
    /// Provider not configured
    #[error("Provider not configured: {0}")]
    ProviderNotConfigured(String),

    /// API key not set
    #[error("API key not set for provider: {0}")]
    ApiKeyNotSet(String),

    /// Request failed
    #[error("Request failed: {0}")]
    RequestFailed(String),

    /// Response parsing error
    #[error("Failed to parse response: {0}")]
    ParseError(String),

    /// Streaming error
    #[error("Streaming error: {0}")]
    StreamError(String),

    /// Tool call error
    #[error("Tool call failed: {name} - {message}")]
    ToolCallFailed { name: String, message: String },

    /// Action not found
    #[error("Action not found: {0}")]
    ActionNotFound(String),

    /// Action execution error
    #[error("Action execution failed: {0}")]
    ActionExecutionFailed(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded, retry after {retry_after:?} seconds")]
    RateLimitExceeded { retry_after: Option<u64> },

    /// Authentication error
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Model not available
    #[error("Model not available: {0}")]
    ModelNotAvailable(String),

    /// Context length exceeded
    #[error("Context length exceeded: {current} tokens, max {max} tokens")]
    ContextLengthExceeded { current: usize, max: usize },

    /// Session error
    #[error("Session error: {0}")]
    SessionError(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl AIError {
    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            AIError::NetworkError(_) | AIError::RateLimitExceeded { .. } | AIError::StreamError(_)
        )
    }
}

/// Result type for AI operations
pub type AIResult<T> = Result<T, AIError>;

impl From<serde_json::Error> for AIError {
    fn from(err: serde_json::Error) -> Self {
        AIError::ParseError(err.to_string())
    }
}
