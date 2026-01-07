//! Integration tests for IPC communication
//!
//! These tests verify the IPC infrastructure works correctly.
//!
//! Note: Full IPC server/client tests are skipped on CI due to platform-specific
//! socket behavior. The unit tests in the library cover the core functionality.

use auroraview_mcp_server::protocol::{Request, Response};

/// Test JSON-RPC request/response serialization
#[test]
fn test_jsonrpc_serialization() {
    // Test request serialization
    let request = Request::with_params(
        42,
        "tools.call",
        serde_json::json!({
            "name": "test_tool",
            "arguments": {"arg1": "value1"}
        }),
    );

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("\"jsonrpc\":\"2.0\""));
    assert!(json.contains("\"id\":42"));
    assert!(json.contains("\"method\":\"tools.call\""));

    // Test response serialization
    let response = Response::success(Some(42.into()), serde_json::json!({"result": "ok"}));
    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("\"jsonrpc\":\"2.0\""));
    assert!(json.contains("\"result\""));
}
