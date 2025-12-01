//! High-performance JSON operations
//!
//! This module provides simd-json accelerated JSON parsing and serialization.
//! The pure Rust API can be used by both CLI and Python bindings.

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
