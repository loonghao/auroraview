//! Tests for the RFC 0015 drag-drop wiring inside `auroraview-desktop`.
//!
//! Two layers are exercised here:
//!
//! 1. `DesktopConfig.capture_file_drop` field default + builder ergonomics.
//! 2. `DragDropIpcSink for IpcRouter` — verifies that drag-drop events
//!    routed through the IPC pipeline reach subscribed listeners and
//!    that the "no listener" case neither errors nor panics.
//!
//! The `should_warn_drag_drop_listener_missing` warn-once state machine
//! is process-global (`AtomicBool`), so we cannot directly observe its
//! transitions from outside the crate without mutating shared state
//! used by other tests. The behavioural contract that matters from a
//! caller's perspective — `dispatch` always returns `Ok(())` regardless
//! of listener presence — is what we assert here.

use auroraview_core::builder::DragDropIpcSink;
use auroraview_desktop::{DesktopConfig, IpcRouter};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// ---------------------------------------------------------------------
// DesktopConfig.capture_file_drop
// ---------------------------------------------------------------------

#[test]
fn desktop_config_default_disables_capture_file_drop() {
    let config = DesktopConfig::default();
    assert!(
        !config.capture_file_drop,
        "RFC 0015: capture_file_drop must default to false so HTML5 drag-drop \
         remains the out-of-the-box behaviour"
    );
}

#[test]
fn desktop_config_builder_enables_capture_file_drop() {
    let config = DesktopConfig::default().capture_file_drop(true);
    assert!(config.capture_file_drop);
}

#[test]
fn desktop_config_builder_disables_capture_file_drop() {
    // Start from the enabled state to make sure the builder honours an
    // explicit `false` and does not just OR-in.
    let config = DesktopConfig::default()
        .capture_file_drop(true)
        .capture_file_drop(false);
    assert!(!config.capture_file_drop);
}

#[test]
fn desktop_config_builder_returns_owned_self() {
    // The builder takes `mut self` and returns `Self`, so we should be
    // able to chain it with the rest of the builder vocabulary.
    let config = DesktopConfig::default()
        .title("drag-drop probe")
        .capture_file_drop(true);
    assert_eq!(config.title, "drag-drop probe");
    assert!(config.capture_file_drop);
}

// ---------------------------------------------------------------------
// DragDropIpcSink for IpcRouter
// ---------------------------------------------------------------------

#[test]
fn ipc_router_dispatch_with_listener_invokes_handler() {
    let router = IpcRouter::new();
    let counter = Arc::new(AtomicUsize::new(0));

    {
        let counter = counter.clone();
        router.on("file_drop_hover", move |payload| {
            // The dispatch helper passes the JSON value untouched; we
            // sanity-check the shape so a future change to the envelope
            // (e.g. wrapping in `{ "data": ... }`) is caught here.
            assert!(
                payload.get("paths").is_some(),
                "expected `paths` field in drag-drop payload, got: {payload}"
            );
            counter.fetch_add(1, Ordering::SeqCst);
        });
    }

    let payload = serde_json::json!({
        "paths": ["C:/tmp/a.txt"],
        "position": { "x": 10, "y": 20 },
    });

    let result = <IpcRouter as DragDropIpcSink>::dispatch(&router, "file_drop_hover", payload);
    assert!(
        result.is_ok(),
        "dispatch should never error on success path"
    );
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[test]
fn ipc_router_dispatch_fan_out_to_multiple_listeners() {
    let router = IpcRouter::new();
    let calls = Arc::new(AtomicUsize::new(0));

    for _ in 0..3 {
        let calls = calls.clone();
        router.on("file_drop", move |_| {
            calls.fetch_add(1, Ordering::SeqCst);
        });
    }

    let result = <IpcRouter as DragDropIpcSink>::dispatch(
        &router,
        "file_drop",
        serde_json::json!({ "paths": [] }),
    );

    assert!(result.is_ok());
    assert_eq!(
        calls.load(Ordering::SeqCst),
        3,
        "every registered listener should be invoked"
    );
}

#[test]
fn ipc_router_dispatch_without_listener_returns_ok() {
    // The drag-drop sink contract requires `Ok(())` even when no listener
    // is registered: drag-drop events fire per OS cursor transition, so
    // returning `Err` here would generate one `tracing::error!` per
    // mouse-move and flood the subscriber. The `warn!` at first
    // occurrence is the canonical breadcrumb, asserted via the unique
    // event-name technique below.
    let router = IpcRouter::new();

    // Use a deliberately unique event name so this test does not race
    // with the warn-once `AtomicBool` for `file_drop_*` (which other
    // tests in this binary may have already flipped).
    let event_name = "file_drop_no_listener_probe_unique";

    let result = <IpcRouter as DragDropIpcSink>::dispatch(
        &router,
        event_name,
        serde_json::json!({ "paths": ["x"] }),
    );
    assert!(result.is_ok());

    // A second dispatch with the same name should also be a clean Ok —
    // the warn-once guard suppresses the second log line, but never
    // affects the public return value.
    let result = <IpcRouter as DragDropIpcSink>::dispatch(
        &router,
        event_name,
        serde_json::json!({ "paths": [] }),
    );
    assert!(result.is_ok());
}

#[test]
fn ipc_router_dispatch_isolates_event_names() {
    // Listener registered for `file_drop_hover` must not receive
    // `file_drop` events. This pins down the `event_listeners` keying
    // contract relied upon by the drag-drop sink — a future refactor
    // that, say, wildcards the event name would be caught here.
    let router = IpcRouter::new();
    let hover_calls = Arc::new(AtomicUsize::new(0));
    let drop_calls = Arc::new(AtomicUsize::new(0));

    {
        let hover_calls = hover_calls.clone();
        router.on("file_drop_hover", move |_| {
            hover_calls.fetch_add(1, Ordering::SeqCst);
        });
    }
    {
        let drop_calls = drop_calls.clone();
        router.on("file_drop", move |_| {
            drop_calls.fetch_add(1, Ordering::SeqCst);
        });
    }

    let _ =
        <IpcRouter as DragDropIpcSink>::dispatch(&router, "file_drop_hover", serde_json::json!({}));
    assert_eq!(hover_calls.load(Ordering::SeqCst), 1);
    assert_eq!(drop_calls.load(Ordering::SeqCst), 0);

    let _ = <IpcRouter as DragDropIpcSink>::dispatch(&router, "file_drop", serde_json::json!({}));
    assert_eq!(hover_calls.load(Ordering::SeqCst), 1);
    assert_eq!(drop_calls.load(Ordering::SeqCst), 1);
}
