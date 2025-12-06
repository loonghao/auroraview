//! Integration tests for Protocol functionality
//!
//! These tests verify the complete protocol registration and handling workflow.

use _core::webview::protocol::{ProtocolHandler, ProtocolResponse};
use rstest::*;

/// Fixture: Create a new ProtocolHandler
#[fixture]
fn protocol_handler() -> ProtocolHandler {
    ProtocolHandler::new()
}

#[rstest]
fn test_register_and_handle_protocol(protocol_handler: ProtocolHandler) {
    protocol_handler.register("dcc", |uri| {
        if uri.contains("test") {
            Some(ProtocolResponse::text("Test response"))
        } else {
            None
        }
    });

    let response = protocol_handler.handle("dcc://test/resource");
    assert!(
        response.is_some(),
        "Registered protocol should return response"
    );

    let resp = response.unwrap();
    assert_eq!(resp.status, 200);
    assert_eq!(resp.mime_type, "text/plain");
    assert_eq!(String::from_utf8(resp.data).unwrap(), "Test response");
}

#[rstest]
fn test_unregister_protocol(protocol_handler: ProtocolHandler) {
    protocol_handler.register("test", |_| Some(ProtocolResponse::text("data")));
    assert!(
        protocol_handler.handle("test://resource").is_some(),
        "Protocol should be registered"
    );

    protocol_handler.unregister("test");
    assert!(
        protocol_handler.handle("test://resource").is_none(),
        "Protocol should be unregistered"
    );
}

#[rstest]
fn test_clear_all_protocols(protocol_handler: ProtocolHandler) {
    protocol_handler.register("proto1", |_| Some(ProtocolResponse::text("data1")));
    protocol_handler.register("proto2", |_| Some(ProtocolResponse::text("data2")));

    assert!(protocol_handler.handle("proto1://test").is_some());
    assert!(protocol_handler.handle("proto2://test").is_some());

    protocol_handler.clear();

    assert!(
        protocol_handler.handle("proto1://test").is_none(),
        "All protocols should be cleared"
    );
    assert!(
        protocol_handler.handle("proto2://test").is_none(),
        "All protocols should be cleared"
    );
}

#[rstest]
fn test_multiple_protocol_registrations(protocol_handler: ProtocolHandler) {
    protocol_handler.register("api", |uri| {
        if uri.contains("users") {
            Some(ProtocolResponse::json(&serde_json::json!({"users": []})))
        } else {
            None
        }
    });

    protocol_handler.register("assets", |uri| {
        if uri.contains("image") {
            Some(ProtocolResponse::html("<img src='test.png'/>"))
        } else {
            None
        }
    });

    // Test first protocol
    let response1 = protocol_handler.handle("api://users");
    assert!(response1.is_some());
    assert_eq!(response1.unwrap().mime_type, "application/json");

    // Test second protocol
    let response2 = protocol_handler.handle("assets://image.png");
    assert!(response2.is_some());
    assert_eq!(response2.unwrap().mime_type, "text/html");
}

#[rstest]
fn test_protocol_response_types() {
    // Test different response types
    let text_resp = ProtocolResponse::text("Hello");
    assert_eq!(text_resp.mime_type, "text/plain");
    assert_eq!(text_resp.status, 200);

    let html_resp = ProtocolResponse::html("<h1>Title</h1>");
    assert_eq!(html_resp.mime_type, "text/html");

    let json_resp = ProtocolResponse::json(&serde_json::json!({"key": "value"}));
    assert_eq!(json_resp.mime_type, "application/json");

    let not_found = ProtocolResponse::not_found();
    assert_eq!(not_found.status, 404);
}

#[rstest]
fn test_protocol_handler_concurrent_access(protocol_handler: ProtocolHandler) {
    use std::sync::Arc;
    use std::thread;

    let handler = Arc::new(protocol_handler);
    handler.register("concurrent", |_| Some(ProtocolResponse::text("OK")));

    let mut handles = vec![];
    for _ in 0..10 {
        let handler_clone: Arc<ProtocolHandler> = Arc::clone(&handler);
        let handle = thread::spawn(move || handler_clone.handle("concurrent://test"));
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.join().unwrap();
        assert!(result.is_some(), "Concurrent access should work");
    }
}
