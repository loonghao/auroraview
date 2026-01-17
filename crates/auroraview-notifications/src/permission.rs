//! Notification permission handling (Web Notifications API compatible).

use serde::{Deserialize, Serialize};

/// Permission state for notifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PermissionState {
    /// Permission not yet requested.
    #[default]
    Default,
    /// Permission granted.
    Granted,
    /// Permission denied.
    Denied,
}

impl PermissionState {
    /// Returns true if notifications are allowed.
    pub fn is_granted(&self) -> bool {
        matches!(self, Self::Granted)
    }

    /// Returns true if permission was denied.
    pub fn is_denied(&self) -> bool {
        matches!(self, Self::Denied)
    }

    /// Returns true if permission hasn't been requested yet.
    pub fn is_default(&self) -> bool {
        matches!(self, Self::Default)
    }
}

impl std::fmt::Display for PermissionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Default => write!(f, "default"),
            Self::Granted => write!(f, "granted"),
            Self::Denied => write!(f, "denied"),
        }
    }
}

/// Permission configuration for an origin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    /// The origin this permission applies to.
    pub origin: String,
    /// Current permission state.
    pub state: PermissionState,
    /// When the permission was last updated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Permission {
    /// Creates a new permission with default state.
    pub fn new(origin: impl Into<String>) -> Self {
        Self {
            origin: origin.into(),
            state: PermissionState::Default,
            updated_at: None,
        }
    }

    /// Creates a granted permission.
    pub fn granted(origin: impl Into<String>) -> Self {
        Self {
            origin: origin.into(),
            state: PermissionState::Granted,
            updated_at: Some(chrono::Utc::now()),
        }
    }

    /// Creates a denied permission.
    pub fn denied(origin: impl Into<String>) -> Self {
        Self {
            origin: origin.into(),
            state: PermissionState::Denied,
            updated_at: Some(chrono::Utc::now()),
        }
    }

    /// Grants the permission.
    pub fn grant(&mut self) {
        self.state = PermissionState::Granted;
        self.updated_at = Some(chrono::Utc::now());
    }

    /// Denies the permission.
    pub fn deny(&mut self) {
        self.state = PermissionState::Denied;
        self.updated_at = Some(chrono::Utc::now());
    }

    /// Resets to default state.
    pub fn reset(&mut self) {
        self.state = PermissionState::Default;
        self.updated_at = Some(chrono::Utc::now());
    }
}
