//! Bookmark manager implementation

use crate::{Bookmark, BookmarkError, BookmarkFolder, BookmarkId, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// Bookmark manager
///
/// Manages bookmarks and folders with optional persistence.
#[derive(Debug)]
pub struct BookmarkManager {
    inner: Arc<RwLock<BookmarkStore>>,
    storage_path: Option<PathBuf>,
}

#[derive(Debug, Default)]
struct BookmarkStore {
    bookmarks: HashMap<BookmarkId, Bookmark>,
    folders: HashMap<BookmarkId, BookmarkFolder>,
}

impl BookmarkManager {
    /// Create a new bookmark manager
    ///
    /// If `data_dir` is provided, bookmarks will be persisted to disk.
    pub fn new(data_dir: Option<&Path>) -> Self {
        let storage_path = data_dir.map(|p| p.join("bookmarks.json"));

        let mut manager = Self {
            inner: Arc::new(RwLock::new(BookmarkStore::default())),
            storage_path,
        };

        // Try to load existing bookmarks
        if let Some(ref path) = manager.storage_path {
            if path.exists() {
                let _ = manager.load();
            }
        }

        // Initialize special folders
        manager.init_special_folders();

        manager
    }

    /// Initialize special folders (bookmarks bar, other bookmarks)
    fn init_special_folders(&self) {
        use crate::folder::special_folders;

        let mut store = self.inner.write().unwrap();

        // Bookmarks bar
        if !store.folders.contains_key(special_folders::BOOKMARKS_BAR) {
            let folder = BookmarkFolder::with_id(special_folders::BOOKMARKS_BAR, "Bookmarks Bar");
            store.folders.insert(folder.id.clone(), folder);
        }

        // Other bookmarks
        if !store.folders.contains_key(special_folders::OTHER_BOOKMARKS) {
            let folder = BookmarkFolder::with_id(special_folders::OTHER_BOOKMARKS, "Other Bookmarks");
            store.folders.insert(folder.id.clone(), folder);
        }
    }

    /// Add a bookmark
    pub fn add(&self, url: impl Into<String>, title: impl Into<String>) -> BookmarkId {
        let bookmark = Bookmark::new(title, url);
        let id = bookmark.id.clone();

        let mut store = self.inner.write().unwrap();
        store.bookmarks.insert(id.clone(), bookmark);
        drop(store);

        let _ = self.save();
        id
    }

    /// Add a bookmark to a specific folder
    pub fn add_to_folder(
        &self,
        url: impl Into<String>,
        title: impl Into<String>,
        folder_id: &str,
    ) -> Result<BookmarkId> {
        let store = self.inner.read().unwrap();
        if !store.folders.contains_key(folder_id) {
            return Err(BookmarkError::FolderNotFound(folder_id.to_string()));
        }
        drop(store);

        let bookmark = Bookmark::new(title, url).with_parent(folder_id);
        let id = bookmark.id.clone();

        let mut store = self.inner.write().unwrap();
        store.bookmarks.insert(id.clone(), bookmark);
        drop(store);

        let _ = self.save();
        Ok(id)
    }

    /// Get a bookmark by ID
    pub fn get(&self, id: &str) -> Option<Bookmark> {
        let store = self.inner.read().unwrap();
        store.bookmarks.get(id).cloned()
    }

    /// Update a bookmark
    pub fn update(&self, id: &str, title: Option<&str>, url: Option<&str>) -> Result<()> {
        let mut store = self.inner.write().unwrap();
        let bookmark = store
            .bookmarks
            .get_mut(id)
            .ok_or_else(|| BookmarkError::NotFound(id.to_string()))?;

        if let Some(title) = title {
            bookmark.set_title(title);
        }
        if let Some(url) = url {
            bookmark.set_url(url);
        }

        drop(store);
        let _ = self.save();
        Ok(())
    }

    /// Remove a bookmark
    pub fn remove(&self, id: &str) -> bool {
        let mut store = self.inner.write().unwrap();
        let removed = store.bookmarks.remove(id).is_some();
        drop(store);

        if removed {
            let _ = self.save();
        }
        removed
    }

    /// Check if a URL is bookmarked
    pub fn is_bookmarked(&self, url: &str) -> bool {
        let store = self.inner.read().unwrap();
        store.bookmarks.values().any(|b| b.url == url)
    }

    /// Find bookmark by URL
    pub fn find_by_url(&self, url: &str) -> Option<Bookmark> {
        let store = self.inner.read().unwrap();
        store.bookmarks.values().find(|b| b.url == url).cloned()
    }

    /// Get all bookmarks
    pub fn all(&self) -> Vec<Bookmark> {
        let store = self.inner.read().unwrap();
        store.bookmarks.values().cloned().collect()
    }

    /// Get bookmarks in a folder
    pub fn in_folder(&self, folder_id: &str) -> Vec<Bookmark> {
        let store = self.inner.read().unwrap();
        let mut bookmarks: Vec<_> = store
            .bookmarks
            .values()
            .filter(|b| b.parent_id.as_deref() == Some(folder_id))
            .cloned()
            .collect();
        bookmarks.sort_by_key(|b| b.position);
        bookmarks
    }

    /// Get root bookmarks (without parent folder)
    pub fn root_bookmarks(&self) -> Vec<Bookmark> {
        let store = self.inner.read().unwrap();
        let mut bookmarks: Vec<_> = store
            .bookmarks
            .values()
            .filter(|b| b.parent_id.is_none())
            .cloned()
            .collect();
        bookmarks.sort_by_key(|b| b.position);
        bookmarks
    }

    /// Search bookmarks
    pub fn search(&self, query: &str) -> Vec<Bookmark> {
        let store = self.inner.read().unwrap();
        store
            .bookmarks
            .values()
            .filter(|b| b.matches(query))
            .cloned()
            .collect()
    }

    /// Move bookmark to folder
    pub fn move_to_folder(&self, bookmark_id: &str, folder_id: Option<&str>) -> Result<()> {
        if let Some(fid) = folder_id {
            let store = self.inner.read().unwrap();
            if !store.folders.contains_key(fid) {
                return Err(BookmarkError::FolderNotFound(fid.to_string()));
            }
            drop(store);
        }

        let mut store = self.inner.write().unwrap();
        let bookmark = store
            .bookmarks
            .get_mut(bookmark_id)
            .ok_or_else(|| BookmarkError::NotFound(bookmark_id.to_string()))?;

        bookmark.set_parent(folder_id.map(String::from));
        drop(store);

        let _ = self.save();
        Ok(())
    }

    // ========== Folder operations ==========

    /// Create a folder
    pub fn create_folder(&self, name: impl Into<String>) -> BookmarkId {
        let folder = BookmarkFolder::new(name);
        let id = folder.id.clone();

        let mut store = self.inner.write().unwrap();
        store.folders.insert(id.clone(), folder);
        drop(store);

        let _ = self.save();
        id
    }

    /// Create a subfolder
    pub fn create_subfolder(
        &self,
        name: impl Into<String>,
        parent_id: &str,
    ) -> Result<BookmarkId> {
        let store = self.inner.read().unwrap();
        if !store.folders.contains_key(parent_id) {
            return Err(BookmarkError::FolderNotFound(parent_id.to_string()));
        }
        drop(store);

        let folder = BookmarkFolder::new(name).with_parent(parent_id);
        let id = folder.id.clone();

        let mut store = self.inner.write().unwrap();
        store.folders.insert(id.clone(), folder);
        drop(store);

        let _ = self.save();
        Ok(id)
    }

    /// Get a folder by ID
    pub fn get_folder(&self, id: &str) -> Option<BookmarkFolder> {
        let store = self.inner.read().unwrap();
        store.folders.get(id).cloned()
    }

    /// Get all folders
    pub fn all_folders(&self) -> Vec<BookmarkFolder> {
        let store = self.inner.read().unwrap();
        store.folders.values().cloned().collect()
    }

    /// Get subfolders
    pub fn subfolders(&self, parent_id: &str) -> Vec<BookmarkFolder> {
        let store = self.inner.read().unwrap();
        let mut folders: Vec<_> = store
            .folders
            .values()
            .filter(|f| f.parent_id.as_deref() == Some(parent_id))
            .cloned()
            .collect();
        folders.sort_by_key(|f| f.position);
        folders
    }

    /// Delete a folder (and optionally its contents)
    pub fn delete_folder(&self, id: &str, delete_contents: bool) -> Result<()> {
        let mut store = self.inner.write().unwrap();

        if !store.folders.contains_key(id) {
            return Err(BookmarkError::FolderNotFound(id.to_string()));
        }

        if delete_contents {
            // Delete all bookmarks in folder
            store.bookmarks.retain(|_, b| b.parent_id.as_deref() != Some(id));

            // Delete all subfolders (recursively would need more complex logic)
            store.folders.retain(|_, f| f.parent_id.as_deref() != Some(id));
        } else {
            // Move contents to root
            for bookmark in store.bookmarks.values_mut() {
                if bookmark.parent_id.as_deref() == Some(id) {
                    bookmark.parent_id = None;
                }
            }
            for folder in store.folders.values_mut() {
                if folder.parent_id.as_deref() == Some(id) {
                    folder.parent_id = None;
                }
            }
        }

        store.folders.remove(id);
        drop(store);

        let _ = self.save();
        Ok(())
    }

    /// Rename a folder
    pub fn rename_folder(&self, id: &str, name: impl Into<String>) -> Result<()> {
        let mut store = self.inner.write().unwrap();
        let folder = store
            .folders
            .get_mut(id)
            .ok_or_else(|| BookmarkError::FolderNotFound(id.to_string()))?;

        folder.set_name(name);
        drop(store);

        let _ = self.save();
        Ok(())
    }

    // ========== Persistence ==========

    /// Save bookmarks to disk
    pub fn save(&self) -> Result<()> {
        let Some(ref path) = self.storage_path else {
            return Ok(());
        };

        let store = self.inner.read().unwrap();

        #[derive(serde::Serialize)]
        struct Data<'a> {
            bookmarks: &'a HashMap<BookmarkId, Bookmark>,
            folders: &'a HashMap<BookmarkId, BookmarkFolder>,
        }

        let data = Data {
            bookmarks: &store.bookmarks,
            folders: &store.folders,
        };

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(&data)?;
        std::fs::write(path, json)?;

        Ok(())
    }

    /// Load bookmarks from disk
    pub fn load(&mut self) -> Result<()> {
        let Some(ref path) = self.storage_path else {
            return Ok(());
        };

        if !path.exists() {
            return Ok(());
        }

        let json = std::fs::read_to_string(path)?;

        #[derive(serde::Deserialize)]
        struct Data {
            bookmarks: HashMap<BookmarkId, Bookmark>,
            folders: HashMap<BookmarkId, BookmarkFolder>,
        }

        let data: Data = serde_json::from_str(&json)?;

        let mut store = self.inner.write().unwrap();
        store.bookmarks = data.bookmarks;
        store.folders = data.folders;

        Ok(())
    }

    /// Export bookmarks to JSON string
    pub fn export(&self) -> Result<String> {
        let store = self.inner.read().unwrap();

        #[derive(serde::Serialize)]
        struct Data<'a> {
            bookmarks: &'a HashMap<BookmarkId, Bookmark>,
            folders: &'a HashMap<BookmarkId, BookmarkFolder>,
        }

        let data = Data {
            bookmarks: &store.bookmarks,
            folders: &store.folders,
        };

        Ok(serde_json::to_string_pretty(&data)?)
    }

    /// Import bookmarks from JSON string
    pub fn import(&self, json: &str) -> Result<()> {
        #[derive(serde::Deserialize)]
        struct Data {
            bookmarks: HashMap<BookmarkId, Bookmark>,
            folders: HashMap<BookmarkId, BookmarkFolder>,
        }

        let data: Data = serde_json::from_str(json)?;

        let mut store = self.inner.write().unwrap();
        store.bookmarks.extend(data.bookmarks);
        store.folders.extend(data.folders);
        drop(store);

        let _ = self.save();
        Ok(())
    }

    /// Clear all bookmarks (keeps special folders)
    pub fn clear(&self) {
        let mut store = self.inner.write().unwrap();
        store.bookmarks.clear();
        drop(store);

        self.init_special_folders();
        let _ = self.save();
    }

    /// Get bookmark count
    pub fn count(&self) -> usize {
        let store = self.inner.read().unwrap();
        store.bookmarks.len()
    }

    /// Get folder count
    pub fn folder_count(&self) -> usize {
        let store = self.inner.read().unwrap();
        store.folders.len()
    }
}

impl Clone for BookmarkManager {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            storage_path: self.storage_path.clone(),
        }
    }
}

impl Default for BookmarkManager {
    fn default() -> Self {
        Self::new(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_get_bookmark() {
        let manager = BookmarkManager::new(None);

        let id = manager.add("https://github.com", "GitHub");
        let bookmark = manager.get(&id).unwrap();

        assert_eq!(bookmark.title, "GitHub");
        assert_eq!(bookmark.url, "https://github.com");
    }

    #[test]
    fn test_is_bookmarked() {
        let manager = BookmarkManager::new(None);

        manager.add("https://github.com", "GitHub");

        assert!(manager.is_bookmarked("https://github.com"));
        assert!(!manager.is_bookmarked("https://gitlab.com"));
    }

    #[test]
    fn test_remove_bookmark() {
        let manager = BookmarkManager::new(None);

        let id = manager.add("https://github.com", "GitHub");
        assert!(manager.is_bookmarked("https://github.com"));

        assert!(manager.remove(&id));
        assert!(!manager.is_bookmarked("https://github.com"));
    }

    #[test]
    fn test_search() {
        let manager = BookmarkManager::new(None);

        manager.add("https://github.com", "GitHub");
        manager.add("https://gitlab.com", "GitLab");
        manager.add("https://rust-lang.org", "Rust");

        let results = manager.search("git");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_folders() {
        let manager = BookmarkManager::new(None);

        let folder_id = manager.create_folder("Development");
        manager.add_to_folder("https://github.com", "GitHub", &folder_id).unwrap();

        let bookmarks = manager.in_folder(&folder_id);
        assert_eq!(bookmarks.len(), 1);
        assert_eq!(bookmarks[0].title, "GitHub");
    }

    #[test]
    fn test_special_folders_exist() {
        use crate::folder::special_folders;

        let manager = BookmarkManager::new(None);

        assert!(manager.get_folder(special_folders::BOOKMARKS_BAR).is_some());
        assert!(manager.get_folder(special_folders::OTHER_BOOKMARKS).is_some());
    }
}
