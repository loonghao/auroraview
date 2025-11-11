//! Python bindings utilities
//!
//! Helper functions for converting between Python and Rust types.

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

/// Convert Python object to JSON Value (recursive, supports nested structures)
///
/// Supports Python types: str, int, float, bool, None, list, dict
fn py_to_json_recursive(value: &Bound<'_, PyAny>) -> PyResult<serde_json::Value> {
    // Try basic types first
    if let Ok(s) = value.extract::<String>() {
        return Ok(serde_json::Value::String(s));
    }

    // IMPORTANT: In Python, bool is a subclass of int. Check bool BEFORE int/float
    if let Ok(b) = value.extract::<bool>() {
        return Ok(serde_json::Value::Bool(b));
    }

    if let Ok(i) = value.extract::<i64>() {
        return Ok(serde_json::Value::Number(i.into()));
    }

    if let Ok(f) = value.extract::<f64>() {
        return Ok(serde_json::json!(f));
    }

    // Check for None
    if value.is_none() {
        return Ok(serde_json::Value::Null);
    }

    // Check for list
    if let Ok(list) = value.downcast::<PyList>() {
        let mut json_array = Vec::new();
        for item in list.iter() {
            json_array.push(py_to_json_recursive(&item)?);
        }
        return Ok(serde_json::Value::Array(json_array));
    }

    // Check for dict
    if let Ok(dict) = value.downcast::<PyDict>() {
        let mut json_obj = serde_json::Map::new();
        for (key, val) in dict.iter() {
            let key_str = key.extract::<String>()?;
            let json_val = py_to_json_recursive(&val)?;
            json_obj.insert(key_str, json_val);
        }
        return Ok(serde_json::Value::Object(json_obj));
    }

    // Unsupported type - convert to string representation
    tracing::warn!(
        "[WARNING] [py_to_json_recursive] Unsupported Python type: {}, converting to string",
        value.get_type().name()?
    );
    Ok(serde_json::Value::String(value.to_string()))
}

/// Convert Python dict to JSON Value
///
/// Supports Python types: str, int, float, bool, None, list, dict (with nesting)
pub fn py_dict_to_json(dict: &Bound<'_, PyDict>) -> PyResult<serde_json::Value> {
    let mut json_obj = serde_json::Map::new();

    for (key, value) in dict.iter() {
        let key_str = key.extract::<String>()?;
        let json_value = py_to_json_recursive(&value)?;
        json_obj.insert(key_str, json_value);
    }

    Ok(serde_json::Value::Object(json_obj))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::Python;

    #[test]
    fn test_py_dict_to_json() {
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            dict.set_item("string", "value").unwrap();
            dict.set_item("number", 42).unwrap();
            dict.set_item("float", std::f64::consts::PI).unwrap();
            dict.set_item("bool", true).unwrap();

            let json = py_dict_to_json(&dict).unwrap();

            assert_eq!(json["string"], "value");
            assert_eq!(json["number"], 42);
            assert_eq!(json["bool"], true);
        });
    }
}

#[test]
fn test_py_dict_to_json_nested_and_edge_types() {
    Python::with_gil(|py| {
        let dict = PyDict::new(py);
        let inner = PyDict::new(py);
        inner.set_item("nested", 3).unwrap();
        let list = pyo3::types::PyList::empty(py);
        list.append(1).unwrap();
        list.append(2).unwrap();
        list.append("x").unwrap();
        list.append(true).unwrap();
        list.append(py.None()).unwrap();
        list.append(inner.as_any()).unwrap();
        dict.set_item("list", list).unwrap();

        // Add unsupported type (tuple) → falls back to string
        let tuple = pyo3::types::PyTuple::new(py, [1, 2, 3]).unwrap();
        dict.set_item("tuple", &tuple).unwrap();

        let json = py_dict_to_json(&dict).unwrap();
        assert_eq!(json["list"][0], 1);
        assert_eq!(json["list"][2], "x");
        assert_eq!(json["list"][3], true);
        assert!(json["list"][4].is_null());
        assert_eq!(json["list"][5]["nested"], 3);
        // Tuple serialized via Display to string like "(1, 2, 3)"
        assert!(json["tuple"].is_string());
    });
}
