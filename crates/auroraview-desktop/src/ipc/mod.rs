//! IPC (Inter-Process Communication) for desktop mode
//!
//! Handles communication between WebView JavaScript and Rust backend.

mod handler;
mod message;

#[cfg(any(test, feature = "test-helpers"))]
pub use handler::reset_drag_drop_warn_guards;
pub use handler::IpcRouter;
pub use message::{IpcMessage, IpcResponse};
