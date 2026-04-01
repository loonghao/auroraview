//! AuroraView History Management
//!
//! This crate provides browsing history functionality that can be used
//! by both WebView and Browser applications.
//!
//! # Features
//!
//! - History entry storage
//! - Persistent storage (JSON file)
//! - Full-text search
//! - Visit count tracking
//! - Date-based filtering
//!
//! # Example
//!
//! ```rust,ignore
//! use auroraview_history::{HistoryManager, HistoryEntry};
//!
//! let manager = HistoryManager::new(None);
//!
//! // Record a visit
//! manager.visit("https://github.com", "GitHub");
//!
//! // Search history
//! let results = manager.search("git");
//!
//! // Get recent history
//! let recent = manager.recent(10);
//! ```

mod entry;
mod error;
mod manager;
mod search;

/// Single history entry with URL, title, and visit metadata.
pub use entry::HistoryEntry;
/// Error and result types for history operations.
pub use error::{HistoryError, Result};
/// History manager for recording, querying, and persisting browsing history.
pub use manager::HistoryManager;
/// Search configuration and result types for history queries.
pub use search::{SearchOptions, SearchResult};

/// Unique identifier for history entries
pub type HistoryId = String;
