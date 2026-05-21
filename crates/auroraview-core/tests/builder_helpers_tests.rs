//! Contract tests for `attach_drag_drop_handler` (RFC 0015 §6.1).
//!
//! These tests guard the helper's design contract:
//!
//! - `capture=false` short-circuits without cloning the `Arc` sink.
//! - `capture=true` clones the sink exactly once and dispatches events
//!   through `DragDropIpcSink::dispatch` with the documented event-name
//!   and JSON payload shape.
//! - `Over` events are filtered (too frequent).
//! - Sink errors are logged once per event but never propagate.
//! - The trait object is `Send + Sync + 'static`.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::Mutex;

use auroraview_core::builder::{
    attach_drag_drop_handler, create_drag_drop_handler, DispatchError, DragDropEventData,
    DragDropEventType, DragDropIpcSink, NoopDragDropSink,
};

// ---------------------------------------------------------------------------
// Local test sinks
// ---------------------------------------------------------------------------

/// Counts dispatch calls and records `(event_name, data)` for assertions.
#[derive(Default)]
struct CountingSink {
    count: AtomicUsize,
    events: Mutex<Vec<(String, serde_json::Value)>>,
}

impl CountingSink {
    fn count(&self) -> usize {
        self.count.load(Ordering::SeqCst)
    }

    fn events(&self) -> Vec<(String, serde_json::Value)> {
        self.events.lock().unwrap().clone()
    }
}

impl DragDropIpcSink for CountingSink {
    fn dispatch(
        &self,
        event_name: &str,
        data: serde_json::Value,
    ) -> Result<(), DispatchError> {
        self.count.fetch_add(1, Ordering::SeqCst);
        self.events
            .lock()
            .unwrap()
            .push((event_name.to_string(), data));
        Ok(())
    }
}

/// Sink that always returns a backend error, used to verify the helper's
/// error path swallows the failure and emits a single tracing record.
#[derive(Default)]
struct ErrorSink {
    count: AtomicUsize,
}

#[derive(Debug, thiserror::Error)]
#[error("synthetic dispatch failure")]
struct SyntheticBackendError;

impl DragDropIpcSink for ErrorSink {
    fn dispatch(
        &self,
        _event_name: &str,
        _data: serde_json::Value,
    ) -> Result<(), DispatchError> {
        self.count.fetch_add(1, Ordering::SeqCst);
        Err(DispatchError::backend(SyntheticBackendError))
    }
}

// ---------------------------------------------------------------------------
// §6.1 attach_drag_drop_handler contract tests
// ---------------------------------------------------------------------------

#[test]
fn attach_drag_drop_handler_smoke_capture_false() {
    // Smoke test: the helper compiles and runs without panic when given
    // a fresh `WebViewBuilder` and the shared `NoopDragDropSink`.
    let sink: Arc<NoopDragDropSink> = Arc::new(NoopDragDropSink);
    let builder = wry::WebViewBuilder::new();
    let _builder = attach_drag_drop_handler(builder, false, &sink);
}

#[test]
fn attach_drag_drop_handler_does_not_clone_sink_when_capture_false() {
    // RFC 0015 §3.3: with `capture=false`, the helper takes only a borrow
    // of `Arc<S>` and must NOT increment the Arc's strong count. This
    // guards against accidental ownership creep in future refactors.
    let sink: Arc<NoopDragDropSink> = Arc::new(NoopDragDropSink);
    let before = Arc::strong_count(&sink);

    let builder = wry::WebViewBuilder::new();
    let _builder = attach_drag_drop_handler(builder, false, &sink);

    assert_eq!(Arc::strong_count(&sink), before);
}

#[test]
fn attach_drag_drop_handler_clones_sink_exactly_once_when_capture_true() {
    // RFC 0015 §3.3 dual contract: with `capture=true`, the helper must
    // perform exactly one `Arc::clone` (one atomic increment) so the
    // wry callback owns a `Send + Sync + 'static` reference. Stronger
    // claims (two clones, leak) would indicate the helper is silently
    // duplicating shared state.
    let sink: Arc<CountingSink> = Arc::new(CountingSink::default());
    let before = Arc::strong_count(&sink);

    let builder = wry::WebViewBuilder::new();
    let returned = attach_drag_drop_handler(builder, true, &sink);

    // Exactly +1: the closure stored inside the wry builder owns one Arc.
    assert_eq!(Arc::strong_count(&sink), before + 1);

    // Drop the builder (and therefore the closure inside it) and verify
    // the strong count returns to the baseline. Catches leaks via
    // accidental boxing into a `'static` slot outside the closure.
    drop(returned);
    assert_eq!(Arc::strong_count(&sink), before);
}

#[test]
fn attach_drag_drop_handler_dispatches_to_sink_when_capture_true() {
    // The `attach_drag_drop_handler` builder side effect is hard to drive
    // synchronously without spinning a real event loop. Instead, we
    // exercise the same dispatch closure shape by driving
    // `create_drag_drop_handler` directly: the helper composes this
    // primitive with `Arc::clone(sink) + sink.dispatch`, so the
    // event-name mapping, JSON payload shape, and `Over` filtering live
    // here.
    let sink: Arc<CountingSink> = Arc::new(CountingSink::default());
    let dispatch_sink = Arc::clone(&sink);
    let handler = create_drag_drop_handler(move |event_name, data| {
        dispatch_sink
            .dispatch(event_name, data)
            .expect("counting sink never errors");
    });

    // Build representative payloads matching the four wry variants.
    let enter = wry::DragDropEvent::Enter {
        paths: vec![std::path::PathBuf::from("/tmp/a.txt")],
        position: (10, 20),
    };
    let over = wry::DragDropEvent::Over {
        position: (15, 25),
    };
    let drop_evt = wry::DragDropEvent::Drop {
        paths: vec![std::path::PathBuf::from("/tmp/b.png")],
        position: (30, 40),
    };
    let leave = wry::DragDropEvent::Leave;

    handler(enter);
    handler(over);
    handler(drop_evt);
    handler(leave);

    // `Over` is filtered (RFC 0015 §5: too frequent); the other three
    // variants reach the sink in order.
    assert_eq!(sink.count(), 3, "Over events must be filtered out");

    let events = sink.events();
    let names: Vec<&str> = events.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(
        names,
        vec!["file_drop_hover", "file_drop", "file_drop_cancelled"]
    );

    // Spot-check JSON shape:
    // file_drop_hover → { hovering: true, paths: [...], position: {x,y} }
    let hover = &events[0].1;
    assert_eq!(hover["hovering"], serde_json::json!(true));
    assert_eq!(hover["paths"], serde_json::json!(["/tmp/a.txt"]));
    assert!(hover["position"].is_object());

    // file_drop → { paths: [...], position: {x,y}, timestamp: u64 }
    let drop = &events[1].1;
    assert_eq!(drop["paths"], serde_json::json!(["/tmp/b.png"]));
    assert!(drop["timestamp"].is_u64());
    // No `hovering` field on `file_drop` (RFC 0015 §5 note).
    assert!(drop.get("hovering").is_none());

    // file_drop_cancelled → { hovering: false, reason: "left_window" }
    let leave_payload = &events[2].1;
    assert_eq!(leave_payload["hovering"], serde_json::json!(false));
    assert_eq!(leave_payload["reason"], serde_json::json!("left_window"));
}

#[test]
fn dragdrop_dispatch_error_is_swallowed() {
    // RFC 0015 §3.3: if the sink returns an error, the helper logs and
    // continues — drag-drop must never block the WebView. We simulate
    // the helper's closure body manually (matching its source) so we
    // can drive it without a wry event loop. The key invariants:
    //
    //   1. `sink.dispatch` is called once per non-`Over` event.
    //   2. Errors do not propagate (no panic / no Result return).
    let sink: Arc<ErrorSink> = Arc::new(ErrorSink::default());
    let closure_sink = Arc::clone(&sink);
    let handler = create_drag_drop_handler(move |event_name, data| {
        // Mirror the helper body in helpers.rs: log on error, never bubble.
        if let Err(err) = closure_sink.dispatch(event_name, data) {
            // Use an inline tracing call so test output is observable
            // when run with `RUST_LOG=error` but the test itself does
            // not depend on a tracing subscriber being active.
            tracing::error!(
                target: "auroraview::drag_drop",
                "Failed to dispatch {} via DragDropIpcSink: {}",
                event_name,
                err
            );
        }
    });

    // Two non-`Over` events drive two dispatches.
    handler(wry::DragDropEvent::Enter {
        paths: vec![],
        position: (0, 0),
    });
    handler(wry::DragDropEvent::Leave);

    assert_eq!(sink.count.load(Ordering::SeqCst), 2);
}

#[test]
fn dragdropipcsink_blanket_send_sync() {
    // Compile-time check: `dyn DragDropIpcSink` must be usable behind
    // an `Arc<dyn DragDropIpcSink>` and crossed between threads. Any
    // future change that drops the `Send + Sync + 'static` bound from
    // the trait will break this build.
    fn assert_send_sync<T: Send + Sync + ?Sized>() {}
    assert_send_sync::<dyn DragDropIpcSink>();

    // Also exercise the runtime side: send the sink across a thread.
    let sink: Arc<dyn DragDropIpcSink> = Arc::new(NoopDragDropSink);
    let handle = std::thread::spawn(move || {
        sink.dispatch("file_drop", serde_json::json!({ "paths": [] }))
            .expect("noop dispatch always succeeds");
    });
    handle.join().expect("worker thread must not panic");
}

#[test]
fn drag_drop_event_data_to_json_skips_hovering_on_drop() {
    // RFC 0015 §5: `file_drop` payload deliberately omits `hovering`.
    // Front-end code must distinguish state via event-name, not a
    // unified `hovering` boolean.
    let drop = DragDropEventData {
        event_type: DragDropEventType::Drop,
        paths: vec!["/tmp/x".into()],
        position: Some((1.0, 2.0)),
        timestamp: Some(123),
    };
    let json = drop.to_json();
    assert!(json.get("hovering").is_none());
    assert_eq!(json["timestamp"], serde_json::json!(123));
}
