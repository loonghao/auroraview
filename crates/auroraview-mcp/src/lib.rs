//! `AuroraView` MCP Server - exposes `AuroraView` as a standard MCP server.
//!
//! # Features
//!
//! - `screenshot` - Capture `WebView` screenshot
//! - `eval_js` - Evaluate JavaScript in `WebView` context
//! - `load_url` - Navigate `WebView` to URL
//! - `send_event` - Send event to `WebView`
//!
//! # Transport
//!
//! - HTTP/SSE via `StreamableHttpService` (rmcp)
//! - mDNS broadcast for auto-discovery
//! - AG-UI SSE events at `/agui/events`

#![warn(missing_docs)]

pub mod adapter;
pub mod agui;
pub mod cdp;
pub mod error;
pub mod mcp_server;
pub mod mdns;
pub mod oauth;
pub mod python_bindings;
pub mod registry;
pub mod runner;
pub mod types;

pub use adapter::{CdpAdapterConfig, CdpAuroraViewAdapter, DEFAULT_CDP_TIMEOUT};
