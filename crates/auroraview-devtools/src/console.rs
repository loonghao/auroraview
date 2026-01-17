//! Console message types

use serde::{Deserialize, Serialize};

/// Console message type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConsoleMessageType {
    Log,
    Debug,
    Info,
    Warning,
    Error,
}

impl Default for ConsoleMessageType {
    fn default() -> Self {
        Self::Log
    }
}

/// Console message from DevTools
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleMessage {
    /// Message type
    pub message_type: ConsoleMessageType,
    /// Message text
    pub text: String,
    /// Source URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Line number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    /// Column number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,
    /// Stack trace (if error)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack_trace: Option<String>,
    /// Timestamp (milliseconds since epoch)
    pub timestamp: i64,
}

impl ConsoleMessage {
    /// Create a new log message
    pub fn log(text: impl Into<String>) -> Self {
        Self {
            message_type: ConsoleMessageType::Log,
            text: text.into(),
            source: None,
            line: None,
            column: None,
            stack_trace: None,
            timestamp: chrono_timestamp(),
        }
    }

    /// Create a new error message
    pub fn error(text: impl Into<String>) -> Self {
        Self {
            message_type: ConsoleMessageType::Error,
            text: text.into(),
            source: None,
            line: None,
            column: None,
            stack_trace: None,
            timestamp: chrono_timestamp(),
        }
    }

    /// Create a new warning message
    pub fn warning(text: impl Into<String>) -> Self {
        Self {
            message_type: ConsoleMessageType::Warning,
            text: text.into(),
            source: None,
            line: None,
            column: None,
            stack_trace: None,
            timestamp: chrono_timestamp(),
        }
    }

    /// Create a new info message
    pub fn info(text: impl Into<String>) -> Self {
        Self {
            message_type: ConsoleMessageType::Info,
            text: text.into(),
            source: None,
            line: None,
            column: None,
            stack_trace: None,
            timestamp: chrono_timestamp(),
        }
    }

    /// Create a new debug message
    pub fn debug(text: impl Into<String>) -> Self {
        Self {
            message_type: ConsoleMessageType::Debug,
            text: text.into(),
            source: None,
            line: None,
            column: None,
            stack_trace: None,
            timestamp: chrono_timestamp(),
        }
    }

    /// Set source location
    pub fn with_source(mut self, url: impl Into<String>, line: u32, column: u32) -> Self {
        self.source = Some(url.into());
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    /// Set stack trace
    pub fn with_stack_trace(mut self, stack: impl Into<String>) -> Self {
        self.stack_trace = Some(stack.into());
        self
    }

    /// Check if this is an error message
    pub fn is_error(&self) -> bool {
        self.message_type == ConsoleMessageType::Error
    }

    /// Check if this is a warning message
    pub fn is_warning(&self) -> bool {
        self.message_type == ConsoleMessageType::Warning
    }
}

/// Get current timestamp in milliseconds
fn chrono_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_message_types() {
        let log = ConsoleMessage::log("test");
        assert_eq!(log.message_type, ConsoleMessageType::Log);

        let error = ConsoleMessage::error("error");
        assert!(error.is_error());

        let warning = ConsoleMessage::warning("warning");
        assert!(warning.is_warning());
    }

    #[test]
    fn test_console_message_with_source() {
        let msg = ConsoleMessage::log("test").with_source("script.js", 10, 5);

        assert_eq!(msg.source, Some("script.js".to_string()));
        assert_eq!(msg.line, Some(10));
        assert_eq!(msg.column, Some(5));
    }
}
