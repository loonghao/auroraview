//! Setting value types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A setting value that can hold various types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SettingValue {
    /// Null value.
    Null,
    /// Boolean value.
    Bool(bool),
    /// Integer value.
    Integer(i64),
    /// Float value.
    Float(f64),
    /// String value.
    String(String),
    /// Array of values.
    Array(Vec<SettingValue>),
    /// Object/map of values.
    Object(HashMap<String, SettingValue>),
}

impl SettingValue {
    /// Returns the type name of this value.
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Bool(_) => "bool",
            Self::Integer(_) => "integer",
            Self::Float(_) => "float",
            Self::String(_) => "string",
            Self::Array(_) => "array",
            Self::Object(_) => "object",
        }
    }

    /// Returns true if this value is null.
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Returns the boolean value if this is a Bool.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(v) => Some(*v),
            _ => None,
        }
    }

    /// Returns the integer value if this is an Integer.
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Self::Integer(v) => Some(*v),
            _ => None,
        }
    }

    /// Returns the float value if this is a Float.
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(v) => Some(*v),
            Self::Integer(v) => Some(*v as f64),
            _ => None,
        }
    }

    /// Returns the string value if this is a String.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(v) => Some(v),
            _ => None,
        }
    }

    /// Returns the array if this is an Array.
    pub fn as_array(&self) -> Option<&Vec<SettingValue>> {
        match self {
            Self::Array(v) => Some(v),
            _ => None,
        }
    }

    /// Returns the object if this is an Object.
    pub fn as_object(&self) -> Option<&HashMap<String, SettingValue>> {
        match self {
            Self::Object(v) => Some(v),
            _ => None,
        }
    }
}

impl Default for SettingValue {
    fn default() -> Self {
        Self::Null
    }
}

impl From<bool> for SettingValue {
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}

impl From<i32> for SettingValue {
    fn from(v: i32) -> Self {
        Self::Integer(v as i64)
    }
}

impl From<i64> for SettingValue {
    fn from(v: i64) -> Self {
        Self::Integer(v)
    }
}

impl From<f64> for SettingValue {
    fn from(v: f64) -> Self {
        Self::Float(v)
    }
}

impl From<String> for SettingValue {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}

impl From<&str> for SettingValue {
    fn from(v: &str) -> Self {
        Self::String(v.to_string())
    }
}

impl<T: Into<SettingValue>> From<Vec<T>> for SettingValue {
    fn from(v: Vec<T>) -> Self {
        Self::Array(v.into_iter().map(Into::into).collect())
    }
}

impl<T: Into<SettingValue>> From<HashMap<String, T>> for SettingValue {
    fn from(v: HashMap<String, T>) -> Self {
        Self::Object(v.into_iter().map(|(k, v)| (k, v.into())).collect())
    }
}
