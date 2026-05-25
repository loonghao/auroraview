//! High-level helper functions for WebView building
//!
//! This module provides convenience functions that integrate
//! drag-drop and IPC handling with the existing IpcHandler/IpcMessage types.

use super::drag_drop::{DragDropEventData, DragDropEventType, DragDropHandler};
use super::ipc::{IpcMessageHandler, IpcMessageType, ParsedIpcMessage};
use std::sync::Arc;

/// Errors that may occur while dispatching a drag-drop event into the IPC pipeline.
///
/// Currently a single variant. The underlying `IpcHandler::handle_message`
/// error type is `String`, which cannot be split into semantic variants
/// (`Disconnected` / `Serialization`). Once the IPC error type is enum-ified
/// (a separate refactor across the IPC subsystem, decoupled from this RFC),
/// new variants can be appended here.
///
/// `#[non_exhaustive]` guarantees that adding new variants is not a breaking
/// change.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum DispatchError {
    /// Backend-specific dispatch failure.
    #[error(transparent)]
    Backend(Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl DispatchError {
    /// Wrap any `Send + Sync + 'static` error as a `Backend` variant.
    pub fn backend<E>(err: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Backend(Box::new(err))
    }
}

/// Trait abstraction over the IPC entry point expected by
/// [`attach_drag_drop_handler`].
///
/// Implementations are responsible for forwarding events into the IPC
/// pipeline. They do not log on failure: the helper logs through a single
/// `tracing::error!` call so all sinks share a uniform log format.
///
/// # Contract
///
/// The caller (`attach_drag_drop_handler` → `DragDropHandler::into_handler`)
/// guarantees that **`Over` events are never dispatched**. The `Over`
/// variant fires per-pixel during a drag and is filtered at the
/// `DragDropHandler` level before any callback or sink is invoked.
/// Implementations may therefore assume `event_name` is one of:
///
/// - `"file_drop_hover"` (Enter)
/// - `"file_drop"` (Drop)
/// - `"file_drop_cancelled"` (Leave)
///
/// If an `Over` event somehow reaches `dispatch` (e.g. a caller bypasses
/// `into_handler`), the implementation is free to return `Ok(())` and
/// discard it — but this should never happen in normal operation.
pub trait DragDropIpcSink: Send + Sync + 'static {
    /// Forward a single drag-drop event into the IPC pipeline.
    ///
    /// `event_name` is guaranteed to be one of `"file_drop_hover"`,
    /// `"file_drop"`, or `"file_drop_cancelled"` — never `"file_drop_over"`.
    /// See the trait-level [Contract](#contract) section.
    fn dispatch(&self, event_name: &str, data: serde_json::Value) -> Result<(), DispatchError>;
}

/// No-op [`DragDropIpcSink`] used at call sites that pass `capture=false`
/// unconditionally (RFC 0016 Browser-mode controller and business tabs).
///
/// Because [`attach_drag_drop_handler`] short-circuits on `capture=false`
/// and never calls `dispatch`, a single shared no-op type keeps every
/// "never-attached" sink generic-arg consistent and avoids duplicated
/// type definitions across crates.
///
/// For test-only sinks that need to count or fail dispatches, define a
/// dedicated test helper instead.
#[derive(Debug, Default)]
pub struct NoopDragDropSink;

impl DragDropIpcSink for NoopDragDropSink {
    fn dispatch(&self, _event_name: &str, _data: serde_json::Value) -> Result<(), DispatchError> {
        Ok(())
    }
}

/// Conditionally attach the drag-drop proxy handler to a `wry::WebViewBuilder`.
///
/// - `capture == false` — the builder is returned unchanged. The helper
///   does **not** call `with_drag_drop_handler`. The WebView falls back to
///   the browser-native HTML5 drag-drop semantics. `ipc_sink` is borrowed,
///   not cloned: the caller's `Arc::strong_count` stays exactly the same.
/// - `capture == true` — the helper performs exactly one `Arc::clone` of
///   `ipc_sink`, builds a `Send + Sync + 'static` closure, and registers it
///   via `with_drag_drop_handler`. Events are filtered through
///   `DragDropHandler::into_handler` (`Over` events are dropped) and
///   forwarded as `file_drop_hover` / `file_drop` / `file_drop_cancelled`
///   into `sink.dispatch(...)`. If `dispatch` returns an error, the helper
///   logs a single `tracing::error!` and discards the event (drag-drop
///   must never block the WebView).
///
/// # Borrow form
///
/// `ipc_sink: &Arc<S>` (not `Arc<S>`) so that:
/// - the `capture == false` path performs zero atomic operations on the Arc;
/// - the `capture == true` path performs exactly one `Arc::clone`.
///
/// # Lifetime parameter `'a`
///
/// `wry::WebViewBuilder<'a>` is `'static` when constructed with
/// `WebViewBuilder::new()` but borrows the `WebContext` lifetime when built
/// with `WebViewBuilder::new_with_web_context(&mut web_context)`. The
/// generic lifetime keeps the helper usable in both forms.
///
/// # Static dispatch
///
/// `where S: DragDropIpcSink` (no `?Sized`). Each call site instantiates a
/// separate monomorphized copy.
///
/// # Note on upstream behavior
///
/// Due to a wry/WebView2 limitation, registering `with_drag_drop_handler`
/// (regardless of its return value) suppresses HTML5 `dragover` / `drop`
/// events inside the WebView. See RFC 0015 §2 and
/// <https://github.com/tauri-apps/wry/issues/157>.
pub fn attach_drag_drop_handler<'a, S>(
    builder: wry::WebViewBuilder<'a>,
    capture: bool,
    ipc_sink: &Arc<S>,
) -> wry::WebViewBuilder<'a>
where
    S: DragDropIpcSink,
{
    if !capture {
        // ipc_sink is only borrowed; strong_count is unaffected.
        return builder;
    }

    let sink = Arc::clone(ipc_sink);

    builder.with_drag_drop_handler(create_drag_drop_handler(move |event_name, data| {
        if let Err(err) = sink.dispatch(event_name, data) {
            tracing::error!(
                target: "auroraview::drag_drop",
                "Failed to dispatch {} via DragDropIpcSink: {}",
                event_name,
                err
            );
        }
    }))
}

/// Browser-mode convenience: explicitly skip drag-drop capture.
///
/// Forwards to [`attach_drag_drop_handler`] with `capture=false` and a
/// shared no-op sink so every Browser-mode call site funnels through the
/// **same** auditable entry point. The CI scanner
/// ([`scripts/ci/check_browser_no_drag_drop_capture.py`]) only needs to
/// look at `attach_drag_drop_handler` second arguments — wrapping the
/// passthrough in the canonical helper keeps its expansion grep-able and
/// makes accidental removal of the call site visible to the scanner.
///
/// # Cost
///
/// Zero allocation: a single `&'static Arc<NoopDragDropSink>` is reused
/// across every call. `attach_drag_drop_handler` short-circuits on
/// `capture=false` and returns the builder unchanged without ever
/// touching the sink, so the wrapper is observably equivalent to the
/// previous identity passthrough.
///
/// # Rationale
///
/// Multi-webview overlays cannot maintain a coherent drop state machine
/// across pixel boundaries (RFC 0016 §2.1). Pages needing absolute file
/// paths via IPC should use a top-level `AuroraView` instance with
/// `capture_file_drop=True` instead.
#[inline]
pub fn skip_drag_drop_capture(builder: wry::WebViewBuilder<'_>) -> wry::WebViewBuilder<'_> {
    use std::sync::OnceLock;
    static NOOP_SINK: OnceLock<Arc<NoopDragDropSink>> = OnceLock::new();
    let sink = NOOP_SINK.get_or_init(|| Arc::new(NoopDragDropSink));
    attach_drag_drop_handler(builder, false, sink)
}

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
        // Defense-in-depth: `into_handler` already short-circuits Over events,
        // but guard here too in case the caller bypasses `into_handler`.
        if data.event_type == DragDropEventType::Over {
            return;
        }

        let event_name = data.event_type.as_event_name();
        let json_data = data.to_json();
        callback(event_name, json_data);
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
