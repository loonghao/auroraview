//! Integration tests for IPC JSON operations
//!
//! These tests verify the high-performance JSON functionality.

use _core::ipc::json;
use rstest::*;
use serde_json::json;

#[rstest]
#[case(r#"{"key": "value"}"#)]
#[case(r#"{"number": 42}"#)]
#[case(r#"{"bool": true}"#)]
#[case(r#"{"array": [1, 2, 3]}"#)]
fn test_json_from_str_valid(#[case] json_str: &str) {
    let result = json::from_str(json_str);
    assert!(result.is_ok());
}

#[rstest]
#[case(r#"{"key": "value""#, "parse error")]
#[case(r#"{key: value}"#, "parse error")]
#[case(r#"invalid json"#, "parse error")]
fn test_json_from_str_invalid(#[case] json_str: &str, #[case] expected_error: &str) {
    let result = json::from_str(json_str);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains(expected_error));
}

#[rstest]
fn test_json_from_str_object() {
    let json_str = r#"{"name": "test", "value": 123}"#;
    let result = json::from_str(json_str).unwrap();

    assert!(result.is_object());
    assert_eq!(result["name"], "test");
    assert_eq!(result["value"], 123);
}

#[rstest]
fn test_json_from_str_array() {
    let json_str = r#"[1, 2, 3, 4, 5]"#;
    let result = json::from_str(json_str).unwrap();

    assert!(result.is_array());
    let arr = result.as_array().unwrap();
    assert_eq!(arr.len(), 5);
    assert_eq!(arr[0], 1);
    assert_eq!(arr[4], 5);
}

#[rstest]
fn test_json_from_str_nested() {
    let json_str = r#"{"outer": {"inner": {"deep": "value"}}}"#;
    let result = json::from_str(json_str).unwrap();

    assert_eq!(result["outer"]["inner"]["deep"], "value");
}

#[rstest]
fn test_json_to_string_object() {
    let value = json!({
        "name": "test",
        "value": 42
    });

    let result = json::to_string(&value);
    assert!(result.is_ok());

    let json_str = result.unwrap();
    assert!(json_str.contains("name"));
    assert!(json_str.contains("test"));
    assert!(json_str.contains("value"));
    assert!(json_str.contains("42"));
}

#[rstest]
fn test_json_to_string_array() {
    let value = json!([1, 2, 3, 4, 5]);

    let result = json::to_string(&value);
    assert!(result.is_ok());

    let json_str = result.unwrap();
    assert!(json_str.contains("1"));
    assert!(json_str.contains("5"));
}

#[rstest]
#[case(json!({"key": "value"}))]
#[case(json!([1, 2, 3]))]
#[case(json!({"nested": {"deep": "value"}}))]
#[case(json!({"array": [1, 2, 3], "object": {"key": "value"}}))]
fn test_json_roundtrip(#[case] original: serde_json::Value) {
    // Serialize to string
    let json_str = json::to_string(&original).unwrap();

    // Parse back
    let parsed = json::from_str(&json_str).unwrap();

    // Should be equal
    assert_eq!(original, parsed);
}

#[rstest]
fn test_json_empty_object() {
    let json_str = r#"{}"#;
    let result = json::from_str(json_str).unwrap();
    assert!(result.is_object());
    assert_eq!(result.as_object().unwrap().len(), 0);
}

#[rstest]
fn test_json_empty_array() {
    let json_str = r#"[]"#;
    let result = json::from_str(json_str).unwrap();
    assert!(result.is_array());
    assert_eq!(result.as_array().unwrap().len(), 0);
}

#[rstest]
#[case(r#"null"#)]
#[case(r#"true"#)]
#[case(r#"false"#)]
#[case(r#"123"#)]
#[case(r#"123.456"#)]
#[case(r#""string""#)]
fn test_json_primitive_types(#[case] json_str: &str) {
    let result = json::from_str(json_str);
    assert!(result.is_ok());
}

#[rstest]
fn test_json_unicode_strings() {
    let json_str = r#"{"emoji": "🚀", "chinese": "你好", "japanese": "こんにちは"}"#;
    let result = json::from_str(json_str).unwrap();

    assert_eq!(result["emoji"], "🚀");
    assert_eq!(result["chinese"], "你好");
    assert_eq!(result["japanese"], "こんにちは");
}

#[rstest]
fn test_json_escaped_characters() {
    let json_str = r#"{"escaped": "line1\nline2\ttab"}"#;
    let result = json::from_str(json_str).unwrap();

    let escaped_str = result["escaped"].as_str().unwrap();
    assert!(escaped_str.contains('\n'));
    assert!(escaped_str.contains('\t'));
}

#[rstest]
fn test_json_large_numbers() {
    let json_str = r#"{"small": 1, "large": 9007199254740991}"#;
    let result = json::from_str(json_str).unwrap();

    assert_eq!(result["small"], 1);
    assert_eq!(result["large"], 9007199254740991_i64);
}

#[rstest]
fn test_json_floating_point() {
    let json_str = r#"{"pi": 3.14159, "e": 2.71828}"#;
    let result = json::from_str(json_str).unwrap();

    let pi = result["pi"].as_f64().unwrap();
    let e = result["e"].as_f64().unwrap();

    assert!((pi - std::f64::consts::PI).abs() < 0.001);
    assert!((e - std::f64::consts::E).abs() < 0.001);
}

#[rstest]
fn test_json_deeply_nested() {
    let mut nested = json!("value");
    for _ in 0..10 {
        nested = json!({"level": nested});
    }

    let json_str = json::to_string(&nested).unwrap();
    let parsed = json::from_str(&json_str).unwrap();

    assert_eq!(nested, parsed);
}

#[rstest]
fn test_json_mixed_array() {
    let json_str = r#"[1, "string", true, null, {"key": "value"}, [1, 2, 3]]"#;
    let result = json::from_str(json_str).unwrap();

    let arr = result.as_array().unwrap();
    assert_eq!(arr.len(), 6);
    assert_eq!(arr[0], 1);
    assert_eq!(arr[1], "string");
    assert_eq!(arr[2], true);
    assert!(arr[3].is_null());
    assert!(arr[4].is_object());
    assert!(arr[5].is_array());
}
