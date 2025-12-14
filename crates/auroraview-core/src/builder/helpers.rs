//! High-level helper functions for WebView building
//!
//! This module provides convenience functions that integrate
//! drag-drop and IPC handling with the existing IpcHandler/IpcMessage types.

use super::drag_drop::{DragDropEventData, DragDropEventType, DragDropHandler};
use super::ipc::{IpcMessageHandler, IpcMessageType, ParsedIpcMessage};
use std::sync::Arc;

/// Create a drag-drop handler that sends events to an IPC callback
///
/// This is a convenience function that creates a `DragDropHandler` which
/// converts drag-drop events to a format suitable for IPC messaging.
///
/// # Arguments
/// * `callback` - Callback that receives (event_name, data) pairs
///
/// # Returns
/// A closure suitable for `WebViewBuilder::with_drag_drop_handler`
pub fn create_drag_drop_handler<F>(callback: F) -> impl Fn(wry::DragDropEvent) -> bool + 'static
where
    F: Fn(&str, serde_json::Value) + Send + Sync + 'static,
{
    let callback = Arc::new(callback);

    DragDropHandler::new(move |data: DragDropEventData| {
        let event_name = data.event_type.as_event_name();
        let json_data = data.to_json();

        // Skip Over events (too frequent)
        if data.event_type != DragDropEventType::Over {
            callback(event_name, json_data);
        }
    })
    .into_handler()
}

/// Create an IPC handler that routes messages to appropriate callbacks
///
/// This is a convenience function that creates an `IpcMessageHandler` which
/// parses IPC messages and routes them to the appropriate callback.
///
/// # Arguments
/// * `on_event` - Callback for event messages (event_name, detail)
/// * `on_call` - Callback for call messages (method, params, id)
/// * `on_invoke` - Callback for invoke messages (cmd, args, id)
/// * `on_js_callback` - Callback for JS callback results (callback_id, data)
///
/// # Returns
/// A closure suitable for `WebViewBuilder::with_ipc_handler`
pub fn create_ipc_handler<E, C, I, J>(
    on_event: E,
    on_call: C,
    on_invoke: I,
    on_js_callback: J,
) -> impl Fn(wry::http::Request<String>) + 'static
where
    E: Fn(String, serde_json::Value) + Send + Sync + 'static,
    C: Fn(String, serde_json::Value, Option<String>) + Send + Sync + 'static,
    I: Fn(String, serde_json::Value, Option<String>) + Send + Sync + 'static,
    J: Fn(String, serde_json::Value) + Send + Sync + 'static,
{
    let on_event = Arc::new(on_event);
    let on_call = Arc::new(on_call);
    let on_invoke = Arc::new(on_invoke);
    let on_js_callback = Arc::new(on_js_callback);

    IpcMessageHandler::new(move |msg: ParsedIpcMessage| match msg.msg_type {
        IpcMessageType::Event => {
            if let Some(name) = msg.name {
                on_event(name, msg.data);
            }
        }
        IpcMessageType::Call => {
            if let Some(name) = msg.name {
                on_call(name, msg.data, msg.id);
            }
        }
        IpcMessageType::Invoke => {
            if let Some(name) = msg.name {
                on_invoke(name, msg.data, msg.id);
            }
        }
        IpcMessageType::JsCallbackResult => {
            if let Some(callback_id) = msg.name {
                on_js_callback(callback_id, msg.data);
            }
        }
        IpcMessageType::Unknown(_) => {
            tracing::warn!("[IpcHandler] Unknown message type");
        }
    })
    .into_handler()
}

/// Simplified IPC handler that only handles events and calls
///
/// This is a simpler version of `create_ipc_handler` for common use cases
/// where only events and calls need to be handled.
pub fn create_simple_ipc_handler<E, C>(
    on_event: E,
    on_call: C,
) -> impl Fn(wry::http::Request<String>) + 'static
where
    E: Fn(String, serde_json::Value) + Send + Sync + 'static,
    C: Fn(String, serde_json::Value, Option<String>) + Send + Sync + 'static,
{
    create_ipc_handler(
        on_event,
        on_call,
        |_cmd, _args, _id| {
            // Invoke not handled
        },
        |_callback_id, _data| {
            // JS callback not handled
        },
    )
}
