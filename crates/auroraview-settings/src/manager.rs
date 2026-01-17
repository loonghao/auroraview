//! Settings manager with schema validation and persistence.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use parking_lot::RwLock;

use crate::error::{Result, SettingsError};
use crate::schema::{SchemaRegistry, SettingSchema};
use crate::store::SettingsStore;
use crate::value::SettingValue;

/// Callback type for setting change notifications.
pub type ChangeCallback = Box<dyn Fn(&str, &SettingValue, &SettingValue) + Send + Sync>;

/// Internal state for the settings manager.
struct SettingsState {
    store: SettingsStore,
    defaults: SettingsStore,
    registry: SchemaRegistry,
    callbacks: Vec<ChangeCallback>,
}

/// Main settings manager with validation and persistence.
pub struct SettingsManager {
    inner: Arc<RwLock<SettingsState>>,
    storage_path: Option<PathBuf>,
}

impl SettingsManager {
    /// Creates a new settings manager.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(SettingsState {
                store: SettingsStore::new(),
                defaults: SettingsStore::new(),
                registry: SchemaRegistry::new(),
                callbacks: Vec::new(),
            })),
            storage_path: None,
        }
    }

    /// Creates a new settings manager with a storage path.
    pub fn with_storage(path: impl AsRef<Path>) -> Self {
        let mut manager = Self::new();
        manager.storage_path = Some(path.as_ref().to_path_buf());
        manager
    }

    /// Registers a setting schema.
    pub fn register_schema(&self, schema: SettingSchema) {
        let mut state = self.inner.write();
        // Set the default value
        state.defaults.set(schema.key.clone(), schema.default.clone());
        state.registry.register(schema);
    }

    /// Registers multiple schemas.
    pub fn register_schemas(&self, schemas: impl IntoIterator<Item = SettingSchema>) {
        for schema in schemas {
            self.register_schema(schema);
        }
    }

    /// Gets the schema registry.
    pub fn registry(&self) -> SchemaRegistry {
        let state = self.inner.read();
        // Clone the registry for external use
        let mut registry = SchemaRegistry::new();
        for schema in state.registry.all() {
            registry.register(schema.clone());
        }
        registry
    }

    /// Gets a setting value, falling back to default if not set.
    pub fn get(&self, key: &str) -> Option<SettingValue> {
        let state = self.inner.read();
        state
            .store
            .get(key)
            .or_else(|| state.defaults.get(key))
            .cloned()
    }

    /// Gets a string setting.
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.get(key).and_then(|v| v.as_str().map(String::from))
    }

    /// Gets a boolean setting.
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(|v| v.as_bool())
    }

    /// Gets an integer setting.
    pub fn get_integer(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(|v| v.as_integer())
    }

    /// Gets a float setting.
    pub fn get_float(&self, key: &str) -> Option<f64> {
        self.get(key).and_then(|v| v.as_float())
    }

    /// Sets a setting value with validation.
    pub fn set(&self, key: impl Into<String>, value: SettingValue) -> Result<()> {
        let key = key.into();
        let mut state = self.inner.write();

        // Validate against schema if registered
        if let Some(schema) = state.registry.get(&key) {
            schema
                .validate(&value)
                .map_err(|reason| SettingsError::ValidationFailed {
                    key: key.clone(),
                    reason,
                })?;
        }

        // Get old value for change notification
        let old_value = state
            .store
            .get(&key)
            .or_else(|| state.defaults.get(&key))
            .cloned()
            .unwrap_or(SettingValue::Null);

        // Set the new value
        state.store.set(key.clone(), value.clone());

        // Notify callbacks
        for callback in &state.callbacks {
            callback(&key, &old_value, &value);
        }

        Ok(())
    }

    /// Resets a setting to its default value.
    pub fn reset(&self, key: &str) -> Result<()> {
        let mut state = self.inner.write();

        let old_value = state.store.get(key).cloned();
        let default_value = state.defaults.get(key).cloned();

        if let Some(old) = old_value {
            state.store.remove(key);

            if let Some(default) = &default_value {
                for callback in &state.callbacks {
                    callback(key, &old, default);
                }
            }
        }

        Ok(())
    }

    /// Resets all settings to defaults.
    pub fn reset_all(&self) {
        let mut state = self.inner.write();
        state.store.clear();
    }

    /// Registers a change callback.
    pub fn on_change<F>(&self, callback: F)
    where
        F: Fn(&str, &SettingValue, &SettingValue) + Send + Sync + 'static,
    {
        let mut state = self.inner.write();
        state.callbacks.push(Box::new(callback));
    }

    /// Loads settings from the storage path.
    pub fn load(&self) -> Result<()> {
        let Some(path) = &self.storage_path else {
            return Ok(());
        };

        if !path.exists() {
            return Ok(());
        }

        let content = std::fs::read_to_string(path)?;
        let map: std::collections::HashMap<String, SettingValue> = serde_json::from_str(&content)?;

        let mut state = self.inner.write();
        state.store = SettingsStore::from_map(map);

        Ok(())
    }

    /// Saves settings to the storage path.
    pub fn save(&self) -> Result<()> {
        let Some(path) = &self.storage_path else {
            return Ok(());
        };

        let state = self.inner.read();
        let map = state.store.clone().into_map();
        let content = serde_json::to_string_pretty(&map)?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(path, content)?;

        Ok(())
    }

    /// Returns all non-default settings.
    pub fn user_settings(&self) -> SettingsStore {
        let state = self.inner.read();
        state.store.clone()
    }

    /// Returns all settings including defaults.
    pub fn all_settings(&self) -> SettingsStore {
        let state = self.inner.read();
        let mut result = state.defaults.clone();
        result.merge(state.store.clone());
        result
    }
}

impl Default for SettingsManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for SettingsManager {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            storage_path: self.storage_path.clone(),
        }
    }
}
