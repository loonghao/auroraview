//! Notification system for AuroraView.
//!
//! This crate provides a notification system with support for:
//! - Multiple notification types (info, success, warning, error)
//! - Permission management (Web Notifications API compatible)
//! - Actions/buttons in notifications
//! - Auto-dismiss with configurable duration
//! - Notification history
//!
//! # Example
//!
//! ```rust
//! use auroraview_notifications::{NotificationManager, Notification, NotificationType};
//!
//! let manager = NotificationManager::new();
//!
//! // Show a simple notification
//! let id = manager.notify(Notification::new("Hello", "Welcome to AuroraView")).unwrap();
//!
//! // Show a notification with type
//! manager.notify(
//!     Notification::new("Success", "Operation completed")
//!         .with_type(NotificationType::Success)
//! ).unwrap();
//!
//! // Dismiss a notification
//! manager.dismiss(id).unwrap();
//! ```

mod error;
mod manager;
mod notification;
mod permission;

pub use error::{NotificationError, Result};
pub use manager::NotificationManager;
pub use notification::{Notification, NotificationAction, NotificationType};
pub use permission::{Permission, PermissionState};
