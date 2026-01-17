//! Settings storage with nested key support.

use std::collections::HashMap;

use crate::value::SettingValue;

/// In-memory settings store with nested key support.
#[derive(Debug, Default, Clone)]
pub struct SettingsStore {
    values: HashMap<String, SettingValue>,
}

impl SettingsStore {
    /// Creates a new empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets a value by key.
    pub fn get(&self, key: &str) -> Option<&SettingValue> {
        self.values.get(key)
    }

    /// Sets a value by key.
    pub fn set(&mut self, key: impl Into<String>, value: SettingValue) {
        self.values.insert(key.into(), value);
    }

    /// Removes a value by key.
    pub fn remove(&mut self, key: &str) -> Option<SettingValue> {
        self.values.remove(key)
    }

    /// Returns true if the key exists.
    pub fn contains(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    /// Returns all keys.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.values.keys().map(|s| s.as_str())
    }

    /// Returns keys matching a prefix.
    pub fn keys_with_prefix(&self, prefix: &str) -> impl Iterator<Item = &str> {
        let prefix = prefix.to_string();
        self.values
            .keys()
            .filter(move |k| k.starts_with(&prefix))
            .map(|s| s.as_str())
    }

    /// Clears all values.
    pub fn clear(&mut self) {
        self.values.clear();
    }

    /// Returns the number of settings.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns true if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Merges another store into this one.
    pub fn merge(&mut self, other: SettingsStore) {
        self.values.extend(other.values);
    }

    /// Converts to a HashMap.
    pub fn into_map(self) -> HashMap<String, SettingValue> {
        self.values
    }

    /// Creates from a HashMap.
    pub fn from_map(map: HashMap<String, SettingValue>) -> Self {
        Self { values: map }
    }
}

impl From<HashMap<String, SettingValue>> for SettingsStore {
    fn from(map: HashMap<String, SettingValue>) -> Self {
        Self::from_map(map)
    }
}

impl IntoIterator for SettingsStore {
    type Item = (String, SettingValue);
    type IntoIter = std::collections::hash_map::IntoIter<String, SettingValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}

impl<'a> IntoIterator for &'a SettingsStore {
    type Item = (&'a String, &'a SettingValue);
    type IntoIter = std::collections::hash_map::Iter<'a, String, SettingValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.iter()
    }
}
