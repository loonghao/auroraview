//! AuroraView Core - Python-facing WebView class
//!
//! This module is split into multiple files for maintainability:
//! - `main.rs`: Constructor, lifecycle methods, property getters
//! - `js.rs`: JavaScript execution methods
//! - `storage.rs`: Storage and Cookie APIs
//! - `events.rs`: Event callback methods
//! - `dialogs.rs`: File and message dialog methods
//! - `bom.rs`: Browser Object Model APIs
//! - `multiwindow.rs`: Multi-window management APIs
//! - `plugins.rs`: Plugin system integration

use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use super::config::WebViewConfig;
use super::event_loop::UserEvent;
use super::webview_inner::WebViewInner;
use crate::bindings::webview::py_dict_to_json;
use crate::ipc::{IpcHandler, JsCallbackManager, MessageQueue, WebViewMessage};

// Sub-modules containing #[pymethods] implementations
#[cfg(feature = "templates")]
mod api; // API registration methods (uses Askama templates)
mod bom;
mod dialogs;
mod dom; // DOM operation methods (high-performance)
mod events;
mod js;
mod main;
mod multiwindow;
pub mod plugins;
mod storage;

pub use plugins::PluginManager;

/// Thread-safe event emitter for cross-thread event emission
///
/// This class can be safely used from any thread (including ProcessPlugin background threads)
/// to emit events to the JavaScript frontend. It only holds an Arc<MessageQueue> which is
/// thread-safe.
///
/// Example:
///     emitter = webview.create_emitter()
///     # Can be used from any thread:
///     emitter.emit("my_event", {"data": "value"})
#[pyclass(name = "EventEmitter")]
pub struct EventEmitter {
    message_queue: Arc<MessageQueue>,
}

#[pymethods]
impl EventEmitter {
    /// Emit an event to JavaScript (thread-safe)
    ///
    /// Args:
    ///     event_name (str): Name of the event
    ///     data (dict): Data to send with the event
    fn emit(&self, event_name: &str, data: &Bound<'_, PyDict>) -> PyResult<()> {
        let json_data = py_dict_to_json(data)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        self.message_queue.push(WebViewMessage::EmitEvent {
            event_name: event_name.to_string(),
            data: json_data,
        });

        Ok(())
    }
}

/// Python-facing WebView class
/// Supports both standalone and embedded modes (for DCC integration)
#[pyclass(name = "WebView", unsendable)]
pub struct AuroraView {
    pub(crate) inner: Rc<RefCell<Option<WebViewInner>>>,
    pub(crate) config: Rc<RefCell<WebViewConfig>>,
    pub(crate) ipc_handler: Arc<IpcHandler>,
    /// Thread-safe message queue for cross-thread communication
    pub(crate) message_queue: Arc<MessageQueue>,
    /// Event loop proxy for sending close events (standalone mode only)
    pub(crate) event_loop_proxy: Rc<RefCell<Option<tao::event_loop::EventLoopProxy<UserEvent>>>>,
    /// JavaScript callback manager for async execution
    pub(crate) js_callback_manager: Arc<JsCallbackManager>,
    /// Callback invoked when WebView2 HWND is created (Windows only)
    pub(crate) on_hwnd_created: Rc<RefCell<Option<Py<PyAny>>>>,
    /// Cached HWND value (Windows only) - stored at AuroraView level
    /// so it can be accessed even when inner RefCell is borrowed
    #[cfg(target_os = "windows")]
    pub(crate) cached_hwnd: Rc<RefCell<Option<u64>>>,
}

#[pymethods]
impl AuroraView {
    /// Create a thread-safe event emitter
    ///
    /// Returns an EventEmitter that can be used from any thread to emit events
    /// to the JavaScript frontend. This is useful for plugin callbacks that run
    /// on background threads.
    ///
    /// Example:
    ///     emitter = webview.create_emitter()
    ///     plugins.set_emit_callback(emitter.emit)
    fn create_emitter(&self) -> EventEmitter {
        EventEmitter {
            message_queue: Arc::clone(&self.message_queue),
        }
    }
}

/// Implement Drop to track when AuroraView is destroyed
impl Drop for AuroraView {
    fn drop(&mut self) {
        let title = self.config.borrow().title.clone();
        tracing::warn!(
            "[CLOSE] [AuroraView::drop] WebView '{}' is being destroyed!",
            title
        );
    }
}
