//! Bookmark folder data structure

use crate::BookmarkId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A bookmark folder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkFolder {
    /// Unique identifier
    pub id: BookmarkId,

    /// Folder name
    pub name: String,

    /// Parent folder ID (None for root)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<BookmarkId>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Position in parent folder (for ordering)
    #[serde(default)]
    pub position: u32,

    /// Icon (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

impl BookmarkFolder {
    /// Create a new folder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            parent_id: None,
            created_at: Utc::now(),
            position: 0,
            icon: None,
        }
    }

    /// Create a folder with a specific ID
    pub fn with_id(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            parent_id: None,
            created_at: Utc::now(),
            position: 0,
            icon: None,
        }
    }

    /// Set parent folder
    pub fn with_parent(mut self, parent_id: impl Into<BookmarkId>) -> Self {
        self.parent_id = Some(parent_id.into());
        self
    }

    /// Set position
    pub fn with_position(mut self, position: u32) -> Self {
        self.position = position;
        self
    }

    /// Set icon
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Update the name
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// Move to a different parent folder
    pub fn set_parent(&mut self, parent_id: Option<BookmarkId>) {
        self.parent_id = parent_id;
    }
}

/// Special folder IDs
pub mod special_folders {
    /// Bookmarks bar folder ID
    pub const BOOKMARKS_BAR: &str = "bookmarks_bar";
    /// Other bookmarks folder ID
    pub const OTHER_BOOKMARKS: &str = "other_bookmarks";
    /// Mobile bookmarks folder ID (reserved for sync)
    #[allow(dead_code)]
    pub const MOBILE_BOOKMARKS: &str = "mobile_bookmarks";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_folder_creation() {
        let folder = BookmarkFolder::new("Development");

        assert_eq!(folder.name, "Development");
        assert!(folder.parent_id.is_none());
        assert!(!folder.id.is_empty());
    }

    #[test]
    fn test_folder_with_methods() {
        let parent = BookmarkFolder::new("Parent");
        let folder = BookmarkFolder::new("Child")
            .with_parent(parent.id)
            .with_position(2);

        assert!(folder.parent_id.is_some());
        assert_eq!(folder.position, 2);
    }
}
