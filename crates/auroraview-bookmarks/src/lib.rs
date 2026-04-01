//! AuroraView Bookmark Management
//!
//! This crate provides bookmark management functionality that can be used
//! by both WebView and Browser applications.
//!
//! # Features
//!
//! - Bookmark storage with folders
//! - Persistent storage (JSON file)
//! - Thread-safe operations
//! - Favicon support
//!
//! # Example
//!
//! ```rust,ignore
//! use auroraview_bookmarks::{BookmarkManager, Bookmark};
//!
//! let manager = BookmarkManager::new(None);
//!
//! // Add a bookmark
//! let id = manager.add_bookmark("https://github.com", "GitHub");
//!
//! // Check if URL is bookmarked
//! assert!(manager.is_bookmarked("https://github.com"));
//!
//! // Get all bookmarks
//! let bookmarks = manager.all();
//! ```

mod bookmark;
mod error;
mod folder;
mod manager;

/// Single bookmark entry with URL, title, and metadata.
pub use bookmark::Bookmark;
/// Error and result types for bookmark operations.
pub use error::{BookmarkError, Result};
/// Bookmark folder for hierarchical organization.
pub use folder::BookmarkFolder;
/// Bookmark manager for CRUD operations and persistence.
pub use manager::BookmarkManager;

/// Unique identifier for bookmarks and folders
pub type BookmarkId = String;
