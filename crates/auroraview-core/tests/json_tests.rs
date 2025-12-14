//! JSON utility tests

use auroraview_core::json::{from_bytes, from_str, to_string, to_value};
use serde::Serialize;

#[test]
fn test_from_str() {
    let json = r#"{"name": "test", "value": 42}"#;
    let value = from_str(json).unwrap();
    assert_eq!(value["name"], "test");
    assert_eq!(value["value"], 42);
}

#[test]
fn test_to_string() {
    let value = serde_json::json!({"name": "test", "value": 42});
    let json = to_string(&value).unwrap();
    assert!(json.contains("name"));
    assert!(json.contains("test"));
}

#[test]
fn test_from_bytes() {
    let bytes = br#"{"key": "value"}"#.to_vec();
    let value = from_bytes(bytes).unwrap();
    assert_eq!(value["key"], "value");
}

#[test]
fn test_to_value() {
    #[derive(Serialize)]
    struct Test {
        name: String,
    }
    let t = Test {
        name: "hello".to_string(),
    };
    let value = to_value(&t).unwrap();
    assert_eq!(value["name"], "hello");
}
