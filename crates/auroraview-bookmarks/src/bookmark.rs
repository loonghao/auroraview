//! Bookmark data structure

use crate::BookmarkId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A bookmark entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    /// Unique identifier
    pub id: BookmarkId,

    /// Display title
    pub title: String,

    /// URL
    pub url: String,

    /// Favicon URL (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,

    /// Parent folder ID (None for root)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<BookmarkId>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last modified timestamp
    pub modified_at: DateTime<Utc>,

    /// Position in parent folder (for ordering)
    #[serde(default)]
    pub position: u32,

    /// Tags for categorization
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

impl Bookmark {
    /// Create a new bookmark
    pub fn new(title: impl Into<String>, url: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.into(),
            url: url.into(),
            favicon: None,
            parent_id: None,
            created_at: now,
            modified_at: now,
            position: 0,
            tags: Vec::new(),
        }
    }

    /// Create a bookmark with a specific ID
    pub fn with_id(
        id: impl Into<String>,
        title: impl Into<String>,
        url: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: id.into(),
            title: title.into(),
            url: url.into(),
            favicon: None,
            parent_id: None,
            created_at: now,
            modified_at: now,
            position: 0,
            tags: Vec::new(),
        }
    }

    /// Set parent folder
    pub fn with_parent(mut self, parent_id: impl Into<BookmarkId>) -> Self {
        self.parent_id = Some(parent_id.into());
        self
    }

    /// Set favicon
    pub fn with_favicon(mut self, favicon: impl Into<String>) -> Self {
        self.favicon = Some(favicon.into());
        self
    }

    /// Set position
    pub fn with_position(mut self, position: u32) -> Self {
        self.position = position;
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Set tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Update the title
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
        self.modified_at = Utc::now();
    }

    /// Update the URL
    pub fn set_url(&mut self, url: impl Into<String>) {
        self.url = url.into();
        self.modified_at = Utc::now();
    }

    /// Update the favicon
    pub fn set_favicon(&mut self, favicon: Option<String>) {
        self.favicon = favicon;
        self.modified_at = Utc::now();
    }

    /// Move to a different folder
    pub fn set_parent(&mut self, parent_id: Option<BookmarkId>) {
        self.parent_id = parent_id;
        self.modified_at = Utc::now();
    }

    /// Check if bookmark matches a search query
    pub fn matches(&self, query: &str) -> bool {
        let query = query.to_lowercase();
        self.title.to_lowercase().contains(&query)
            || self.url.to_lowercase().contains(&query)
            || self.tags.iter().any(|t| t.to_lowercase().contains(&query))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bookmark_creation() {
        let bookmark = Bookmark::new("GitHub", "https://github.com");

        assert_eq!(bookmark.title, "GitHub");
        assert_eq!(bookmark.url, "https://github.com");
        assert!(bookmark.parent_id.is_none());
        assert!(!bookmark.id.is_empty());
    }

    #[test]
    fn test_bookmark_with_methods() {
        let bookmark = Bookmark::new("Test", "https://test.com")
            .with_favicon("https://test.com/favicon.ico")
            .with_position(5)
            .with_tag("test");

        assert_eq!(
            bookmark.favicon,
            Some("https://test.com/favicon.ico".to_string())
        );
        assert_eq!(bookmark.position, 5);
        assert_eq!(bookmark.tags, vec!["test".to_string()]);
    }

    #[test]
    fn test_bookmark_matches() {
        let bookmark = Bookmark::new("GitHub", "https://github.com").with_tag("code");

        assert!(bookmark.matches("git"));
        assert!(bookmark.matches("hub"));
        assert!(bookmark.matches("github.com"));
        assert!(bookmark.matches("code"));
        assert!(!bookmark.matches("gitlab"));
    }
}
