//! Download item data structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Unique identifier for a download
pub type DownloadId = String;

/// Download state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DownloadState {
    /// Download is pending/queued
    Pending,
    /// Download is in progress
    Downloading,
    /// Download is paused
    Paused,
    /// Download completed successfully
    Completed,
    /// Download failed
    Failed,
    /// Download was cancelled
    Cancelled,
}

impl Default for DownloadState {
    fn default() -> Self {
        Self::Pending
    }
}

/// A download item
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadItem {
    /// Unique identifier
    pub id: DownloadId,
    /// Source URL
    pub url: String,
    /// File name
    pub filename: String,
    /// Save path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub save_path: Option<PathBuf>,
    /// MIME type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Total size in bytes (None if unknown)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_bytes: Option<u64>,
    /// Received bytes
    pub received_bytes: u64,
    /// Download state
    pub state: DownloadState,
    /// Error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Completion timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
    /// Download speed in bytes per second
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<u64>,
}

impl DownloadItem {
    /// Create a new download item
    pub fn new(url: impl Into<String>, filename: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            url: url.into(),
            filename: filename.into(),
            save_path: None,
            mime_type: None,
            total_bytes: None,
            received_bytes: 0,
            state: DownloadState::Pending,
            error: None,
            created_at: Utc::now(),
            completed_at: None,
            speed: None,
        }
    }

    /// Create a download item with a specific ID
    pub fn with_id(
        id: impl Into<String>,
        url: impl Into<String>,
        filename: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            url: url.into(),
            filename: filename.into(),
            save_path: None,
            mime_type: None,
            total_bytes: None,
            received_bytes: 0,
            state: DownloadState::Pending,
            error: None,
            created_at: Utc::now(),
            completed_at: None,
            speed: None,
        }
    }

    /// Set save path
    pub fn with_save_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.save_path = Some(path.into());
        self
    }

    /// Set MIME type
    pub fn with_mime_type(mut self, mime_type: impl Into<String>) -> Self {
        self.mime_type = Some(mime_type.into());
        self
    }

    /// Set total size
    pub fn with_total_bytes(mut self, total: u64) -> Self {
        self.total_bytes = Some(total);
        self
    }

    /// Start downloading
    pub fn start(&mut self) {
        if self.state == DownloadState::Pending || self.state == DownloadState::Paused {
            self.state = DownloadState::Downloading;
        }
    }

    /// Pause download
    pub fn pause(&mut self) {
        if self.state == DownloadState::Downloading {
            self.state = DownloadState::Paused;
            self.speed = None;
        }
    }

    /// Resume download
    pub fn resume(&mut self) {
        if self.state == DownloadState::Paused {
            self.state = DownloadState::Downloading;
        }
    }

    /// Cancel download
    pub fn cancel(&mut self) {
        if self.state != DownloadState::Completed {
            self.state = DownloadState::Cancelled;
            self.speed = None;
        }
    }

    /// Mark as completed
    pub fn complete(&mut self) {
        self.state = DownloadState::Completed;
        self.completed_at = Some(Utc::now());
        self.speed = None;
        if let Some(total) = self.total_bytes {
            self.received_bytes = total;
        }
    }

    /// Mark as failed
    pub fn fail(&mut self, error: impl Into<String>) {
        self.state = DownloadState::Failed;
        self.error = Some(error.into());
        self.speed = None;
    }

    /// Update progress
    pub fn update_progress(&mut self, received: u64, total: Option<u64>) {
        self.received_bytes = received;
        if let Some(t) = total {
            self.total_bytes = Some(t);
        }
    }

    /// Update speed
    pub fn update_speed(&mut self, bytes_per_second: u64) {
        self.speed = Some(bytes_per_second);
    }

    /// Get progress as percentage (0-100)
    pub fn progress(&self) -> Option<u8> {
        self.total_bytes.map(|total| {
            if total == 0 {
                100
            } else {
                ((self.received_bytes as f64 / total as f64) * 100.0).min(100.0) as u8
            }
        })
    }

    /// Check if download is active
    pub fn is_active(&self) -> bool {
        self.state == DownloadState::Downloading
    }

    /// Check if download is finished (completed, failed, or cancelled)
    pub fn is_finished(&self) -> bool {
        matches!(
            self.state,
            DownloadState::Completed | DownloadState::Failed | DownloadState::Cancelled
        )
    }

    /// Check if download can be resumed
    pub fn can_resume(&self) -> bool {
        self.state == DownloadState::Paused
    }

    /// Check if download can be paused
    pub fn can_pause(&self) -> bool {
        self.state == DownloadState::Downloading
    }

    /// Check if download can be cancelled
    pub fn can_cancel(&self) -> bool {
        !self.is_finished()
    }

    /// Get estimated time remaining in seconds
    pub fn eta(&self) -> Option<u64> {
        match (self.speed, self.total_bytes) {
            (Some(speed), Some(total)) if speed > 0 => {
                let remaining = total.saturating_sub(self.received_bytes);
                Some(remaining / speed)
            }
            _ => None,
        }
    }

    /// Get the domain from URL
    pub fn domain(&self) -> Option<&str> {
        self.url
            .strip_prefix("https://")
            .or_else(|| self.url.strip_prefix("http://"))
            .and_then(|s| s.split('/').next())
    }

    /// Get file extension
    pub fn extension(&self) -> Option<&str> {
        self.filename.rsplit('.').next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_item_creation() {
        let item = DownloadItem::new("https://example.com/file.zip", "file.zip");

        assert_eq!(item.filename, "file.zip");
        assert_eq!(item.state, DownloadState::Pending);
        assert_eq!(item.received_bytes, 0);
    }

    #[test]
    fn test_download_state_transitions() {
        let mut item = DownloadItem::new("https://example.com/file.zip", "file.zip");

        assert!(item.can_cancel());
        assert!(!item.can_pause());
        assert!(!item.can_resume());

        item.start();
        assert_eq!(item.state, DownloadState::Downloading);
        assert!(item.can_pause());

        item.pause();
        assert_eq!(item.state, DownloadState::Paused);
        assert!(item.can_resume());

        item.resume();
        assert_eq!(item.state, DownloadState::Downloading);

        item.complete();
        assert_eq!(item.state, DownloadState::Completed);
        assert!(item.is_finished());
        assert!(!item.can_cancel());
    }

    #[test]
    fn test_progress() {
        let mut item = DownloadItem::new("https://example.com/file.zip", "file.zip")
            .with_total_bytes(1000);

        item.update_progress(500, None);
        assert_eq!(item.progress(), Some(50));

        item.update_progress(1000, None);
        assert_eq!(item.progress(), Some(100));
    }

    #[test]
    fn test_eta() {
        let mut item = DownloadItem::new("https://example.com/file.zip", "file.zip")
            .with_total_bytes(1000);

        item.update_progress(500, None);
        item.update_speed(100); // 100 bytes/sec

        assert_eq!(item.eta(), Some(5)); // 500 remaining / 100 = 5 seconds
    }

    #[test]
    fn test_domain_and_extension() {
        let item = DownloadItem::new("https://example.com/path/file.zip", "file.zip");

        assert_eq!(item.domain(), Some("example.com"));
        assert_eq!(item.extension(), Some("zip"));
    }
}
