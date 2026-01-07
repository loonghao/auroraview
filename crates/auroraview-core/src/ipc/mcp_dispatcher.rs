//! MCP Tool Dispatcher Interface
//!
//! This module defines the interface for dispatching MCP tool calls
//! from the async MCP server thread to the main thread's event loop.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────┐         ┌─────────────────────────┐
//! │   MCP Server        │         │   Main Thread           │
//! │   (Tokio Runtime)   │         │   (Event Loop)          │
//! │                     │         │                         │
//! │   Tool Handler ─────┼─────────┼──> MessageQueue ────────┤
//! │                     │         │         │               │
//! │   await response ◄──┼─────────┼─── response_tx ◄────────┤
//! │                     │         │                         │
//! └─────────────────────┘         └─────────────────────────┘
//! ```
//!
//! This design ensures:
//! 1. MCP requests don't block waiting for Python GIL
//! 2. Python callbacks execute on the main thread (required by DCC apps)
//! 3. The MCP server remains responsive

use serde_json::Value;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Result type for MCP tool execution
pub type McpToolResult = Result<Value, String>;

/// Future type for async tool execution
pub type McpToolFuture = Pin<Box<dyn Future<Output = McpToolResult> + Send>>;

/// Opaque handler type for Python callbacks
/// This is type-erased to avoid PyO3 dependency in core
#[allow(dead_code)]
pub type OpaqueHandler = Box<dyn std::any::Any + Send + Sync>;

/// Trait for dispatching MCP tool calls to the main thread
///
/// Implementations should:
/// 1. Push a message to the main thread's message queue
/// 2. Return a future that resolves when the tool execution completes
/// 3. Handle timeouts appropriately
pub trait McpToolDispatcher: Send + Sync {
    /// Dispatch a tool call to be executed on the main thread (without handler)
    ///
    /// This is a simpler version that doesn't include the handler.
    /// Use `dispatch_with_handler` when you need to pass a Python handler.
    ///
    /// # Arguments
    /// * `tool_name` - Name of the tool to execute
    /// * `args` - JSON arguments for the tool
    ///
    /// # Returns
    /// A future that resolves to the tool result
    fn dispatch(&self, tool_name: String, args: Value) -> McpToolFuture;

    /// Check if the dispatcher is connected to the main thread
    fn is_connected(&self) -> bool;
}

/// A no-op dispatcher that returns an error
///
/// Used when no dispatcher is configured (fallback behavior)
#[derive(Clone, Default)]
pub struct NoOpDispatcher;

impl McpToolDispatcher for NoOpDispatcher {
    fn dispatch(&self, tool_name: String, _args: Value) -> McpToolFuture {
        Box::pin(async move {
            Err(format!(
                "No dispatcher configured for tool: {}. \
                 MCP tools require a connection to the WebView event loop.",
                tool_name
            ))
        })
    }

    fn is_connected(&self) -> bool {
        false
    }
}

/// Shared dispatcher reference
pub type SharedDispatcher = Arc<dyn McpToolDispatcher>;
