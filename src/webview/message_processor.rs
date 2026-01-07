//! Message processing utilities for WebView
//!
//! This module provides a unified message processor to reduce code duplication
//! between `process_events()` and `process_ipc_only()`.
//!
//! NOTE: This module is prepared for future refactoring. Currently the message
//! processing logic is duplicated in `event_loop.rs` and `backend/native.rs`.
//! Once stabilized, we can migrate to use this unified processor.

#![allow(dead_code)]

use crate::ipc::WebViewMessage;
use crate::webview::js_assets;
use pyo3::types::PyAnyMethods;
use std::sync::{Arc, Mutex};
use wry::WebView as WryWebView;

/// Execute an MCP tool call on the main thread
///
/// This function is called from various message processing locations to handle
/// MCP tool calls. It executes the Python handler and sends the result back
/// through the response channel.
#[cfg(all(feature = "mcp-server", feature = "python-bindings"))]
pub fn execute_mcp_tool(
    tool_name: &str,
    args: serde_json::Value,
    handler: pyo3::Py<pyo3::types::PyAny>,
    response_tx: std::sync::Arc<
        std::sync::Mutex<Option<tokio::sync::oneshot::Sender<Result<serde_json::Value, String>>>>,
    >,
    context: &str,
) {
    tracing::info!(
        "[{}] Executing MCP tool: {} with args: {}",
        context,
        tool_name,
        args
    );

    let result = pyo3::Python::attach(|py| {
        // Check if args is empty - if so, call without arguments
        let call_result = match &args {
            serde_json::Value::Object(map) if map.is_empty() => {
                // No arguments - call function without parameters
                handler.call0(py)
            }
            serde_json::Value::Null => {
                // Null args - call function without parameters
                handler.call0(py)
            }
            _ => {
                // Has arguments - convert and pass as kwargs
                let py_args = match pythonize::pythonize(py, &args) {
                    Ok(obj) => obj,
                    Err(e) => {
                        return Err(format!("Failed to convert args to Python: {}", e));
                    }
                };
                // Use call with kwargs - Python functions expect keyword arguments
                let kwargs_dict = match py_args.downcast::<pyo3::types::PyDict>() {
                    Ok(dict) => dict,
                    Err(_) => {
                        return Err("Arguments must be a JSON object".to_string());
                    }
                };
                handler.call(py, (), Some(&kwargs_dict))
            }
        };

        // Handle the result
        match call_result {
            Ok(result) => {
                // Convert result back to JSON
                match pythonize::depythonize::<serde_json::Value>(result.bind(py)) {
                    Ok(json) => Ok(json),
                    Err(e) => Err(format!("Failed to convert result to JSON: {}", e)),
                }
            }
            Err(e) => Err(format!("Python handler error: {}", e)),
        }
    });

    // Send the result back through the response channel
    if let Ok(mut guard) = response_tx.lock() {
        if let Some(tx) = guard.take() {
            if let Err(e) = tx.send(result) {
                tracing::error!(
                    "[{}] Failed to send MCP tool result for {}: {:?}",
                    context,
                    tool_name,
                    e
                );
            }
        }
    }
}

/// Execute a Python callback on the main thread
///
/// This function is called from various message processing locations to handle
/// deferred Python callbacks. It executes the Python callback and handles any errors.
#[cfg(feature = "python-bindings")]
pub fn execute_python_callback(
    callback_id: u64,
    event_name: &str,
    data: serde_json::Value,
    ipc_handler: &crate::ipc::IpcHandler,
    context: &str,
) {
    tracing::info!(
        "[{}] Executing Python callback: id={}, event={}, data={}",
        context,
        callback_id,
        event_name,
        data
    );

    if let Err(e) = ipc_handler.execute_deferred_callback(callback_id, event_name, data) {
        tracing::error!(
            "[{}] Failed to execute deferred callback {}: {}",
            context,
            callback_id,
            e
        );
    }
}

/// Process a single WebView message
///
/// This is the unified message handler used by both `process_events()` and
/// `process_ipc_only()`. It handles all message types consistently.
///
/// # Arguments
/// * `webview` - Reference to the locked WebView
/// * `message` - The message to process
/// * `context` - A string identifying the caller (for logging)
pub fn process_message(webview: &WryWebView, message: WebViewMessage, context: &str) {
    match message {
        WebViewMessage::EvalJs(script) => {
            tracing::debug!("[{}] Processing EvalJs: {}", context, script);
            if let Err(e) = webview.evaluate_script(&script) {
                tracing::error!("[{}] Failed to execute JavaScript: {}", context, e);
            }
        }
        WebViewMessage::EmitEvent { event_name, data } => {
            tracing::debug!(
                "[OK] [{}] Emitting event: {} with data: {}",
                context,
                event_name,
                data
            );
            let json_str = data.to_string();
            let escaped_json = json_str.replace('\\', "\\\\").replace('\'', "\\'");
            let script = format!(
                "window.dispatchEvent(new CustomEvent('{}', {{ detail: JSON.parse('{}') }}));",
                event_name, escaped_json
            );
            if let Err(e) = webview.evaluate_script(&script) {
                tracing::error!("[{}] Failed to emit event: {}", context, e);
            } else {
                tracing::debug!("[OK] [{}] Event emitted successfully", context);
            }
        }
        WebViewMessage::LoadUrl(url) => {
            // Use native WebView load_url() instead of JavaScript window.location.href
            // This is more reliable, especially after splash screen loading
            tracing::info!("[{}] Loading URL via native API: {}", context, url);
            if let Err(e) = webview.load_url(&url) {
                tracing::error!("[{}] Failed to load URL: {}", context, e);
            }
        }
        WebViewMessage::LoadHtml(html) => {
            tracing::debug!("[{}] Processing LoadHtml ({} bytes)", context, html.len());
            if let Err(e) = webview.load_html(&html) {
                tracing::error!("[{}] Failed to load HTML: {}", context, e);
            }
        }
        WebViewMessage::WindowEvent { event_type, data } => {
            let event_name = event_type.as_str();
            let json_str = data.to_string();
            let escaped_json = json_str.replace('\\', "\\\\").replace('\'', "\\'");
            let script = js_assets::build_emit_event_script(event_name, &escaped_json);
            tracing::debug!(
                "[WINDOW_EVENT] [{}] Emitting window event: {}",
                context,
                event_name
            );
            if let Err(e) = webview.evaluate_script(&script) {
                tracing::error!("[{}] Failed to emit window event: {}", context, e);
            }
        }
        WebViewMessage::SetVisible(_) => {
            // SetVisible is handled at window level, not in message processing
        }
        WebViewMessage::EvalJsAsync {
            script,
            callback_id,
        } => {
            let async_script = js_assets::build_eval_js_async_script(&script, callback_id);
            if let Err(e) = webview.evaluate_script(&async_script) {
                tracing::error!(
                    "[{}] Failed to execute async JavaScript (id={}): {}",
                    context,
                    callback_id,
                    e
                );
            }
        }
        WebViewMessage::Reload => {
            if let Err(e) = webview.evaluate_script("location.reload()") {
                tracing::error!("[{}] Failed to reload: {}", context, e);
            }
        }
        WebViewMessage::StopLoading => {
            if let Err(e) = webview.evaluate_script("window.stop()") {
                tracing::error!("[{}] Failed to stop loading: {}", context, e);
            }
        }
        WebViewMessage::Close => {
            // Close is handled at event loop level, not in message processing
            tracing::debug!(
                "[{}] Close message received (handled at event loop level)",
                context
            );
        }
        #[cfg(feature = "python-bindings")]
        WebViewMessage::PythonCallbackDeferred { .. } => {
            // PythonCallbackDeferred is handled at event loop level, not in message processing
            // This is because it needs access to the IpcHandler which is not available here
            tracing::debug!(
                "[{}] PythonCallbackDeferred message received (handled at event loop level)",
                context
            );
        }
        #[cfg(all(feature = "mcp-server", feature = "python-bindings"))]
        WebViewMessage::McpToolCall {
            tool_name,
            args,
            handler,
            response_tx,
        } => {
            // Use the dedicated function to execute MCP tool
            execute_mcp_tool(&tool_name, args, handler, response_tx, context);
        }
    }
}

/// Process all messages in a queue using the unified handler.
///
/// Returns `(processed_count, close_requested)`.
///
/// Note: `WebViewMessage::Close` is a *control* message. We don't execute any JS
/// here, but we do surface it to the caller so mode-specific code (event loop,
/// embedded/Qt host, etc.) can perform the correct shutdown.
pub fn process_message_queue(
    webview: &Arc<Mutex<WryWebView>>,
    message_queue: &crate::ipc::MessageQueue,
    context: &str,
) -> (usize, bool) {
    if let Ok(webview_guard) = webview.lock() {
        let mut close_requested = false;

        let count = message_queue.process_all(|message| {
            tracing::trace!("[{}] processing message: {:?}", context, message);

            if matches!(&message, WebViewMessage::Close) {
                close_requested = true;
            }

            process_message(&webview_guard, message, context);
        });

        if count > 0 {
            tracing::debug!("[{}] processed {} messages from queue", context, count);
        } else {
            tracing::trace!("[{}] no messages in queue", context);
        }

        (count, close_requested)
    } else {
        tracing::error!("[{}] failed to lock WebView", context);
        (0, false)
    }
}
