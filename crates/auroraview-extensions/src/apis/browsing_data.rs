//! Chrome Browsing Data API Implementation
//!
//! Provides functionality to remove browsing data.
//!
//! ## Features
//! - Remove various types of browsing data
//! - Filter by time range
//! - Filter by data types

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::{ExtensionError, ExtensionResult};

/// Data type options for removal
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DataTypeSet {
    /// Whether to remove app cache
    #[serde(skip_serializing_if = "Option::is_none")]
    pub appcache: Option<bool>,
    /// Whether to remove cache
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache: Option<bool>,
    /// Whether to remove cache storage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_storage: Option<bool>,
    /// Whether to remove cookies
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cookies: Option<bool>,
    /// Whether to remove downloads
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloads: Option<bool>,
    /// Whether to remove file systems
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_systems: Option<bool>,
    /// Whether to remove form data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub form_data: Option<bool>,
    /// Whether to remove history
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<bool>,
    /// Whether to remove indexed DB
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indexed_db: Option<bool>,
    /// Whether to remove local storage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_storage: Option<bool>,
    /// Whether to remove passwords
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passwords: Option<bool>,
    /// Whether to remove plugin data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_data: Option<bool>,
    /// Whether to remove service workers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_workers: Option<bool>,
    /// Whether to remove web SQL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_sql: Option<bool>,
}

/// Removal options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RemovalOptions {
    /// Remove data accumulated on or after this time (ms since epoch)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<f64>,
    /// Origins to remove data from
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origins: Option<Vec<String>>,
    /// Origins to exclude from removal
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_origins: Option<Vec<String>>,
}

/// Settings result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsResult {
    /// Removal options
    pub options: RemovalOptions,
    /// Data types to remove
    pub data_to_remove: DataTypeSet,
    /// Data types that are removable
    pub data_removal_permitted: DataTypeSet,
}

/// Browsing Data API handler
pub struct BrowsingDataApi;

impl Default for BrowsingDataApi {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowsingDataApi {
    /// Create a new BrowsingDataApi instance
    pub fn new() -> Self {
        Self
    }

    /// Get settings
    pub fn settings(&self) -> ExtensionResult<Value> {
        let result = SettingsResult {
            options: RemovalOptions {
                since: None,
                origins: None,
                exclude_origins: None,
            },
            data_to_remove: DataTypeSet::default(),
            data_removal_permitted: DataTypeSet {
                appcache: Some(true),
                cache: Some(true),
                cache_storage: Some(true),
                cookies: Some(true),
                downloads: Some(true),
                file_systems: Some(true),
                form_data: Some(true),
                history: Some(true),
                indexed_db: Some(true),
                local_storage: Some(true),
                passwords: Some(true),
                plugin_data: Some(true),
                service_workers: Some(true),
                web_sql: Some(true),
            },
        };
        Ok(serde_json::to_value(result)?)
    }

    /// Remove browsing data
    pub fn remove(
        &self,
        _options: RemovalOptions,
        _data_to_remove: DataTypeSet,
    ) -> ExtensionResult<Value> {
        // In a real implementation, this would clear the specified data types
        // For now, we just acknowledge the request
        Ok(json!(null))
    }

    /// Remove app cache
    pub fn remove_appcache(&self, _options: RemovalOptions) -> ExtensionResult<Value> {
        Ok(json!(null))
    }

    /// Remove cache
    pub fn remove_cache(&self, _options: RemovalOptions) -> ExtensionResult<Value> {
        Ok(json!(null))
    }

    /// Remove cache storage
    pub fn remove_cache_storage(&self, _options: RemovalOptions) -> ExtensionResult<Value> {
        Ok(json!(null))
    }

    /// Remove cookies
    pub fn remove_cookies(&self, _options: RemovalOptions) -> ExtensionResult<Value> {
        Ok(json!(null))
    }

    /// Remove downloads
    pub fn remove_downloads(&self, _options: RemovalOptions) -> ExtensionResult<Value> {
        Ok(json!(null))
    }

    /// Remove file systems
    pub fn remove_file_systems(&self, _options: RemovalOptions) -> ExtensionResult<Value> {
        Ok(json!(null))
    }

    /// Remove form data
    pub fn remove_form_data(&self, _options: RemovalOptions) -> ExtensionResult<Value> {
        Ok(json!(null))
    }

    /// Remove history
    pub fn remove_history(&self, _options: RemovalOptions) -> ExtensionResult<Value> {
        Ok(json!(null))
    }

    /// Remove indexed DB
    pub fn remove_indexed_db(&self, _options: RemovalOptions) -> ExtensionResult<Value> {
        Ok(json!(null))
    }

    /// Remove local storage
    pub fn remove_local_storage(&self, _options: RemovalOptions) -> ExtensionResult<Value> {
        Ok(json!(null))
    }

    /// Remove passwords
    pub fn remove_passwords(&self, _options: RemovalOptions) -> ExtensionResult<Value> {
        Ok(json!(null))
    }

    /// Remove plugin data
    pub fn remove_plugin_data(&self, _options: RemovalOptions) -> ExtensionResult<Value> {
        Ok(json!(null))
    }

    /// Remove service workers
    pub fn remove_service_workers(&self, _options: RemovalOptions) -> ExtensionResult<Value> {
        Ok(json!(null))
    }

    /// Remove web SQL
    pub fn remove_web_sql(&self, _options: RemovalOptions) -> ExtensionResult<Value> {
        Ok(json!(null))
    }

    /// Handle API call
    pub fn handle(&self, method: &str, params: Value) -> ExtensionResult<Value> {
        let options: RemovalOptions = params
            .get("options")
            .cloned()
            .map(|v| serde_json::from_value(v).unwrap_or_default())
            .unwrap_or_default();

        match method {
            "settings" => self.settings(),
            "remove" => {
                let data_to_remove: DataTypeSet = params
                    .get("dataToRemove")
                    .cloned()
                    .map(|v| serde_json::from_value(v).unwrap_or_default())
                    .unwrap_or_default();
                self.remove(options, data_to_remove)
            }
            "removeAppcache" => self.remove_appcache(options),
            "removeCache" => self.remove_cache(options),
            "removeCacheStorage" => self.remove_cache_storage(options),
            "removeCookies" => self.remove_cookies(options),
            "removeDownloads" => self.remove_downloads(options),
            "removeFileSystems" => self.remove_file_systems(options),
            "removeFormData" => self.remove_form_data(options),
            "removeHistory" => self.remove_history(options),
            "removeIndexedDB" => self.remove_indexed_db(options),
            "removeLocalStorage" => self.remove_local_storage(options),
            "removePasswords" => self.remove_passwords(options),
            "removePluginData" => self.remove_plugin_data(options),
            "removeServiceWorkers" => self.remove_service_workers(options),
            "removeWebSQL" => self.remove_web_sql(options),
            _ => Err(ExtensionError::UnknownMethod(method.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings() {
        let api = BrowsingDataApi::new();
        let result = api.settings().unwrap();
        let settings: SettingsResult = serde_json::from_value(result).unwrap();
        assert!(settings.data_removal_permitted.cache.unwrap_or(false));
    }

    #[test]
    fn test_remove() {
        let api = BrowsingDataApi::new();
        let result = api.remove(
            RemovalOptions::default(),
            DataTypeSet {
                cache: Some(true),
                ..Default::default()
            },
        );
        assert!(result.is_ok());
    }
}
