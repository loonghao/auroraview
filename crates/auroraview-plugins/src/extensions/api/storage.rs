use serde_json::Value;

use crate::extensions::ExtensionsPlugin;
use auroraview_plugin_core::{PluginError, PluginResult};

impl ExtensionsPlugin {
    /// Handle storage API calls
    pub fn handle_storage_api(
        &self,
        extension_id: &str,
        method: &str,
        params: &Value,
    ) -> PluginResult<Value> {
        let area = params
            .get("area")
            .and_then(|v| v.as_str())
            .unwrap_or("local");
        let storage_key = format!("{}:{}", extension_id, area);

        match method {
            "get" => {
                let state = self.state.read();
                let data = state.storage.get(&storage_key).cloned().unwrap_or_default();

                // Handle different key formats
                let keys = params.get("keys");
                let result = match keys {
                    None | Some(Value::Null) => {
                        // Return all data
                        data
                    }
                    Some(Value::String(key)) => {
                        // Single key
                        let mut result = std::collections::HashMap::new();
                        if let Some(value) = data.get(key) {
                            result.insert(key.clone(), value.clone());
                        }
                        result
                    }
                    Some(Value::Array(arr)) => {
                        // Array of keys
                        let mut result = std::collections::HashMap::new();
                        for key in arr {
                            if let Some(key_str) = key.as_str() {
                                if let Some(value) = data.get(key_str) {
                                    result.insert(key_str.to_string(), value.clone());
                                }
                            }
                        }
                        result
                    }
                    Some(Value::Object(obj)) => {
                        // Object with defaults
                        let mut result = std::collections::HashMap::new();
                        for (key, default) in obj {
                            let value = data.get(key).cloned().unwrap_or(default.clone());
                            result.insert(key.clone(), value);
                        }
                        result
                    }
                    _ => data,
                };

                serde_json::to_value(result).map_err(PluginError::serialization_error)
            }
            "set" => {
                let items: std::collections::HashMap<String, Value> = params
                    .get("items")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .ok_or_else(|| PluginError::invalid_args("items is required"))?;

                let mut state = self.state.write();
                let data = state.storage.entry(storage_key.clone()).or_default();

                // Track changes for onChanged event
                let mut changes = std::collections::HashMap::new();
                for (key, new_value) in items {
                    let old_value = data.get(&key).cloned();
                    changes.insert(
                        key.clone(),
                        serde_json::json!({
                            "oldValue": old_value,
                            "newValue": new_value
                        }),
                    );
                    data.insert(key, new_value);
                }

                // Persist to disk via callback
                let cbs = self.callbacks.read();
                if let Some(ref persist_cb) = cbs.on_storage_persist {
                    persist_cb(extension_id, &storage_key, data);
                }
                drop(cbs);

                tracing::debug!(
                    "storage.set: {} keys updated for {}",
                    changes.len(),
                    storage_key
                );

                Ok(serde_json::json!({}))
            }
            "remove" => {
                let keys: Vec<String> = match params.get("keys") {
                    Some(Value::String(s)) => vec![s.clone()],
                    Some(Value::Array(arr)) => arr
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect(),
                    _ => return Err(PluginError::invalid_args("keys is required")),
                };

                let mut state = self.state.write();
                if let Some(data) = state.storage.get_mut(&storage_key) {
                    for key in keys {
                        data.remove(&key);
                    }
                }

                Ok(serde_json::json!({}))
            }
            "clear" => {
                let mut state = self.state.write();
                state.storage.remove(&storage_key);
                Ok(serde_json::json!({}))
            }
            "getBytesInUse" => {
                let state = self.state.read();
                let data = state.storage.get(&storage_key).cloned().unwrap_or_default();
                let json = serde_json::to_string(&data).unwrap_or_default();
                Ok(serde_json::json!(json.len()))
            }
            "setAccessLevel" => {
                // Not implemented for now
                Ok(serde_json::json!({}))
            }
            _ => Err(PluginError::command_not_found(&format!(
                "storage.{}",
                method
            ))),
        }
    }
}
