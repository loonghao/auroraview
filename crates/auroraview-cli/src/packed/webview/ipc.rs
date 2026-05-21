//! IPC message handling for WebView module
//!
//! This module handles IPC messages from the WebView, routing them to
//! either the PluginRouter or Python backend.

use serde_json::Value;
use std::sync::{Arc, RwLock};

use auroraview_plugins::{PluginRequest, PluginRouter};
use tao::event_loop::EventLoopProxy;

use crate::packed::backend::PythonBackend;
use crate::packed::events::UserEvent;

/// Handle window commands from JavaScript
///
/// These commands are handled directly by Rust since the window is controlled by Rust.
/// Supported commands:
/// - close: Close the window and exit the application
pub fn handle_window_command(
    method: &str,
    _params: &Value,
    proxy: &EventLoopProxy<UserEvent>,
) -> Result<Value, String> {
    tracing::info!("[Rust] Window command: {}", method);

    match method {
        "close" => {
            tracing::info!("[Rust] Closing window via window.close() API");
            // Send close event to event loop
            if let Err(e) = proxy.send_event(UserEvent::CloseWindow) {
                tracing::error!("[Rust] Failed to send CloseWindow event: {}", e);
                return Err(format!("Failed to close window: {}", e));
            }
            Ok(serde_json::json!({"success": true}))
        }
        _ => {
            tracing::warn!("[Rust] Unknown window command: {}", method);
            Err(format!("Unknown window command: {}", method))
        }
    }
}

/// Handle IPC message from WebView
///
/// This function routes messages to either:
/// 1. PluginRouter - for native plugin commands (plugin:*, shell, process, etc.)
/// 2. Python backend - for application-specific API calls (api.*)
#[allow(clippy::too_many_lines)]
pub fn handle_ipc_message(
    body: &str,
    python_backend: &Arc<RwLock<Option<Arc<PythonBackend>>>>,
    plugin_router: &Arc<RwLock<PluginRouter>>,
    proxy: &EventLoopProxy<UserEvent>,
) {
    tracing::debug!("[Rust] IPC message received: {}", body);

    // Parse the message
    let msg: serde_json::Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("Failed to parse IPC message: {}", e);
            let _ = proxy.send_event(UserEvent::BackendError {
                message: format!("Invalid IPC payload: {}", e),
                source: "ipc".to_string(),
            });
            return;
        }
    };

    let msg_type = msg.get("type").and_then(|v| v.as_str()).unwrap_or("");

    match msg_type {
        "call" => {
            handle_call_message(&msg, python_backend, plugin_router, proxy);
        }
        "plugin" => {
            handle_plugin_message(&msg, plugin_router, proxy);
        }
        "event" => {
            handle_event_message(&msg, proxy);
        }
        "set_html" => {
            handle_set_html_message(&msg, proxy);
        }
        "invoke" => {
            handle_invoke_message(&msg, plugin_router, proxy);
        }
        _ => {
            tracing::warn!("Unknown IPC message type: {}", msg_type);
            if let Some(id) = msg.get("id").and_then(|v| v.as_str()) {
                let error_response = serde_json::json!({
                    "id": id,
                    "ok": false,
                    "result": null,
                    "error": {
                        "name": "UnknownMessageType",
                        "message": format!("Unknown IPC message type: {}", msg_type)
                    }
                });
                let _ = proxy.send_event(UserEvent::PythonResponse(error_response.to_string()));
            }
        }
    }
}

/// Handle "call" message type
fn handle_call_message(
    msg: &Value,
    python_backend: &Arc<RwLock<Option<Arc<PythonBackend>>>>,
    plugin_router: &Arc<RwLock<PluginRouter>>,
    proxy: &EventLoopProxy<UserEvent>,
) {
    let id = msg
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let method = msg.get("method").and_then(|v| v.as_str()).unwrap_or("");
    let params = msg
        .get("params")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    tracing::debug!("[Rust] API call: {} (id: {})", method, id);

    // Check if this is a window command (window.*)
    if let Some(window_method) = method.strip_prefix("window.") {
        let result = handle_window_command(window_method, &params, proxy);
        let response = serde_json::json!({
            "id": id,
            "ok": result.is_ok(),
            "result": result.as_ref().ok(),
            "error": result.as_ref().err().map(|e| serde_json::json!({
                "name": "WindowError",
                "message": e.to_string()
            }))
        });
        let _ = proxy.send_event(UserEvent::PythonResponse(response.to_string()));
    }
    // Check if this is a plugin command (plugin:*)
    else if method.starts_with("plugin:") {
        handle_plugin_call(msg, id, method, plugin_router, proxy);
    } else {
        handle_api_call(msg, id, method, params, python_backend, proxy);
    }
}

/// Handle plugin call in "call" message
fn handle_plugin_call(
    msg: &Value,
    id: String,
    method: &str,
    plugin_router: &Arc<RwLock<PluginRouter>>,
    proxy: &EventLoopProxy<UserEvent>,
) {
    if let Some(request) = PluginRequest::from_invoke(
        method,
        msg.get("params")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
    ) {
        let response = match plugin_router.read() {
            Ok(router) => router.handle(request),
            Err(e) => {
                tracing::error!("Plugin router lock poisoned: {}", e);
                let error_response = serde_json::json!({
                    "id": id,
                    "ok": false,
                    "result": null,
                    "error": {
                        "name": "InternalError",
                        "message": "Plugin router unavailable"
                    }
                });
                let _ = proxy.send_event(UserEvent::PythonResponse(error_response.to_string()));
                return;
            }
        };

        // Send response back via event loop
        let result = serde_json::json!({
            "id": id,
            "ok": response.success,
            "result": response.data,
            "error": response.error
        });
        let _ = proxy.send_event(UserEvent::PythonResponse(result.to_string()));
    } else {
        tracing::warn!("Invalid plugin command format: {}", method);
        let error_response = serde_json::json!({
            "id": id,
            "ok": false,
            "result": null,
            "error": {
                "name": "InvalidFormat",
                "message": format!("Invalid plugin command format: {}", method)
            }
        });
        let _ = proxy.send_event(UserEvent::PythonResponse(error_response.to_string()));
    }
}

/// Handle API call in "call" message (forward to Python backend)
fn handle_api_call(
    _msg: &Value,
    id: String,
    method: &str,
    params: Value,
    python_backend: &Arc<RwLock<Option<Arc<PythonBackend>>>>,
    proxy: &EventLoopProxy<UserEvent>,
) {
    let backend = python_backend.read().ok().and_then(|state| state.clone());
    if let Some(backend) = backend {
        let request = serde_json::json!({
            "id": id,
            "method": method,
            "params": params
        });

        if let Err(e) = backend.send_request(&request.to_string()) {
            tracing::error!("Failed to send request to Python: {}", e);
            let error_response = serde_json::json!({
                "id": id,
                "ok": false,
                "result": null,
                "error": {
                    "name": "PythonBackendError",
                    "message": format!("{}", e)
                }
            });
            let _ = proxy.send_event(UserEvent::PythonResponse(error_response.to_string()));
        }
    } else {
        tracing::warn!("No Python backend available for API call: {}", method);
        let error_response = serde_json::json!({
            "id": id,
            "ok": false,
            "result": null,
            "error": {
                "name": "NoPythonBackend",
                "message": "No Python backend available for API calls"
            }
        });
        let _ = proxy.send_event(UserEvent::PythonResponse(error_response.to_string()));
    }
}

/// Handle "plugin" message type (direct plugin invocation)
fn handle_plugin_message(
    msg: &Value,
    plugin_router: &Arc<RwLock<PluginRouter>>,
    proxy: &EventLoopProxy<UserEvent>,
) {
    let plugin = msg.get("plugin").and_then(|v| v.as_str()).unwrap_or("");
    let command = msg.get("command").and_then(|v| v.as_str()).unwrap_or("");
    let args = msg.get("args").cloned().unwrap_or(serde_json::Value::Null);
    let id = msg
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    tracing::debug!("Plugin call: {}|{} (id: {})", plugin, command, id);

    let request = PluginRequest::new(plugin, command, args);
    let response = match plugin_router.read() {
        Ok(router) => router.handle(request),
        Err(e) => {
            tracing::error!("Plugin router lock poisoned: {}", e);
            let error_response = serde_json::json!({
                "id": id,
                "ok": false,
                "result": null,
                "error": {
                    "name": "InternalError",
                    "message": "Plugin router unavailable"
                }
            });
            let _ = proxy.send_event(UserEvent::PythonResponse(error_response.to_string()));
            return;
        }
    };

    // Send response back via event loop
    let result = serde_json::json!({
        "id": id,
        "ok": response.success,
        "result": response.data,
        "error": response.error
    });
    let _ = proxy.send_event(UserEvent::PythonResponse(result.to_string()));
}

/// IPC sink that forwards drag-drop events from `attach_drag_drop_handler`
/// into the packed mode's IPC pipeline.
///
/// This wraps [`handle_ipc_message`] by re-encoding the drag-drop event as
/// the same `{ "type": "event", "event": ..., ... }` envelope the WebView
/// would have produced.
pub struct PackedDragDropSink {
    pub python_backend: Arc<RwLock<Option<Arc<PythonBackend>>>>,
    pub plugin_router: Arc<RwLock<PluginRouter>>,
    pub proxy: EventLoopProxy<UserEvent>,
}

impl auroraview_core::builder::DragDropIpcSink for PackedDragDropSink {
    fn dispatch(
        &self,
        event_name: &str,
        data: serde_json::Value,
    ) -> Result<(), auroraview_core::builder::DispatchError> {
        let envelope = serde_json::json!({
            "type": "event",
            "event": event_name,
            "data": data,
        });
        handle_ipc_message(
            &envelope.to_string(),
            &self.python_backend,
            &self.plugin_router,
            &self.proxy,
        );
        Ok(())
    }
}

/// Handle "event" message type
fn handle_event_message(msg: &Value, proxy: &EventLoopProxy<UserEvent>) {
    let event = msg.get("event").and_then(|v| v.as_str()).unwrap_or("");
    tracing::debug!("Event received: {}", event);

    // Handle loading screen ready event
    if event == "loading_screen_ready" {
        tracing::info!("[Rust] Received loading_screen_ready event from WebView");
        let _ = proxy.send_event(UserEvent::LoadingScreenReady);
    }
    // Handle navigate to app event (frontend requests navigation)
    else if event == "navigate_to_app" {
        tracing::info!("[Rust] Received navigate_to_app event from WebView");
        let _ = proxy.send_event(UserEvent::NavigateToApp);
    }
    // Handle page ready event (auroraview bridge initialized after navigation)
    else if event == "__auroraview_ready" {
        tracing::info!("[Rust] Received __auroraview_ready event - page bridge initialized");
        let _ = proxy.send_event(UserEvent::PageReady);
    }
}

/// Handle "set_html" message type (for dynamic HTML like Browser component)
fn handle_set_html_message(msg: &Value, proxy: &EventLoopProxy<UserEvent>) {
    let html = msg
        .get("html")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let title = msg.get("title").and_then(|v| v.as_str()).map(String::from);

    tracing::info!(
        "[Rust] Received set_html command (html_len: {}, title: {:?})",
        html.len(),
        title
    );

    let _ = proxy.send_event(UserEvent::SetHtml { html, title });
}

/// Handle "invoke" message type (Tauri-style invoke command)
fn handle_invoke_message(
    msg: &Value,
    plugin_router: &Arc<RwLock<PluginRouter>>,
    proxy: &EventLoopProxy<UserEvent>,
) {
    let id = msg
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let cmd = msg.get("cmd").and_then(|v| v.as_str()).unwrap_or("");
    let args = msg.get("args").cloned().unwrap_or(serde_json::Value::Null);

    tracing::debug!("[Rust] Invoke command: {} (id: {})", cmd, id);

    // Handle plugin commands (plugin:name|command format)
    if cmd.starts_with("plugin:") {
        handle_invoke_plugin(msg, id, cmd, args, plugin_router, proxy);
    } else {
        tracing::warn!("Unsupported invoke command: {}", cmd);
        let error_response = serde_json::json!({
            "id": id,
            "ok": false,
            "result": null,
            "error": {
                "name": "UnsupportedCommand",
                "message": format!("Unsupported invoke command: {}", cmd)
            }
        });
        let _ = proxy.send_event(UserEvent::PythonResponse(error_response.to_string()));
    }
}

/// Handle plugin invoke command
fn handle_invoke_plugin(
    _msg: &Value,
    id: String,
    cmd: &str,
    args: Value,
    plugin_router: &Arc<RwLock<PluginRouter>>,
    proxy: &EventLoopProxy<UserEvent>,
) {
    let plugin_part = cmd.strip_prefix("plugin:").unwrap_or("");
    if let Some((plugin, command)) = plugin_part.split_once('|') {
        tracing::debug!(
            "[Rust] Plugin invoke: plugin={}, command={}, id={}",
            plugin,
            command,
            id
        );

        let request = PluginRequest::new(plugin, command, args);
        let response = match plugin_router.read() {
            Ok(router) => router.handle(request),
            Err(e) => {
                tracing::error!("Plugin router lock poisoned: {}", e);
                let error_response = serde_json::json!({
                    "id": id,
                    "ok": false,
                    "result": null,
                    "error": {
                        "name": "InternalError",
                        "message": "Plugin router unavailable"
                    }
                });
                let _ = proxy.send_event(UserEvent::PythonResponse(error_response.to_string()));
                return;
            }
        };

        // Send response back via event loop
        let result = serde_json::json!({
            "id": id,
            "ok": response.success,
            "result": response.data,
            "error": response.error
        });
        let _ = proxy.send_event(UserEvent::PythonResponse(result.to_string()));
    } else {
        tracing::warn!("Invalid invoke plugin command format: {}", cmd);
        let error_response = serde_json::json!({
            "id": id,
            "ok": false,
            "result": null,
            "error": {
                "name": "InvalidFormat",
                "message": format!("Invalid plugin command format: {}", cmd)
            }
        });
        let _ = proxy.send_event(UserEvent::PythonResponse(error_response.to_string()));
    }
}

#[cfg(test)]
mod tests {
    use super::PackedDragDropSink;

    /// `PackedDragDropSink` is passed to `attach_drag_drop_handler` as the
    /// generic `S: DragDropIpcSink` parameter, which requires
    /// `Send + Sync + 'static`. That bound currently holds because every
    /// field (`Arc<RwLock<...>>`, `EventLoopProxy<UserEvent>`) implements
    /// both traits on the supported platforms.
    ///
    /// `tao::EventLoopProxy<T>` is the only field whose `Sync` impl is
    /// not part of `std`. If a future `tao` upgrade ever removes its
    /// `Sync` impl, the helper call site would fail with a confusing
    /// "S does not implement Sync" error that points at user code rather
    /// than at the upstream regression. These compile-time checks force
    /// the build to fail at the type definition itself instead.
    #[allow(dead_code)]
    fn assert_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<PackedDragDropSink>();
        assert_sync::<PackedDragDropSink>();
    }
}
