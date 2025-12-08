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
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use super::config::WebViewConfig;
use super::event_loop::UserEvent;
use super::webview_inner::WebViewInner;
use crate::ipc::{IpcHandler, JsCallbackManager, MessageQueue};

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
