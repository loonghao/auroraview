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

pub use error::{TabError, Result};
pub use event::TabEvent;
pub use group::{TabGroup, TabGroupId};
pub use manager::TabManager;
pub use session::{Session, SessionManager};
pub use state::{SecurityState, TabId, TabState};
