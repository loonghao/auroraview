//! Tests for IPC router

use auroraview_desktop::{IpcMessage, IpcResponse, IpcRouter};
use serde_json::json;

#[test]
fn test_ipc_router_new() {
    let router = IpcRouter::new();
    // Should create without panic
    drop(router);
}

#[test]
fn test_ipc_router_register_handler() {
    let router = IpcRouter::new();

    router.register("test.echo", |params| params);

    // Handler should be registered
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

    // Unregister non-existent
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
