//! IPC (Inter-Process Communication) Module - Python Bindings
//!
//! This module provides Python-specific IPC functionality, building on
//! the platform-agnostic abstractions from `auroraview-core::ipc`.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    auroraview-core/ipc                       │
//! │  (Platform-agnostic: IpcMessage, IpcMetrics)                 │
//! └─────────────────────────────────────────────────────────────┘
//!                              ↑
//!                              │ uses
//! ┌─────────────────────────────────────────────────────────────┐
//! │                 src/ipc (Python bindings)                    │
//! │  - IpcHandler (Python callback management)                   │
//! │  - json_to_python, python_to_json                            │
//! │  - ThreadedBackend (PyO3-specific)                           │
//! │  - MessageQueue (WebView integration)                        │
//! └─────────────────────────────────────────────────────────────┘
//! ```

pub mod async_handler;
pub mod backend;
pub mod batch;
pub mod handler;
pub mod js_callback;
pub mod json;
pub mod message_queue;
pub mod threaded;

// Re-export core IPC types
pub use auroraview_core::ipc::{IpcMessage, IpcMetrics, IpcMetricsSnapshot, IpcMode};

// Re-export Python-specific types
pub use handler::IpcHandler;
pub use message_queue::{MessageQueue, WebViewMessage, WindowEventType};

// Re-export for API consumers
#[cfg(feature = "python-bindings")]
#[allow(unused_imports)]
pub use backend::IpcBackend;
#[cfg(feature = "python-bindings")]
#[allow(unused_imports)]
pub use handler::PythonCallback;
#[allow(unused_imports)]
pub use message_queue::MessageQueueConfig;
#[cfg(feature = "python-bindings")]
#[allow(unused_imports)]
pub use threaded::{ThreadedBackend, ThreadedConfig};

// Re-export async handler
pub use async_handler::{AsyncIpcConfig, AsyncIpcHandler, AsyncIpcMessage};

// Re-export JS callback manager
pub use js_callback::{JsCallbackManager, JsCallbackResult};

// Re-export JSON conversion functions
#[cfg(feature = "python-bindings")]
pub use json::{json_to_python, python_to_json};

/// Helper function to convert Python dict to JSON value
#[cfg(feature = "python-bindings")]
pub fn dict_to_json(
    dict: &pyo3::Bound<'_, pyo3::types::PyDict>,
) -> pyo3::PyResult<serde_json::Value> {
    python_to_json(dict.as_any())
}
