//! AuroraView Embedded MCP Server
//!
//! This crate provides an embedded MCP (Model Context Protocol) Server with Streamable HTTP transport
//! for AI assistant integration. It allows AuroraView applications to expose tools and
//! resources to AI assistants like Claude, Cursor, and Copilot.
//!
//! # Features
//!
//! - **Streamable HTTP Transport**: Modern, efficient transport replacing deprecated SSE
//! - **Built on rmcp SDK**: Uses the official Rust MCP SDK for protocol compliance
//! - **Dynamic Tool Registration**: Register tools at runtime
//! - **Async Support**: Full async/await support with Tokio
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                         AI Assistant                             │
//! │                    (Claude, Cursor, Copilot)                    │
//! └─────────────────────────────────────────────────────────────────┘
//!                                 │
//!                  MCP Protocol (Streamable HTTP)
//!                                 │
//!                                 ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                      Embedded MCP Server                         │
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
//! │  │ Streamable  │  │    Tool     │  │      rmcp SDK           │  │
//! │  │   HTTP      │  │   Registry  │  │      Service            │  │
//! │  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use auroraview_mcp::{McpServer, McpConfig, Tool};
//!
//! // Create server with config
//! let config = McpConfig::default();
//! let server = McpServer::new(config);
//!
//! // Register a tool
//! server.register_tool(Tool::new("echo", "Echo back the input")
//!     .with_param("message", "string", "Message to echo")
//!     .with_handler(|args| {
//!         let msg = args.get("message").unwrap();
//!         Ok(serde_json::json!({ "echoed": msg }))
//!     }));
//!
//! // Start server (Streamable HTTP on /mcp endpoint)
//! let port = server.start().await?;
//! println!("MCP Server running at http://127.0.0.1:{}/mcp", port);
//! ```

pub mod config;
#[cfg(feature = "python")]
pub mod dispatcher;
pub mod error;
pub mod protocol;
pub mod server;
pub mod tool;

#[cfg(feature = "python")]
pub mod python;

pub use config::McpConfig;
pub use error::{McpError, McpResult};
pub use server::McpServer;
pub use tool::{Tool, ToolRegistry};

#[cfg(feature = "python")]
pub use dispatcher::{PythonMcpDispatcher, SharedPythonDispatcher};
#[cfg(feature = "python")]
pub use python::{register_mcp_module, PyMcpConfig, PyMcpServer};

/// MCP protocol version
pub const PROTOCOL_VERSION: &str = "2024-11-05";

/// Default server name
pub const DEFAULT_SERVER_NAME: &str = "auroraview-embedded";

// ============================================================================
// Standalone Python Module Entry Point
// ============================================================================
// When built with `standalone-pymodule` feature, this creates _mcp.pyd
// that can be imported directly as `from auroraview import _mcp`

#[cfg(all(feature = "python", feature = "ext-module"))]
use pyo3::prelude::*;

/// Standalone Python module for auroraview-mcp
/// This is the entry point when building _mcp.pyd separately
#[cfg(all(feature = "python", feature = "ext-module"))]
#[pymodule]
fn _mcp(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Register all MCP classes and functions
    python::register_mcp_module(m)?;

    // Add module metadata
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__doc__", "AuroraView Embedded MCP Server")?;
    m.add("PROTOCOL_VERSION", PROTOCOL_VERSION)?;

    Ok(())
}
