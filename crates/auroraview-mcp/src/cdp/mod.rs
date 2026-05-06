//! Chrome DevTools Protocol (CDP) client module for AuroraView MCP Server.
//!
//! This module provides a minimal async CDP client for communicating with
//! Chrome/WebView2 instances via WebSocket.
//!
//! # Module structure
//!
//! - `mod.rs` — type definitions (`CdpError`, `CdpClient`, etc.)
//! - `connect.rs` — connection establishment and request/response handling
//! - `browser.rs` — `Browser.*` CDP methods
//! - `page.rs` — `Page.*` CDP methods
//! - `dom.rs` — `DOM.*` CDP methods
//! - `runtime.rs` — `Runtime.*` CDP methods
//! - `network.rs` — `Network.*` CDP methods
//! - `emulation.rs` — `Emulation.*` CDP methods
//! - `security.rs` — `Security.*` CDP methods

// ---------------------------------------------------------------------------
// Sub-modules
// ---------------------------------------------------------------------------

pub mod connect;
pub mod browser;
pub mod page;
pub mod dom;
pub mod runtime;
pub mod network;
pub mod emulation;
pub mod security;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

use serde::Deserialize;

/// Errors produced by the CDP client.
#[derive(Debug, thiserror::Error)]
pub enum CdpError {
    /// HTTP error during CDP endpoint discovery (GET /json/version).
    #[error("HTTP target discovery failed: {0}")]
    Http(#[from] reqwest::Error),

    /// WebSocket error during CDP communication.
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    /// JSON serialization or deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Base64 decoding error (e.g. when decoding screenshot data).
    #[error("base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),

    /// CDP returned an error message in the response.
    #[error("CDP method {0} returned error: {1}")]
    Remote(String, String),

    /// CDP response is missing an expected field.
    #[error("unexpected CDP response for {0}: missing `{1}` field")]
    MalformedResponse(String, &'static str),

    /// CDP connection was closed before receiving a response.
    #[error("CDP connection closed before receiving response for {0}")]
    ConnectionClosed(String),

    /// CDP request timed out waiting for a response.
    #[error("CDP method {0} timed out after {1:?}")]
    Timeout(String, std::time::Duration),
}

// ---------------------------------------------------------------------------
// CDP response types
// ---------------------------------------------------------------------------

/// `http://<host>:<port>/json/version` response shape (subset we care about).
#[derive(Debug, Deserialize)]
struct VersionInfo {
    #[serde(rename = "webSocketDebuggerUrl")]
    web_socket_debugger_url: String,
    #[serde(rename = "Browser", default)]
    browser: String,
    #[serde(rename = "Protocol-Version", default)]
    protocol_version: String,
}

/// Static information returned by `Browser.getVersion`.
#[derive(Debug, Clone)]
pub struct BrowserVersion {
    /// Product identifier (e.g. "Chrome/120.0.6099.109").
    pub product: String,
    /// CDP protocol version string.
    pub protocol_version: String,
}

// ---------------------------------------------------------------------------
// CdpClient struct definition
// ---------------------------------------------------------------------------

use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream,
};

/// Async CDP client holding a single browser-level WebSocket.
///
/// Implements `Clone` by wrapping the WebSocket in an `Arc<Mutex<..>>`.
/// This allows the client to be shared across multiple tool calls.
#[derive(Clone)]
pub struct CdpClient {
    pub(crate) inner: Arc<Mutex<CdpClientInner>>,
    /// Endpoint URL we connected to, kept around for diagnostics.
    pub endpoint: String,
}

/// Inner state of `CdpClient` (not `Clone`).
pub struct CdpClientInner {
    pub(crate) ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
    pub(crate) next_id: u64,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cdp_error_display() {
        // Test that CdpError implements Display correctly
        let err = CdpError::ConnectionClosed("Browser.getVersion".to_string());
        let display = format!("{err}");
        assert!(!display.is_empty());
        assert!(display.contains("Browser.getVersion"));
    }
}
