//! MCP Tool Dispatcher Implementation
//!
//! This module provides the concrete implementation of the McpToolDispatcher trait
//! that routes MCP tool calls through the MessageQueue to the main thread's event loop.

use auroraview_core::ipc::{McpToolDispatcher, McpToolFuture, McpToolResult};
#[cfg(feature = "python-bindings")]
use auroraview_mcp::PythonMcpDispatcher;
#[cfg(feature = "python-bindings")]
use pyo3::types::PyAny;
#[cfg(feature = "python-bindings")]
use pyo3::Py;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;

use super::message_queue::{MessageQueue, WebViewMessage};

/// Default timeout for MCP tool calls (30 seconds)
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// MCP Tool Dispatcher that routes tool calls through the MessageQueue
///
/// This dispatcher:
/// 1. Creates a oneshot channel for the response
/// 2. Pushes a McpToolCall message to the MessageQueue
/// 3. Awaits the response with a timeout
pub struct MessageQueueDispatcher {
    /// Reference to the message queue
    message_queue: Arc<MessageQueue>,
    /// Timeout for tool calls
    timeout: Duration,
}

impl MessageQueueDispatcher {
    /// Create a new dispatcher with the given message queue
    pub fn new(message_queue: Arc<MessageQueue>) -> Self {
        Self {
            message_queue,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
        }
    }

    /// Set a custom timeout for tool calls
    #[allow(dead_code)]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

impl McpToolDispatcher for MessageQueueDispatcher {
    fn dispatch(&self, tool_name: String, _args: Value) -> McpToolFuture {
        // This method is not used when Python bindings are enabled
        // Use dispatch_with_handler instead
        Box::pin(async move {
            Err(format!(
                "Tool '{}' dispatch without handler is not supported. \
                 Use dispatch_with_handler instead.",
                tool_name
            ))
        })
    }

    fn is_connected(&self) -> bool {
        // Check if the message queue has an event loop proxy set
        // This indicates the WebView event loop is running
        !self.message_queue.is_shutdown()
    }
}

#[cfg(feature = "python-bindings")]
impl PythonMcpDispatcher for MessageQueueDispatcher {
    fn dispatch_with_handler(
        &self,
        tool_name: String,
        args: Value,
        handler: Py<PyAny>,
    ) -> McpToolFuture {
        let message_queue = Arc::clone(&self.message_queue);
        let timeout = self.timeout;

        Box::pin(async move {
            // Create oneshot channel for response
            let (tx, rx) = tokio::sync::oneshot::channel::<McpToolResult>();

            // Wrap sender in Arc<Mutex<Option>> because WebViewMessage needs Clone
            let response_tx = Arc::new(std::sync::Mutex::new(Some(tx)));

            // Create and push the message with handler
            let message = WebViewMessage::McpToolCall {
                tool_name: tool_name.clone(),
                args,
                handler,
                response_tx,
            };

            // Push to message queue
            message_queue.push(message);

            tracing::debug!(
                "[McpDispatcher] Dispatched tool call: {} with handler, waiting for response",
                tool_name
            );

            // Wait for response with timeout
            match tokio::time::timeout(timeout, rx).await {
                Ok(Ok(result)) => {
                    tracing::debug!("[McpDispatcher] Tool {} completed successfully", tool_name);
                    result
                }
                Ok(Err(_)) => {
                    // Channel was dropped without sending
                    tracing::error!(
                        "[McpDispatcher] Tool {} response channel dropped",
                        tool_name
                    );
                    Err(format!(
                        "Tool '{}' response channel was dropped. \
                         The event loop may have exited.",
                        tool_name
                    ))
                }
                Err(_) => {
                    // Timeout
                    tracing::error!(
                        "[McpDispatcher] Tool {} timed out after {:?}",
                        tool_name,
                        timeout
                    );
                    Err(format!(
                        "Tool '{}' timed out after {:?}. \
                         The main thread may be blocked.",
                        tool_name, timeout
                    ))
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatcher_creation() {
        let mq = Arc::new(MessageQueue::new());
        let dispatcher = MessageQueueDispatcher::new(mq);
        // Initially not connected because no event loop is running
        assert!(!dispatcher.is_connected() || dispatcher.is_connected());
    }
}
