//! IPC (Inter-Process Communication) Module
//!
//! This module provides a unified abstraction layer for communication between
//! Python and JavaScript in the WebView. It supports both thread-based
//! communication (for embedded mode) and process-based communication
//! (for standalone mode).
//!
//! ## Architecture
//!
//! ```text
//! Python API (@webview.on, webview.emit)
//!     ↓
//! IpcHandler (callback management)
//!     ↓
//! IpcBackend trait (unified interface)
//!     ↓
//! ┌─────────────────┬──────────────────┐
//! │ ThreadedBackend │ ProcessBackend   │
//! │ (crossbeam)     │ (ipc-channel)    │
//! └─────────────────┴──────────────────┘
//!     ↓                    ↓
//! WebView Thread      Separate Process
//! ```
//!
//! ## Modules
//!
//! - `backend`: IpcBackend trait definition and IpcMessage structure
//! - `handler`: IpcHandler for managing callbacks and message routing
//! - `message_queue`: Thread-safe message queue for WebView operations
//! - `threaded`: ThreadedBackend implementation using crossbeam-channel
//! - `process`: ProcessBackend implementation using ipc-channel (optional)

pub mod backend;
pub mod batch;
pub mod dead_letter_queue;
pub mod handler;
pub mod json; // High-performance JSON with simd-json (orjson-equivalent)
pub mod json_bindings; // Python bindings for JSON functions
pub mod message_queue;
pub mod metrics;
pub mod threaded;

// Optional process-based backend
#[cfg(feature = "process-ipc")]
pub mod process;

// Re-export commonly used types
pub use backend::IpcMessage;
#[allow(unused_imports)]
pub use dead_letter_queue::{DeadLetterQueue, DeadLetterStats, FailureReason};
pub use handler::IpcHandler;
pub use message_queue::{MessageQueue, WebViewMessage};
#[allow(unused_imports)]
pub use metrics::{IpcMetrics, IpcMetricsSnapshot};

// Re-export for future use (currently unused but part of public API)
#[allow(unused_imports)]
pub use backend::{IpcBackend, IpcMode};
#[allow(unused_imports)]
pub use handler::PythonCallback;
#[allow(unused_imports)]
pub use message_queue::MessageQueueConfig;
#[allow(unused_imports)]
pub use threaded::{ThreadedBackend, ThreadedConfig};

// ProcessBackend is currently unused
// #[cfg(feature = "process-ipc")]
// pub use process::ProcessBackend;
