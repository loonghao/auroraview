//! IPC (Inter-Process Communication) for DCC mode
//!
//! Shared IPC infrastructure for communication between WebView and Rust backend.

mod handler;
mod message;

pub use handler::IpcRouter;
pub use message::{IpcError, IpcMessage, IpcResponse};
