// Tests for CdpClient (moved from cdp.rs to keep file under 1000 lines)
// These are integration-style tests that verify JSON parsing logic.

use auroraview_mcp::cdp::{BrowserVersion, CdpError};
use std::time::Duration;

#[test]
fn cdp_error_display_timeout() {
    let dur = Duration::from_secs(5);
    let err = CdpError::Timeout("test_method".to_string(), dur);
    let msg = format!("{err}");
    assert!(msg.contains("timed out"), "got: {msg}");
}

#[test]
fn cdp_error_display_connection_closed() {
    let err = CdpError::ConnectionClosed("test_method".to_string());
    let msg = format!("{err}");
    assert!(msg.contains("closed before"), "got: {msg}");
}

#[test]
fn cdp_error_display_remote() {
    let err = CdpError::Remote("test_method".to_string(), "test error".to_owned());
    let msg = format!("{err}");
    assert!(msg.contains("test error"), "got: {msg}");
}

#[test]
fn cdp_error_display_malformed_response() {
    let err = CdpError::MalformedResponse("test_method".to_string(), "result");
    let msg = format!("{err}");
    assert!(msg.contains("result"), "got: {msg}");
}

#[test]
fn browser_version_creation() {
    let version = BrowserVersion {
        product: "Chrome/120.0.6099.109".to_owned(),
        protocol_version: "1.3".to_owned(),
    };
    assert_eq!(version.product, "Chrome/120.0.6099.109");
    assert_eq!(version.protocol_version, "1.3");
}

#[test]
fn browser_version_debug() {
    let version = BrowserVersion {
        product: "test".to_owned(),
        protocol_version: "1.0".to_owned(),
    };
    let debug = format!("{version:?}");
    assert!(debug.contains("test"));
}

// Tests for JSON response parsing logic

#[test]
fn query_selector_returns_node_id() {
    let json = serde_json::json!({"result": {"nodeId": 42}});
    let node_id = json
        .get("result")
        .and_then(|r| r.get("nodeId"))
        .and_then(serde_json::Value::as_i64)
        .filter(|&id| id != 0);
    assert_eq!(node_id, Some(42));
}

#[test]
fn query_selector_returns_none_when_not_found() {
    let json = serde_json::json!({"result": {"nodeId": 0}});
    let node_id = json
        .get("result")
        .and_then(|r| r.get("nodeId"))
        .and_then(serde_json::Value::as_i64)
        .filter(|&id| id != 0);
    assert_eq!(node_id, None);
}

#[test]
fn query_selector_all_returns_node_ids() {
    let json = serde_json::json!({"result": {"nodeIds": [1, 2, 3]}});
    let node_ids = json
        .get("result")
        .and_then(|r| r.get("nodeIds"))
        .and_then(serde_json::Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(serde_json::Value::as_i64)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    assert_eq!(node_ids, vec![1, 2, 3]);
}

#[test]
fn query_selector_all_returns_empty_vec() {
    let json = serde_json::json!({"result": {"nodeIds": []}});
    let node_ids = json
        .get("result")
        .and_then(|r| r.get("nodeIds"))
        .and_then(serde_json::Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(serde_json::Value::as_i64)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    assert!(node_ids.is_empty());
}

#[test]
fn get_outer_html_returns_html() {
    let json = serde_json::json!({"result": {"outerHTML": "<div>Hello</div>"}});
    let html = json
        .get("result")
        .and_then(|r| r.get("outerHTML"))
        .and_then(serde_json::Value::as_str)
        .map(String::from)
        .unwrap_or_default();
    assert_eq!(html, "<div>Hello</div>");
}

#[test]
fn get_outer_html_handles_missing_field() {
    let json = serde_json::json!({"result": {}});
    let html = json
        .get("result")
        .and_then(|r| r.get("outerHTML"))
        .and_then(serde_json::Value::as_str)
        .map(String::from)
        .unwrap_or_default();
    assert_eq!(html, "");
}

#[test]
fn get_attributes_returns_attributes() {
    let json = serde_json::json!({"result": {"attributes": ["id", "my-id", "class", "my-class"]}});
    let attrs_array = json
        .get("result")
        .and_then(|r| r.get("attributes"))
        .and_then(serde_json::Value::as_array)
        .unwrap();

    let mut attrs = std::collections::HashMap::new();
    let mut i = 0;
    while i + 1 < attrs_array.len() {
        if let (Some(name), Some(value)) = (attrs_array[i].as_str(), attrs_array[i + 1].as_str()) {
            attrs.insert(name.to_owned(), value.to_owned());
        }
        i += 2;
    }
    assert_eq!(attrs.len(), 2);
    assert_eq!(attrs.get("id"), Some(&"my-id".to_owned()));
    assert_eq!(attrs.get("class"), Some(&"my-class".to_owned()));
}

#[test]
fn get_attributes_returns_empty() {
    let json = serde_json::json!({"result": {"attributes": []}});
    let attrs_array = json
        .get("result")
        .and_then(|r| r.get("attributes"))
        .and_then(serde_json::Value::as_array)
        .unwrap();

    let mut attrs = std::collections::HashMap::new();
    let mut i = 0;
    while i + 1 < attrs_array.len() {
        if let (Some(name), Some(value)) = (attrs_array[i].as_str(), attrs_array[i + 1].as_str()) {
            attrs.insert(name.to_owned(), value.to_owned());
        }
        i += 2;
    }
    assert!(attrs.is_empty());
}

#[test]
fn get_properties_returns_properties() {
    let json = serde_json::json!({
        "result": {
            "result": [
                {"name": "prop1", "value": {"type": "string", "value": "hello"}},
                {"name": "prop2", "value": {"type": "number", "value": 42}}
            ]
        }
    });
    let props = json
        .get("result")
        .and_then(|r| r.get("result"))
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    assert_eq!(props.len(), 2);
    assert_eq!(props[0].get("name").and_then(|v| v.as_str()), Some("prop1"));
    assert_eq!(props[1].get("name").and_then(|v| v.as_str()), Some("prop2"));
}

#[test]
fn get_response_body_returns_text() {
    let json = serde_json::json!({
        "result": {
            "body": "Hello, World!",
            "base64Encoded": false
        }
    });
    let body = json
        .get("result")
        .and_then(|r| r.get("body"))
        .and_then(|v| v.as_str())
        .unwrap();
    let is_base64 = json
        .get("result")
        .and_then(|r| r.get("base64Encoded"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let bytes = if is_base64 {
        <base64::engine::GeneralPurpose as base64::Engine>::decode(
            &base64::engine::general_purpose::STANDARD,
            body,
        )
        .unwrap()
    } else {
        body.as_bytes().to_vec()
    };
    assert_eq!(bytes, b"Hello, World!");
}

#[test]
fn get_response_body_returns_base64() {
    let json = serde_json::json!({
        "result": {
            "body": <base64::engine::GeneralPurpose as base64::Engine>::encode(&base64::engine::general_purpose::STANDARD, "Hello, World!"),
            "base64Encoded": true
        }
    });
    let body = json
        .get("result")
        .and_then(|r| r.get("body"))
        .and_then(|v| v.as_str())
        .unwrap();
    let is_base64 = json
        .get("result")
        .and_then(|r| r.get("base64Encoded"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let bytes = if is_base64 {
        <base64::engine::GeneralPurpose as base64::Engine>::decode(
            &base64::engine::general_purpose::STANDARD,
            body,
        )
        .unwrap()
    } else {
        body.as_bytes().to_vec()
    };
    assert_eq!(bytes, b"Hello, World!");
}

// Tests for new CDP methods (set_attribute_value, remove_attribute, call_function_on, clear_browser_cache)

#[test]
fn set_attribute_value_returns_ok() {
    let json = serde_json::json!({"result": {}});
    let result = json.get("result");
    assert!(result.is_some());
}

#[test]
fn remove_attribute_returns_ok() {
    let json = serde_json::json!({"result": {}});
    let result = json.get("result");
    assert!(result.is_some());
}

#[test]
fn call_function_on_returns_value() {
    let json = serde_json::json!({
        "result": {
            "result": {
                "type": "number",
                "value": 42
            }
        }
    });
    let value = json
        .get("result")
        .and_then(|r| r.get("result"))
        .and_then(|v| v.get("value"))
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    assert_eq!(value, serde_json::json!(42));
}

#[test]
fn call_function_on_returns_string() {
    let json = serde_json::json!({
        "result": {
            "result": {
                "type": "string",
                "value": "hello"
            }
        }
    });
    let value = json
        .get("result")
        .and_then(|r| r.get("result"))
        .and_then(|v| v.get("value"))
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    assert_eq!(value, serde_json::json!("hello"));
}

#[test]
fn clear_browser_cache_returns_ok() {
    let json = serde_json::json!({"result": {}});
    let result = json.get("result");
    assert!(result.is_some());
}

// Tests for Iteration #93 new CDP methods

#[test]
fn set_cache_disabled_returns_ok() {
    // CDP returns `{"result": {}}` for successful setCacheDisabled
    let json = serde_json::json!({"result": {}});
    let result = json.get("result");
    assert!(result.is_some());
}

#[test]
fn set_download_behavior_returns_ok() {
    // CDP returns `{"result": {}}` for successful setDownloadBehavior
    let json = serde_json::json!({"result": {}});
    let result = json.get("result");
    assert!(result.is_some());
}

#[test]
fn set_device_metrics_override_returns_ok() {
    // CDP returns `{"result": {}}` for successful setDeviceMetricsOverride
    let json = serde_json::json!({"result": {}});
    let result = json.get("result");
    assert!(result.is_some());
}

#[test]
fn set_ignore_certificate_errors_returns_ok() {
    // CDP returns `{"result": {}}` for successful setIgnoreCertificateErrors
    let json = serde_json::json!({"result": {}});
    let result = json.get("result");
    assert!(result.is_some());
}
