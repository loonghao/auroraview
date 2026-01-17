//! Bookmark management

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Unique identifier for a bookmark
pub type BookmarkId = String;

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
    pub favicon: Option<String>,
    /// Parent folder ID (None for root)
    pub parent_id: Option<BookmarkId>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last modified timestamp
    pub modified_at: DateTime<Utc>,
    /// Position in parent folder
    pub position: u32,
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
        }
    }

    /// Create a bookmark with ID
    pub fn with_id(id: impl Into<String>, title: impl Into<String>, url: impl Into<String>) -> Self {
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
        }
    }

    /// Set parent folder
    pub fn with_parent(mut self, parent_id: BookmarkId) -> Self {
        self.parent_id = Some(parent_id);
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
}

/// A bookmark folder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkFolder {
    /// Unique identifier
    pub id: BookmarkId,
    /// Folder name
    pub name: String,
    /// Parent folder ID (None for root)
    pub parent_id: Option<BookmarkId>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Position in parent folder
    pub position: u32,
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
        }
    }
}

/// Bookmark storage
#[derive(Debug, Default, Serialize, Deserialize)]
struct BookmarkStorage {
    bookmarks: HashMap<BookmarkId, Bookmark>,
    folders: HashMap<BookmarkId, BookmarkFolder>,
}

/// Bookmark manager
pub struct BookmarkManager {
    storage: RwLock<BookmarkStorage>,
    data_path: Option<PathBuf>,
}

impl BookmarkManager {
    /// Create a new bookmark manager
    pub fn new(data_dir: Option<&str>) -> Self {
        let data_path = data_dir
            .map(PathBuf::from)
            .or_else(|| dirs::data_dir().map(|d| d.join("auroraview").join("browser")))
            .map(|d| d.join("bookmarks.json"));

        let storage = data_path
            .as_ref()
            .and_then(|p| fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        Self {
            storage: RwLock::new(storage),
            data_path,
        }
    }

    /// Add a bookmark
    pub fn add(&self, bookmark: Bookmark) -> BookmarkId {
        let id = bookmark.id.clone();
        {
            let mut storage = self.storage.write();
            storage.bookmarks.insert(id.clone(), bookmark);
        }
        self.save();
        id
    }

    /// Add a bookmark from URL and title
    pub fn add_bookmark(&self, url: &str, title: &str) -> BookmarkId {
        let bookmark = Bookmark::new(title, url);
        self.add(bookmark)
    }

    /// Remove a bookmark
    pub fn remove(&self, id: &BookmarkId) -> Option<Bookmark> {
        let result = {
            let mut storage = self.storage.write();
            storage.bookmarks.remove(id)
        };
        if result.is_some() {
            self.save();
        }
        result
    }

    /// Get a bookmark by ID
    pub fn get(&self, id: &BookmarkId) -> Option<Bookmark> {
        self.storage.read().bookmarks.get(id).cloned()
    }

    /// Get all bookmarks
    pub fn all(&self) -> Vec<Bookmark> {
        let storage = self.storage.read();
        let mut bookmarks: Vec<_> = storage.bookmarks.values().cloned().collect();
        bookmarks.sort_by_key(|b| b.position);
        bookmarks
    }

    /// Get bookmarks in a folder
    pub fn in_folder(&self, folder_id: Option<&BookmarkId>) -> Vec<Bookmark> {
        let storage = self.storage.read();
        let mut bookmarks: Vec<_> = storage
            .bookmarks
            .values()
            .filter(|b| b.parent_id.as_ref() == folder_id)
            .cloned()
            .collect();
        bookmarks.sort_by_key(|b| b.position);
        bookmarks
    }

    /// Get bookmarks bar items (root level)
    pub fn bar_items(&self) -> Vec<Bookmark> {
        self.in_folder(None)
    }

    /// Update a bookmark
    pub fn update(&self, id: &BookmarkId, title: Option<String>, url: Option<String>) -> bool {
        let updated = {
            let mut storage = self.storage.write();
            if let Some(bookmark) = storage.bookmarks.get_mut(id) {
                if let Some(t) = title {
                    bookmark.title = t;
                }
                if let Some(u) = url {
                    bookmark.url = u;
                }
                bookmark.modified_at = Utc::now();
                true
            } else {
                false
            }
        };
        if updated {
            self.save();
        }
        updated
    }

    /// Check if a URL is bookmarked
    pub fn is_bookmarked(&self, url: &str) -> bool {
        self.storage.read().bookmarks.values().any(|b| b.url == url)
    }

    /// Find bookmark by URL
    pub fn find_by_url(&self, url: &str) -> Option<Bookmark> {
        self.storage
            .read()
            .bookmarks
            .values()
            .find(|b| b.url == url)
            .cloned()
    }

    /// Add a folder
    pub fn add_folder(&self, folder: BookmarkFolder) -> BookmarkId {
        let id = folder.id.clone();
        {
            let mut storage = self.storage.write();
            storage.folders.insert(id.clone(), folder);
        }
        self.save();
        id
    }

    /// Get all folders
    pub fn folders(&self) -> Vec<BookmarkFolder> {
        let storage = self.storage.read();
        let mut folders: Vec<_> = storage.folders.values().cloned().collect();
        folders.sort_by_key(|f| f.position);
        folders
    }

    /// Save to disk
    fn save(&self) {
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

    /// Clear all bookmarks
    pub fn clear(&self) {
        {
            let mut storage = self.storage.write();
            storage.bookmarks.clear();
            storage.folders.clear();
        }
        self.save();
    }

    /// Get bookmark count
    pub fn count(&self) -> usize {
        self.storage.read().bookmarks.len()
    }
}

impl Default for BookmarkManager {
    fn default() -> Self {
        Self::new(None)
    }
}
