//! Signal-slot system tests

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use auroraview_core::signals::{
    CallbackBridge, ChannelBridge, ConnectionGuard, EventBridge, EventBus, FilterMiddleware,
    LogLevel, LoggingMiddleware, Signal, SignalRegistry, WebViewSignals,
};

// ============================================================================
// Signal core tests
// ============================================================================

#[test]
fn signal_connect_emit() {
    let signal: Signal<i32> = Signal::new();
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let _conn = signal.connect(move |value| {
        counter_clone.fetch_add(value as usize, Ordering::SeqCst);
    });

    signal.emit(5);
    signal.emit(3);

    assert_eq!(counter.load(Ordering::SeqCst), 8);
}

#[test]
fn signal_disconnect() {
    let signal: Signal<i32> = Signal::new();
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let conn = signal.connect(move |value| {
        counter_clone.fetch_add(value as usize, Ordering::SeqCst);
    });

    signal.emit(5);
    assert_eq!(counter.load(Ordering::SeqCst), 5);

    signal.disconnect(conn);
    signal.emit(3);
    assert_eq!(counter.load(Ordering::SeqCst), 5);
}

#[test]
fn signal_multiple_handlers() {
    let signal: Signal<i32> = Signal::new();
    let counter1 = Arc::new(AtomicUsize::new(0));
    let counter2 = Arc::new(AtomicUsize::new(0));
    let c1 = counter1.clone();
    let c2 = counter2.clone();

    let _conn1 = signal.connect(move |v| {
        c1.fetch_add(v as usize, Ordering::SeqCst);
    });
    let _conn2 = signal.connect(move |v| {
        c2.fetch_add(v as usize * 2, Ordering::SeqCst);
    });

    signal.emit(5);

    assert_eq!(counter1.load(Ordering::SeqCst), 5);
    assert_eq!(counter2.load(Ordering::SeqCst), 10);
}

#[test]
fn connect_once() {
    let signal: Signal<i32> = Signal::new();
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let _conn = signal.connect_once(move |value| {
        counter_clone.fetch_add(value as usize, Ordering::SeqCst);
    });

    signal.emit(5);
    signal.emit(3); // Should not trigger again

    assert_eq!(counter.load(Ordering::SeqCst), 5);
}

#[test]
fn signal_registry() {
    let registry = SignalRegistry::new();
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let _conn = registry.connect("test_event", move |_| {
        counter_clone.fetch_add(1, Ordering::SeqCst);
    });

    registry.emit("test_event", serde_json::json!({"key": "value"}));
    registry.emit("test_event", serde_json::json!(null));

    assert_eq!(counter.load(Ordering::SeqCst), 2);
}

#[test]
fn webview_signals() {
    let signals = WebViewSignals::new();
    let loaded = Arc::new(AtomicUsize::new(0));
    let loaded_clone = loaded.clone();

    signals.page_loaded.connect(move |_| {
        loaded_clone.fetch_add(1, Ordering::SeqCst);
    });

    signals.page_loaded.emit(());

    assert_eq!(loaded.load(Ordering::SeqCst), 1);
}

#[test]
fn webview_custom_signals() {
    let signals = WebViewSignals::new();
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    signals.on("custom_event", move |data| {
        if let Some(n) = data.get("count").and_then(|v| v.as_u64()) {
            counter_clone.fetch_add(n as usize, Ordering::SeqCst);
        }
    });

    signals.emit_custom("custom_event", serde_json::json!({"count": 42}));

    assert_eq!(counter.load(Ordering::SeqCst), 42);
}

#[test]
fn registry_connect_creates_signal() {
    let registry = SignalRegistry::new();
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let _conn = registry.connect("new_event", move |_| {
        counter_clone.fetch_add(1, Ordering::SeqCst);
    });

    assert!(registry.contains("new_event"));

    registry.emit("new_event", serde_json::json!({}));
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[test]
fn registry_connect_once() {
    let registry = SignalRegistry::new();
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let _conn = registry.connect_once("one_time", move |_| {
        counter_clone.fetch_add(1, Ordering::SeqCst);
    });

    registry.emit("one_time", serde_json::json!(1));
    registry.emit("one_time", serde_json::json!(2));

    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[test]
fn registry_disconnect() {
    let registry = SignalRegistry::new();
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let conn = registry.connect("my_event", move |_| {
        counter_clone.fetch_add(1, Ordering::SeqCst);
    });

    registry.emit("my_event", serde_json::json!({}));
    assert_eq!(counter.load(Ordering::SeqCst), 1);

    assert!(registry.disconnect("my_event", conn));

    registry.emit("my_event", serde_json::json!({}));
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[test]
fn registry_remove_signal() {
    let registry = SignalRegistry::new();

    registry.connect("temp_signal", |_| {});
    assert!(registry.contains("temp_signal"));

    assert!(registry.remove("temp_signal"));
    assert!(!registry.contains("temp_signal"));

    assert!(!registry.remove("non_existent"));
}

#[test]
fn registry_names() {
    let registry = SignalRegistry::new();

    registry.connect("event_a", |_| {});
    registry.connect("event_b", |_| {});

    let names = registry.names();
    assert!(names.contains(&"event_a".to_string()));
    assert!(names.contains(&"event_b".to_string()));
}

// ============================================================================
// ConnectionGuard tests
// ============================================================================

#[test]
fn connection_guard_auto_disconnect_on_drop() {
    let signal = Arc::new(Signal::<i32>::new());
    let counter = Arc::new(AtomicUsize::new(0));

    {
        let c = counter.clone();
        let guard = ConnectionGuard::new(
            signal.clone(),
            signal.connect(move |_| {
                c.fetch_add(1, Ordering::SeqCst);
            }),
        );
        assert!(guard.is_attached());
        signal.emit(1);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
        // guard drops here
    }

    // After drop, handler should be disconnected
    signal.emit(2);
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[test]
fn connection_guard_detach() {
    let signal = Arc::new(Signal::<i32>::new());
    let counter = Arc::new(AtomicUsize::new(0));

    let conn_id = {
        let c = counter.clone();
        let guard = ConnectionGuard::new(
            signal.clone(),
            signal.connect(move |_| {
                c.fetch_add(1, Ordering::SeqCst);
            }),
        );
        signal.emit(1);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
        guard.detach()
    };

    // Detached — guard dropped but handler still active
    signal.emit(2);
    assert_eq!(counter.load(Ordering::SeqCst), 2);

    // Manual cleanup
    signal.disconnect(conn_id);
    signal.emit(3);
    assert_eq!(counter.load(Ordering::SeqCst), 2);
}

#[test]
fn connection_guard_manual_disconnect() {
    let signal = Arc::new(Signal::<i32>::new());
    let counter = Arc::new(AtomicUsize::new(0));

    let c = counter.clone();
    let guard = ConnectionGuard::new(
        signal.clone(),
        signal.connect(move |_| {
            c.fetch_add(1, Ordering::SeqCst);
        }),
    );

    signal.emit(1);
    assert_eq!(counter.load(Ordering::SeqCst), 1);

    guard.disconnect();
    signal.emit(2);
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

// ============================================================================
// ChannelBridge tests
// ============================================================================

#[test]
fn channel_bridge_basic() {
    let (bridge, receiver) = ChannelBridge::new("test_ch");

    bridge
        .emit("page:load", serde_json::json!({"url": "https://example.com"}))
        .unwrap();

    let msg = receiver.recv().unwrap();
    assert_eq!(msg.event, "page:load");
    assert_eq!(msg.data["url"], "https://example.com");
}

#[test]
fn channel_bridge_multiple_messages() {
    let (bridge, receiver) = ChannelBridge::new("multi_ch");

    for i in 0..5u64 {
        bridge
            .emit("tick", serde_json::json!({"i": i}))
            .unwrap();
    }

    for i in 0..5u64 {
        let msg = receiver.recv().unwrap();
        assert_eq!(msg.event, "tick");
        assert_eq!(msg.data["i"], i);
    }
}

#[test]
fn channel_bridge_is_connected() {
    let (bridge, _receiver) = ChannelBridge::new("connected_ch");
    assert!(bridge.is_connected());
    bridge.disconnect().unwrap();
    assert!(!bridge.is_connected());
}

#[test]
fn channel_bridge_emit_after_disconnect_fails() {
    let (bridge, _receiver) = ChannelBridge::new("disc_ch");
    bridge.disconnect().unwrap();

    let result = bridge.emit("event", serde_json::json!(null));
    assert!(result.is_err());
}

#[test]
fn channel_bridge_bounded() {
    let (bridge, receiver) = ChannelBridge::bounded("bounded_ch", 2);

    bridge.emit("e1", serde_json::json!(1)).unwrap();
    bridge.emit("e2", serde_json::json!(2)).unwrap();

    let m1 = receiver.recv().unwrap();
    let m2 = receiver.recv().unwrap();
    assert_eq!(m1.event, "e1");
    assert_eq!(m2.event, "e2");
}

// ============================================================================
// EventBus tests
// ============================================================================

#[test]
fn event_bus_basic_on_emit() {
    let bus = EventBus::new();
    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();

    bus.on("click", move |_| {
        c.fetch_add(1, Ordering::SeqCst);
    });

    bus.emit("click", serde_json::json!(null));
    bus.emit("click", serde_json::json!(null));
    assert_eq!(counter.load(Ordering::SeqCst), 2);
}

#[test]
fn event_bus_once() {
    let bus = EventBus::new();
    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();

    bus.once("init", move |_| {
        c.fetch_add(1, Ordering::SeqCst);
    });

    bus.emit("init", serde_json::json!(null));
    bus.emit("init", serde_json::json!(null));
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[test]
fn event_bus_off() {
    let bus = EventBus::new();
    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();

    let conn = bus.on("resize", move |_| {
        c.fetch_add(1, Ordering::SeqCst);
    });

    bus.emit("resize", serde_json::json!(null));
    assert_eq!(counter.load(Ordering::SeqCst), 1);

    bus.off("resize", conn);
    bus.emit("resize", serde_json::json!(null));
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[test]
fn event_bus_off_all() {
    let bus = EventBus::new();
    let counter = Arc::new(AtomicUsize::new(0));
    let c1 = counter.clone();
    let c2 = counter.clone();

    bus.on("scroll", move |_| {
        c1.fetch_add(1, Ordering::SeqCst);
    });
    bus.on("scroll", move |_| {
        c2.fetch_add(1, Ordering::SeqCst);
    });

    bus.emit("scroll", serde_json::json!(null));
    assert_eq!(counter.load(Ordering::SeqCst), 2);

    bus.off_all("scroll");
    bus.emit("scroll", serde_json::json!(null));
    assert_eq!(counter.load(Ordering::SeqCst), 2);
}

#[test]
fn event_bus_clear() {
    let bus = EventBus::new();
    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();

    bus.on("a", move |_| {
        c.fetch_add(1, Ordering::SeqCst);
    });
    bus.on("b", |_| {});

    assert_eq!(bus.event_count(), 2);

    bus.clear();
    assert_eq!(bus.event_count(), 0);

    bus.emit("a", serde_json::json!(null));
    assert_eq!(counter.load(Ordering::SeqCst), 0);
}

#[test]
fn event_bus_named() {
    let bus = EventBus::named("dcc_bus");
    assert_eq!(bus.name(), Some("dcc_bus"));
}

#[test]
fn event_bus_default_name() {
    let bus = EventBus::new();
    assert!(bus.name().is_none());
}

#[test]
fn event_bus_has_handlers() {
    let bus = EventBus::new();
    assert!(!bus.has_handlers("key"));

    bus.on("key", |_| {});
    assert!(bus.has_handlers("key"));
}

#[test]
fn event_bus_handler_count() {
    let bus = EventBus::new();
    bus.on("ev", |_| {});
    bus.on("ev", |_| {});
    bus.on("other", |_| {});

    assert_eq!(bus.handler_count("ev"), 2);
    assert_eq!(bus.total_handler_count(), 3);
}

#[test]
fn event_bus_event_names() {
    let bus = EventBus::new();
    bus.on("alpha", |_| {});
    bus.on("beta", |_| {});

    let mut names = bus.event_names();
    names.sort();
    assert!(names.contains(&"alpha".to_string()));
    assert!(names.contains(&"beta".to_string()));
}

#[test]
fn event_bus_debug() {
    let bus = EventBus::named("debug_bus");
    bus.on("ev", |_| {});
    let s = format!("{:?}", bus);
    assert!(s.contains("EventBus"));
    assert!(s.contains("debug_bus"));
}

#[test]
fn event_bus_with_channel_bridge() {
    let bus = EventBus::new();
    let (bridge, receiver) = ChannelBridge::new("ch_bridge");
    bus.add_bridge(bridge);

    bus.emit("nav:go", serde_json::json!({"url": "https://example.com"}));

    let msg = receiver.recv_timeout(std::time::Duration::from_millis(200)).unwrap();
    assert_eq!(msg.event, "nav:go");
}

#[test]
fn event_bus_emit_local_skips_bridge() {
    let bus = EventBus::new();
    let (bridge, receiver) = ChannelBridge::new("ch_bridge");
    bus.add_bridge(bridge);

    let local_count = Arc::new(AtomicUsize::new(0));
    let lc = local_count.clone();
    bus.on("nav:go", move |_| {
        lc.fetch_add(1, Ordering::SeqCst);
    });

    bus.emit_local("nav:go", serde_json::json!(null));

    assert_eq!(local_count.load(Ordering::SeqCst), 1);
    // Bridge should not receive
    assert!(receiver.try_recv().is_err());
}

#[test]
fn event_bus_with_filter_middleware() {
    let bus = EventBus::new();
    bus.use_middleware(FilterMiddleware::new().deny_pattern("internal:.*").unwrap());

    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();

    bus.on("internal:secret", move |_| {
        c.fetch_add(1, Ordering::SeqCst);
    });

    let c2 = counter.clone();
    bus.on("public:data", move |_| {
        c2.fetch_add(10, Ordering::SeqCst);
    });

    bus.emit("internal:secret", serde_json::json!(null));
    assert_eq!(counter.load(Ordering::SeqCst), 0);

    bus.emit("public:data", serde_json::json!(null));
    assert_eq!(counter.load(Ordering::SeqCst), 10);
}

#[test]
fn event_bus_with_logging_middleware() {
    let bus = EventBus::new();
    bus.use_middleware(LoggingMiddleware::new(LogLevel::Debug));
    assert_eq!(bus.middleware_count(), 1);

    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();
    bus.on("logged_event", move |_| {
        c.fetch_add(1, Ordering::SeqCst);
    });

    bus.emit("logged_event", serde_json::json!({"msg": "hello"}));
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[test]
fn event_bus_bridge_count() {
    let bus = EventBus::new();
    assert_eq!(bus.bridge_count(), 0);

    let (b1, _r1) = ChannelBridge::new("b1");
    let (b2, _r2) = ChannelBridge::new("b2");
    bus.add_bridge(b1);
    bus.add_bridge(b2);
    assert_eq!(bus.bridge_count(), 2);

    bus.remove_bridge("b1");
    assert_eq!(bus.bridge_count(), 1);
}

#[test]
fn event_bus_callback_bridge() {
    let bus = EventBus::new();
    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();

    bus.add_bridge(CallbackBridge::new("cb", move |_event, _data| {
        c.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }));

    bus.emit("any:event", serde_json::json!(null));
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

// ============================================================================
// Concurrency tests
// ============================================================================

#[test]
fn signal_concurrent_emit() {
    use std::thread;

    let signal = Arc::new(Signal::<u64>::new());
    let counter = Arc::new(AtomicUsize::new(0));

    let num_threads = 8;
    let emits_per_thread = 50;

    let c = counter.clone();
    let _conn = signal.connect(move |_| {
        c.fetch_add(1, Ordering::SeqCst);
    });

    let mut handles = vec![];
    for _ in 0..num_threads {
        let sig = signal.clone();
        handles.push(thread::spawn(move || {
            for i in 0..emits_per_thread {
                sig.emit(i as u64);
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(
        counter.load(Ordering::SeqCst),
        num_threads * emits_per_thread
    );
}

#[test]
fn event_bus_concurrent_emit() {
    use std::thread;

    let bus = Arc::new(EventBus::new());
    let counter = Arc::new(AtomicUsize::new(0));

    let c = counter.clone();
    bus.on("ping", move |_| {
        c.fetch_add(1, Ordering::SeqCst);
    });

    let num_threads = 5;
    let emits_per_thread = 20;

    let mut handles = vec![];
    for _ in 0..num_threads {
        let b = bus.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..emits_per_thread {
                b.emit("ping", serde_json::json!(null));
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(
        counter.load(Ordering::SeqCst),
        num_threads * emits_per_thread
    );
}

#[test]
fn registry_emit_to_nonexistent_signal() {
    let registry = SignalRegistry::new();
    // Emitting to a non-existent signal should not panic
    registry.emit("does_not_exist", serde_json::json!(null));
}

#[test]
fn webview_signals_resized() {
    let signals = WebViewSignals::new();
    let result = Arc::new(std::sync::Mutex::new(None));
    let r = result.clone();

    signals.resized.connect(move |(w, h)| {
        *r.lock().unwrap() = Some((w, h));
    });

    signals.resized.emit((1920u32, 1080u32));

    let val = result.lock().unwrap();
    assert_eq!(*val, Some((1920u32, 1080u32)));
}

#[test]
fn webview_signals_moved() {
    let signals = WebViewSignals::new();
    let result = Arc::new(std::sync::Mutex::new(None));
    let r = result.clone();

    signals.moved.connect(move |(x, y)| {
        *r.lock().unwrap() = Some((x, y));
    });

    signals.moved.emit((100i32, 200i32));

    let val = result.lock().unwrap();
    assert_eq!(*val, Some((100i32, 200i32)));
}

#[test]
fn webview_signals_lifecycle() {
    let signals = WebViewSignals::new();
    let closed = Arc::new(AtomicUsize::new(0));
    let focused = Arc::new(AtomicUsize::new(0));
    let minimized = Arc::new(AtomicUsize::new(0));

    let c = closed.clone();
    signals.closed.connect(move |_| { c.fetch_add(1, Ordering::SeqCst); });

    let f = focused.clone();
    signals.focused.connect(move |_| { f.fetch_add(1, Ordering::SeqCst); });

    let m = minimized.clone();
    signals.minimized.connect(move |_| { m.fetch_add(1, Ordering::SeqCst); });

    signals.closed.emit(());
    signals.focused.emit(());
    signals.minimized.emit(());

    assert_eq!(closed.load(Ordering::SeqCst), 1);
    assert_eq!(focused.load(Ordering::SeqCst), 1);
    assert_eq!(minimized.load(Ordering::SeqCst), 1);
}

// ============================================================================
// R15 Extensions
// ============================================================================

#[test]
fn signal_emit_no_handlers_no_panic() {
    let signal = Signal::<u32>::new();
    signal.emit(42);
}

#[test]
fn signal_single_connect_and_emit() {
    let signal = Signal::<u32>::new();
    let value = Arc::new(std::sync::Mutex::new(0u32));
    let v = value.clone();
    let _conn = signal.connect(move |n| { *v.lock().unwrap() = n; });
    signal.emit(99);
    assert_eq!(*value.lock().unwrap(), 99);
}

#[test]
fn signal_two_connects_both_called() {
    let signal = Signal::<u32>::new();
    let count = Arc::new(AtomicUsize::new(0));
    let c1 = count.clone();
    let c2 = count.clone();
    let _conn1 = signal.connect(move |_| { c1.fetch_add(1, Ordering::SeqCst); });
    let _conn2 = signal.connect(move |_| { c2.fetch_add(1, Ordering::SeqCst); });
    signal.emit(0);
    assert_eq!(count.load(Ordering::SeqCst), 2);
}

#[test]
fn event_bus_emit_many_events_no_panic() {
    let bus = EventBus::new();
    bus.on("click", |_| {});
    for i in 0..20 {
        bus.emit("click", serde_json::json!(i));
    }
}

#[test]
fn event_bus_multiple_handlers_same_event_r15() {
    let bus = EventBus::new();
    let count = Arc::new(AtomicUsize::new(0));

    for _ in 0..5 {
        let c = count.clone();
        bus.on("multi_r15", move |_| { c.fetch_add(1, Ordering::SeqCst); });
    }

    bus.emit("multi_r15", serde_json::json!(null));
    assert_eq!(count.load(Ordering::SeqCst), 5);
}

#[test]
fn webview_signals_custom_event_fires() {
    let signals = WebViewSignals::new();
    let count = Arc::new(AtomicUsize::new(0));
    let c = count.clone();
    signals.on("scene_loaded_r15", move |_| { c.fetch_add(1, Ordering::SeqCst); });
    signals.emit_custom("scene_loaded_r15", serde_json::json!({}));
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[test]
fn event_bus_clear_removes_all_handlers_r15() {
    let bus = EventBus::new();
    bus.on("a_r15", |_| {});
    bus.on("b_r15", |_| {});
    bus.on("c_r15", |_| {});
    assert_eq!(bus.event_count(), 3);
    bus.clear();
    assert_eq!(bus.event_count(), 0);
}


