//! Download manager implementation

use crate::{DownloadError, DownloadId, DownloadItem, DownloadQueue, DownloadState, Result};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Download manager
///
/// Manages downloads with queue support and optional persistence.
#[derive(Debug)]
pub struct DownloadManager {
    inner: Arc<RwLock<DownloadStore>>,
    download_dir: PathBuf,
    storage_path: Option<PathBuf>,
}

#[derive(Debug, Default)]
struct DownloadStore {
    downloads: HashMap<DownloadId, DownloadItem>,
    queue: DownloadQueue,
}

impl DownloadManager {
    /// Create a new download manager
    ///
    /// If `download_dir` is None, uses system default downloads folder.
    pub fn new(download_dir: Option<&Path>) -> Self {
        let download_dir = download_dir
            .map(PathBuf::from)
            .unwrap_or_else(|| dirs::download_dir().unwrap_or_else(|| PathBuf::from(".")));

        Self {
            inner: Arc::new(RwLock::new(DownloadStore::default())),
            download_dir,
            storage_path: None,
        }
    }

    /// Create a download manager with persistence
    pub fn with_persistence(download_dir: Option<&Path>, data_dir: &Path) -> Self {
        let mut manager = Self::new(download_dir);
        manager.storage_path = Some(data_dir.join("downloads.json"));

        // Load existing downloads
        if let Some(ref path) = manager.storage_path {
            if path.exists() {
                let _ = manager.load();
            }
        }

        manager
    }

    /// Set maximum concurrent downloads
    pub fn set_max_concurrent(&self, max: usize) {
        let mut store = self.inner.write();
        store.queue.set_max_concurrent(max);
    }

    /// Get download directory
    pub fn download_dir(&self) -> &Path {
        &self.download_dir
    }

    /// Set download directory
    pub fn set_download_dir(&mut self, dir: impl Into<PathBuf>) {
        self.download_dir = dir.into();
    }

    // ========== Download Operations ==========

    /// Add a download
    pub fn add(&self, url: impl Into<String>, filename: impl Into<String>) -> DownloadId {
        let url = url.into();
        let filename = filename.into();
        let save_path = self.download_dir.join(&filename);

        let item = DownloadItem::new(&url, &filename).with_save_path(save_path);
        let id = item.id.clone();

        let mut store = self.inner.write();
        store.downloads.insert(id.clone(), item);
        store.queue.enqueue(id.clone());
        drop(store);

        let _ = self.save();
        id
    }

    /// Add a download with custom item
    pub fn add_item(&self, item: DownloadItem) -> DownloadId {
        let id = item.id.clone();

        let mut store = self.inner.write();
        store.downloads.insert(id.clone(), item);
        store.queue.enqueue(id.clone());
        drop(store);

        let _ = self.save();
        id
    }

    /// Get a download by ID
    pub fn get(&self, id: &DownloadId) -> Option<DownloadItem> {
        let store = self.inner.read();
        store.downloads.get(id).cloned()
    }

    /// Get all downloads
    pub fn all(&self) -> Vec<DownloadItem> {
        let store = self.inner.read();
        store.downloads.values().cloned().collect()
    }

    /// Get downloads by state
    pub fn by_state(&self, state: DownloadState) -> Vec<DownloadItem> {
        let store = self.inner.read();
        store
            .downloads
            .values()
            .filter(|d| d.state == state)
            .cloned()
            .collect()
    }

    /// Get active downloads
    pub fn active(&self) -> Vec<DownloadItem> {
        self.by_state(DownloadState::Downloading)
    }

    /// Get completed downloads
    pub fn completed(&self) -> Vec<DownloadItem> {
        self.by_state(DownloadState::Completed)
    }

    /// Get pending downloads
    pub fn pending(&self) -> Vec<DownloadItem> {
        self.by_state(DownloadState::Pending)
    }

    /// Remove a download
    pub fn remove(&self, id: &DownloadId) -> bool {
        let mut store = self.inner.write();
        let removed = store.downloads.remove(id).is_some();
        store.queue.remove(id);
        drop(store);

        if removed {
            let _ = self.save();
        }
        removed
    }

    /// Clear completed downloads
    pub fn clear_completed(&self) {
        let mut store = self.inner.write();
        store
            .downloads
            .retain(|_, d| d.state != DownloadState::Completed);
        drop(store);
        let _ = self.save();
    }

    /// Clear all downloads
    pub fn clear(&self) {
        let mut store = self.inner.write();
        store.downloads.clear();
        store.queue.clear();
        drop(store);
        let _ = self.save();
    }

    // ========== Download Control ==========

    /// Start a download
    pub fn start(&self, id: &DownloadId) -> Result<()> {
        let mut store = self.inner.write();

        let download = store
            .downloads
            .get_mut(id)
            .ok_or_else(|| DownloadError::NotFound(id.clone()))?;

        if download.state != DownloadState::Pending && download.state != DownloadState::Paused {
            return Err(DownloadError::InvalidState(format!(
                "Cannot start download in {:?} state",
                download.state
            )));
        }

        download.start();
        store.queue.mark_active(id);
        drop(store);

        let _ = self.save();
        Ok(())
    }

    /// Pause a download
    pub fn pause(&self, id: &DownloadId) -> Result<()> {
        let mut store = self.inner.write();

        let download = store
            .downloads
            .get_mut(id)
            .ok_or_else(|| DownloadError::NotFound(id.clone()))?;

        if download.state != DownloadState::Downloading {
            return Err(DownloadError::InvalidState(format!(
                "Cannot pause download in {:?} state",
                download.state
            )));
        }

        download.pause();
        drop(store);

        let _ = self.save();
        Ok(())
    }

    /// Resume a download
    pub fn resume(&self, id: &DownloadId) -> Result<()> {
        let mut store = self.inner.write();

        let download = store
            .downloads
            .get_mut(id)
            .ok_or_else(|| DownloadError::NotFound(id.clone()))?;

        if download.state != DownloadState::Paused {
            return Err(DownloadError::InvalidState(format!(
                "Cannot resume download in {:?} state",
                download.state
            )));
        }

        download.resume();
        drop(store);

        let _ = self.save();
        Ok(())
    }

    /// Cancel a download
    pub fn cancel(&self, id: &DownloadId) -> Result<()> {
        let mut store = self.inner.write();

        let download = store
            .downloads
            .get_mut(id)
            .ok_or_else(|| DownloadError::NotFound(id.clone()))?;

        if download.is_finished() {
            return Err(DownloadError::InvalidState(
                "Download already finished".to_string(),
            ));
        }

        download.cancel();
        store.queue.mark_finished(id);
        drop(store);

        let _ = self.save();
        Ok(())
    }

    /// Mark download as complete
    pub fn complete(&self, id: &DownloadId) -> Result<()> {
        let mut store = self.inner.write();

        let download = store
            .downloads
            .get_mut(id)
            .ok_or_else(|| DownloadError::NotFound(id.clone()))?;

        download.complete();
        store.queue.mark_finished(id);
        drop(store);

        let _ = self.save();
        Ok(())
    }

    /// Mark download as failed
    pub fn fail(&self, id: &DownloadId, error: impl Into<String>) -> Result<()> {
        let mut store = self.inner.write();

        let download = store
            .downloads
            .get_mut(id)
            .ok_or_else(|| DownloadError::NotFound(id.clone()))?;

        download.fail(error);
        store.queue.mark_finished(id);
        drop(store);

        let _ = self.save();
        Ok(())
    }

    // ========== Progress Updates ==========

    /// Update download progress
    pub fn update_progress(&self, id: &DownloadId, received: u64, total: Option<u64>) {
        let mut store = self.inner.write();
        if let Some(download) = store.downloads.get_mut(id) {
            download.update_progress(received, total);
        }
    }

    /// Update download speed
    pub fn update_speed(&self, id: &DownloadId, bytes_per_second: u64) {
        let mut store = self.inner.write();
        if let Some(download) = store.downloads.get_mut(id) {
            download.update_speed(bytes_per_second);
        }
    }

    // ========== Queue Operations ==========

    /// Get next download to start (from queue)
    pub fn next_to_start(&self) -> Option<DownloadId> {
        let mut store = self.inner.write();
        store.queue.next()
    }

    /// Check if can start new download
    pub fn can_start_new(&self) -> bool {
        let store = self.inner.read();
        store.queue.can_start()
    }

    /// Get queue statistics
    pub fn queue_stats(&self) -> (usize, usize, usize) {
        let store = self.inner.read();
        (
            store.queue.pending_count(),
            store.queue.active_count(),
            store.downloads.len(),
        )
    }

    // ========== Statistics ==========

    /// Get download count
    pub fn count(&self) -> usize {
        let store = self.inner.read();
        store.downloads.len()
    }

    /// Get total bytes downloaded
    pub fn total_bytes_downloaded(&self) -> u64 {
        let store = self.inner.read();
        store
            .downloads
            .values()
            .filter(|d| d.state == DownloadState::Completed)
            .filter_map(|d| d.total_bytes)
            .sum()
    }

    // ========== Persistence ==========

    /// Save downloads to disk
    pub fn save(&self) -> Result<()> {
        let Some(ref path) = self.storage_path else {
            return Ok(());
        };

        let store = self.inner.read();
        let downloads: Vec<_> = store.downloads.values().cloned().collect();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(&downloads)?;
        std::fs::write(path, json)?;

        Ok(())
    }

    /// Load downloads from disk
    pub fn load(&mut self) -> Result<()> {
        let Some(ref path) = self.storage_path else {
            return Ok(());
        };

        if !path.exists() {
            return Ok(());
        }

        let json = std::fs::read_to_string(path)?;
        let downloads: Vec<DownloadItem> = serde_json::from_str(&json)?;

        let mut store = self.inner.write();
        for download in downloads {
            // Re-queue incomplete downloads
            if download.state == DownloadState::Pending
                || download.state == DownloadState::Downloading
            {
                store.queue.enqueue(download.id.clone());
            }
            store.downloads.insert(download.id.clone(), download);
        }

        Ok(())
    }

    /// Export downloads to JSON
    pub fn export(&self) -> Result<String> {
        let store = self.inner.read();
        let downloads: Vec<_> = store.downloads.values().cloned().collect();
        Ok(serde_json::to_string_pretty(&downloads)?)
    }
}

impl Clone for DownloadManager {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            download_dir: self.download_dir.clone(),
            storage_path: self.storage_path.clone(),
        }
    }
}

impl Default for DownloadManager {
    fn default() -> Self {
        Self::new(None)
    }
}

/// Platform-specific directories
mod dirs {
    use std::path::PathBuf;

    pub fn download_dir() -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            std::env::var("USERPROFILE")
                .ok()
                .map(|p| PathBuf::from(p).join("Downloads"))
        }

        #[cfg(target_os = "macos")]
        {
            std::env::var("HOME")
                .ok()
                .map(|p| PathBuf::from(p).join("Downloads"))
        }

        #[cfg(target_os = "linux")]
        {
            std::env::var("HOME")
                .ok()
                .map(|p| PathBuf::from(p).join("Downloads"))
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_download() {
        let manager = DownloadManager::new(Some(Path::new("/tmp")));

        let id = manager.add("https://example.com/file.zip", "file.zip");
        let download = manager.get(&id).unwrap();

        assert_eq!(download.filename, "file.zip");
        assert_eq!(download.state, DownloadState::Pending);
    }

    #[test]
    fn test_download_lifecycle() {
        let manager = DownloadManager::new(Some(Path::new("/tmp")));

        let id = manager.add("https://example.com/file.zip", "file.zip");

        manager.start(&id).unwrap();
        assert_eq!(manager.get(&id).unwrap().state, DownloadState::Downloading);

        manager.pause(&id).unwrap();
        assert_eq!(manager.get(&id).unwrap().state, DownloadState::Paused);

        manager.resume(&id).unwrap();
        assert_eq!(manager.get(&id).unwrap().state, DownloadState::Downloading);

        manager.complete(&id).unwrap();
        assert_eq!(manager.get(&id).unwrap().state, DownloadState::Completed);
    }

    #[test]
    fn test_progress_update() {
        let manager = DownloadManager::new(Some(Path::new("/tmp")));

        let id = manager.add("https://example.com/file.zip", "file.zip");
        manager.start(&id).unwrap();

        manager.update_progress(&id, 500, Some(1000));
        manager.update_speed(&id, 100);

        let download = manager.get(&id).unwrap();
        assert_eq!(download.progress(), Some(50));
        assert_eq!(download.speed, Some(100));
    }

    #[test]
    fn test_clear_completed() {
        let manager = DownloadManager::new(Some(Path::new("/tmp")));

        let id1 = manager.add("https://example.com/1.zip", "1.zip");
        let id2 = manager.add("https://example.com/2.zip", "2.zip");

        manager.start(&id1).unwrap();
        manager.complete(&id1).unwrap();

        assert_eq!(manager.count(), 2);
        manager.clear_completed();
        assert_eq!(manager.count(), 1);
    }
}
