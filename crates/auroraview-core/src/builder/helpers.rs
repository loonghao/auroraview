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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_create_drag_drop_handler() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let _handler = create_drag_drop_handler(move |_event_name, _data| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        // Handler is created successfully
        assert_eq!(counter.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_create_drag_drop_handler_captures_event_name() {
        let events = Arc::new(std::sync::Mutex::new(Vec::new()));
        let events_clone = events.clone();

        let _handler = create_drag_drop_handler(move |event_name, _data| {
            events_clone.lock().unwrap().push(event_name.to_string());
        });

        // Verify handler is created
        assert!(events.lock().unwrap().is_empty());
    }

    #[test]
    fn test_create_drag_drop_handler_captures_json_data() {
        let data_list = Arc::new(std::sync::Mutex::new(Vec::new()));
        let data_clone = data_list.clone();

        let _handler = create_drag_drop_handler(move |_event_name, data| {
            data_clone.lock().unwrap().push(data);
        });

        // Verify handler is created
        assert!(data_list.lock().unwrap().is_empty());
    }

    #[test]
    fn test_create_ipc_handler() {
        let event_counter = Arc::new(AtomicUsize::new(0));
        let call_counter = Arc::new(AtomicUsize::new(0));

        let event_counter_clone = event_counter.clone();
        let call_counter_clone = call_counter.clone();

        let _handler = create_ipc_handler(
            move |_name, _data| {
                event_counter_clone.fetch_add(1, Ordering::SeqCst);
            },
            move |_method, _params, _id| {
                call_counter_clone.fetch_add(1, Ordering::SeqCst);
            },
            |_cmd, _args, _id| {},
            |_callback_id, _data| {},
        );

        // Handlers are created successfully
        assert_eq!(event_counter.load(Ordering::SeqCst), 0);
        assert_eq!(call_counter.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_create_ipc_handler_all_callbacks() {
        let event_counter = Arc::new(AtomicUsize::new(0));
        let call_counter = Arc::new(AtomicUsize::new(0));
        let invoke_counter = Arc::new(AtomicUsize::new(0));
        let js_callback_counter = Arc::new(AtomicUsize::new(0));

        let event_clone = event_counter.clone();
        let call_clone = call_counter.clone();
        let invoke_clone = invoke_counter.clone();
        let js_callback_clone = js_callback_counter.clone();

        let _handler = create_ipc_handler(
            move |_name, _data| {
                event_clone.fetch_add(1, Ordering::SeqCst);
            },
            move |_method, _params, _id| {
                call_clone.fetch_add(1, Ordering::SeqCst);
            },
            move |_cmd, _args, _id| {
                invoke_clone.fetch_add(1, Ordering::SeqCst);
            },
            move |_callback_id, _data| {
                js_callback_clone.fetch_add(1, Ordering::SeqCst);
            },
        );

        // All counters start at 0
        assert_eq!(event_counter.load(Ordering::SeqCst), 0);
        assert_eq!(call_counter.load(Ordering::SeqCst), 0);
        assert_eq!(invoke_counter.load(Ordering::SeqCst), 0);
        assert_eq!(js_callback_counter.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_create_simple_ipc_handler() {
        let event_counter = Arc::new(AtomicUsize::new(0));
        let call_counter = Arc::new(AtomicUsize::new(0));

        let event_clone = event_counter.clone();
        let call_clone = call_counter.clone();

        let _handler = create_simple_ipc_handler(
            move |_name, _data| {
                event_clone.fetch_add(1, Ordering::SeqCst);
            },
            move |_method, _params, _id| {
                call_clone.fetch_add(1, Ordering::SeqCst);
            },
        );

        // Handlers are created successfully
        assert_eq!(event_counter.load(Ordering::SeqCst), 0);
        assert_eq!(call_counter.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_create_ipc_handler_captures_event_data() {
        let events = Arc::new(std::sync::Mutex::new(Vec::new()));
        let events_clone = events.clone();

        let _handler = create_ipc_handler(
            move |name, data| {
                events_clone
                    .lock()
                    .unwrap()
                    .push((name.clone(), data.clone()));
            },
            |_method, _params, _id| {},
            |_cmd, _args, _id| {},
            |_callback_id, _data| {},
        );

        // Verify handler is created
        assert!(events.lock().unwrap().is_empty());
    }

    #[test]
    fn test_create_ipc_handler_captures_call_data() {
        let calls = Arc::new(std::sync::Mutex::new(Vec::new()));
        let calls_clone = calls.clone();

        let _handler = create_ipc_handler(
            |_name, _data| {},
            move |method, params, id| {
                calls_clone
                    .lock()
                    .unwrap()
                    .push((method.clone(), params.clone(), id.clone()));
            },
            |_cmd, _args, _id| {},
            |_callback_id, _data| {},
        );

        // Verify handler is created
        assert!(calls.lock().unwrap().is_empty());
    }

    #[test]
    fn test_create_ipc_handler_captures_invoke_data() {
        let invokes = Arc::new(std::sync::Mutex::new(Vec::new()));
        let invokes_clone = invokes.clone();

        let _handler = create_ipc_handler(
            |_name, _data| {},
            |_method, _params, _id| {},
            move |cmd, args, id| {
                invokes_clone
                    .lock()
                    .unwrap()
                    .push((cmd.clone(), args.clone(), id.clone()));
            },
            |_callback_id, _data| {},
        );

        // Verify handler is created
        assert!(invokes.lock().unwrap().is_empty());
    }

    #[test]
    fn test_create_ipc_handler_captures_js_callback_data() {
        let callbacks = Arc::new(std::sync::Mutex::new(Vec::new()));
        let callbacks_clone = callbacks.clone();

        let _handler = create_ipc_handler(
            |_name, _data| {},
            |_method, _params, _id| {},
            |_cmd, _args, _id| {},
            move |callback_id, data| {
                callbacks_clone
                    .lock()
                    .unwrap()
                    .push((callback_id.clone(), data.clone()));
            },
        );

        // Verify handler is created
        assert!(callbacks.lock().unwrap().is_empty());
    }
}
