//! History manager implementation

use crate::{HistoryEntry, HistoryId, Result, SearchOptions, SearchResult};
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// History manager
///
/// Manages browsing history with optional persistence.
#[derive(Debug)]
pub struct HistoryManager {
    inner: Arc<RwLock<HistoryStore>>,
    storage_path: Option<PathBuf>,
    max_entries: usize,
}

#[derive(Debug, Default)]
struct HistoryStore {
    entries: HashMap<HistoryId, HistoryEntry>,
}

impl HistoryManager {
    /// Default maximum entries
    const DEFAULT_MAX_ENTRIES: usize = 10000;

    /// Create a new history manager
    ///
    /// If `data_dir` is provided, history will be persisted to disk.
    pub fn new(data_dir: Option<&Path>) -> Self {
        let storage_path = data_dir.map(|p| p.join("history.json"));

        let mut manager = Self {
            inner: Arc::new(RwLock::new(HistoryStore::default())),
            storage_path,
            max_entries: Self::DEFAULT_MAX_ENTRIES,
        };

        // Try to load existing history
        if let Some(ref path) = manager.storage_path {
            if path.exists() {
                let _ = manager.load();
            }
        }

        manager
    }

    /// Set maximum entries
    pub fn with_max_entries(mut self, max: usize) -> Self {
        self.max_entries = max;
        self
    }

    /// Record a visit
    pub fn visit(&self, url: impl Into<String>, title: impl Into<String>) -> HistoryId {
        let url = url.into();
        let title = title.into();

        let mut store = self.inner.write().unwrap();

        // Check if entry exists
        if let Some(entry) = store.entries.values_mut().find(|e| e.url == url) {
            entry.record_visit();
            if !title.is_empty() {
                entry.set_title(&title);
            }
            let id = entry.id.clone();
            drop(store);
            let _ = self.save();
            return id;
        }

        // Create new entry
        let entry = HistoryEntry::new(&url, title);
        let id = entry.id.clone();
        store.entries.insert(id.clone(), entry);

        // Enforce max entries
        self.enforce_max_entries(&mut store);

        drop(store);
        let _ = self.save();
        id
    }

    /// Record a typed visit (user typed URL directly)
    pub fn typed_visit(&self, url: impl Into<String>, title: impl Into<String>) -> HistoryId {
        let url = url.into();
        let title = title.into();

        let mut store = self.inner.write().unwrap();

        // Check if entry exists
        if let Some(entry) = store.entries.values_mut().find(|e| e.url == url) {
            entry.record_typed_visit();
            if !title.is_empty() {
                entry.set_title(&title);
            }
            let id = entry.id.clone();
            drop(store);
            let _ = self.save();
            return id;
        }

        // Create new entry with typed visit
        let mut entry = HistoryEntry::new(&url, title);
        entry.typed_count = 1;
        let id = entry.id.clone();
        store.entries.insert(id.clone(), entry);

        self.enforce_max_entries(&mut store);

        drop(store);
        let _ = self.save();
        id
    }

    /// Enforce maximum entries limit
    fn enforce_max_entries(&self, store: &mut HistoryStore) {
        if store.entries.len() <= self.max_entries {
            return;
        }

        // Remove oldest entries
        let mut entries: Vec<_> = store.entries.values().collect();
        entries.sort_by_key(|e| e.last_visit);

        let to_remove = store.entries.len() - self.max_entries;
        let ids_to_remove: Vec<_> = entries
            .iter()
            .take(to_remove)
            .map(|e| e.id.clone())
            .collect();

        for id in ids_to_remove {
            store.entries.remove(&id);
        }
    }

    /// Get a history entry by ID
    pub fn get(&self, id: &str) -> Option<HistoryEntry> {
        let store = self.inner.read().unwrap();
        store.entries.get(id).cloned()
    }

    /// Get entry by URL
    pub fn get_by_url(&self, url: &str) -> Option<HistoryEntry> {
        let store = self.inner.read().unwrap();
        store.entries.values().find(|e| e.url == url).cloned()
    }

    /// Delete a history entry
    pub fn delete(&self, id: &str) -> bool {
        let mut store = self.inner.write().unwrap();
        let removed = store.entries.remove(id).is_some();
        drop(store);

        if removed {
            let _ = self.save();
        }
        removed
    }

    /// Delete entries by URL
    pub fn delete_url(&self, url: &str) -> bool {
        let mut store = self.inner.write().unwrap();
        let initial_len = store.entries.len();
        store.entries.retain(|_, e| e.url != url);
        let removed = store.entries.len() < initial_len;
        drop(store);

        if removed {
            let _ = self.save();
        }
        removed
    }

    /// Get recent history entries
    pub fn recent(&self, limit: usize) -> Vec<HistoryEntry> {
        let store = self.inner.read().unwrap();
        let mut entries: Vec<_> = store.entries.values().cloned().collect();
        entries.sort_by(|a, b| b.last_visit.cmp(&a.last_visit));
        entries.truncate(limit);
        entries
    }

    /// Get all history entries
    pub fn all(&self) -> Vec<HistoryEntry> {
        let store = self.inner.read().unwrap();
        store.entries.values().cloned().collect()
    }

    /// Search history
    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        self.search_with_options(query, SearchOptions::default())
    }

    /// Search history with options
    pub fn search_with_options(&self, query: &str, options: SearchOptions) -> Vec<SearchResult> {
        let store = self.inner.read().unwrap();

        let mut results: Vec<_> = store
            .entries
            .values()
            .filter(|e| e.matches(query) && options.matches(e))
            .map(|e| SearchResult::new(e.clone(), query))
            .collect();

        // Sort by relevance
        results.sort_by(|a, b| b.score.cmp(&a.score));

        // Apply limit
        if let Some(limit) = options.limit {
            results.truncate(limit);
        }

        results
    }

    /// Get frequently visited sites
    pub fn frequent(&self, limit: usize) -> Vec<HistoryEntry> {
        let store = self.inner.read().unwrap();
        let mut entries: Vec<_> = store.entries.values().cloned().collect();
        entries.sort_by(|a, b| b.visit_count.cmp(&a.visit_count));
        entries.truncate(limit);
        entries
    }

    /// Get entries by domain
    pub fn by_domain(&self, domain: &str) -> Vec<HistoryEntry> {
        let store = self.inner.read().unwrap();
        store
            .entries
            .values()
            .filter(|e| e.domain() == Some(domain))
            .cloned()
            .collect()
    }

    /// Get entries in date range
    pub fn in_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<HistoryEntry> {
        let store = self.inner.read().unwrap();
        store
            .entries
            .values()
            .filter(|e| e.last_visit >= start && e.last_visit <= end)
            .cloned()
            .collect()
    }

    /// Get entries from today
    pub fn today(&self) -> Vec<HistoryEntry> {
        let now = Utc::now();
        let start = now - Duration::days(1);
        self.in_range(start, now)
    }

    /// Get entries from this week
    pub fn this_week(&self) -> Vec<HistoryEntry> {
        let now = Utc::now();
        let start = now - Duration::weeks(1);
        self.in_range(start, now)
    }

    /// Get entries from this month
    pub fn this_month(&self) -> Vec<HistoryEntry> {
        let now = Utc::now();
        let start = now - Duration::days(30);
        self.in_range(start, now)
    }

    /// Delete entries older than specified days
    pub fn delete_older_than(&self, days: i64) -> usize {
        let cutoff = Utc::now() - Duration::days(days);

        let mut store = self.inner.write().unwrap();
        let initial_len = store.entries.len();
        store.entries.retain(|_, e| e.last_visit >= cutoff);
        let removed = initial_len - store.entries.len();
        drop(store);

        if removed > 0 {
            let _ = self.save();
        }
        removed
    }

    /// Delete all history for a domain
    pub fn delete_domain(&self, domain: &str) -> usize {
        let mut store = self.inner.write().unwrap();
        let initial_len = store.entries.len();
        store.entries.retain(|_, e| e.domain() != Some(domain));
        let removed = initial_len - store.entries.len();
        drop(store);

        if removed > 0 {
            let _ = self.save();
        }
        removed
    }

    /// Clear all history
    pub fn clear(&self) {
        let mut store = self.inner.write().unwrap();
        store.entries.clear();
        drop(store);
        let _ = self.save();
    }

    /// Get entry count
    pub fn count(&self) -> usize {
        let store = self.inner.read().unwrap();
        store.entries.len()
    }

    // ========== Persistence ==========

    /// Save history to disk
    pub fn save(&self) -> Result<()> {
        let Some(ref path) = self.storage_path else {
            return Ok(());
        };

        let store = self.inner.read().unwrap();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string(&store.entries)?;
        std::fs::write(path, json)?;

        Ok(())
    }

    /// Load history from disk
    pub fn load(&mut self) -> Result<()> {
        let Some(ref path) = self.storage_path else {
            return Ok(());
        };

        if !path.exists() {
            return Ok(());
        }

        let json = std::fs::read_to_string(path)?;
        let entries: HashMap<HistoryId, HistoryEntry> = serde_json::from_str(&json)?;

        let mut store = self.inner.write().unwrap();
        store.entries = entries;

        Ok(())
    }

    /// Export history to JSON string
    pub fn export(&self) -> Result<String> {
        let store = self.inner.read().unwrap();
        Ok(serde_json::to_string_pretty(&store.entries)?)
    }

    /// Import history from JSON string
    pub fn import(&self, json: &str) -> Result<()> {
        let entries: HashMap<HistoryId, HistoryEntry> = serde_json::from_str(json)?;

        let mut store = self.inner.write().unwrap();
        store.entries.extend(entries);
        drop(store);

        let _ = self.save();
        Ok(())
    }
}

impl Clone for HistoryManager {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            storage_path: self.storage_path.clone(),
            max_entries: self.max_entries,
        }
    }
}

impl Default for HistoryManager {
    fn default() -> Self {
        Self::new(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visit() {
        let manager = HistoryManager::new(None);

        let id = manager.visit("https://github.com", "GitHub");
        let entry = manager.get(&id).unwrap();

        assert_eq!(entry.url, "https://github.com");
        assert_eq!(entry.title, "GitHub");
        assert_eq!(entry.visit_count, 1);
    }

    #[test]
    fn test_multiple_visits() {
        let manager = HistoryManager::new(None);

        manager.visit("https://github.com", "GitHub");
        manager.visit("https://github.com", "GitHub - Updated");

        let entry = manager.get_by_url("https://github.com").unwrap();
        assert_eq!(entry.visit_count, 2);
        assert_eq!(entry.title, "GitHub - Updated");
    }

    #[test]
    fn test_search() {
        let manager = HistoryManager::new(None);

        manager.visit("https://github.com", "GitHub");
        manager.visit("https://gitlab.com", "GitLab");
        manager.visit("https://rust-lang.org", "Rust");

        let results = manager.search("git");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_recent() {
        let manager = HistoryManager::new(None);

        manager.visit("https://first.com", "First");
        std::thread::sleep(std::time::Duration::from_millis(10));
        manager.visit("https://second.com", "Second");
        std::thread::sleep(std::time::Duration::from_millis(10));
        manager.visit("https://third.com", "Third");

        let recent = manager.recent(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].url, "https://third.com");
    }

    #[test]
    fn test_frequent() {
        let manager = HistoryManager::new(None);

        manager.visit("https://once.com", "Once");
        manager.visit("https://twice.com", "Twice");
        manager.visit("https://twice.com", "Twice");
        manager.visit("https://thrice.com", "Thrice");
        manager.visit("https://thrice.com", "Thrice");
        manager.visit("https://thrice.com", "Thrice");

        let frequent = manager.frequent(2);
        assert_eq!(frequent.len(), 2);
        assert_eq!(frequent[0].url, "https://thrice.com");
        assert_eq!(frequent[1].url, "https://twice.com");
    }

    #[test]
    fn test_delete() {
        let manager = HistoryManager::new(None);

        let id = manager.visit("https://github.com", "GitHub");
        assert!(manager.get(&id).is_some());

        assert!(manager.delete(&id));
        assert!(manager.get(&id).is_none());
    }

    #[test]
    fn test_clear() {
        let manager = HistoryManager::new(None);

        manager.visit("https://github.com", "GitHub");
        manager.visit("https://gitlab.com", "GitLab");
        assert_eq!(manager.count(), 2);

        manager.clear();
        assert_eq!(manager.count(), 0);
    }
}
