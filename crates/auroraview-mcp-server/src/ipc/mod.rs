//! IPC (Inter-Process Communication) module.
//!
//! This module provides IPC communication between the MCP Sidecar process
//! and the main AuroraView process using `ipckit` LocalSocket.
//!
//! ## Architecture
//!
//! ```text
//! Main Process (AuroraView)          Sidecar Process (MCP Server)
//! ┌─────────────────────────┐        ┌─────────────────────────┐
//! │  IPC Server             │        │  IPC Client             │
//! │  (LocalSocketListener)  │◄──────►│  (LocalSocketStream)    │
//! │                         │        │                         │
//! │  - Handles tool.call    │        │  - Sends tool.call      │
//! │  - Returns results      │        │  - Receives results     │
//! └─────────────────────────┘        └─────────────────────────┘
//! ```

mod client;
mod server;

pub use client::IpcClient;
pub use server::IpcServer;

use std::io;
use thiserror::Error;

/// IPC error types.
#[derive(Debug, Error)]
pub enum IpcError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Send failed: {0}")]
    SendFailed(String),

    #[error("Receive failed: {0}")]
    ReceiveFailed(String),

    #[error("Timeout after {0}ms")]
    Timeout(u64),

    #[error("Connection closed")]
    ConnectionClosed,

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Result type for IPC operations.
pub type IpcResult<T> = Result<T, IpcError>;

/// Generate a unique IPC channel name.
///
/// Format: `auroraview_mcp_{pid}_{nonce}`
///
/// The nonce is a random UUID to prevent other processes from guessing
/// the channel name.
pub fn generate_channel_name(pid: u32) -> String {
    let nonce = uuid::Uuid::new_v4().to_string().replace('-', "")[..16].to_string();
    format!("auroraview_mcp_{}_{}", pid, nonce)
}

/// Generate a random authentication token.
pub fn generate_auth_token() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_channel_name() {
        let name = generate_channel_name(12345);
        assert!(name.starts_with("auroraview_mcp_12345_"));
        assert!(name.len() > 25); // pid + nonce
    }

    #[test]
    fn test_generate_auth_token() {
        let token1 = generate_auth_token();
        let token2 = generate_auth_token();
        assert_ne!(token1, token2);
        assert_eq!(token1.len(), 36); // UUID format
    }
}
