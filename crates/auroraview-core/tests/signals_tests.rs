//! Signal-slot system tests

use auroraview_core::signals::{Signal, SignalRegistry, WebViewSignals};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[test]
fn test_signal_connect_emit() {
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
fn test_signal_disconnect() {
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
    assert_eq!(counter.load(Ordering::SeqCst), 5); // Still 5, handler disconnected
}

#[test]
fn test_signal_multiple_handlers() {
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
fn test_connect_once() {
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
fn test_signal_registry() {
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
fn test_webview_signals() {
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
fn test_webview_custom_signals() {
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
fn test_registry_connect_creates_signal() {
    let registry = SignalRegistry::new();
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    // connect() creates the signal automatically
    let _conn = registry.connect("new_event", move |_| {
        counter_clone.fetch_add(1, Ordering::SeqCst);
    });

    // Signal now exists
    assert!(registry.contains("new_event"));

    registry.emit("new_event", serde_json::json!({}));
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[test]
fn test_registry_connect_once() {
    let registry = SignalRegistry::new();
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let _conn = registry.connect_once("one_time", move |_| {
        counter_clone.fetch_add(1, Ordering::SeqCst);
    });

    registry.emit("one_time", serde_json::json!(1));
    registry.emit("one_time", serde_json::json!(2)); // Won't trigger

    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[test]
fn test_registry_disconnect() {
    let registry = SignalRegistry::new();
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let conn = registry.connect("my_event", move |_| {
        counter_clone.fetch_add(1, Ordering::SeqCst);
    });

    registry.emit("my_event", serde_json::json!({}));
    assert_eq!(counter.load(Ordering::SeqCst), 1);

    // Disconnect
    assert!(registry.disconnect("my_event", conn));

    registry.emit("my_event", serde_json::json!({}));
    assert_eq!(counter.load(Ordering::SeqCst), 1); // Still 1
}

#[test]
fn test_registry_remove_signal() {
    let registry = SignalRegistry::new();

    registry.connect("temp_signal", |_| {});
    assert!(registry.contains("temp_signal"));

    assert!(registry.remove("temp_signal"));
    assert!(!registry.contains("temp_signal"));

    // Remove non-existent signal returns false
    assert!(!registry.remove("non_existent"));
}

#[test]
fn test_registry_names() {
    let registry = SignalRegistry::new();

    registry.connect("event_a", |_| {});
    registry.connect("event_b", |_| {});

    let names = registry.names();
    assert!(names.contains(&"event_a".to_string()));
    assert!(names.contains(&"event_b".to_string()));
}
