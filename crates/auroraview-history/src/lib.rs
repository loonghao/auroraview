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

pub use entry::HistoryEntry;
pub use error::{HistoryError, Result};
pub use manager::HistoryManager;
pub use search::{SearchOptions, SearchResult};

/// Unique identifier for history entries
pub type HistoryId = String;
