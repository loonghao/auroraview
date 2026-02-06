//! Chrome Storage API Implementation
//!
//! Implements `chrome.storage.local` and `chrome.storage.sync` APIs.
//!
//! ## Storage Areas
//!
//! - `local` - Local storage (persisted on device)
//! - `sync` - Synced storage (in AuroraView, this is the same as local)
//! - `session` - Session storage (cleared when browser closes)
//!
//! ## Example
//!
//! ```javascript
//! // Set values
//! await chrome.storage.local.set({ key: "value" });
//!
//! // Get values
//! const result = await chrome.storage.local.get(["key"]);
//! console.log(result.key); // "value"
//!
//! // Remove values
//! await chrome.storage.local.remove(["key"]);
//!
//! // Clear all
//! await chrome.storage.local.clear();
//! ```

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;

use crate::error::ExtensionResult;
use crate::ExtensionId;

/// Storage area type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum StorageArea {
    /// Local storage (persisted)
    #[default]
    Local,
    /// Sync storage (in AuroraView, same as local)
    Sync,
    /// Session storage (not persisted)
    Session,
    /// Managed storage (read-only, enterprise policies)
    Managed,
}

/// Storage change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageChange {
    /// Old value (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_value: Option<Value>,
    /// New value (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_value: Option<Value>,
}

/// Storage changes map
pub type StorageChanges = HashMap<String, StorageChange>;

/// Storage backend for extensions
pub struct StorageBackend {
    /// Storage directory
    storage_dir: PathBuf,
    /// In-memory cache for session storage
    session_cache: Arc<RwLock<HashMap<ExtensionId, HashMap<String, Value>>>>,
    /// In-memory cache for local/sync storage (write-through)
    #[allow(dead_code)]
    local_cache: Arc<RwLock<HashMap<ExtensionId, HashMap<String, Value>>>>,
}

impl StorageBackend {
    /// Create a new storage backend
    pub fn new(storage_dir: PathBuf) -> Self {
        Self {
            storage_dir,
            session_cache: Arc::new(RwLock::new(HashMap::new())),
            local_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get storage file path for an extension
    fn get_storage_path(&self, extension_id: &str, area: StorageArea) -> PathBuf {
        let area_name = match area {
            StorageArea::Local => "local",
            StorageArea::Sync => "sync",
            StorageArea::Session => "session",
            StorageArea::Managed => "managed",
        };
        self.storage_dir
            .join(extension_id)
            .join(format!("{}.json", area_name))
    }

    /// Load storage data from disk
    async fn load_storage(
        &self,
        extension_id: &str,
        area: StorageArea,
    ) -> ExtensionResult<HashMap<String, Value>> {
        let path = self.get_storage_path(extension_id, area);

        if !path.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(&path).await?;
        let data: HashMap<String, Value> = serde_json::from_str(&content)?;
        Ok(data)
    }

    /// Save storage data to disk
    async fn save_storage(
        &self,
        extension_id: &str,
        area: StorageArea,
        data: &HashMap<String, Value>,
    ) -> ExtensionResult<()> {
        let path = self.get_storage_path(extension_id, area);

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let content = serde_json::to_string_pretty(data)?;
        fs::write(&path, content).await?;
        Ok(())
    }

    /// Get values from storage
    pub async fn get(
        &self,
        extension_id: &str,
        area: StorageArea,
        keys: Option<Vec<String>>,
    ) -> ExtensionResult<HashMap<String, Value>> {
        // For session storage, use in-memory cache only
        if area == StorageArea::Session {
            let cache = self.session_cache.read();
            let ext_data = cache.get(extension_id).cloned().unwrap_or_default();

            return Ok(match keys {
                Some(keys) => ext_data
                    .into_iter()
                    .filter(|(k, _)| keys.contains(k))
                    .collect(),
                None => ext_data,
            });
        }

        // For local/sync, load from disk (or cache)
        let data = self.load_storage(extension_id, area).await?;

        Ok(match keys {
            Some(keys) => data.into_iter().filter(|(k, _)| keys.contains(k)).collect(),
            None => data,
        })
    }

    /// Set values in storage
    pub async fn set(
        &self,
        extension_id: &str,
        area: StorageArea,
        items: HashMap<String, Value>,
    ) -> ExtensionResult<StorageChanges> {
        let mut changes = StorageChanges::new();

        // For session storage, use in-memory cache only
        if area == StorageArea::Session {
            let mut cache = self.session_cache.write();
            let ext_data = cache.entry(extension_id.to_string()).or_default();

            for (key, new_value) in items {
                let old_value = ext_data.insert(key.clone(), new_value.clone());
                changes.insert(
                    key,
                    StorageChange {
                        old_value,
                        new_value: Some(new_value),
                    },
                );
            }

            return Ok(changes);
        }

        // For local/sync, persist to disk
        let mut data = self.load_storage(extension_id, area).await?;

        for (key, new_value) in items {
            let old_value = data.insert(key.clone(), new_value.clone());
            changes.insert(
                key,
                StorageChange {
                    old_value,
                    new_value: Some(new_value),
                },
            );
        }

        self.save_storage(extension_id, area, &data).await?;

        Ok(changes)
    }

    /// Remove values from storage
    pub async fn remove(
        &self,
        extension_id: &str,
        area: StorageArea,
        keys: Vec<String>,
    ) -> ExtensionResult<StorageChanges> {
        let mut changes = StorageChanges::new();

        // For session storage, use in-memory cache only
        if area == StorageArea::Session {
            let mut cache = self.session_cache.write();
            if let Some(ext_data) = cache.get_mut(extension_id) {
                for key in keys {
                    if let Some(old_value) = ext_data.remove(&key) {
                        changes.insert(
                            key,
                            StorageChange {
                                old_value: Some(old_value),
                                new_value: None,
                            },
                        );
                    }
                }
            }
            return Ok(changes);
        }

        // For local/sync, persist to disk
        let mut data = self.load_storage(extension_id, area).await?;

        for key in keys {
            if let Some(old_value) = data.remove(&key) {
                changes.insert(
                    key,
                    StorageChange {
                        old_value: Some(old_value),
                        new_value: None,
                    },
                );
            }
        }

        self.save_storage(extension_id, area, &data).await?;

        Ok(changes)
    }

    /// Clear all values from storage
    pub async fn clear(
        &self,
        extension_id: &str,
        area: StorageArea,
    ) -> ExtensionResult<StorageChanges> {
        let mut changes = StorageChanges::new();

        // For session storage, use in-memory cache only
        if area == StorageArea::Session {
            let mut cache = self.session_cache.write();
            if let Some(ext_data) = cache.remove(extension_id) {
                for (key, old_value) in ext_data {
                    changes.insert(
                        key,
                        StorageChange {
                            old_value: Some(old_value),
                            new_value: None,
                        },
                    );
                }
            }
            return Ok(changes);
        }

        // For local/sync, clear and save
        let data = self.load_storage(extension_id, area).await?;

        for (key, old_value) in data {
            changes.insert(
                key,
                StorageChange {
                    old_value: Some(old_value),
                    new_value: None,
                },
            );
        }

        self.save_storage(extension_id, area, &HashMap::new())
            .await?;

        Ok(changes)
    }

    /// Get bytes in use for storage
    pub async fn get_bytes_in_use(
        &self,
        extension_id: &str,
        area: StorageArea,
        keys: Option<Vec<String>>,
    ) -> ExtensionResult<usize> {
        let data = self.get(extension_id, area, keys).await?;
        let json = serde_json::to_string(&data)?;
        Ok(json.len())
    }
}

/// Storage API handler for JavaScript bridge
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum StorageRequest {
    /// Get values
    #[serde(rename = "get")]
    Get {
        area: StorageArea,
        keys: Option<Vec<String>>,
    },
    /// Set values
    #[serde(rename = "set")]
    Set {
        area: StorageArea,
        items: HashMap<String, Value>,
    },
    /// Remove values
    #[serde(rename = "remove")]
    Remove {
        area: StorageArea,
        keys: Vec<String>,
    },
    /// Clear all values
    #[serde(rename = "clear")]
    Clear { area: StorageArea },
    /// Get bytes in use
    #[serde(rename = "getBytesInUse")]
    GetBytesInUse {
        area: StorageArea,
        keys: Option<Vec<String>>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_storage_set_get() {
        let dir = tempdir().unwrap();
        let backend = StorageBackend::new(dir.path().to_path_buf());

        let mut items = HashMap::new();
        items.insert("key1".to_string(), Value::String("value1".to_string()));
        items.insert("key2".to_string(), Value::Number(42.into()));

        backend
            .set("test-ext", StorageArea::Local, items)
            .await
            .unwrap();

        let result = backend
            .get("test-ext", StorageArea::Local, None)
            .await
            .unwrap();
        assert_eq!(
            result.get("key1"),
            Some(&Value::String("value1".to_string()))
        );
        assert_eq!(result.get("key2"), Some(&Value::Number(42.into())));
    }

    #[tokio::test]
    async fn test_session_storage() {
        let dir = tempdir().unwrap();
        let backend = StorageBackend::new(dir.path().to_path_buf());

        let mut items = HashMap::new();
        items.insert(
            "session_key".to_string(),
            Value::String("session_value".to_string()),
        );

        backend
            .set("test-ext", StorageArea::Session, items)
            .await
            .unwrap();

        let result = backend
            .get("test-ext", StorageArea::Session, None)
            .await
            .unwrap();
        assert_eq!(
            result.get("session_key"),
            Some(&Value::String("session_value".to_string()))
        );

        // Session storage should not persist to disk
        let path = backend.get_storage_path("test-ext", StorageArea::Session);
        assert!(!path.exists());
    }
}
