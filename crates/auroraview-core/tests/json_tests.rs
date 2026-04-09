//! JSON utility tests

use auroraview_core::json::{
    from_bytes, from_slice, from_str, from_value, serialize_to_js_literal, to_js_literal,
    to_string, to_string_pretty, to_value,
};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// from_str
// ---------------------------------------------------------------------------

#[test]
fn test_from_str() {
    let json = r#"{"name": "test", "value": 42}"#;
    let value = from_str(json).unwrap();
    assert_eq!(value["name"], "test");
    assert_eq!(value["value"], 42);
}

#[test]
fn test_from_str_empty_object() {
    let value = from_str("{}").unwrap();
    assert!(value.is_object());
    assert_eq!(value.as_object().unwrap().len(), 0);
}

#[test]
fn test_from_str_array() {
    let value = from_str(r#"[1, 2, 3]"#).unwrap();
    assert!(value.is_array());
    assert_eq!(value[0], 1);
    assert_eq!(value[2], 3);
}

#[test]
fn test_from_str_nested() {
    let json = r#"{"outer": {"inner": "value"}}"#;
    let value = from_str(json).unwrap();
    assert_eq!(value["outer"]["inner"], "value");
}

#[test]
fn test_from_str_error() {
    let result = from_str("not valid json");
    assert!(result.is_err());
    let msg = result.unwrap_err();
    assert!(msg.contains("JSON parse error"));
}

#[test]
fn test_from_str_unicode() {
    let json = r#"{"greeting": "你好世界"}"#;
    let value = from_str(json).unwrap();
    assert_eq!(value["greeting"], "你好世界");
}

// ---------------------------------------------------------------------------
// from_slice
// ---------------------------------------------------------------------------

#[test]
fn test_from_slice_basic() {
    let mut bytes = br#"{"key": "value"}"#.to_vec();
    let value = from_slice(&mut bytes).unwrap();
    assert_eq!(value["key"], "value");
}

#[test]
fn test_from_slice_number() {
    let mut bytes = b"42".to_vec();
    let value = from_slice(&mut bytes).unwrap();
    assert_eq!(value, 42);
}

#[test]
fn test_from_slice_error() {
    let mut bytes = b"invalid".to_vec();
    let result = from_slice(&mut bytes);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// from_bytes
// ---------------------------------------------------------------------------

#[test]
fn test_from_bytes() {
    let bytes = br#"{"key": "value"}"#.to_vec();
    let value = from_bytes(bytes).unwrap();
    assert_eq!(value["key"], "value");
}

#[test]
fn test_from_bytes_boolean() {
    let value = from_bytes(b"true".to_vec()).unwrap();
    assert_eq!(value, true);
}

#[test]
fn test_from_bytes_null() {
    let value = from_bytes(b"null".to_vec()).unwrap();
    assert!(value.is_null());
}

// ---------------------------------------------------------------------------
// to_string
// ---------------------------------------------------------------------------

#[test]
fn test_to_string() {
    let value = serde_json::json!({"name": "test", "value": 42});
    let json = to_string(&value).unwrap();
    assert!(json.contains("name"));
    assert!(json.contains("test"));
}

#[test]
fn test_to_string_roundtrip() {
    let original = serde_json::json!({"a": 1, "b": [2, 3], "c": null});
    let s = to_string(&original).unwrap();
    let parsed = from_str(&s).unwrap();
    assert_eq!(original["a"], parsed["a"]);
    assert_eq!(original["c"], parsed["c"]);
}

#[test]
fn test_to_string_unicode_preserved() {
    let value = serde_json::json!({"msg": "日本語テスト"});
    let s = to_string(&value).unwrap();
    // serde_json preserves Unicode in to_string
    assert!(s.contains("日本語テスト"));
}

// ---------------------------------------------------------------------------
// to_string_pretty
// ---------------------------------------------------------------------------

#[test]
fn test_to_string_pretty() {
    let value = serde_json::json!({"x": 1, "y": 2});
    let pretty = to_string_pretty(&value).unwrap();
    // Pretty-printed JSON has newlines and indentation
    assert!(pretty.contains('\n'));
    assert!(pretty.contains("  "));
    assert!(pretty.contains("\"x\""));
}

#[test]
fn test_to_string_pretty_roundtrip() {
    let original = serde_json::json!({"list": [1, 2, 3]});
    let pretty = to_string_pretty(&original).unwrap();
    let parsed = from_str(&pretty).unwrap();
    assert_eq!(original["list"][0], parsed["list"][0]);
    assert_eq!(original["list"][2], parsed["list"][2]);
}

// ---------------------------------------------------------------------------
// to_value
// ---------------------------------------------------------------------------

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

#[test]
fn test_to_value_vec() {
    let v = vec![1u32, 2, 3];
    let value = to_value(&v).unwrap();
    assert!(value.is_array());
    assert_eq!(value[0], 1);
}

#[test]
fn test_to_value_nested_struct() {
    #[derive(Serialize)]
    struct Inner {
        x: i32,
    }
    #[derive(Serialize)]
    struct Outer {
        inner: Inner,
    }
    let o = Outer { inner: Inner { x: 99 } };
    let value = to_value(&o).unwrap();
    assert_eq!(value["inner"]["x"], 99);
}

// ---------------------------------------------------------------------------
// from_value
// ---------------------------------------------------------------------------

#[test]
fn test_from_value_string() {
    let value = serde_json::json!("hello");
    let s: String = from_value(value).unwrap();
    assert_eq!(s, "hello");
}

#[test]
fn test_from_value_struct() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Point {
        x: f64,
        y: f64,
    }
    let value = serde_json::json!({"x": 1.5, "y": 2.5});
    let p: Point = from_value(value).unwrap();
    assert_eq!(p, Point { x: 1.5, y: 2.5 });
}

#[test]
fn test_from_value_error() {
    #[allow(dead_code)]
    #[derive(Debug, Deserialize)]
    struct Strict {
        required_field: String,
    }
    let value = serde_json::json!({"other_field": "value"});
    let result: Result<Strict, _> = from_value(value);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("JSON deserialize error"));
}

// ---------------------------------------------------------------------------
// to_js_literal
// ---------------------------------------------------------------------------

#[test]
fn test_to_js_literal_simple() {
    let value = serde_json::json!({"key": "value"});
    let literal = to_js_literal(&value);
    assert!(literal.contains("key"));
    assert!(literal.contains("value"));
}

#[test]
fn test_to_js_literal_unicode() {
    let data = serde_json::json!({"message": "你好世界", "emoji": "🎉"});
    let js_literal = to_js_literal(&data);
    assert!(js_literal.contains("你好世界"));
    assert!(js_literal.contains("🎉"));
}

#[test]
fn test_to_js_literal_special_chars() {
    let data = serde_json::json!({"path": "C:\\Users\\test", "quote": "He said \"hello\""});
    let js_literal = to_js_literal(&data);
    assert!(js_literal.contains("C:\\\\Users\\\\test"));
    assert!(js_literal.contains("\\\"hello\\\""));
}

#[test]
fn test_to_js_literal_nested() {
    let data = serde_json::json!({
        "user": {"name": "张三", "age": 25},
        "tags": ["中文", "test"]
    });
    let js_literal = to_js_literal(&data);
    assert!(js_literal.contains("张三"));
    assert!(js_literal.contains("中文"));
}

#[test]
fn test_to_js_literal_number() {
    let value = serde_json::json!(42);
    assert_eq!(to_js_literal(&value), "42");
}

#[test]
fn test_to_js_literal_boolean() {
    assert_eq!(to_js_literal(&serde_json::json!(true)), "true");
    assert_eq!(to_js_literal(&serde_json::json!(false)), "false");
}

#[test]
fn test_to_js_literal_null() {
    assert_eq!(to_js_literal(&serde_json::Value::Null), "null");
}

#[test]
fn test_to_js_literal_empty_object() {
    let value = serde_json::json!({});
    assert_eq!(to_js_literal(&value), "{}");
}

#[test]
fn test_to_js_literal_empty_array() {
    let value = serde_json::json!([]);
    assert_eq!(to_js_literal(&value), "[]");
}

// ---------------------------------------------------------------------------
// serialize_to_js_literal
// ---------------------------------------------------------------------------

#[test]
fn test_serialize_to_js_literal() {
    #[derive(serde::Serialize)]
    struct TestStruct {
        name: String,
        value: i32,
    }
    let data = TestStruct {
        name: "测试".to_string(),
        value: 100,
    };
    let js_literal = serialize_to_js_literal(&data).unwrap();
    assert!(js_literal.contains("测试"));
    assert!(js_literal.contains("100"));
}

#[test]
fn test_serialize_to_js_literal_vec() {
    let v = vec![1u32, 2, 3];
    let s = serialize_to_js_literal(&v).unwrap();
    assert_eq!(s, "[1,2,3]");
}

#[test]
fn test_serialize_to_js_literal_string() {
    let s = "hello".to_string();
    let literal = serialize_to_js_literal(&s).unwrap();
    assert_eq!(literal, r#""hello""#);
}
