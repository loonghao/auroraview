//! chrome.storage API handler

use serde_json::Value;
use std::sync::Arc;

use crate::apis::ApiHandler;
use crate::error::{ExtensionError, ExtensionResult};
use crate::storage::{StorageArea, StorageBackend};

/// Storage API handler
pub struct StorageApiHandler {
    backend: Arc<StorageBackend>,
}

impl StorageApiHandler {
    /// Create a new storage API handler
    pub fn new(backend: Arc<StorageBackend>) -> Self {
        Self { backend }
    }
}

impl ApiHandler for StorageApiHandler {
    fn namespace(&self) -> &str {
        "storage"
    }

    fn handle(&self, method: &str, params: Value, extension_id: &str) -> ExtensionResult<Value> {
        // Parse area from params
        let area: StorageArea = params
            .get("area")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        // Use tokio runtime for async operations
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| ExtensionError::Runtime("No tokio runtime available".to_string()))?;

        match method {
            "get" => {
                let keys: Option<Vec<String>> = params
                    .get("keys")
                    .and_then(|v| serde_json::from_value(v.clone()).ok());

                let backend = self.backend.clone();
                let ext_id = extension_id.to_string();

                let result = rt.block_on(async move { backend.get(&ext_id, area, keys).await })?;

                Ok(serde_json::to_value(result)?)
            }
            "set" => {
                let items = params
                    .get("items")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .ok_or_else(|| {
                        ExtensionError::InvalidArgument("items is required".to_string())
                    })?;

                let backend = self.backend.clone();
                let ext_id = extension_id.to_string();

                let _changes =
                    rt.block_on(async move { backend.set(&ext_id, area, items).await })?;

                Ok(serde_json::json!({}))
            }
            "remove" => {
                let keys: Vec<String> = params
                    .get("keys")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .ok_or_else(|| {
                        ExtensionError::InvalidArgument("keys is required".to_string())
                    })?;

                let backend = self.backend.clone();
                let ext_id = extension_id.to_string();

                let _changes =
                    rt.block_on(async move { backend.remove(&ext_id, area, keys).await })?;

                Ok(serde_json::json!({}))
            }
            "clear" => {
                let backend = self.backend.clone();
                let ext_id = extension_id.to_string();

                let _changes = rt.block_on(async move { backend.clear(&ext_id, area).await })?;

                Ok(serde_json::json!({}))
            }
            "getBytesInUse" => {
                let keys: Option<Vec<String>> = params
                    .get("keys")
                    .and_then(|v| serde_json::from_value(v.clone()).ok());

                let backend = self.backend.clone();
                let ext_id = extension_id.to_string();

                let bytes = rt
                    .block_on(async move { backend.get_bytes_in_use(&ext_id, area, keys).await })?;

                Ok(serde_json::json!(bytes))
            }
            _ => Err(ExtensionError::ApiNotSupported(format!(
                "storage.{} is not supported",
                method
            ))),
        }
    }

    fn methods(&self) -> Vec<&str> {
        vec!["get", "set", "remove", "clear", "getBytesInUse"]
    }
}
