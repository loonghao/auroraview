//! Thread-safe WebView Proxy for cross-thread operations
//!
//! This module provides a `WebViewProxy` that can be safely shared across threads.
//! It only contains thread-safe fields (`Arc<...>`) and allows operations like
//! `eval_js`, `emit`, etc. to be called from any thread.
//!
//! ## Problem
//! `AuroraView` is marked as `unsendable` because it contains non-Send fields
//! like `Rc<RefCell<...>>`. When calling methods from a different thread than
//! the one that created the WebView, PyO3 panics with:
//! ```text
//! assertion `left == right` failed: _core::webview::core::AuroraView is unsendable,
//! but sent to another thread
//! ```
//!
//! ## Solution
//! `WebViewProxy` only contains `Arc<...>` fields which are `Send + Sync`.
//! Operations are performed via the thread-safe `MessageQueue` which is
//! processed by the WebView's event loop on the correct thread.
//!
//! ## Usage
//! ```python
//! # In HWND mode - WebView runs in background thread
//! def create_webview_thread():
//!     webview = WebView(...)
//!     proxy = webview.get_proxy()  # Get thread-safe proxy
//!     self._proxy = proxy          # Store for cross-thread access
//!     webview.show_blocking()
//!
//! # From DCC main thread - safe!
//! self._proxy.eval_js("console.log('Hello from DCC!')")
//! ```

use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::sync::Arc;

use crate::ipc::{JsCallbackManager, MessageQueue, WebViewMessage};

/// Thread-safe proxy for cross-thread WebView operations
///
/// This proxy can be safely shared across threads and used to
/// communicate with the WebView running in another thread.
/// All operations are performed via a message queue that is
/// processed by the WebView's event loop.
#[pyclass(name = "WebViewProxy")]
#[derive(Clone)]
pub struct WebViewProxy {
    /// Thread-safe message queue for WebView operations
    message_queue: Arc<MessageQueue>,
    /// Thread-safe JavaScript callback manager
    js_callback_manager: Arc<JsCallbackManager>,
}

// SAFETY: WebViewProxy only contains Arc<...> fields which are Send + Sync
unsafe impl Send for WebViewProxy {}
unsafe impl Sync for WebViewProxy {}

impl WebViewProxy {
    /// Create a new WebViewProxy from the given components
    pub fn new(
        message_queue: Arc<MessageQueue>,
        js_callback_manager: Arc<JsCallbackManager>,
    ) -> Self {
        Self {
            message_queue,
            js_callback_manager,
        }
    }
}

#[pymethods]
impl WebViewProxy {
    /// Execute JavaScript code in the WebView (thread-safe)
    ///
    /// This method can be called from ANY thread. The script is queued
    /// and will be executed by the WebView's event loop.
    ///
    /// Args:
    ///     script (str): JavaScript code to execute
    ///
    /// Example:
    ///     >>> proxy = webview.get_proxy()
    ///     >>> proxy.eval_js("console.log('Hello from another thread!')")
    fn eval_js(&self, script: &str) -> PyResult<()> {
        tracing::debug!("[WebViewProxy] Queueing JavaScript execution: {}", script);
        self.message_queue
            .push(WebViewMessage::EvalJs(script.to_string()));
        Ok(())
    }

    /// Execute JavaScript code asynchronously with result callback (thread-safe)
    ///
    /// Args:
    ///     script (str): JavaScript code to execute
    ///     callback (callable): Python function(result, error) to call with result
    ///     timeout_ms (int): Timeout in milliseconds (default: 5000)
    #[pyo3(signature = (script, callback, timeout_ms=5000))]
    fn eval_js_async(&self, script: &str, callback: Py<PyAny>, timeout_ms: u64) -> PyResult<()> {
        let callback_id = self.js_callback_manager.next_callback_id();

        tracing::debug!(
            "[WebViewProxy] Queueing async JavaScript (id={}, timeout={}ms): {}",
            callback_id,
            timeout_ms,
            script
        );

        self.js_callback_manager
            .register_callback_with_timeout(callback_id, callback, timeout_ms);

        self.message_queue.push(WebViewMessage::EvalJsAsync {
            script: script.to_string(),
            callback_id,
        });

        Ok(())
    }

    /// Emit an event to JavaScript (thread-safe)
    ///
    /// Args:
    ///     event_name (str): Name of the event
    ///     data (dict): Data to send with the event
    fn emit(&self, event_name: &str, data: &Bound<'_, PyDict>) -> PyResult<()> {
        tracing::debug!("[WebViewProxy] Emitting event: {}", event_name);

        // Convert Python dict to JSON
        let json_data = crate::ipc::dict_to_json(data)?;

        self.message_queue.push(WebViewMessage::EmitEvent {
            event_name: event_name.to_string(),
            data: json_data,
        });

        Ok(())
    }

    /// Load a URL in the WebView (thread-safe)
    ///
    /// Args:
    ///     url (str): URL to load
    fn load_url(&self, url: &str) -> PyResult<()> {
        tracing::debug!("[WebViewProxy] Loading URL: {}", url);
        self.message_queue
            .push(WebViewMessage::LoadUrl(url.to_string()));
        Ok(())
    }

    /// Load HTML content in the WebView (thread-safe)
    ///
    /// Args:
    ///     html (str): HTML content to load
    fn load_html(&self, html: &str) -> PyResult<()> {
        tracing::debug!("[WebViewProxy] Loading HTML content");
        self.message_queue
            .push(WebViewMessage::LoadHtml(html.to_string()));
        Ok(())
    }

    /// Reload the current page (thread-safe)
    fn reload(&self) -> PyResult<()> {
        tracing::debug!("[WebViewProxy] Reloading page");
        self.message_queue.push(WebViewMessage::Reload);
        Ok(())
    }

    /// Close the WebView window (thread-safe)
    ///
    /// This sends a close message to the WebView's event loop,
    /// which will trigger the window to close and the event loop to exit.
    ///
    /// Example:
    ///     >>> proxy = webview.get_proxy()
    ///     >>> proxy.close()  # Closes the window from another thread
    fn close(&self) -> PyResult<()> {
        tracing::info!("[WebViewProxy] Requesting window close");
        self.message_queue.push(WebViewMessage::Close);
        Ok(())
    }

    /// Check if the proxy is valid (message queue is available)
    fn is_valid(&self) -> bool {
        // The proxy is always valid as long as it exists
        // The message queue might be disconnected but that's handled in push()
        true
    }

    fn __repr__(&self) -> String {
        format!(
            "WebViewProxy(queue_len={}, pending_callbacks={})",
            self.message_queue.len(),
            self.js_callback_manager.pending_count()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webview_proxy_is_send_and_sync() {
        // Compile-time check: WebViewProxy must be Send + Sync
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<WebViewProxy>();
    }

    #[test]
    fn test_webview_proxy_clone() {
        let mq = Arc::new(MessageQueue::new());
        let cb = Arc::new(JsCallbackManager::new());
        let proxy = WebViewProxy::new(mq.clone(), cb.clone());

        // Clone should work
        let proxy2 = proxy.clone();
        assert!(proxy2.is_valid());
    }

    #[test]
    fn test_webview_proxy_eval_js_queues_message() {
        let mq = Arc::new(MessageQueue::new());
        let cb = Arc::new(JsCallbackManager::new());
        let proxy = WebViewProxy::new(mq.clone(), cb.clone());

        // Queue should be empty initially
        assert_eq!(mq.len(), 0);

        // After eval_js, queue should have one message
        proxy.eval_js("console.log('test')").unwrap();
        assert_eq!(mq.len(), 1);
    }

    #[test]
    fn test_webview_proxy_load_url_queues_message() {
        let mq = Arc::new(MessageQueue::new());
        let cb = Arc::new(JsCallbackManager::new());
        let proxy = WebViewProxy::new(mq.clone(), cb.clone());

        proxy.load_url("https://example.com").unwrap();
        assert_eq!(mq.len(), 1);
    }

    #[test]
    fn test_webview_proxy_load_html_queues_message() {
        let mq = Arc::new(MessageQueue::new());
        let cb = Arc::new(JsCallbackManager::new());
        let proxy = WebViewProxy::new(mq.clone(), cb.clone());

        proxy.load_html("<html></html>").unwrap();
        assert_eq!(mq.len(), 1);
    }

    #[test]
    fn test_webview_proxy_reload_queues_message() {
        let mq = Arc::new(MessageQueue::new());
        let cb = Arc::new(JsCallbackManager::new());
        let proxy = WebViewProxy::new(mq.clone(), cb.clone());

        proxy.reload().unwrap();
        assert_eq!(mq.len(), 1);
    }

    #[test]
    fn test_webview_proxy_thread_safety() {
        use std::thread;

        let mq = Arc::new(MessageQueue::new());
        let cb = Arc::new(JsCallbackManager::new());
        let proxy = WebViewProxy::new(mq.clone(), cb.clone());

        // Clone proxy and send to another thread
        let proxy_clone = proxy.clone();
        let handle = thread::spawn(move || {
            proxy_clone.eval_js("from_thread()").unwrap();
        });

        handle.join().unwrap();

        // Message should be in queue from the other thread
        assert_eq!(mq.len(), 1);
    }

    #[test]
    fn test_webview_proxy_multiple_threads() {
        use std::thread;

        let mq = Arc::new(MessageQueue::new());
        let cb = Arc::new(JsCallbackManager::new());
        let proxy = WebViewProxy::new(mq.clone(), cb.clone());

        let mut handles = vec![];
        for i in 0..10 {
            let proxy_clone = proxy.clone();
            let handle = thread::spawn(move || {
                proxy_clone.eval_js(&format!("thread_{}", i)).unwrap();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // All 10 messages should be in queue
        assert_eq!(mq.len(), 10);
    }
}
