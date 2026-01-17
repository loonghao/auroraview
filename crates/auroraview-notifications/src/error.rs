//! Error types for notification operations.

use thiserror::Error;
use uuid::Uuid;

/// Result type for notification operations.
pub type Result<T> = std::result::Result<T, NotificationError>;

/// Errors that can occur during notification operations.
#[derive(Debug, Error)]
pub enum NotificationError {
    /// Notification not found.
    #[error("Notification not found: {0}")]
    NotFound(Uuid),

    /// Permission denied.
    #[error("Notification permission denied")]
    PermissionDenied,

    /// Permission not yet requested.
    #[error("Notification permission not requested")]
    PermissionNotRequested,

    /// Invalid notification data.
    #[error("Invalid notification: {0}")]
    InvalidNotification(String),

    /// Maximum notifications reached.
    #[error("Maximum notifications reached: {0}")]
    MaxNotificationsReached(usize),
}
