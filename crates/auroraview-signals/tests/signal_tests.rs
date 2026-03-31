//! Integration tests for auroraview-signals
//!
//! Tests cover Signal, SignalRegistry, EventBus, and emit-path clone
//! optimisations (last-handler move, bridge-path cloning).

use auroraview_signals::prelude::*;
use serde_json::json;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

// ---------------------------------------------------------------------------
// Signal<T> — emit optimisation
// ---------------------------------------------------------------------------

#[test]
fn emit_zero_handlers_no_panic() {
    let signal: Signal<String> = Signal::new();
    signal.emit("unused".to_string()); // must not panic
}

#[test]
fn emit_single_handler_no_clone_needed() {
    let signal: Signal<i32> = Signal::new();
    let sum = Arc::new(AtomicUsize::new(0));
    let s = sum.clone();
    signal.connect(move |x| { s.fetch_add(x as usize, Ordering::SeqCst); });
    signal.emit(7);
    assert_eq!(sum.load(Ordering::SeqCst), 7);
}

#[test]
fn emit_multiple_handlers_all_receive_value() {
    let signal: Signal<u64> = Signal::new();
    let sum = Arc::new(AtomicUsize::new(0));

    for _ in 0..5 {
        let s = sum.clone();
        signal.connect(move |x| { s.fetch_add(x as usize, Ordering::SeqCst); });
    }

    signal.emit(10);
    // All 5 handlers should have received 10
    assert_eq!(sum.load(Ordering::SeqCst), 50);
}

#[test]
fn emit_count_returns_correct_count() {
    let signal: Signal<i32> = Signal::new();
    signal.connect(|_| {});
    signal.connect(|_| {});
    signal.connect(|_| {});
    assert_eq!(signal.emit_count(0), 3);
}

#[test]
fn emit_count_zero_when_no_handlers() {
    let signal: Signal<i32> = Signal::new();
    assert_eq!(signal.emit_count(42), 0);
}

/// Verify the last-handler move optimisation: a handler that receives the
/// final value can mutate it without needing a clone.  We test correctness
/// by using a non-Copy type (String) with multiple handlers.
#[test]
fn emit_non_copy_type_multi_handler() {
    let signal: Signal<String> = Signal::new();
    let results = Arc::new(parking_lot::Mutex::new(Vec::<String>::new()));

    for _ in 0..3 {
        let r = results.clone();
        signal.connect(move |s| r.lock().push(s));
    }

    signal.emit("hello".to_string());

    let r = results.lock();
    assert_eq!(r.len(), 3);
    assert!(r.iter().all(|s| s == "hello"));
}

// ---------------------------------------------------------------------------
// connect_once — value received exactly once
// ---------------------------------------------------------------------------

#[test]
fn connect_once_fires_once() {
    let signal: Signal<i32> = Signal::new();
    let count = Arc::new(AtomicUsize::new(0));
    let c = count.clone();
    signal.connect_once(move |_| { c.fetch_add(1, Ordering::SeqCst); });

    signal.emit(1);
    signal.emit(1);
    signal.emit(1);
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

// ---------------------------------------------------------------------------
// connect_guard — RAII auto-disconnect
// ---------------------------------------------------------------------------

#[test]
fn connect_guard_auto_disconnects_on_drop() {
    let signal = Arc::new(Signal::<i32>::new());
    let count = Arc::new(AtomicUsize::new(0));
    let c = count.clone();

    {
        let _guard = signal.connect_guard(move |_| { c.fetch_add(1, Ordering::SeqCst); });
        signal.emit(1);
        assert_eq!(count.load(Ordering::SeqCst), 1);
    } // guard dropped → handler disconnected

    signal.emit(1);
    assert_eq!(count.load(Ordering::SeqCst), 1); // still 1
}

// ---------------------------------------------------------------------------
// Deadlock safety: handler calls connect/disconnect during emit
// ---------------------------------------------------------------------------

#[test]
fn emit_handler_disconnect_all_no_deadlock() {
    let signal = Arc::new(Signal::<i32>::new());
    let s = signal.clone();
    signal.connect(move |_| s.disconnect_all());
    signal.emit(1); // must not deadlock
    assert_eq!(signal.handler_count(), 0);
}

#[test]
fn emit_handler_connect_new_no_deadlock() {
    let signal = Arc::new(Signal::<i32>::new());
    let s = signal.clone();
    signal.connect(move |_| { s.connect(|_| {}); });
    signal.emit(1); // must not deadlock
    assert_eq!(signal.handler_count(), 2);
}

// ---------------------------------------------------------------------------
// SignalRegistry
// ---------------------------------------------------------------------------

#[test]
fn registry_emit_delivers_to_all_handlers() {
    let reg = SignalRegistry::new();
    let count = Arc::new(AtomicUsize::new(0));

    for _ in 0..4 {
        let c = count.clone();
        reg.connect("ev", move |_| { c.fetch_add(1, Ordering::SeqCst); });
    }

    reg.emit("ev", json!(null));
    assert_eq!(count.load(Ordering::SeqCst), 4);
}

#[test]
fn registry_emit_nonexistent_returns_zero() {
    let reg = SignalRegistry::new();
    assert_eq!(reg.emit("ghost", json!(null)), 0);
}

#[test]
fn registry_connect_once_fires_once() {
    let reg = SignalRegistry::new();
    let count = Arc::new(AtomicUsize::new(0));
    let c = count.clone();
    reg.connect_once("once_ev", move |_| { c.fetch_add(1, Ordering::SeqCst); });

    reg.emit("once_ev", json!(null));
    reg.emit("once_ev", json!(null));
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[test]
fn registry_remove_clears_handlers() {
    let reg = SignalRegistry::new();
    reg.connect("sig", |_| {});
    assert!(reg.contains("sig"));
    reg.remove("sig");
    assert!(!reg.contains("sig"));
}

// ---------------------------------------------------------------------------
// EventBus
// ---------------------------------------------------------------------------

#[test]
fn bus_emit_reaches_local_handlers() {
    let bus = EventBus::new();
    let count = Arc::new(AtomicUsize::new(0));
    let c = count.clone();
    bus.on("ev", move |_| { c.fetch_add(1, Ordering::SeqCst); });
    bus.emit("ev", json!(null));
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[test]
fn bus_emit_skips_bridges_when_no_bridge_registered() {
    let bus = EventBus::new();
    let count = Arc::new(AtomicUsize::new(0));
    let c = count.clone();
    bus.on("ev", move |_| { c.fetch_add(1, Ordering::SeqCst); });
    let n = bus.emit("ev", json!(null));
    assert_eq!(n, 1);
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[test]
fn bus_emit_local_skips_bridges() {
    use auroraview_signals::bridge::CallbackBridge;
    let bus = EventBus::new();
    let bridge_hit = Arc::new(AtomicUsize::new(0));
    let bh = bridge_hit.clone();
    bus.add_bridge(CallbackBridge::new("b", move |_, _| {
        bh.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }));

    let local_hit = Arc::new(AtomicUsize::new(0));
    let lh = local_hit.clone();
    bus.on("ev", move |_| { lh.fetch_add(1, Ordering::SeqCst); });

    bus.emit_local("ev", json!(null));
    assert_eq!(local_hit.load(Ordering::SeqCst), 1);
    assert_eq!(bridge_hit.load(Ordering::SeqCst), 0);
}

#[test]
fn bus_emit_to_bridges_skips_local() {
    use auroraview_signals::bridge::CallbackBridge;
    let bus = EventBus::new();
    let bridge_hit = Arc::new(AtomicUsize::new(0));
    let bh = bridge_hit.clone();
    bus.add_bridge(CallbackBridge::new("b", move |_, _| {
        bh.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }));

    let local_hit = Arc::new(AtomicUsize::new(0));
    let lh = local_hit.clone();
    bus.on("ev", move |_| { lh.fetch_add(1, Ordering::SeqCst); });

    bus.emit_to_bridges("ev", json!(null)).unwrap();
    assert_eq!(local_hit.load(Ordering::SeqCst), 0);
    assert_eq!(bridge_hit.load(Ordering::SeqCst), 1);
}

#[test]
fn bus_once_fires_once() {
    let bus = EventBus::new();
    let count = Arc::new(AtomicUsize::new(0));
    let c = count.clone();
    bus.once("ev", move |_| { c.fetch_add(1, Ordering::SeqCst); });
    bus.emit("ev", json!(null));
    bus.emit("ev", json!(null));
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[test]
fn bus_off_stops_handler() {
    let bus = EventBus::new();
    let count = Arc::new(AtomicUsize::new(0));
    let c = count.clone();
    let id = bus.on("ev", move |_| { c.fetch_add(1, Ordering::SeqCst); });
    bus.emit("ev", json!(null));
    bus.off("ev", id);
    bus.emit("ev", json!(null));
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[test]
fn bus_middleware_filter_blocks_event() {
    use auroraview_signals::middleware::FilterMiddleware;
    let bus = EventBus::new();
    bus.use_middleware(FilterMiddleware::new().deny_pattern("private:.*").unwrap());

    let hit = Arc::new(AtomicUsize::new(0));
    let h = hit.clone();
    bus.on("private:data", move |_| { h.fetch_add(1, Ordering::SeqCst); });

    let n = bus.emit("private:data", json!(null));
    assert_eq!(n, 0);
    assert_eq!(hit.load(Ordering::SeqCst), 0);
}

#[test]
fn bus_middleware_allows_other_events() {
    use auroraview_signals::middleware::FilterMiddleware;
    let bus = EventBus::new();
    bus.use_middleware(FilterMiddleware::new().deny_pattern("private:.*").unwrap());

    let hit = Arc::new(AtomicUsize::new(0));
    let h = hit.clone();
    bus.on("public:event", move |_| { h.fetch_add(1, Ordering::SeqCst); });

    bus.emit("public:event", json!(null));
    assert_eq!(hit.load(Ordering::SeqCst), 1);
}
