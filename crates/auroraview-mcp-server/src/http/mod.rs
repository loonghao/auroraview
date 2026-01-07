//! HTTP Server module for MCP Sidecar.
//!
//! This module implements the MCP Streamable HTTP server that exposes
//! tools registered via IPC to external AI agents.
//!
//! ## Architecture
//!
//! ```text
//! AI Agent (Claude, Cursor, etc.)
//!     │
//!     │ HTTP POST /mcp
//!     ▼
//! ┌─────────────────────────────────────┐
//! │  Sidecar HTTP Server                │
//! │  (Streamable HTTP Transport)        │
//! │                                     │
//! │  ┌─────────────────────────────┐   │
//! │  │  McpSidecarService          │   │
//! │  │  - Implements ServerHandler │   │
//! │  │  - Forwards tool.call to    │   │
//! │  │    IPC Client               │   │
//! │  └─────────────────────────────┘   │
//! └─────────────────────────────────────┘
//!     │
//!     │ IPC (tool.call)
//!     ▼
//! ┌─────────────────────────────────────┐
//! │  Main Process (IPC Server)          │
//! │  - Executes tools in Python/Rust    │
//! │  - Returns results                  │
//! └─────────────────────────────────────┘
//! ```

mod server;
mod service;

pub use server::{HttpServer, HttpServerConfig};
pub use service::McpSidecarService;

