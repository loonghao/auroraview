//! Integration tests for auroraview-signals
//!
//! Tests cover Signal, SignalRegistry, EventBus, and emit-path clone
//! optimisations (last-handler move, bridge-path cloning).

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use auroraview_signals::prelude::*;
use serde_json::json;

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
// connect_ref / emit_ref — zero-clone path
// ---------------------------------------------------------------------------

#[test]
fn connect_ref_zero_clone_path_receives_value() {
    let signal: Signal<String> = Signal::new();
    let results = Arc::new(parking_lot::Mutex::new(Vec::<String>::new()));
    let r = results.clone();
    signal.connect_ref(move |s| r.lock().push(s.clone()));

    signal.emit("alpha".to_string());
    signal.emit("beta".to_string());

    let r = results.lock();
    assert_eq!(*r, vec!["alpha", "beta"]);
}

#[test]
fn connect_ref_and_connect_both_called_on_emit() {
    let signal: Signal<i32> = Signal::new();
    let ref_sum = Arc::new(AtomicUsize::new(0));
    let val_sum = Arc::new(AtomicUsize::new(0));

    let rs = ref_sum.clone();
    signal.connect_ref(move |x| { rs.fetch_add(*x as usize, Ordering::SeqCst); });

    let vs = val_sum.clone();
    signal.connect(move |x| { vs.fetch_add(x as usize, Ordering::SeqCst); });

    signal.emit(5);
    assert_eq!(ref_sum.load(Ordering::SeqCst), 5);
    assert_eq!(val_sum.load(Ordering::SeqCst), 5);
}

#[test]
fn emit_ref_does_not_call_value_handlers() {
    let signal: Signal<i32> = Signal::new();
    let ref_hit = Arc::new(AtomicUsize::new(0));
    let val_hit = Arc::new(AtomicUsize::new(0));

    let rh = ref_hit.clone();
    signal.connect_ref(move |_| { rh.fetch_add(1, Ordering::SeqCst); });

    let vh = val_hit.clone();
    signal.connect(move |_| { vh.fetch_add(1, Ordering::SeqCst); });

    let n = signal.emit_ref(&99);
    assert_eq!(n, 1);
    assert_eq!(ref_hit.load(Ordering::SeqCst), 1);
    assert_eq!(val_hit.load(Ordering::SeqCst), 0);
}

#[test]
fn connect_ref_guard_auto_disconnects() {
    let signal = Arc::new(Signal::<i32>::new());
    let count = Arc::new(AtomicUsize::new(0));
    let c = count.clone();

    {
        let _guard = signal.connect_ref_guard(move |_| { c.fetch_add(1, Ordering::SeqCst); });
        signal.emit(1);
        assert_eq!(count.load(Ordering::SeqCst), 1);
    } // guard dropped → disconnected

    signal.emit(1);
    assert_eq!(count.load(Ordering::SeqCst), 1); // still 1
}

#[test]
fn disconnect_all_removes_ref_handlers() {
    let signal: Signal<i32> = Signal::new();
    signal.connect_ref(|_| {});
    signal.connect_ref(|_| {});
    signal.connect(|_| {});
    assert_eq!(signal.handler_count(), 3);

    signal.disconnect_all();
    assert_eq!(signal.handler_count(), 0);

    let n = signal.emit_ref(&0);
    assert_eq!(n, 0);
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

// ---------------------------------------------------------------------------
// Concurrent emit stress test
// ---------------------------------------------------------------------------

#[test]
fn concurrent_emit_from_many_threads_all_receive() {
    use std::thread;

    let signal = Arc::new(Signal::<u64>::new());
    let total = Arc::new(AtomicUsize::new(0));

    // 4 handlers
    for _ in 0..4 {
        let t = total.clone();
        signal.connect(move |v| { t.fetch_add(v as usize, Ordering::SeqCst); });
    }

    // 8 threads each emitting 1 → total expected = 8 × 4 × 1 = 32
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let s = signal.clone();
            thread::spawn(move || s.emit(1_u64))
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(total.load(Ordering::SeqCst), 32);
}

#[test]
fn concurrent_connect_and_emit_no_panic() {
    use std::thread;

    let signal = Arc::new(Signal::<i32>::new());
    let connected = Arc::new(AtomicUsize::new(0));

    let connectors: Vec<_> = (0..5)
        .map(|_| {
            let s = signal.clone();
            let c = connected.clone();
            thread::spawn(move || {
                s.connect(move |_| { c.fetch_add(1, Ordering::SeqCst); });
            })
        })
        .collect();

    let emitters: Vec<_> = (0..5)
        .map(|_| {
            let s = signal.clone();
            thread::spawn(move || s.emit(1))
        })
        .collect();

    for h in connectors.into_iter().chain(emitters) {
        h.join().unwrap();
    }
    // must not panic — correctness of count is non-deterministic
}

// ---------------------------------------------------------------------------
// ChannelBridge — bounded capacity and disconnect
// ---------------------------------------------------------------------------

#[test]
fn channel_bridge_bounded_delivers_within_capacity() {
    use auroraview_signals::bridge::ChannelBridge;

    let (bridge, receiver) = ChannelBridge::bounded("bounded", 4);

    bridge.emit("ev", json!(1)).unwrap();
    bridge.emit("ev", json!(2)).unwrap();

    let m1 = receiver.recv().unwrap();
    let m2 = receiver.recv().unwrap();
    assert_eq!(m1.data, json!(1));
    assert_eq!(m2.data, json!(2));
}

#[test]
fn channel_bridge_send_fails_when_receiver_dropped() {
    use auroraview_signals::bridge::ChannelBridge;

    let (bridge, receiver) = ChannelBridge::new("ch");
    // Drop receiver — subsequent sends will fail because channel has no reader
    drop(receiver);

    let res = bridge.emit("ev", json!(null));
    assert!(res.is_err(), "send should fail after receiver is dropped");
}

#[test]
fn channel_bridge_disconnected_returns_err() {
    use auroraview_signals::bridge::ChannelBridge;

    let (bridge, _rx) = ChannelBridge::new("ch");
    bridge.disconnect().unwrap();
    assert!(!bridge.is_connected());

    let res = bridge.emit("ev", json!(null));
    assert!(res.is_err());
}

#[test]
fn channel_bridge_message_carries_event_and_data() {
    use auroraview_signals::bridge::{ChannelBridge, ChannelMessage};

    let (bridge, rx) = ChannelBridge::new("ch");
    bridge.emit("user:login", json!({"user": "alice"})).unwrap();

    let ChannelMessage { event, data } = rx.recv().unwrap();
    assert_eq!(event, "user:login");
    assert_eq!(data["user"], "alice");
}

// ---------------------------------------------------------------------------
// MultiBridge — all bridges fail → returns Err
// ---------------------------------------------------------------------------

#[test]
fn multi_bridge_all_fail_returns_err() {
    use auroraview_signals::bridge::{BridgeError, CallbackBridge, MultiBridge};

    let multi = MultiBridge::new("multi");
    multi.add(CallbackBridge::new("b1", |_, _| {
        Err(BridgeError::SendFailed("b1 down".into()))
    }));
    multi.add(CallbackBridge::new("b2", |_, _| {
        Err(BridgeError::SendFailed("b2 down".into()))
    }));

    let res = multi.emit("ev", json!(null));
    assert!(res.is_err());
}

#[test]
fn multi_bridge_partial_fail_returns_ok() {
    use auroraview_signals::bridge::{BridgeError, CallbackBridge, MultiBridge};
    use std::sync::atomic::{AtomicUsize, Ordering};

    let hit = Arc::new(AtomicUsize::new(0));
    let h = hit.clone();

    let multi = MultiBridge::new("multi");
    multi.add(CallbackBridge::new("good", move |_, _| {
        h.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }));
    multi.add(CallbackBridge::new("bad", |_, _| {
        Err(BridgeError::SendFailed("bad".into()))
    }));

    let res = multi.emit("ev", json!(null));
    assert!(res.is_ok()); // partial success → Ok
    assert_eq!(hit.load(Ordering::SeqCst), 1);
}

// ---------------------------------------------------------------------------
// FilterMiddleware — deny_by_default + runtime patterns
// ---------------------------------------------------------------------------

/// Directly verify FilterMiddleware::deny_by_default logic at the middleware level.
///
/// Note: `Regex::is_match` performs a *substring* search, so "safe:.*" would
/// match "unsafe:thing" because it contains the substring "safe:thing".
/// Use "internal:thing" which has no overlap with "safe:" prefix.
#[test]
fn filter_deny_by_default_middleware_level() {
    use auroraview_signals::middleware::FilterMiddleware;

    let filter = FilterMiddleware::deny_by_default();
    filter.add_allow_pattern("safe:.*").unwrap();

    let mut data = json!(null);

    // "internal:thing" contains no "safe:" substring → denied
    let res = filter.before_emit("internal:thing", &mut data);
    assert!(!res.should_continue(), "expected Stop but got {:?}", res);

    // "safe:thing" matches allow pattern → allowed
    let res2 = filter.before_emit("safe:thing", &mut data);
    assert!(res2.should_continue(), "expected Continue but got {:?}", res2);
}

/// Verify deny_by_default blocks events via EventBus middleware pipeline.
#[test]
fn filter_deny_by_default_via_event_bus() {
    use auroraview_signals::middleware::FilterMiddleware;

    let filter = FilterMiddleware::deny_by_default();
    filter.add_allow_pattern("safe:.*").unwrap();

    let bus = EventBus::new();
    bus.use_middleware(filter);

    let blocked_count = Arc::new(AtomicUsize::new(0));
    let bc = blocked_count.clone();
    bus.on("internal:thing", move |_| { bc.fetch_add(1, Ordering::SeqCst); });

    let allowed_count = Arc::new(AtomicUsize::new(0));
    let ac = allowed_count.clone();
    bus.on("safe:thing", move |_| { ac.fetch_add(1, Ordering::SeqCst); });

    let n_blocked = bus.emit("internal:thing", json!(null));
    let n_allowed = bus.emit("safe:thing", json!(null));

    assert_eq!(n_blocked, 0, "middleware should block internal:thing");
    assert_eq!(n_allowed, 1, "safe:thing should pass deny_by_default filter");
    assert_eq!(blocked_count.load(Ordering::SeqCst), 0);
    assert_eq!(allowed_count.load(Ordering::SeqCst), 1);
}

#[test]
fn filter_runtime_add_deny_pattern_blocks_new_events() {
    use auroraview_signals::middleware::FilterMiddleware;

    let filter = FilterMiddleware::new();
    let bus = EventBus::new();

    // Add deny pattern at runtime (after construction)
    filter.add_deny_pattern("secret:.*").unwrap();
    bus.use_middleware(filter);

    let hit = Arc::new(AtomicUsize::new(0));
    let h = hit.clone();
    bus.on("secret:data", move |_| { h.fetch_add(1, Ordering::SeqCst); });

    bus.emit("secret:data", json!(null));
    assert_eq!(hit.load(Ordering::SeqCst), 0);
}

#[test]
fn filter_clear_allows_all_after_clear() {
    use auroraview_signals::middleware::FilterMiddleware;

    let filter = FilterMiddleware::new().deny_pattern("blocked:.*").unwrap();
    let hit = Arc::new(AtomicUsize::new(0));
    let h = hit.clone();

    // Confirm deny pattern works before clear
    let mut data = json!(null);
    let res = filter.before_emit("blocked:event", &mut data);
    assert!(!res.should_continue());

    // After clear, all events pass
    filter.clear();
    let res2 = filter.before_emit("blocked:event", &mut data);
    assert!(res2.should_continue());

    let _ = h.fetch_add(0, Ordering::SeqCst); // suppress unused
}

// ---------------------------------------------------------------------------
// TransformMiddleware — global transform + runtime add
// ---------------------------------------------------------------------------

#[test]
fn transform_global_transform_applied_to_all_events() {
    use auroraview_signals::middleware::TransformMiddleware;

    let transform = TransformMiddleware::new().set_global_transform(|data| {
        if let Some(obj) = data.as_object_mut() {
            obj.insert("global".to_string(), json!(true));
        }
    });

    let mut data = json!({});
    transform.before_emit("any:event", &mut data);
    assert_eq!(data["global"], true);
}

#[test]
fn transform_runtime_add_applies_to_subsequent_events() {
    use auroraview_signals::middleware::TransformMiddleware;

    let transform = TransformMiddleware::new();
    transform
        .add_runtime_transform("dyn:.*", |data| {
            if let Some(obj) = data.as_object_mut() {
                obj.insert("dynamic".to_string(), json!(1));
            }
        })
        .unwrap();

    let mut data = json!({});
    transform.before_emit("dyn:event", &mut data);
    assert_eq!(data["dynamic"], 1);

    // Non-matching event is unchanged
    let mut data2 = json!({});
    transform.before_emit("other:event", &mut data2);
    assert!(data2.get("dynamic").is_none());
}

// ---------------------------------------------------------------------------
// ConnectionGuard — detach / manual disconnect / is_attached
// ---------------------------------------------------------------------------

#[test]
fn connection_guard_detach_keeps_handler_alive() {
    let signal = Arc::new(Signal::<i32>::new());
    let count = Arc::new(AtomicUsize::new(0));
    let c = count.clone();

    let conn_id = {
        let guard = signal.connect_guard(move |_| { c.fetch_add(1, Ordering::SeqCst); });
        assert!(guard.is_attached());
        guard.detach() // detach before drop
    };

    signal.emit(1);
    assert_eq!(count.load(Ordering::SeqCst), 1); // still connected

    signal.disconnect(conn_id);
    signal.emit(1);
    assert_eq!(count.load(Ordering::SeqCst), 1); // now gone
}

#[test]
fn connection_guard_manual_disconnect_returns_true() {
    let signal = Arc::new(Signal::<i32>::new());
    let guard = signal.connect_guard(|_| {});
    let removed = guard.disconnect(); // explicit disconnect
    assert!(removed);
    assert_eq!(signal.handler_count(), 0);
}

#[test]
fn connection_guard_id_accessible() {
    let signal = Arc::new(Signal::<i32>::new());
    let guard = signal.connect_guard(|_| {});
    let id = guard.id();
    assert!(signal.connections().contains(&id));
}

// ---------------------------------------------------------------------------
// SignalRegistry — emit_or_create / get_or_create / get / disconnect_all
// ---------------------------------------------------------------------------

#[test]
fn registry_emit_or_create_creates_and_delivers() {
    let reg = SignalRegistry::new();
    let count = Arc::new(AtomicUsize::new(0));
    let c = count.clone();
    reg.connect("auto", move |_| { c.fetch_add(1, Ordering::SeqCst); });

    // emit_or_create on existing signal
    let n = reg.emit_or_create("auto", json!(null));
    assert_eq!(n, 1);
    assert_eq!(count.load(Ordering::SeqCst), 1);

    // emit_or_create on nonexistent signal creates it (0 handlers)
    let n2 = reg.emit_or_create("new_auto", json!(null));
    assert_eq!(n2, 0);
    assert!(reg.contains("new_auto"));
}

#[test]
fn registry_get_returns_none_for_unknown() {
    let reg = SignalRegistry::new();
    assert!(reg.get("ghost").is_none());
}

#[test]
fn registry_get_returns_some_after_connect() {
    let reg = SignalRegistry::new();
    reg.connect("known", |_| {});
    assert!(reg.get("known").is_some());
}

#[test]
fn registry_disconnect_all_clears_handlers() {
    let reg = SignalRegistry::new();
    reg.connect("ev", |_| {});
    reg.connect("ev", |_| {});
    assert_eq!(reg.handler_count("ev"), 2);

    reg.disconnect_all("ev");
    assert_eq!(reg.handler_count("ev"), 0);

    // Signal still exists (just empty)
    assert!(reg.contains("ev"));
}

#[test]
fn registry_is_connected_false_when_no_handlers() {
    let reg = SignalRegistry::new();
    assert!(!reg.is_connected("ev"));
    reg.connect("ev", |_| {});
    assert!(reg.is_connected("ev"));
}

// ---------------------------------------------------------------------------
// EventBus — clear / off_all / bridge_names / middleware_count
// ---------------------------------------------------------------------------

#[test]
fn bus_clear_removes_all_handlers() {
    let bus = EventBus::new();
    bus.on("a", |_| {});
    bus.on("b", |_| {});
    assert_eq!(bus.total_handler_count(), 2);

    bus.clear();
    assert_eq!(bus.total_handler_count(), 0);
    assert_eq!(bus.event_count(), 0);
}

#[test]
fn bus_off_all_removes_handlers_for_event() {
    let bus = EventBus::new();
    bus.on("ev", |_| {});
    bus.on("ev", |_| {});
    bus.on("other", |_| {});
    assert_eq!(bus.handler_count("ev"), 2);

    bus.off_all("ev");
    assert_eq!(bus.handler_count("ev"), 0);
    assert_eq!(bus.handler_count("other"), 1); // unchanged
}

#[test]
fn bus_bridge_names_lists_all_bridges() {
    use auroraview_signals::bridge::CallbackBridge;

    let bus = EventBus::new();
    bus.add_bridge(CallbackBridge::new("alpha", |_, _| Ok(())));
    bus.add_bridge(CallbackBridge::new("beta", |_, _| Ok(())));

    let mut names = bus.bridge_names();
    names.sort();
    assert_eq!(names, vec!["alpha", "beta"]);
}

#[test]
fn bus_middleware_count_reflects_added_middleware() {
    use auroraview_signals::middleware::LoggingMiddleware;

    let bus = EventBus::new();
    assert_eq!(bus.middleware_count(), 0);

    bus.use_middleware(LoggingMiddleware::new(LogLevel::Debug));
    assert_eq!(bus.middleware_count(), 1);

    bus.use_middleware(LoggingMiddleware::new(LogLevel::Info));
    assert_eq!(bus.middleware_count(), 2);
}

#[test]
fn bus_remove_bridge_decrements_count() {
    use auroraview_signals::bridge::CallbackBridge;

    let bus = EventBus::new();
    bus.add_bridge(CallbackBridge::new("target", |_, _| Ok(())));
    bus.add_bridge(CallbackBridge::new("keep", |_, _| Ok(())));
    assert_eq!(bus.bridge_count(), 2);

    let removed = bus.remove_bridge("target");
    assert!(removed);
    assert_eq!(bus.bridge_count(), 1);
}

#[test]
fn bus_named_has_expected_name() {
    let bus = EventBus::named("pipeline");
    assert_eq!(bus.name(), Some("pipeline"));
}

// ---------------------------------------------------------------------------
// WebViewBridge — from_arc + prefix filter skips non-matching
// ---------------------------------------------------------------------------

#[test]
fn webview_bridge_from_arc_forwards_events() {
    use auroraview_signals::prelude::{WebViewBridge, WebViewSender};
    use std::sync::atomic::{AtomicUsize, Ordering};

    let count = Arc::new(AtomicUsize::new(0));
    let c = count.clone();
    let sender: WebViewSender = Arc::new(move |_msg| {
        c.fetch_add(1, Ordering::SeqCst);
        Ok(())
    });

    let bridge = WebViewBridge::from_arc("wv", sender);
    bridge.emit("app:event", json!(null)).unwrap();
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[test]
fn webview_bridge_prefix_filter_skips_non_matching() {
    use auroraview_signals::prelude::WebViewBridge;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let count = Arc::new(AtomicUsize::new(0));
    let c = count.clone();

    let bridge = WebViewBridge::with_prefix_filter("wv", "ui:", move |_| {
        c.fetch_add(1, Ordering::SeqCst);
        Ok(())
    });

    bridge.emit("ui:click", json!(null)).unwrap();
    assert_eq!(count.load(Ordering::SeqCst), 1);

    // Non-matching prefix — silently skipped (returns Ok)
    let res = bridge.emit("sys:shutdown", json!(null));
    assert!(res.is_ok());
    assert_eq!(count.load(Ordering::SeqCst), 1); // unchanged
}

// ---------------------------------------------------------------------------
// Signal::named / default / debug format
// ---------------------------------------------------------------------------

#[test]
fn signal_named_stores_name() {
    let sig: Signal<i32> = Signal::named("my:signal");
    assert_eq!(sig.name(), Some("my:signal"));
}

#[test]
fn signal_default_creates_unnamed() {
    let sig: Signal<i32> = Signal::default();
    assert_eq!(sig.name(), None);
    assert_eq!(sig.handler_count(), 0);
}

#[test]
fn signal_connections_returns_all_ids() {
    let sig: Signal<i32> = Signal::new();
    let c1 = sig.connect(|_| {});
    let c2 = sig.connect_ref(|_| {});
    let ids = sig.connections();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&c1));
    assert!(ids.contains(&c2));
}

#[test]
fn signal_disconnect_nonexistent_returns_false() {
    use auroraview_signals::connection::ConnectionId;

    let sig: Signal<i32> = Signal::new();
    let fake = ConnectionId::from_raw(u64::MAX);
    assert!(!sig.disconnect(fake));
}
