//! History management

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;

/// A history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// URL visited
    pub url: String,
    /// Page title
    pub title: String,
    /// Visit timestamp
    pub visited_at: DateTime<Utc>,
    /// Number of visits to this URL
    pub visit_count: u32,
    /// Favicon URL (optional)
    pub favicon: Option<String>,
}

impl HistoryEntry {
    /// Create a new history entry
    pub fn new(url: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            title: title.into(),
            visited_at: Utc::now(),
            visit_count: 1,
            favicon: None,
        }
    }

    /// Set favicon
    pub fn with_favicon(mut self, favicon: impl Into<String>) -> Self {
        self.favicon = Some(favicon.into());
        self
    }
}

/// History storage
#[derive(Debug, Default, Serialize, Deserialize)]
struct HistoryStorage {
    entries: VecDeque<HistoryEntry>,
    max_entries: usize,
}

impl HistoryStorage {
    fn new(max_entries: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            max_entries,
        }
    }
}

/// History manager
pub struct HistoryManager {
    storage: RwLock<HistoryStorage>,
    data_path: Option<PathBuf>,
    enabled: bool,
}

impl HistoryManager {
    /// Create a new history manager
    pub fn new(data_dir: Option<&str>, max_entries: usize, enabled: bool) -> Self {
        let data_path = data_dir
            .map(PathBuf::from)
            .or_else(|| dirs::data_dir().map(|d| d.join("auroraview").join("browser")))
            .map(|d| d.join("history.json"));

        let storage = if enabled {
            data_path
                .as_ref()
                .and_then(|p| fs::read_to_string(p).ok())
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_else(|| HistoryStorage::new(max_entries))
        } else {
            HistoryStorage::new(max_entries)
        };

        Self {
            storage: RwLock::new(storage),
            data_path,
            enabled,
        }
    }

    /// Add a history entry
    pub fn add(&self, url: &str, title: &str) {
        if !self.enabled {
            return;
        }

        // Skip internal URLs
        if url.starts_with("about:") || url.starts_with("data:") {
            return;
        }

        {
            let mut storage = self.storage.write();

            // Check if URL already exists and update it
            if let Some(existing) = storage.entries.iter_mut().find(|e| e.url == url) {
                existing.title = title.to_string();
                existing.visited_at = Utc::now();
                existing.visit_count += 1;
            } else {
                // Add new entry
                let entry = HistoryEntry::new(url, title);
                storage.entries.push_front(entry);

                // Remove oldest entries if over limit
                while storage.entries.len() > storage.max_entries {
                    storage.entries.pop_back();
                }
            }
        }

        self.save();
    }

    /// Get history entries (most recent first)
    pub fn get(&self, limit: usize) -> Vec<HistoryEntry> {
        let storage = self.storage.read();
        storage.entries.iter().take(limit).cloned().collect()
    }

    /// Get all history entries
    pub fn all(&self) -> Vec<HistoryEntry> {
        self.storage.read().entries.iter().cloned().collect()
    }

    /// Search history
    pub fn search(&self, query: &str, limit: usize) -> Vec<HistoryEntry> {
        let query = query.to_lowercase();
        let storage = self.storage.read();
        storage
            .entries
            .iter()
            .filter(|e| {
                e.url.to_lowercase().contains(&query) || e.title.to_lowercase().contains(&query)
            })
            .take(limit)
            .cloned()
            .collect()
    }

    /// Remove a history entry by URL
    pub fn remove(&self, url: &str) -> bool {
        let removed = {
            let mut storage = self.storage.write();
            let len_before = storage.entries.len();
            storage.entries.retain(|e| e.url != url);
            storage.entries.len() < len_before
        };
        if removed {
            self.save();
        }
        removed
    }

    /// Clear all history
    pub fn clear(&self) {
        {
            let mut storage = self.storage.write();
            storage.entries.clear();
        }
        self.save();
    }

    /// Clear history older than a date
    pub fn clear_before(&self, before: DateTime<Utc>) {
        {
            let mut storage = self.storage.write();
            storage.entries.retain(|e| e.visited_at >= before);
        }
        self.save();
    }

    /// Get history count
    pub fn count(&self) -> usize {
        self.storage.read().entries.len()
    }

    /// Check if history is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enable/disable history
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.clear();
        }
    }

    /// Save to disk
    fn save(&self) {
        if !self.enabled {
            return;
        }

        if let Some(path) = &self.data_path {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let storage = self.storage.read();
            if let Ok(json) = serde_json::to_string_pretty(&*storage) {
                let _ = fs::write(path, json);
            }
        }
    }

    /// Get entries for today
    pub fn today(&self) -> Vec<HistoryEntry> {
        let today = Utc::now().date_naive();
        let storage = self.storage.read();
        storage
            .entries
            .iter()
            .filter(|e| e.visited_at.date_naive() == today)
            .cloned()
            .collect()
    }

    /// Get entries grouped by date
    pub fn grouped_by_date(&self) -> Vec<(String, Vec<HistoryEntry>)> {
        use std::collections::BTreeMap;

        let storage = self.storage.read();
        let mut groups: BTreeMap<String, Vec<HistoryEntry>> = BTreeMap::new();

        for entry in storage.entries.iter() {
            let date = entry.visited_at.format("%Y-%m-%d").to_string();
            groups.entry(date).or_default().push(entry.clone());
        }

        // Convert to vec and reverse (most recent first)
        groups.into_iter().rev().collect()
    }
}

impl Default for HistoryManager {
    fn default() -> Self {
        Self::new(None, 10000, true)
    }
}
