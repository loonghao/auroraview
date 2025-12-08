//! AuroraView Core - Event Callbacks and IPC
//!
//! This module contains event-related methods:
//! - Navigation event callbacks
//! - Window event callbacks
//! - IPC event registration
//! - Performance monitoring

use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;

use super::AuroraView;
use crate::bindings::webview::py_dict_to_json;
use crate::ipc::WebViewMessage;

#[pymethods]
impl AuroraView {
    // ========================================
    // IPC Event Registration
    // ========================================

    /// Emit an event to JavaScript
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

    /// Emit multiple events to JavaScript in a single batch.
    ///
    /// This is more efficient than calling emit() multiple times because
    /// all events are queued together and processed in one go.
    ///
    /// Args:
    ///     events: List of tuples (event_name, data_dict)
    ///
    /// Example:
    ///     >>> webview.emit_batch([
    ///     ...     ("update", {"field": "name", "value": "John"}),
    ///     ...     ("update", {"field": "email", "value": "john@example.com"}),
    ///     ...     ("batch_complete", {"count": 2}),
    ///     ... ])
    #[pyo3(signature = (events))]
    fn emit_batch(&self, events: Vec<(String, Bound<'_, PyDict>)>) -> PyResult<usize> {
        if events.is_empty() {
            return Ok(0);
        }

        let count = events.len();
        tracing::info!("Emitting {} events in batch", count);

        for (event_name, data) in events {
            let json_data = py_dict_to_json(&data)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

            self.message_queue.push(WebViewMessage::EmitEvent {
                event_name,
                data: json_data,
            });
        }

        Ok(count)
    }

    /// Register a Python callback for JavaScript events
    ///
    /// Args:
    ///     event_name (str): Name of the event to listen for
    ///     callback (callable): Python function to call when event occurs
    fn on(&self, event_name: &str, callback: Py<PyAny>) -> PyResult<()> {
        tracing::debug!("Registering callback for event: {}", event_name);
        self.ipc_handler
            .register_python_callback(event_name, callback);
        Ok(())
    }

    /// Register multiple Python callbacks at once (batch registration)
    ///
    /// This is more efficient than calling on() multiple times because
    /// it logs only once for the entire batch.
    ///
    /// Args:
    ///     callbacks: List of tuples (event_name, callback)
    ///
    /// Example:
    ///     >>> webview.on_batch([
    ///     ...     ("navigation_started", on_nav_start),
    ///     ...     ("navigation_finished", on_nav_finish),
    ///     ...     ("api.echo", handle_echo),
    ///     ... ])
    #[pyo3(signature = (callbacks))]
    fn on_batch(&self, callbacks: Vec<(String, Py<PyAny>)>) -> PyResult<usize> {
        let count = callbacks.len();
        if count == 0 {
            return Ok(0);
        }
        tracing::info!("Registering {} callbacks in batch", count);
        self.ipc_handler.register_python_callbacks_batch(callbacks);
        Ok(count)
    }

    // ========================================
    // Navigation Event Callbacks
    // ========================================

    /// Register a callback for navigation started events (DEPRECATED)
    fn on_navigation_started(&self, callback: Py<PyAny>) -> PyResult<()> {
        self.ipc_handler
            .register_python_callback("navigation_started", callback);
        Ok(())
    }

    /// Register a callback for navigation completed events (DEPRECATED)
    fn on_navigation_completed(&self, callback: Py<PyAny>) -> PyResult<()> {
        self.ipc_handler
            .register_python_callback("navigation_completed", callback);
        Ok(())
    }

    /// Register a callback for navigation failed events (DEPRECATED)
    fn on_navigation_failed(&self, callback: Py<PyAny>) -> PyResult<()> {
        self.ipc_handler
            .register_python_callback("navigation_failed", callback);
        Ok(())
    }

    /// Register a callback for all navigation events (unified API)
    fn on_navigation(&self, callback: Py<PyAny>) -> PyResult<()> {
        self.ipc_handler
            .register_python_callback("navigation", callback);
        Ok(())
    }

    /// Register a callback for load progress changes
    fn on_progress(&self, callback: Py<PyAny>) -> PyResult<()> {
        self.ipc_handler
            .register_python_callback("load_progress", callback);
        Ok(())
    }

    /// Register a callback for load progress changes (DEPRECATED)
    fn on_load_progress(&self, callback: Py<PyAny>) -> PyResult<()> {
        self.on_progress(callback)
    }

    /// Register a callback for title changes
    fn on_title_changed(&self, callback: Py<PyAny>) -> PyResult<()> {
        self.ipc_handler
            .register_python_callback("title_changed", callback);
        Ok(())
    }

    /// Register a callback for URL changes
    fn on_url_changed(&self, callback: Py<PyAny>) -> PyResult<()> {
        self.ipc_handler
            .register_python_callback("url_changed", callback);
        Ok(())
    }

    /// Register a callback for DOM ready event
    fn on_dom_ready(&self, callback: Py<PyAny>) -> PyResult<()> {
        self.ipc_handler
            .register_python_callback("dom_ready", callback);
        Ok(())
    }

    // ========================================
    // Window Event Callbacks
    // ========================================

    /// Register a callback for window show event
    fn on_window_show(&self, callback: Py<PyAny>) -> PyResult<()> {
        self.ipc_handler
            .register_python_callback("window_show", callback);
        Ok(())
    }

    /// Register a callback for window hide event
    fn on_window_hide(&self, callback: Py<PyAny>) -> PyResult<()> {
        self.ipc_handler
            .register_python_callback("window_hide", callback);
        Ok(())
    }

    /// Register a callback for window focus event
    fn on_window_focus(&self, callback: Py<PyAny>) -> PyResult<()> {
        self.ipc_handler
            .register_python_callback("window_focus", callback);
        Ok(())
    }

    /// Register a callback for window blur event
    fn on_window_blur(&self, callback: Py<PyAny>) -> PyResult<()> {
        self.ipc_handler
            .register_python_callback("window_blur", callback);
        Ok(())
    }

    /// Register a callback for window resize event
    fn on_window_resize(&self, callback: Py<PyAny>) -> PyResult<()> {
        self.ipc_handler
            .register_python_callback("window_resize", callback);
        Ok(())
    }

    /// Register a callback for fullscreen state changes
    fn on_fullscreen_changed(&self, callback: Py<PyAny>) -> PyResult<()> {
        self.ipc_handler
            .register_python_callback("fullscreen_changed", callback);
        Ok(())
    }

    // ========================================
    // Performance Monitoring API
    // ========================================

    /// Get performance metrics from the browser
    fn get_performance_metrics(&self, callback: Py<PyAny>) -> PyResult<()> {
        let script = r#"
            (function() {
                var metrics = {};
                if (performance.memory) {
                    metrics.memory = {
                        usedJSHeapSize: performance.memory.usedJSHeapSize,
                        totalJSHeapSize: performance.memory.totalJSHeapSize,
                        jsHeapSizeLimit: performance.memory.jsHeapSizeLimit
                    };
                }
                if (performance.timing) {
                    var t = performance.timing;
                    metrics.timing = {
                        domContentLoaded: t.domContentLoadedEventEnd - t.navigationStart,
                        domComplete: t.domComplete - t.navigationStart,
                        loadComplete: t.loadEventEnd - t.navigationStart,
                        dnsLookup: t.domainLookupEnd - t.domainLookupStart,
                        tcpConnect: t.connectEnd - t.connectStart,
                        serverResponse: t.responseEnd - t.requestStart,
                        domParsing: t.domInteractive - t.responseEnd
                    };
                }
                var resources = performance.getEntriesByType('resource');
                metrics.resourceCount = resources.length;
                metrics.totalResourceSize = resources.reduce(function(sum, r) {
                    return sum + (r.transferSize || 0);
                }, 0);
                var paints = performance.getEntriesByType('paint');
                paints.forEach(function(p) {
                    if (p.name === 'first-paint') {
                        metrics.firstPaint = p.startTime;
                    } else if (p.name === 'first-contentful-paint') {
                        metrics.firstContentfulPaint = p.startTime;
                    }
                });
                return JSON.stringify(metrics);
            })()
        "#;
        self.eval_js_async_internal(script, callback, 5000)
    }

    /// Get IPC statistics
    fn get_ipc_stats(&self) -> PyResult<HashMap<String, usize>> {
        let mut stats = HashMap::new();
        stats.insert(
            "pending_callbacks".to_string(),
            self.js_callback_manager.pending_count(),
        );
        stats.insert("message_queue_size".to_string(), self.message_queue.len());
        stats.insert(
            "registered_events".to_string(),
            self.ipc_handler.registered_event_count(),
        );
        Ok(stats)
    }

    /// Clear performance entries
    fn clear_performance_entries(&self) -> PyResult<()> {
        self.message_queue.push(WebViewMessage::EvalJs(
            "performance.clearResourceTimings(); performance.clearMarks(); performance.clearMeasures();".to_string()
        ));
        Ok(())
    }

    /// Add a performance mark
    fn add_performance_mark(&self, name: &str) -> PyResult<()> {
        let escaped_name = name.replace('\\', "\\\\").replace('\'', "\\'");
        let script = format!("performance.mark('{}')", escaped_name);
        self.message_queue.push(WebViewMessage::EvalJs(script));
        Ok(())
    }

    /// Measure between two performance marks
    fn measure_performance(&self, name: &str, start_mark: &str, end_mark: &str) -> PyResult<()> {
        let escaped_name = name.replace('\\', "\\\\").replace('\'', "\\'");
        let escaped_start = start_mark.replace('\\', "\\\\").replace('\'', "\\'");
        let escaped_end = end_mark.replace('\\', "\\\\").replace('\'', "\\'");
        let script = format!(
            "performance.measure('{}', '{}', '{}')",
            escaped_name, escaped_start, escaped_end
        );
        self.message_queue.push(WebViewMessage::EvalJs(script));
        Ok(())
    }
}
