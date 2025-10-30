//! Python bindings utilities
//!
//! Helper functions for converting between Python and Rust types.

use pyo3::prelude::*;
use pyo3::types::PyDict;

/// Convert Python dict to JSON Value
///
/// Supports basic Python types: str, int, float, bool, None
pub fn py_dict_to_json(dict: &Bound<'_, PyDict>) -> PyResult<serde_json::Value> {
    let mut json_obj = serde_json::Map::new();

    for (key, value) in dict.iter() {
        let key_str = key.extract::<String>()?;
        let json_value = if let Ok(s) = value.extract::<String>() {
            serde_json::Value::String(s)
        } else if let Ok(i) = value.extract::<i64>() {
            serde_json::Value::Number(i.into())
        } else if let Ok(f) = value.extract::<f64>() {
            serde_json::json!(f)
        } else if let Ok(b) = value.extract::<bool>() {
            serde_json::Value::Bool(b)
        } else {
            serde_json::Value::Null
        };
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
            let dict = PyDict::new_bound(py);
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
