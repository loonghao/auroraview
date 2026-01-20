//! High-performance JSON operations
//!
//! This module provides simd-json accelerated JSON parsing and serialization.
//! The pure Rust API can be used by both CLI and Python bindings.
//!
//! ## Key Functions
//!
//! - `from_str`, `from_slice`, `from_bytes` - Parse JSON with SIMD acceleration
//! - `to_string`, `to_string_pretty` - Serialize to JSON string
//! - `to_js_literal` - Convert JSON Value to JavaScript literal (for WebView injection)
//!
//! ## Important: JSON to JavaScript Conversion
//!
//! When injecting JSON data into JavaScript code, use `to_js_literal()`:
//!
//! ```rust,ignore
//! let data = serde_json::json!({"message": "你好"});
//! let js_literal = auroraview_core::json::to_js_literal(&data);
//! // js_literal is valid JavaScript: {"message":"你好"}
//! ```
//!
//! **DO NOT** manually escape JSON strings with `.replace('\\', "\\\\")` etc.
//! This causes encoding issues with Unicode characters (中文乱码).

use serde::{Deserialize, Serialize};

// Re-export Value type
pub use serde_json::Value;

/// Parse JSON from a string slice using SIMD acceleration
///
/// This is 2-3x faster than serde_json::from_str() for typical messages.
#[inline]
pub fn from_str(s: &str) -> Result<Value, String> {
    let mut bytes = s.as_bytes().to_vec();
    simd_json::serde::from_slice(&mut bytes).map_err(|e| format!("JSON parse error: {}", e))
}

/// Parse JSON from mutable bytes (zero-copy, most efficient)
#[inline]
pub fn from_slice(bytes: &mut [u8]) -> Result<Value, String> {
    simd_json::serde::from_slice(bytes).map_err(|e| format!("JSON parse error: {}", e))
}

/// Parse JSON from owned bytes
#[inline]
pub fn from_bytes(mut bytes: Vec<u8>) -> Result<Value, String> {
    simd_json::serde::from_slice(&mut bytes).map_err(|e| format!("JSON parse error: {}", e))
}

/// Serialize a value to JSON string
#[inline]
pub fn to_string<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string(value).map_err(|e| format!("JSON serialize error: {}", e))
}

/// Serialize a value to JSON string with pretty printing
#[inline]
pub fn to_string_pretty<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string_pretty(value).map_err(|e| format!("JSON serialize error: {}", e))
}

/// Deserialize from JSON value
#[inline]
pub fn from_value<T: for<'de> Deserialize<'de>>(value: Value) -> Result<T, String> {
    serde_json::from_value(value).map_err(|e| format!("JSON deserialize error: {}", e))
}

/// Create a JSON value from a serializable type
#[inline]
pub fn to_value<T: Serialize>(value: &T) -> Result<Value, String> {
    serde_json::to_value(value).map_err(|e| format!("JSON value conversion error: {}", e))
}

/// Convert a JSON Value to a JavaScript literal string.
///
/// This is the **canonical way** to prepare JSON data for injection into JavaScript code.
/// The output is a valid JavaScript expression that can be directly embedded in JS.
///
/// # Important
///
/// - **DO NOT** apply additional escaping (like `.replace('\\', "\\\\")`) to the result
/// - The output already handles all necessary escaping for JavaScript
/// - Unicode characters (like Chinese) are preserved correctly
///
/// # Example
///
/// ```rust
/// use serde_json::json;
/// use auroraview_core::json::to_js_literal;
///
/// let data = json!({"message": "你好", "count": 42});
/// let js_literal = to_js_literal(&data);
/// // js_literal = r#"{"message":"你好","count":42}"#
///
/// // Use in JavaScript injection:
/// let script = format!("window.auroraview.trigger('event', {});", js_literal);
/// ```
#[inline]
pub fn to_js_literal(value: &Value) -> String {
    // serde_json::Value::to_string() produces valid JSON which is also valid JavaScript
    // It properly escapes special characters while preserving Unicode
    value.to_string()
}

/// Convert a serializable type to a JavaScript literal string.
///
/// This is a convenience wrapper around `to_js_literal` for types that implement Serialize.
///
/// # Example
///
/// ```rust
/// use serde::Serialize;
/// use auroraview_core::json::serialize_to_js_literal;
///
/// #[derive(Serialize)]
/// struct Event { name: String, data: i32 }
///
/// let event = Event { name: "click".to_string(), data: 42 };
/// let js_literal = serialize_to_js_literal(&event).unwrap();
/// ```
#[inline]
pub fn serialize_to_js_literal<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string(value).map_err(|e| format!("JSON serialize error: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_js_literal_unicode() {
        let data = serde_json::json!({"message": "你好世界", "emoji": "🎉"});
        let js_literal = to_js_literal(&data);
        // Should contain the actual Unicode characters, not escaped \u sequences
        assert!(js_literal.contains("你好世界"));
        assert!(js_literal.contains("🎉"));
    }

    #[test]
    fn test_to_js_literal_special_chars() {
        let data = serde_json::json!({"path": "C:\\Users\\test", "quote": "He said \"hello\""});
        let js_literal = to_js_literal(&data);
        // Should properly escape backslashes and quotes for JSON/JS
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
}
