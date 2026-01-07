//! IPC Handler for WebView Communication
//!
//! This module manages communication between Python and JavaScript,
//! handling event callbacks and message routing.
//!
//! ## Thread Safety
//!
//! Python callbacks registered via `register_python_callback` are executed
//! on the main thread (the thread that created the WebView) via a deferred
//! execution mechanism. This is critical for DCC integration where Python
//! callbacks may need to access non-thread-safe APIs (e.g., Maya cmds).
//!
//! When a JavaScript event is received from the IPC thread:
//! 1. The callback is NOT executed immediately
//! 2. Instead, a `PythonCallbackDeferred` message is pushed to the message queue
//! 3. The main thread's event loop processes this message and executes the callback
//!
//! This ensures all Python callbacks run on the main thread, avoiding GIL
//! contention and thread-safety issues.

use dashmap::DashMap;
#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;
#[cfg(feature = "python-bindings")]
use pyo3::{Py, PyAny};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

// Re-export IpcMessage from backend module
pub use super::backend::IpcMessage;
pub use super::message_queue::{MessageQueue, WebViewMessage};

#[cfg(feature = "python-bindings")]
use super::js_callback::{JsCallbackManager, JsCallbackResult};

/// IPC callback type (Rust closures)
pub type IpcCallback = Arc<dyn Fn(IpcMessage) -> Result<serde_json::Value, String> + Send + Sync>;

/// Python callback wrapper - stores Python callable objects
#[cfg(feature = "python-bindings")]
pub struct PythonCallback {
    /// Python callable object
    pub callback: Py<PyAny>,
}

#[cfg(feature = "python-bindings")]
impl PythonCallback {
    /// Create a new Python callback wrapper
    pub fn new(callback: Py<PyAny>) -> Self {
        Self { callback }
    }

    /// Call the Python callback with the given data
    pub fn call(&self, data: super::json::Value) -> Result<(), String> {
        Python::attach(|py| {
            // Convert JSON value to Python object using the optimized json module
            let py_data = match super::json::json_to_python(py, &data) {
                Ok(obj) => obj,
                Err(e) => {
                    tracing::error!("Failed to convert JSON to Python: {}", e);
                    return Err(format!("Failed to convert JSON to Python: {}", e));
                }
            };

            // Call the Python callback
            match self.callback.call1(py, (py_data,)) {
                Ok(_) => {
                    tracing::debug!("Python callback executed successfully");
                    Ok(())
                }
                Err(e) => {
                    tracing::error!("Python callback error: {}", e);
                    Err(format!("Python callback error: {}", e))
                }
            }
        })
    }
}

/// Global callback ID counter for unique callback identification
#[cfg(feature = "python-bindings")]
static CALLBACK_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Python callback entry with unique ID for deferred execution
#[cfg(feature = "python-bindings")]
pub struct PythonCallbackEntry {
    /// Unique callback ID
    pub id: u64,
    /// The actual callback
    pub callback: PythonCallback,
}

/// IPC handler for managing communication between Python and JavaScript
///
/// Uses DashMap for lock-free concurrent callback storage, improving
/// performance in high-throughput scenarios.
///
/// ## Thread Safety
///
/// Python callbacks are executed on the main thread via deferred execution.
/// When an IPC event is received, callbacks are not executed immediately.
/// Instead, they are scheduled for execution on the main thread's event loop.
pub struct IpcHandler {
    /// Registered event callbacks (Rust closures) - lock-free concurrent map
    callbacks: Arc<DashMap<String, Vec<IpcCallback>>>,

    /// Registered Python callbacks by event name - lock-free concurrent map
    /// Maps event name to list of callback IDs
    #[cfg(feature = "python-bindings")]
    python_callback_events: Arc<DashMap<String, Vec<u64>>>,

    /// Python callbacks by ID for deferred execution
    #[cfg(feature = "python-bindings")]
    python_callbacks_by_id: Arc<DashMap<u64, PythonCallbackEntry>>,

    /// JavaScript callback manager for async execution results
    #[cfg(feature = "python-bindings")]
    js_callback_manager: Option<Arc<JsCallbackManager>>,

    /// Message queue for sending events to WebView
    message_queue: Option<Arc<MessageQueue>>,

    /// Whether to use deferred (main-thread) execution for Python callbacks
    /// When true, callbacks are executed on the main thread via message queue
    /// When false, callbacks are executed immediately (legacy behavior)
    #[cfg(feature = "python-bindings")]
    use_deferred_callbacks: bool,
}

impl IpcHandler {
    /// Create a new IPC handler
    ///
    /// By default, Python callbacks are executed via deferred execution on
    /// the main thread. This can be disabled with `set_deferred_callbacks(false)`.
    pub fn new() -> Self {
        Self {
            callbacks: Arc::new(DashMap::new()),
            #[cfg(feature = "python-bindings")]
            python_callback_events: Arc::new(DashMap::new()),
            #[cfg(feature = "python-bindings")]
            python_callbacks_by_id: Arc::new(DashMap::new()),
            #[cfg(feature = "python-bindings")]
            js_callback_manager: None,
            message_queue: None,
            #[cfg(feature = "python-bindings")]
            use_deferred_callbacks: true, // Safe by default
        }
    }

    /// Set the message queue for sending events to WebView
    pub fn set_message_queue(&mut self, queue: Arc<MessageQueue>) {
        self.message_queue = Some(queue);
    }

    /// Enable or disable deferred (main-thread) execution for Python callbacks
    ///
    /// When enabled (default), Python callbacks are executed on the main thread
    /// via the message queue. This is safer for DCC integration.
    ///
    /// When disabled, callbacks are executed immediately on the IPC thread.
    /// Only disable this if you understand the threading implications.
    #[cfg(feature = "python-bindings")]
    pub fn set_deferred_callbacks(&mut self, enabled: bool) {
        self.use_deferred_callbacks = enabled;
        tracing::info!("Deferred Python callbacks: {}", enabled);
    }

    /// Set the JavaScript callback manager for handling async execution results
    #[cfg(feature = "python-bindings")]
    pub fn set_js_callback_manager(&mut self, manager: Arc<JsCallbackManager>) {
        self.js_callback_manager = Some(manager);
    }

    /// Register a Rust callback for an event
    #[allow(dead_code)]
    pub fn on<F>(&self, event: &str, callback: F)
    where
        F: Fn(IpcMessage) -> Result<serde_json::Value, String> + Send + Sync + 'static,
    {
        self.callbacks
            .entry(event.to_string())
            .or_default()
            .push(Arc::new(callback));
    }

    /// Register a Python callback for an event
    ///
    /// The callback will be executed on the main thread when the event is received.
    /// Returns the unique callback ID for later removal.
    #[cfg(feature = "python-bindings")]
    pub fn register_python_callback(&self, event: &str, callback: Py<PyAny>) -> u64 {
        let id = CALLBACK_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

        // Store callback entry by ID
        self.python_callbacks_by_id.insert(
            id,
            PythonCallbackEntry {
                id,
                callback: PythonCallback::new(callback),
            },
        );

        // Register event -> callback ID mapping
        self.python_callback_events
            .entry(event.to_string())
            .or_default()
            .push(id);

        tracing::debug!("Registered Python callback {} for event: {}", id, event);
        id
    }

    /// Register multiple Python callbacks at once (batch registration)
    ///
    /// This is more efficient than calling register_python_callback multiple times
    /// because it logs only once for the entire batch.
    #[cfg(feature = "python-bindings")]
    pub fn register_python_callbacks_batch(&self, callbacks: Vec<(String, Py<PyAny>)>) {
        let count = callbacks.len();
        for (event, callback) in callbacks {
            self.register_python_callback(&event, callback);
        }
        tracing::info!("Registered {} Python callbacks in batch", count);
    }

    /// Execute a deferred Python callback by ID
    ///
    /// This is called from the main thread's event loop when processing
    /// `PythonCallbackDeferred` messages.
    #[cfg(feature = "python-bindings")]
    pub fn execute_deferred_callback(
        &self,
        callback_id: u64,
        event_name: &str,
        data: serde_json::Value,
    ) -> Result<(), String> {
        if let Some(entry) = self.python_callbacks_by_id.get(&callback_id) {
            tracing::debug!(
                "Executing deferred callback {} for event: {}",
                callback_id,
                event_name
            );
            entry.callback.call(data)
        } else {
            tracing::warn!(
                "Deferred callback {} not found for event: {}",
                callback_id,
                event_name
            );
            Err(format!("Callback {} not found", callback_id))
        }
    }

    /// Emit an event to JavaScript
    ///
    /// Sends an event to the WebView via the message queue.
    /// The event will be dispatched to JavaScript handlers registered with `auroraview.on()`.
    #[allow(dead_code)]
    pub fn emit(&self, event: &str, data: serde_json::Value) -> Result<(), String> {
        tracing::debug!("Emitting IPC event: {}", event);

        if let Some(ref queue) = self.message_queue {
            queue.push(WebViewMessage::EmitEvent {
                event_name: event.to_string(),
                data,
            });
            Ok(())
        } else {
            let err = "Message queue not set - cannot emit event to WebView".to_string();
            tracing::warn!("{}", err);
            Err(err)
        }
    }

    /// Handle incoming message from JavaScript
    ///
    /// ## Thread Safety
    ///
    /// When `use_deferred_callbacks` is enabled (default), Python callbacks are
    /// NOT executed immediately. Instead, `PythonCallbackDeferred` messages are
    /// pushed to the message queue for execution on the main thread.
    ///
    /// This ensures Python callbacks always run on the main thread, which is
    /// critical for DCC integration where Python APIs are not thread-safe.
    #[allow(dead_code)]
    pub fn handle_message(&self, message: IpcMessage) -> Result<serde_json::Value, String> {
        tracing::debug!("Handling IPC message: {}", message.event);

        // Handle internal JS callback result event
        #[cfg(feature = "python-bindings")]
        if message.event == "__js_callback_result__" {
            return self.handle_js_callback_result(&message.data);
        }

        // Handle internal ready event
        // Note: We still process Python callbacks for this event so users can hook into it
        if message.event == "__auroraview_ready" {
            tracing::debug!("WebView bridge ready: {:?}", message.data);
            // Don't return early - let it fall through to Python callback handling
        }

        // First try Python callbacks (only when python-bindings feature is enabled)
        #[cfg(feature = "python-bindings")]
        {
            if let Some(callback_ids) = self.python_callback_events.get(&message.event) {
                let ids: Vec<u64> = callback_ids.value().clone();
                drop(callback_ids); // Release the lock before processing

                if !ids.is_empty() {
                    tracing::info!(
                        "Found {} Python callbacks for event: {}",
                        ids.len(),
                        message.event
                    );

                    for callback_id in ids {
                        if self.use_deferred_callbacks {
                            // Deferred execution: push to message queue for main thread
                            if let Some(ref queue) = self.message_queue {
                                tracing::debug!(
                                    "Deferring Python callback {} for event: {}",
                                    callback_id,
                                    message.event
                                );
                                queue.push(WebViewMessage::PythonCallbackDeferred {
                                    callback_id,
                                    event_name: message.event.clone(),
                                    data: message.data.clone(),
                                });
                            } else {
                                tracing::warn!(
                                    "Cannot defer callback {}: no message queue. Executing immediately.",
                                    callback_id
                                );
                                // Fallback to immediate execution
                                if let Err(e) = self.execute_deferred_callback(
                                    callback_id,
                                    &message.event,
                                    message.data.clone(),
                                ) {
                                    tracing::error!("Python callback error: {}", e);
                                    return Err(e);
                                }
                            }
                        } else {
                            // Immediate execution (legacy behavior)
                            tracing::info!(
                                "Executing Python callback {} immediately for event: {}",
                                callback_id,
                                message.event
                            );
                            if let Err(e) = self.execute_deferred_callback(
                                callback_id,
                                &message.event,
                                message.data.clone(),
                            ) {
                                tracing::error!("Python callback error: {}", e);
                                return Err(e);
                            }
                        }
                    }
                    return Ok(
                        serde_json::json!({"status": "ok", "deferred": self.use_deferred_callbacks}),
                    );
                }
            }
        }

        // For __auroraview_ready, return success even if no Python callback is registered
        if message.event == "__auroraview_ready" {
            return Ok(serde_json::json!({"status": "ok", "message": "ready acknowledged"}));
        }

        // Then try Rust callbacks
        if let Some(event_callbacks) = self.callbacks.get(&message.event) {
            if let Some(callback) = event_callbacks.value().first() {
                match callback(message.clone()) {
                    Ok(result) => return Ok(result),
                    Err(e) => {
                        tracing::error!("IPC callback error: {}", e);
                        return Err(e);
                    }
                }
            }
        }

        // No callback found
        Err(format!(
            "No handler registered for event: {}",
            message.event
        ))
    }

    /// Get the count of registered events (both Rust and Python callbacks)
    pub fn registered_event_count(&self) -> usize {
        let rust_count = self.callbacks.len();
        #[cfg(feature = "python-bindings")]
        let python_count = self.python_callback_events.len();
        #[cfg(not(feature = "python-bindings"))]
        let python_count = 0;
        rust_count + python_count
    }

    /// Handle JavaScript callback result from async execution
    #[cfg(feature = "python-bindings")]
    fn handle_js_callback_result(
        &self,
        data: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let callback_id = data
            .get("callback_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| "Missing callback_id in JS callback result".to_string())?;

        let result_value = data.get("result").cloned();
        let error_value = data.get("error").cloned();

        tracing::debug!(
            "Processing JS callback result: id={}, has_result={}, has_error={}",
            callback_id,
            result_value.is_some(),
            error_value.is_some()
        );

        // Build the callback result
        let js_result = JsCallbackResult {
            value: result_value,
            error: error_value.and_then(|e| {
                e.get("message")
                    .and_then(|m| m.as_str())
                    .map(|s| s.to_string())
            }),
        };

        // Complete the callback if we have a manager
        if let Some(ref manager) = self.js_callback_manager {
            if let Err(e) = manager.complete_callback(callback_id, js_result) {
                tracing::error!("Failed to complete JS callback {}: {}", callback_id, e);
                return Err(e);
            }
        } else {
            tracing::warn!(
                "JS callback result received but no callback manager set (id={})",
                callback_id
            );
        }

        Ok(serde_json::json!({"status": "ok"}))
    }

    /// Remove all callbacks for an event
    #[allow(dead_code)]
    pub fn off(&self, event: &str) {
        self.callbacks.remove(event);
        #[cfg(feature = "python-bindings")]
        {
            // Remove callback IDs for this event and clean up the callbacks
            if let Some((_, ids)) = self.python_callback_events.remove(event) {
                for id in ids {
                    self.python_callbacks_by_id.remove(&id);
                }
            }
        }
    }

    /// Clear all callbacks
    #[allow(dead_code)]
    pub fn clear(&self) {
        self.callbacks.clear();
        #[cfg(feature = "python-bindings")]
        {
            self.python_callback_events.clear();
            self.python_callbacks_by_id.clear();
        }
    }
}

impl Default for IpcHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "python-bindings")]
    use pyo3::types::{PyList, PyModule};

    #[cfg(feature = "python-bindings")]
    fn py_append_collector() -> (Py<PyAny>, Py<PyAny>) {
        Python::attach(|py| {
            let seen = PyList::new(py, [py.None()])?;
            let m = PyModule::from_code(
                py,
                c"def make_cb(seen):\n    def cb(x):\n        seen.append(x)\n    return cb\n",
                c"m.py",
                c"m",
            )
            .unwrap();
            let make_cb = m.getattr("make_cb").unwrap();
            // Keep an owned handle to the list so we can inspect it later
            let seen_obj: Py<PyAny> = seen.clone().unbind().into();
            let seen_bound = seen_obj.bind(py).cast::<PyList>().unwrap();
            let cb = make_cb.call1((seen_bound,)).unwrap().clone().unbind();
            Ok::<(Py<PyAny>, Py<PyAny>), pyo3::PyErr>((cb, seen_obj))
        })
        .unwrap()
    }

    #[cfg(feature = "python-bindings")]
    #[test]
    fn test_python_callback_flow() {
        let handler = IpcHandler::new();
        let (cb, seen_obj) = py_append_collector();
        handler.register_python_callback("evt", cb);

        let msg = IpcMessage {
            event: "evt".to_string(),
            data: serde_json::json!({"a":1}),
            id: None,
        };
        let res = handler.handle_message(msg);
        assert!(res.is_ok());

        Python::attach(|py| {
            let seen = seen_obj.bind(py).cast::<PyList>().unwrap();
            assert_eq!(seen.len(), 1);
            let first = seen.get_item(0).unwrap();
            // first is a Python object converted from JSON dict
            let dict = first.cast::<pyo3::types::PyDict>().unwrap();
            if let Ok(Some(a_val)) = dict.get_item("a") {
                let a: i64 = a_val.extract().unwrap();
                assert_eq!(a, 1);
            } else {
                panic!("missing key a");
            }
        });
    }

    #[test]
    fn test_rust_callback_flow_and_no_handler() {
        let handler = IpcHandler::new();
        handler.on("evt2", |m| {
            assert_eq!(m.event, "evt2");
            Ok(serde_json::json!({"ok": true}))
        });
        let res = handler.handle_message(IpcMessage {
            event: "evt2".to_string(),
            data: serde_json::json!({}),
            id: None,
        });
        assert_eq!(res.unwrap(), serde_json::json!({"ok": true}));

        // No handler case
        let err = handler
            .handle_message(IpcMessage {
                event: "unknown".to_string(),
                data: serde_json::json!({}),
                id: None,
            })
            .unwrap_err();
        assert!(err.contains("No handler registered"));
    }

    #[test]
    fn test_emit_without_message_queue() {
        let handler = IpcHandler::new();
        // Without message queue, emit should return an error
        let result = handler.emit("test_event", serde_json::json!({"data": "test"}));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Message queue not set"));
    }

    #[test]
    fn test_emit_with_message_queue() {
        let mut handler = IpcHandler::new();
        let queue = Arc::new(MessageQueue::new());
        handler.set_message_queue(queue.clone());

        // With message queue, emit should succeed
        let result = handler.emit("test_event", serde_json::json!({"data": "test"}));
        assert!(result.is_ok());

        // Verify message was pushed to queue
        assert_eq!(queue.len(), 1);
    }
}
