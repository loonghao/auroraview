//! IPC (Inter-Process Communication) Core Module
//!
//! This module provides platform-agnostic IPC abstractions that can be used
//! by both pure Rust applications and language bindings (Python, etc.).
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    auroraview-core/ipc                       │
//! │  (Platform-agnostic IPC abstractions)                        │
//! │  - IpcMessage: Message structure                             │
//! │  - IpcMetrics: Performance tracking                          │
//! │  - MessageQueue: Thread-safe queue                           │
//! └─────────────────────────────────────────────────────────────┘
//!                              ↑
//!                              │ uses
//! ┌─────────────────────────────────────────────────────────────┐
//! │              Language Bindings (Python, etc.)                │
//! │  - PythonCallback, json_to_python                            │
//! │  - ThreadedBackend with PyO3                                 │
//! └─────────────────────────────────────────────────────────────┘
//! ```

mod message;
mod metrics;

pub use message::{IpcMessage, IpcMode};
pub use metrics::{IpcMetrics, IpcMetricsSnapshot};
