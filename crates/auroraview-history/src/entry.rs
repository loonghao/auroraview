//! History entry data structure

use crate::HistoryId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Unique identifier (based on URL hash)
    pub id: HistoryId,

    /// Page URL
    pub url: String,

    /// Page title
    pub title: String,

    /// Favicon URL (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,

    /// Number of visits
    pub visit_count: u32,

    /// First visit timestamp
    pub first_visit: DateTime<Utc>,

    /// Last visit timestamp
    pub last_visit: DateTime<Utc>,

    /// Typed count (direct URL entry vs link click)
    #[serde(default)]
    pub typed_count: u32,
}

impl HistoryEntry {
    /// Create a new history entry
    pub fn new(url: impl Into<String>, title: impl Into<String>) -> Self {
        let url = url.into();
        let now = Utc::now();

        Self {
            id: Self::generate_id(&url),
            url,
            title: title.into(),
            favicon: None,
            visit_count: 1,
            first_visit: now,
            last_visit: now,
            typed_count: 0,
        }
    }

    /// Generate ID from URL
    fn generate_id(url: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Record a visit
    pub fn record_visit(&mut self) {
        self.visit_count += 1;
        self.last_visit = Utc::now();
    }

    /// Record a typed visit (user typed URL directly)
    pub fn record_typed_visit(&mut self) {
        self.visit_count += 1;
        self.typed_count += 1;
        self.last_visit = Utc::now();
    }

    /// Update the title
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    /// Update the favicon
    pub fn set_favicon(&mut self, favicon: Option<String>) {
        self.favicon = favicon;
    }

    /// Get the domain from URL
    pub fn domain(&self) -> Option<&str> {
        self.url
            .strip_prefix("https://")
            .or_else(|| self.url.strip_prefix("http://"))
            .and_then(|s| s.split('/').next())
    }

    /// Check if entry matches a search query
    pub fn matches(&self, query: &str) -> bool {
        let query = query.to_lowercase();
        self.title.to_lowercase().contains(&query) || self.url.to_lowercase().contains(&query)
    }

    /// Calculate relevance score for search
    pub fn relevance_score(&self, query: &str) -> u32 {
        let query = query.to_lowercase();
        let mut score = 0u32;

        // Title exact match
        if self.title.to_lowercase() == query {
            score += 100;
        } else if self.title.to_lowercase().starts_with(&query) {
            score += 50;
        } else if self.title.to_lowercase().contains(&query) {
            score += 20;
        }

        // URL match
        if self.url.to_lowercase().contains(&query) {
            score += 15;
        }

        // Boost by visit count (log scale)
        score += (self.visit_count as f64).log2() as u32 * 5;

        // Boost by typed count
        score += self.typed_count * 10;

        // Boost recent visits
        let days_ago = (Utc::now() - self.last_visit).num_days();
        if days_ago < 1 {
            score += 20;
        } else if days_ago < 7 {
            score += 10;
        } else if days_ago < 30 {
            score += 5;
        }

        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_creation() {
        let entry = HistoryEntry::new("https://github.com", "GitHub");

        assert_eq!(entry.url, "https://github.com");
        assert_eq!(entry.title, "GitHub");
        assert_eq!(entry.visit_count, 1);
        assert!(!entry.id.is_empty());
    }

    #[test]
    fn test_record_visit() {
        let mut entry = HistoryEntry::new("https://github.com", "GitHub");
        let initial_visit = entry.last_visit;

        std::thread::sleep(std::time::Duration::from_millis(10));
        entry.record_visit();

        assert_eq!(entry.visit_count, 2);
        assert!(entry.last_visit > initial_visit);
    }

    #[test]
    fn test_domain() {
        let entry = HistoryEntry::new("https://github.com/rust-lang/rust", "Rust");
        assert_eq!(entry.domain(), Some("github.com"));

        let entry = HistoryEntry::new("http://example.com/path", "Example");
        assert_eq!(entry.domain(), Some("example.com"));
    }

    #[test]
    fn test_matches() {
        let entry = HistoryEntry::new("https://github.com", "GitHub");

        assert!(entry.matches("git"));
        assert!(entry.matches("hub"));
        assert!(entry.matches("github.com"));
        assert!(!entry.matches("gitlab"));
    }
}
