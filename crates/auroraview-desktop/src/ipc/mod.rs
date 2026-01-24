//! IPC (Inter-Process Communication) for desktop mode
//!
//! Handles communication between WebView JavaScript and Rust backend.

mod handler;
mod message;

pub use handler::IpcRouter;
pub use message::{IpcMessage, IpcResponse};
