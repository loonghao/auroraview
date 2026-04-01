//! AuroraView Tab Management
//!
//! This crate provides WebView-agnostic tab management functionality.
//! It handles tab state, ordering, groups, and events without depending
//! on any specific WebView implementation.
//!
//! # Features
//!
//! - Tab state management
//! - Tab groups
//! - Session persistence
//! - Tab events
//!
//! # Example
//!
//! ```rust,ignore
//! use auroraview_tabs::{TabManager, TabState};
//!
//! let mut manager = TabManager::new();
//!
//! // Create a tab
//! let id = manager.create("https://github.com");
//!
//! // Update tab state
//! manager.update_title(&id, "GitHub");
//!
//! // Get all tabs
//! let tabs = manager.all();
//! ```

mod error;
mod event;
mod group;
mod manager;
mod session;
mod state;

/// Error and result types for tab operations.
pub use error::{Result, TabError};
/// Tab lifecycle event types (created, closed, activated, etc.).
pub use event::TabEvent;
/// Tab grouping types for organizing related tabs.
pub use group::{TabGroup, TabGroupId};
/// Tab manager for creating, switching, reordering, and closing tabs.
pub use manager::TabManager;
/// Session persistence types for saving and restoring tab state.
pub use session::{Session, SessionManager};
/// Tab state types: identifier, loading state, and security indicators.
pub use state::{SecurityState, TabId, TabState};
