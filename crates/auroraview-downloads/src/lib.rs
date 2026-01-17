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

pub use error::{DownloadError, Result};
pub use item::{DownloadId, DownloadItem, DownloadState};
pub use manager::DownloadManager;
pub use queue::DownloadQueue;
