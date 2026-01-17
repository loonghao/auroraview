//! History search functionality

use crate::HistoryEntry;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Search options
#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    /// Maximum number of results
    pub limit: Option<usize>,

    /// Start date filter
    pub start_date: Option<DateTime<Utc>>,

    /// End date filter
    pub end_date: Option<DateTime<Utc>>,

    /// Domain filter
    pub domain: Option<String>,

    /// Minimum visit count
    pub min_visits: Option<u32>,
}

impl SearchOptions {
    /// Create new search options
    pub fn new() -> Self {
        Self::default()
    }

    /// Set result limit
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set start date
    pub fn start_date(mut self, date: DateTime<Utc>) -> Self {
        self.start_date = Some(date);
        self
    }

    /// Set end date
    pub fn end_date(mut self, date: DateTime<Utc>) -> Self {
        self.end_date = Some(date);
        self
    }

    /// Set domain filter
    pub fn domain(mut self, domain: impl Into<String>) -> Self {
        self.domain = Some(domain.into());
        self
    }

    /// Set minimum visits
    pub fn min_visits(mut self, count: u32) -> Self {
        self.min_visits = Some(count);
        self
    }

    /// Check if an entry matches the filters
    pub fn matches(&self, entry: &HistoryEntry) -> bool {
        // Date range filter
        if let Some(start) = self.start_date {
            if entry.last_visit < start {
                return false;
            }
        }
        if let Some(end) = self.end_date {
            if entry.last_visit > end {
                return false;
            }
        }

        // Domain filter
        if let Some(ref domain) = self.domain {
            if entry.domain() != Some(domain.as_str()) {
                return false;
            }
        }

        // Minimum visits filter
        if let Some(min) = self.min_visits {
            if entry.visit_count < min {
                return false;
            }
        }

        true
    }
}

/// Search result with relevance score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// The matching entry
    pub entry: HistoryEntry,

    /// Relevance score
    pub score: u32,
}

impl SearchResult {
    /// Create a search result
    pub fn new(entry: HistoryEntry, query: &str) -> Self {
        let score = entry.relevance_score(query);
        Self { entry, score }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_search_options_filters() {
        let entry = HistoryEntry::new("https://github.com", "GitHub");

        let opts = SearchOptions::new();
        assert!(opts.matches(&entry));

        let opts = SearchOptions::new().min_visits(10);
        assert!(!opts.matches(&entry));

        let opts = SearchOptions::new().domain("github.com");
        assert!(opts.matches(&entry));

        let opts = SearchOptions::new().domain("gitlab.com");
        assert!(!opts.matches(&entry));
    }

    #[test]
    fn test_date_filters() {
        let entry = HistoryEntry::new("https://github.com", "GitHub");

        let yesterday = Utc::now() - Duration::days(1);
        let tomorrow = Utc::now() + Duration::days(1);

        let opts = SearchOptions::new().start_date(yesterday);
        assert!(opts.matches(&entry));

        let opts = SearchOptions::new().end_date(tomorrow);
        assert!(opts.matches(&entry));

        let opts = SearchOptions::new().start_date(tomorrow);
        assert!(!opts.matches(&entry));
    }
}
