//! Tests for IPC router

use auroraview_desktop::{IpcMessage, IpcResponse, IpcRouter};
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

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
