//! AuroraView MCP Sidecar Server
//!
//! This crate provides a standalone MCP (Model Context Protocol) server
//! that runs as a separate process (Sidecar) alongside the main AuroraView
//! application.
//!
//! ## Architecture
//!
//! ```text
//! Main Process (AuroraView)          Sidecar Process (MCP Server)
//! ┌─────────────────────────┐        ┌─────────────────────────┐
//! │  Python Runtime         │        │  Tokio Runtime          │
//! │  (DCC / System)         │        │  (独立)                 │
//! │  ┌───────────────────┐  │        │  ┌───────────────────┐  │
//! │  │  WebView          │  │        │  │  MCP Server       │  │
//! │  │  (wry/WebView2)   │  │        │  │  (axum + rmcp)    │  │
//! │  └───────────────────┘  │        │  └─────────┬─────────┘  │
//! │           │             │        │            │            │
//! │  ┌────────▼──────────┐  │        │  ┌─────────▼─────────┐  │
//! │  │  IPC Server       │◄─┼────────┼──│  IPC Client       │  │
//! │  │  (tool handlers)  │  │        │  │  (tool forwarding)│  │
//! │  └───────────────────┘  │        │  └───────────────────┘  │
//! └─────────────────────────┘        └─────────────────────────┘
//! ```
//!
//! ## Features
//!
//! - **Process Isolation**: MCP server runs independently, crashes don't affect main process
//! - **IPC Communication**: Uses ipckit LocalSocket (Named Pipe on Windows, Unix Socket on Unix)
//! - **Parent Monitoring**: Auto-exits when parent process dies
//! - **JSON-RPC 2.0**: Standard protocol for tool calls

pub mod http;
pub mod ipc;
pub mod parent_monitor;
pub mod protocol;
pub mod sidecar;

pub use http::{HttpServer, HttpServerConfig, McpSidecarService};
pub use ipc::{generate_auth_token, generate_channel_name, IpcClient, IpcServer};
pub use parent_monitor::{ParentMonitor, ParentMonitorHandle};
pub use protocol::{
    ErrorCode, Request, RequestId, Response, RpcError, ToolCallParams, ToolDefinition,
};
pub use sidecar::{SidecarConfig, SidecarError, SidecarManager, SidecarState};
