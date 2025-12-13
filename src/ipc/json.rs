//! High-performance JSON operations for IPC with Python bindings
//!
//! This module re-exports core JSON functions from auroraview-core and adds
//! PyO3-specific conversion functions for Python interoperability.
//!
//! ## Performance Benefits:
//! - **2-3x faster** than standard serde_json (SIMD acceleration)
//! - **Zero Python dependencies** - no need to install orjson
//! - **Direct PyO3 integration** - optimal Rust ↔ Python conversion
//! - **Memory efficient** - zero-copy parsing where possible

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;
#[cfg(feature = "python-bindings")]
use pyo3::types::{PyDict, PyList};
#[cfg(feature = "python-bindings")]
use pyo3::{Py, PyAny};

// Re-export all core JSON functions
pub use auroraview_core::json::{
    from_bytes, from_slice, from_str, from_value, to_string, to_string_pretty, to_value, Value,
};

/// Convert JSON value to Python object
///
/// This is a critical path for IPC performance, converting Rust JSON
/// to Python objects that can be passed to callbacks.
#[cfg(feature = "python-bindings")]
pub fn json_to_python(py: Python, value: &Value) -> PyResult<Py<PyAny>> {
    match value {
        Value::Null => Ok(py.None()),
        Value::Bool(b) => {
            let obj = b.into_pyobject(py)?;
            Ok(obj.as_any().clone().unbind())
        }
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                let obj = i.into_pyobject(py)?;
                Ok(obj.as_any().clone().unbind())
            } else if let Some(f) = n.as_f64() {
                let obj = f.into_pyobject(py)?;
                Ok(obj.as_any().clone().unbind())
            } else {
                let obj = n.to_string().into_pyobject(py)?;
                Ok(obj.as_any().clone().unbind())
            }
        }
        Value::String(s) => {
            let obj = s.into_pyobject(py)?;
            Ok(obj.as_any().clone().unbind())
        }
        Value::Array(arr) => {
            let py_list = PyList::new(py, arr.iter().map(|_| py.None()))?;
            for (idx, item) in arr.iter().enumerate() {
                let py_item = json_to_python(py, item)?;
                py_list.set_item(idx, py_item)?;
            }
            Ok(py_list.into_any().unbind())
        }
        Value::Object(obj) => {
            let py_dict = PyDict::new(py);
            for (key, val) in obj {
                let py_val = json_to_python(py, val)?;
                py_dict.set_item(key, py_val)?;
            }
            Ok(py_dict.into_any().unbind())
        }
    }
}

/// Convert Python object to JSON value
///
/// Supports Python types: str, int, float, bool, None, list, dict (with nesting)
#[cfg(feature = "python-bindings")]
pub fn python_to_json(value: &Bound<'_, PyAny>) -> PyResult<Value> {
    // Check for None first
    if value.is_none() {
        return Ok(Value::Null);
    }

    // IMPORTANT: Check bool BEFORE int because Python's True/False are subclasses of int
    // (True == 1 and False == 0 in Python), so extract::<i64>() would succeed for booleans
    if let Ok(b) = value.extract::<bool>() {
        // Double-check it's actually a bool type, not just an int that happens to be 0 or 1
        if value.is_instance_of::<pyo3::types::PyBool>() {
            return Ok(Value::Bool(b));
        }
    }

    // Try string
    if let Ok(s) = value.extract::<String>() {
        return Ok(Value::String(s));
    }

    // Try integer
    if let Ok(i) = value.extract::<i64>() {
        return Ok(Value::Number(i.into()));
    }

    // Try float
    if let Ok(f) = value.extract::<f64>() {
        return Ok(serde_json::json!(f));
    }

    // Check for list
    if let Ok(list) = value.cast::<PyList>() {
        let mut json_array = Vec::new();
        for item in list.iter() {
            json_array.push(python_to_json(&item)?);
        }
        return Ok(Value::Array(json_array));
    }

    // Check for dict
    if let Ok(dict) = value.cast::<PyDict>() {
        let mut json_obj = serde_json::Map::new();
        for (key, val) in dict.iter() {
            let key_str = key.extract::<String>()?;
            let json_val = python_to_json(&val)?;
            json_obj.insert(key_str, json_val);
        }
        return Ok(Value::Object(json_obj));
    }

    // Unsupported type - convert to string representation
    Ok(Value::String(value.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str_and_to_string_roundtrip() {
        let s = r#"{"a":1,"b":[1,2,3],"c":null}"#;
        let v = from_str(s).expect("parse ok");
        let out = to_string(&v).expect("serialize ok");
        // serde_json normalizes spacing; reparse and compare
        let v2: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v, v2);
    }

    #[cfg(feature = "python-bindings")]
    #[test]
    fn test_json_to_python_nested_objects() {
        use pyo3::Python;
        let value = serde_json::json!({
            "s": "x",
            "n": 42,
            "f": std::f64::consts::PI,
            "null": null,
            "arr": [1, "y", null],
            "obj": {"k": "v"}
        });
        Python::attach(|py| {
            let obj = json_to_python(py, &value).expect("to py ok");
            let back = python_to_json(obj.bind(py)).expect("roundtrip to json");
            assert_eq!(back["s"], "x");
            assert_eq!(back["n"], 42);
            assert!(back["null"].is_null());
            assert_eq!(back["arr"][1], "y");
            Ok::<(), pyo3::PyErr>(())
        })
        .unwrap();
    }

    #[cfg(feature = "python-bindings")]
    #[test]
    fn test_python_to_json_nested_objects() {
        use pyo3::types::{PyDict as PyDictType, PyList as PyListType};
        use pyo3::Python;
        Python::attach(|py| {
            let dict = PyDictType::new(py);
            dict.set_item("s", "x").unwrap();
            dict.set_item("i", 7).unwrap();
            dict.set_item("f", 2.5).unwrap();
            dict.set_item("none", py.None()).unwrap();
            let list = PyListType::new(py, vec![py.None()])?;
            list.append(1).unwrap();
            list.append("y").unwrap();
            list.append(py.None()).unwrap();
            dict.set_item("arr", list).unwrap();

            let v = python_to_json(dict.as_any()).expect("to json ok");
            assert_eq!(v["s"], "x");
            assert_eq!(v["i"], 7);
            assert!(v["none"].is_null());
            assert_eq!(v["arr"][1], "y");
            Ok::<(), pyo3::PyErr>(())
        })
        .unwrap();
    }

    #[test]
    fn test_from_slice_and_from_bytes_and_pretty() {
        let buf = br#"{"k":1}"#.to_vec();
        let mut slice = buf.clone();
        let v1 = from_slice(&mut slice).expect("slice ok");
        let v2 = from_bytes(buf).expect("bytes ok");
        assert_eq!(v1, v2);
        let pretty = to_string_pretty(&v1).expect("pretty ok");
        assert!(pretty.contains("\n"));
    }

    #[test]
    fn test_value_helpers_and_error() {
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
        struct S {
            a: i32,
        }
        let s = S { a: 5 };
        let val = to_value(&s).expect("to_value ok");
        let back: S = from_value(val).expect("from_value ok");
        assert_eq!(back, s);
        // invalid JSON error path
        assert!(from_str("not json").is_err());
    }

    #[test]
    fn test_from_value_error_wrong_type() {
        let val = serde_json::Value::String("x".to_string());
        let res: Result<i32, _> = from_value(val);
        assert!(res.is_err());
    }

    #[test]
    fn test_from_slice_error() {
        let mut bad = b"{".to_vec(); // invalid JSON
        assert!(from_slice(&mut bad).is_err());
    }
}
