//! Tests for IPC router

use auroraview_dcc::{IpcMessage, IpcResponse, IpcRouter};
use serde_json::json;

#[test]
fn test_ipc_router_new() {
    let router = IpcRouter::new();
    assert!(!router.has_handler("test"));
}

#[test]
fn test_ipc_router_register_and_check() {
    let router = IpcRouter::new();
    
    router.register("test.echo", |params| params);
    
    assert!(router.has_handler("test.echo"));
    assert!(!router.has_handler("test.other"));
}

#[test]
fn test_ipc_router_unregister() {
    let router = IpcRouter::new();
    
    router.register("test.method", |_| json!({}));
    assert!(router.has_handler("test.method"));
    
    let removed = router.unregister("test.method");
    assert!(removed);
    assert!(!router.has_handler("test.method"));
    
    // Unregister non-existent
    let removed = router.unregister("nonexistent");
    assert!(!removed);
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
    assert!(response.error.is_some());
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
    
    let error = response.error.unwrap();
    assert_eq!(error.name, "TestError");
    assert_eq!(error.message, "Something went wrong");
}
