//! AuroraView Download Management
//!
//! This crate provides download management functionality that can be used
//! by both WebView and Browser applications.
//!
//! # Features
//!
//! - Download queue management
//! - Progress tracking
//! - Download history
//! - Persistence
//!
//! # Example
//!
//! ```rust,ignore
//! use auroraview_downloads::{DownloadManager, DownloadItem};
//!
//! let manager = DownloadManager::new(None);
//!
//! // Add a download
//! let id = manager.add("https://example.com/file.zip", "file.zip");
//!
//! // Update progress
//! manager.update_progress(&id, 50, 100);
//!
//! // Get all downloads
//! let downloads = manager.all();
//! ```

mod error;
mod item;
mod manager;
mod queue;

/// Error and result types for download operations.
pub use error::{DownloadError, Result};
/// Download item types: identifier, state, and metadata.
pub use item::{DownloadId, DownloadItem, DownloadState};
/// High-level download manager for tracking and controlling downloads.
pub use manager::DownloadManager;
/// Download queue for concurrent download scheduling.
pub use queue::DownloadQueue;
