//! Tests for IPC router

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use auroraview_desktop::{IpcMessage, IpcResponse, IpcRouter};
use serde_json::json;

#[test]
fn test_ipc_router_new() {
    let router = IpcRouter::new();
    drop(router);
}

#[test]
fn test_ipc_router_register_handler() {
    let router = IpcRouter::new();
    router.register("test.echo", |params| params);
    assert!(router.has_handler("test.echo"));
}

#[test]
fn test_ipc_router_handle_call() {
    let router = IpcRouter::new();
    router.register("test.echo", |params| params);

    let message = json!({
        "type": "call",
        "id": "1",
        "method": "test.echo",
        "params": {"message": "hello"}
    });

    let result = router.handle(&message.to_string());
    assert!(result.is_some());

    let response: IpcResponse = serde_json::from_str(&result.unwrap()).unwrap();
    assert!(response.ok);
    assert_eq!(response.id, "1");
}

#[test]
fn test_ipc_router_method_not_found() {
    let router = IpcRouter::new();

    let message = json!({
        "type": "call",
        "id": "2",
        "method": "nonexistent.method",
        "params": {}
    });

    let result = router.handle(&message.to_string());
    assert!(result.is_some());

    let response: IpcResponse = serde_json::from_str(&result.unwrap()).unwrap();
    assert!(!response.ok);
}

#[test]
fn test_ipc_message_parsing() {
    let json_str = r#"{"type":"call","id":"123","method":"test","params":null}"#;
    let msg: IpcMessage = serde_json::from_str(json_str).unwrap();

    assert_eq!(msg.msg_type, "call");
    assert_eq!(msg.id, Some("123".to_string()));
    assert_eq!(msg.method, Some("test".to_string()));
}

#[test]
fn test_ipc_response_ok() {
    let response = IpcResponse::ok("1".to_string(), json!({"result": "success"}));
    assert!(response.ok);
    assert_eq!(response.id, "1");
    assert!(response.result.is_some());
    assert!(response.error.is_none());
}

#[test]
fn test_ipc_response_err() {
    let response = IpcResponse::err("2".to_string(), "TestError", "Something went wrong");
    assert!(!response.ok);
    assert_eq!(response.id, "2");
    assert!(response.result.is_none());
    assert!(response.error.is_some());
}

#[test]
fn test_ipc_router_has_handler() {
    let router = IpcRouter::new();
    assert!(!router.has_handler("test.method"));
    router.register("test.method", |_| json!(null));
    assert!(router.has_handler("test.method"));
}

#[test]
fn test_ipc_router_unregister() {
    let router = IpcRouter::new();
    router.register("test.method", |_| json!(null));
    assert!(router.has_handler("test.method"));

    let removed = router.unregister("test.method");
    assert!(removed);
    assert!(!router.has_handler("test.method"));

    let removed = router.unregister("test.method");
    assert!(!removed);
}

#[test]
fn test_ipc_router_methods() {
    let router = IpcRouter::new();
    router.register("api.one", |_| json!(null));
    router.register("api.two", |_| json!(null));
    router.register("api.three", |_| json!(null));

    let methods = router.methods();
    assert_eq!(methods.len(), 3);
    assert!(methods.contains(&"api.one".to_string()));
    assert!(methods.contains(&"api.two".to_string()));
    assert!(methods.contains(&"api.three".to_string()));
}

// ============================================================================
// New Tests
// ============================================================================

#[test]
fn test_ipc_router_default() {
    let router = IpcRouter::default();
    assert!(!router.has_handler("any"));
    assert_eq!(router.methods().len(), 0);
}

#[test]
fn test_ipc_router_invalid_json_returns_none() {
    let router = IpcRouter::new();
    let result = router.handle("not valid json {{{{");
    assert!(result.is_none());
}

#[test]
fn test_ipc_router_empty_string_returns_none() {
    let router = IpcRouter::new();
    let result = router.handle("");
    assert!(result.is_none());
}

#[test]
fn test_ipc_router_unknown_message_type_returns_none() {
    let router = IpcRouter::new();
    let message = json!({
        "type": "unknown_type",
        "id": "x",
        "method": "any"
    });
    let result = router.handle(&message.to_string());
    assert!(result.is_none());
}

#[test]
fn test_ipc_router_event_type_returns_none() {
    let router = IpcRouter::new();
    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();

    router.on("page:ready", move |_| {
        c.fetch_add(1, Ordering::SeqCst);
    });

    let message = json!({
        "type": "event",
        "event": "page:ready",
        "detail": {"url": "https://example.com"}
    });

    let result = router.handle(&message.to_string());
    // event type doesn't return response
    assert!(result.is_none());
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[test]
fn test_ipc_router_event_multiple_listeners() {
    let router = IpcRouter::new();
    let counter = Arc::new(AtomicUsize::new(0));

    for _ in 0..3 {
        let c = counter.clone();
        router.on("data:update", move |_| {
            c.fetch_add(1, Ordering::SeqCst);
        });
    }

    let message = json!({
        "type": "event",
        "event": "data:update",
        "detail": null
    });

    router.handle(&message.to_string());
    assert_eq!(counter.load(Ordering::SeqCst), 3);
}

#[test]
fn test_ipc_router_call_with_array_params() {
    let router = IpcRouter::new();
    router.register("math.sum", |params| {
        if let Some(arr) = params.as_array() {
            let sum: i64 = arr.iter().filter_map(|v| v.as_i64()).sum();
            json!(sum)
        } else {
            json!(0)
        }
    });

    let message = json!({
        "type": "call",
        "id": "sum_1",
        "method": "math.sum",
        "params": [1, 2, 3, 4, 5]
    });

    let raw = router.handle(&message.to_string()).unwrap();
    let resp: IpcResponse = serde_json::from_str(&raw).unwrap();
    assert!(resp.ok);
    assert_eq!(resp.result.unwrap(), json!(15));
}

#[test]
fn test_ipc_router_call_returns_complex_result() {
    let router = IpcRouter::new();
    router.register("user.info", |_| {
        json!({
            "name": "Alice",
            "role": "admin",
            "active": true
        })
    });

    let message = json!({
        "type": "call",
        "id": "info_1",
        "method": "user.info",
        "params": {}
    });

    let raw = router.handle(&message.to_string()).unwrap();
    let resp: IpcResponse = serde_json::from_str(&raw).unwrap();
    assert!(resp.ok);
    let result = resp.result.unwrap();
    assert_eq!(result["name"], "Alice");
    assert_eq!(result["role"], "admin");
    assert_eq!(result["active"], true);
}

#[test]
fn test_ipc_router_call_missing_id_returns_none() {
    let router = IpcRouter::new();
    router.register("test.echo", |params| params);

    // No "id" field
    let message = json!({
        "type": "call",
        "method": "test.echo",
        "params": {}
    });

    let result = router.handle(&message.to_string());
    assert!(result.is_none());
}

#[test]
fn test_ipc_router_call_missing_method_returns_none() {
    let router = IpcRouter::new();

    let message = json!({
        "type": "call",
        "id": "x"
        // no "method" field
    });

    let result = router.handle(&message.to_string());
    assert!(result.is_none());
}

#[test]
fn test_ipc_router_invoke_type() {
    let router = IpcRouter::new();
    router.register("fs:read_file", |args| {
        json!({"content": "file contents", "path": args["path"]})
    });

    let message = json!({
        "type": "invoke",
        "id": "inv_1",
        "cmd": "fs:read_file",
        "args": {"path": "/tmp/test.txt"}
    });

    let raw = router.handle(&message.to_string()).unwrap();
    let resp: IpcResponse = serde_json::from_str(&raw).unwrap();
    assert!(resp.ok);
    assert_eq!(resp.result.unwrap()["content"], "file contents");
}

#[test]
fn test_ipc_router_invoke_not_found() {
    let router = IpcRouter::new();

    let message = json!({
        "type": "invoke",
        "id": "inv_2",
        "cmd": "plugin:nonexistent",
        "args": {}
    });

    let raw = router.handle(&message.to_string()).unwrap();
    let resp: IpcResponse = serde_json::from_str(&raw).unwrap();
    assert!(!resp.ok);
}

#[test]
fn test_ipc_response_serde_roundtrip() {
    let original = IpcResponse::ok("42".to_string(), json!({"data": "value"}));
    let json_str = serde_json::to_string(&original).unwrap();
    let restored: IpcResponse = serde_json::from_str(&json_str).unwrap();

    assert_eq!(restored.id, "42");
    assert!(restored.ok);
    assert_eq!(restored.result.unwrap()["data"], "value");
}

#[test]
fn test_ipc_response_err_has_error_info() {
    let resp = IpcResponse::err("99".to_string(), "ValueError", "invalid input");
    let error = resp.error.unwrap();
    assert_eq!(error.name, "ValueError");
    assert_eq!(error.message, "invalid input");
    assert!(error.code.is_none());
}

#[test]
fn test_ipc_router_concurrent_register_and_call() {
    use std::thread;

    let router = Arc::new(IpcRouter::new());
    let counter = Arc::new(AtomicUsize::new(0));

    // Register handlers concurrently
    let mut handles = vec![];
    for i in 0..5usize {
        let r = router.clone();
        let method = format!("concurrent.method{}", i);
        handles.push(thread::spawn(move || {
            r.register(&method, |params| params);
        }));
    }
    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(router.methods().len(), 5);

    // Call all handlers concurrently
    let mut handles = vec![];
    for i in 0..5usize {
        let r = router.clone();
        let c = counter.clone();
        handles.push(thread::spawn(move || {
            let message = json!({
                "type": "call",
                "id": format!("cid_{}", i),
                "method": format!("concurrent.method{}", i),
                "params": {"n": i}
            });
            let result = r.handle(&message.to_string());
            if result.is_some() {
                c.fetch_add(1, Ordering::SeqCst);
            }
        }));
    }
    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(counter.load(Ordering::SeqCst), 5);
}

#[test]
fn test_ipc_message_clone() {
    let msg = IpcMessage {
        msg_type: "call".to_string(),
        event: None,
        method: Some("api.test".to_string()),
        cmd: None,
        params: Some(json!({"x": 1})),
        args: None,
        id: Some("id_clone".to_string()),
        detail: None,
    };

    let cloned = msg.clone();
    assert_eq!(cloned.msg_type, "call");
    assert_eq!(cloned.method, Some("api.test".to_string()));
    assert_eq!(cloned.id, Some("id_clone".to_string()));
}

#[test]
fn test_ipc_router_overwrite_handler() {
    let router = IpcRouter::new();

    // Register initial handler that returns "v1"
    router.register("versioned.method", |_| json!("v1"));

    let msg1 = json!({"type":"call","id":"1","method":"versioned.method","params":null});
    let r1 = router.handle(&msg1.to_string()).unwrap();
    let resp1: IpcResponse = serde_json::from_str(&r1).unwrap();
    assert_eq!(resp1.result.unwrap(), json!("v1"));

    // Overwrite with handler that returns "v2"
    router.register("versioned.method", |_| json!("v2"));

    let msg2 = json!({"type":"call","id":"2","method":"versioned.method","params":null});
    let r2 = router.handle(&msg2.to_string()).unwrap();
    let resp2: IpcResponse = serde_json::from_str(&r2).unwrap();
    assert_eq!(resp2.result.unwrap(), json!("v2"));
}

// ============================================================================
// Invoke extended scenarios
// ============================================================================

#[test]
fn test_ipc_router_invoke_empty_args() {
    let router = IpcRouter::new();
    router.register("plugin:noop", |_| json!({"done": true}));

    let message = json!({
        "type": "invoke",
        "id": "inv_empty",
        "cmd": "plugin:noop",
        "args": {}
    });

    let raw = router.handle(&message.to_string()).unwrap();
    let resp: IpcResponse = serde_json::from_str(&raw).unwrap();
    assert!(resp.ok);
    assert_eq!(resp.result.unwrap()["done"], true);
}

#[test]
fn test_ipc_router_invoke_missing_id_returns_none() {
    let router = IpcRouter::new();
    router.register("plugin:cmd", |_| json!(null));

    let message = json!({
        "type": "invoke",
        "cmd": "plugin:cmd",
        "args": {}
        // no "id"
    });

    let result = router.handle(&message.to_string());
    assert!(result.is_none());
}

#[test]
fn test_ipc_router_invoke_missing_cmd_returns_none() {
    let router = IpcRouter::new();

    let message = json!({
        "type": "invoke",
        "id": "inv_x"
        // no "cmd"
    });

    let result = router.handle(&message.to_string());
    assert!(result.is_none());
}

#[test]
fn test_ipc_router_invoke_complex_return() {
    let router = IpcRouter::new();
    router.register("scene:info", |_| {
        json!({
            "scene": "production",
            "objects": [{"name": "mesh1"}, {"name": "camera1"}],
            "frame_range": [1, 240]
        })
    });

    let msg = json!({"type":"invoke","id":"sc1","cmd":"scene:info","args":{}});
    let raw = router.handle(&msg.to_string()).unwrap();
    let resp: IpcResponse = serde_json::from_str(&raw).unwrap();
    assert!(resp.ok);
    let result = resp.result.unwrap();
    assert_eq!(result["scene"], "production");
    assert_eq!(result["objects"].as_array().unwrap().len(), 2);
}

// ============================================================================
// DCC-specific workflow scenarios (Desktop context)
// ============================================================================

#[test]
fn test_ipc_desktop_export_workflow() {
    use std::sync::{Arc, Mutex};

    let router = IpcRouter::new();
    let log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

    let log_start = log.clone();
    router.register("export.start", move |params| {
        let path = params["path"].as_str().unwrap_or("").to_string();
        log_start.lock().unwrap().push(format!("start:{}", path));
        json!({"status": "started"})
    });

    let log_finish = log.clone();
    router.register("export.finish", move |_| {
        log_finish.lock().unwrap().push("finish".to_string());
        json!({"status": "done"})
    });

    // 1. Start export
    let start_msg = json!({"type":"call","id":"e1","method":"export.start","params":{"path":"/tmp/out.fbx"}});
    let r1 = router.handle(&start_msg.to_string()).unwrap();
    let resp1: IpcResponse = serde_json::from_str(&r1).unwrap();
    assert_eq!(resp1.result.unwrap()["status"], "started");

    // 2. Finish export
    let finish_msg = json!({"type":"call","id":"e2","method":"export.finish","params":{}});
    let r2 = router.handle(&finish_msg.to_string()).unwrap();
    let resp2: IpcResponse = serde_json::from_str(&r2).unwrap();
    assert_eq!(resp2.result.unwrap()["status"], "done");

    let entries = log.lock().unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0], "start:/tmp/out.fbx");
    assert_eq!(entries[1], "finish");
}

#[test]
fn test_ipc_desktop_scene_state_machine() {
    use std::sync::{Arc, Mutex};

    let router = IpcRouter::new();
    let state: Arc<Mutex<String>> = Arc::new(Mutex::new("idle".to_string()));

    let s = state.clone();
    router.register("scene.load", move |params| {
        let name = params["name"].as_str().unwrap_or("unknown").to_string();
        *s.lock().unwrap() = format!("loaded:{}", name);
        json!({"loaded": name})
    });

    let s = state.clone();
    router.register("scene.close", move |_| {
        *s.lock().unwrap() = "idle".to_string();
        json!({"closed": true})
    });

    let s = state.clone();
    router.register("scene.status", move |_| {
        let current = s.lock().unwrap().clone();
        json!({"state": current})
    });

    // Load
    let load_msg = json!({"type":"call","id":"s1","method":"scene.load","params":{"name":"hero_rig"}});
    router.handle(&load_msg.to_string()).unwrap();

    // Check status
    let status_msg = json!({"type":"call","id":"s2","method":"scene.status","params":{}});
    let raw = router.handle(&status_msg.to_string()).unwrap();
    let resp: IpcResponse = serde_json::from_str(&raw).unwrap();
    assert_eq!(resp.result.unwrap()["state"], "loaded:hero_rig");

    // Close
    let close_msg = json!({"type":"call","id":"s3","method":"scene.close","params":{}});
    router.handle(&close_msg.to_string()).unwrap();
    let raw = router.handle(&status_msg.to_string()).unwrap();
    let resp: IpcResponse = serde_json::from_str(&raw).unwrap();
    assert_eq!(resp.result.unwrap()["state"], "idle");
}

// ============================================================================
// IpcMessage edge cases
// ============================================================================

#[test]
fn test_ipc_message_all_fields() {
    let json_str = r#"{"type":"invoke","id":"x1","method":"api.test","cmd":"fs:read","params":{"a":1},"args":{"b":2},"event":"ready","detail":{"c":3}}"#;
    let msg: IpcMessage = serde_json::from_str(json_str).unwrap();

    assert_eq!(msg.msg_type, "invoke");
    assert_eq!(msg.id.as_deref(), Some("x1"));
    assert_eq!(msg.method.as_deref(), Some("api.test"));
    assert_eq!(msg.cmd.as_deref(), Some("fs:read"));
    assert!(msg.params.is_some());
    assert!(msg.args.is_some());
    assert_eq!(msg.event.as_deref(), Some("ready"));
    assert!(msg.detail.is_some());
}

#[test]
fn test_ipc_message_event_type() {
    let json_str = r#"{"type":"event","event":"ui.ready","detail":{"version":"1.0"}}"#;
    let msg: IpcMessage = serde_json::from_str(json_str).unwrap();

    assert_eq!(msg.msg_type, "event");
    assert_eq!(msg.event.as_deref(), Some("ui.ready"));
    assert!(msg.detail.is_some());
    assert_eq!(msg.detail.unwrap()["version"], "1.0");
}

#[test]
fn test_ipc_message_serde_roundtrip() {
    let original = IpcMessage {
        msg_type: "call".to_string(),
        event: None,
        method: Some("api.echo".to_string()),
        cmd: None,
        params: Some(json!({"x": 42})),
        args: None,
        id: Some("roundtrip_id".to_string()),
        detail: None,
    };

    let json_str = serde_json::to_string(&original).unwrap();
    let restored: IpcMessage = serde_json::from_str(&json_str).unwrap();

    assert_eq!(restored.msg_type, "call");
    assert_eq!(restored.method.as_deref(), Some("api.echo"));
    assert_eq!(restored.id.as_deref(), Some("roundtrip_id"));
    assert_eq!(restored.params.unwrap()["x"], 42);
}

// ============================================================================
// IpcResponse edge cases
// ============================================================================

#[test]
fn test_ipc_response_ok_with_array_result() {
    let resp = IpcResponse::ok("arr1".to_string(), json!([1, 2, 3]));
    assert!(resp.ok);
    let result = resp.result.unwrap();
    let arr = result.as_array().unwrap();
    assert_eq!(arr.len(), 3);
    assert_eq!(arr[0], 1);
}

#[test]
fn test_ipc_response_ok_with_bool_result() {
    let resp = IpcResponse::ok("bool1".to_string(), json!(true));
    assert!(resp.ok);
    assert_eq!(resp.result.unwrap(), json!(true));
}

#[test]
fn test_ipc_response_ok_with_number_result() {
    let pi = std::f64::consts::PI;
    let resp = IpcResponse::ok("num1".to_string(), json!(pi));
    assert!(resp.ok);
    let val = resp.result.unwrap();
    assert!((val.as_f64().unwrap() - pi).abs() < 0.001);
}

#[test]
fn test_ipc_response_err_empty_message() {
    let resp = IpcResponse::err("e1".to_string(), "Empty", "");
    assert!(!resp.ok);
    let err = resp.error.unwrap();
    assert_eq!(err.name, "Empty");
    assert!(err.message.is_empty());
}

#[test]
fn test_ipc_response_err_serde_roundtrip() {
    let original = IpcResponse::err("e2".to_string(), "TypeError", "bad type");
    let json_str = serde_json::to_string(&original).unwrap();
    let restored: IpcResponse = serde_json::from_str(&json_str).unwrap();

    assert!(!restored.ok);
    assert_eq!(restored.id, "e2");
    let err = restored.error.unwrap();
    assert_eq!(err.name, "TypeError");
    assert_eq!(err.message, "bad type");
}

// ============================================================================
// Concurrent stress test
// ============================================================================

#[test]
fn test_ipc_router_high_concurrency_calls() {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;
    use std::thread;

    let router = Arc::new(IpcRouter::new());
    let call_count = Arc::new(AtomicU64::new(0));

    // Register a single handler
    let cnt = call_count.clone();
    router.register("stress.add", move |params| {
        let n = params["n"].as_u64().unwrap_or(0);
        cnt.fetch_add(n, Ordering::Relaxed);
        json!({"ok": true})
    });

    let handles: Vec<_> = (0..20)
        .map(|i| {
            let r = router.clone();
            thread::spawn(move || {
                for _ in 0..5 {
                    let msg = json!({
                        "type": "call",
                        "id": format!("s{}_{}", i, 0),
                        "method": "stress.add",
                        "params": {"n": 1}
                    });
                    r.handle(&msg.to_string()).unwrap();
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    // 20 threads x 5 calls x n=1 = 100
    assert_eq!(call_count.load(Ordering::Relaxed), 100);
}

#[test]
fn test_ipc_router_concurrent_event_fire() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::thread;

    let router = Arc::new(IpcRouter::new());
    let fired = Arc::new(AtomicUsize::new(0));

    for _ in 0..4 {
        let f = fired.clone();
        router.on("desktop.tick", move |_| {
            f.fetch_add(1, Ordering::SeqCst);
        });
    }

    let handles: Vec<_> = (0..5)
        .map(|_| {
            let r = router.clone();
            thread::spawn(move || {
                let msg = json!({"type":"event","event":"desktop.tick","detail":null});
                r.handle(&msg.to_string());
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    // 4 listeners x 5 invocations = 20
    assert_eq!(fired.load(Ordering::SeqCst), 20);
}

// ============================================================================
// Handler returning error payload
// ============================================================================

#[test]
fn test_ipc_handler_returns_error_json() {
    let router = IpcRouter::new();
    router.register("api.failing", |_| {
        // Handler returns a result with error field (app-level, not protocol-level)
        json!({"error": "resource not found", "code": 404})
    });

    let msg = json!({"type":"call","id":"f1","method":"api.failing","params":{}});
    let raw = router.handle(&msg.to_string()).unwrap();
    let resp: IpcResponse = serde_json::from_str(&raw).unwrap();

    // Protocol-level: ok = true (handler ran successfully)
    assert!(resp.ok);
    let result = resp.result.unwrap();
    assert_eq!(result["code"], 404);
    assert_eq!(result["error"], "resource not found");
}

// ============================================================================
// Call ID uniqueness and echo
// ============================================================================

#[test]
fn test_ipc_router_response_id_echoes_request() {
    let router = IpcRouter::new();
    router.register("id.echo", |p| p);

    for id_val in &["id-001", "uuid-xxxxxxxx-yyyyyy", "1234567890", "🎯"] {
        let msg = json!({"type":"call","id": id_val,"method":"id.echo","params":{}});
        let raw = router.handle(&msg.to_string()).unwrap();
        let resp: IpcResponse = serde_json::from_str(&raw).unwrap();
        assert_eq!(&resp.id, id_val, "response id should match request id");
    }
}

// ============================================================================
// Unregister then re-register
// ============================================================================

#[test]
fn test_ipc_router_unregister_then_reregister() {
    let router = IpcRouter::new();
    router.register("temp.method", |_| json!({"version": 1}));
    router.unregister("temp.method");

    // After unregister, call returns error
    let msg1 = json!({"type":"call","id":"u1","method":"temp.method","params":{}});
    let r1 = router.handle(&msg1.to_string()).unwrap();
    let resp1: IpcResponse = serde_json::from_str(&r1).unwrap();
    assert!(!resp1.ok);

    // Re-register
    router.register("temp.method", |_| json!({"version": 2}));
    let msg2 = json!({"type":"call","id":"u2","method":"temp.method","params":{}});
    let r2 = router.handle(&msg2.to_string()).unwrap();
    let resp2: IpcResponse = serde_json::from_str(&r2).unwrap();
    assert!(resp2.ok);
    assert_eq!(resp2.result.unwrap()["version"], 2);
}
