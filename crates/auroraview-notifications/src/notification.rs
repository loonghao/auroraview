//! Notification data structures.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type/severity of a notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum NotificationType {
    /// Informational notification.
    #[default]
    Info,
    /// Success notification.
    Success,
    /// Warning notification.
    Warning,
    /// Error notification.
    Error,
}

impl NotificationType {
    /// Returns the default auto-dismiss duration in milliseconds.
    pub fn default_duration(&self) -> Option<u64> {
        match self {
            Self::Info => Some(5000),
            Self::Success => Some(3000),
            Self::Warning => Some(8000),
            Self::Error => None, // Errors don't auto-dismiss
        }
    }
}

/// An action button for a notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationAction {
    /// Unique identifier for this action.
    pub id: String,
    /// Display text for the action button.
    pub label: String,
    /// Optional icon name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

impl NotificationAction {
    /// Creates a new action.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
        }
    }

    /// Sets the icon.
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

/// A notification to display to the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Unique identifier.
    pub id: Uuid,
    /// Title of the notification.
    pub title: String,
    /// Body/message of the notification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// Type/severity.
    #[serde(rename = "type")]
    pub notification_type: NotificationType,
    /// Optional icon URL or name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Optional image URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    /// Action buttons.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<NotificationAction>,
    /// Auto-dismiss duration in milliseconds (None = no auto-dismiss).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<u64>,
    /// Whether the notification requires interaction.
    #[serde(default)]
    pub require_interaction: bool,
    /// Optional tag for grouping/replacing notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// Optional data payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    /// When the notification was created.
    pub created_at: DateTime<Utc>,
    /// When the notification was shown (None if not yet shown).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shown_at: Option<DateTime<Utc>>,
    /// When the notification was dismissed (None if still active).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dismissed_at: Option<DateTime<Utc>>,
}

impl Notification {
    /// Creates a new notification with the given title.
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        let notification_type = NotificationType::default();
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            body: Some(body.into()),
            notification_type,
            icon: None,
            image: None,
            actions: Vec::new(),
            duration: notification_type.default_duration(),
            require_interaction: false,
            tag: None,
            data: None,
            created_at: Utc::now(),
            shown_at: None,
            dismissed_at: None,
        }
    }

    /// Creates a notification with only a title.
    pub fn simple(title: impl Into<String>) -> Self {
        let notification_type = NotificationType::default();
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            body: None,
            notification_type,
            icon: None,
            image: None,
            actions: Vec::new(),
            duration: notification_type.default_duration(),
            require_interaction: false,
            tag: None,
            data: None,
            created_at: Utc::now(),
            shown_at: None,
            dismissed_at: None,
        }
    }

    /// Sets the notification type.
    pub fn with_type(mut self, notification_type: NotificationType) -> Self {
        self.notification_type = notification_type;
        // Update duration to match type if not explicitly set
        self.duration = notification_type.default_duration();
        self
    }

    /// Sets the icon.
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Sets the image.
    pub fn with_image(mut self, image: impl Into<String>) -> Self {
        self.image = Some(image.into());
        self
    }

    /// Adds an action button.
    pub fn with_action(mut self, action: NotificationAction) -> Self {
        self.actions.push(action);
        self
    }

    /// Sets the auto-dismiss duration.
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration = Some(duration_ms);
        self
    }

    /// Disables auto-dismiss.
    pub fn persistent(mut self) -> Self {
        self.duration = None;
        self.require_interaction = true;
        self
    }

    /// Sets the tag for grouping.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }

    /// Sets custom data.
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    /// Returns true if this notification is still active.
    pub fn is_active(&self) -> bool {
        self.dismissed_at.is_none()
    }

    /// Returns true if this notification has been shown.
    pub fn is_shown(&self) -> bool {
        self.shown_at.is_some()
    }

    /// Marks the notification as shown.
    pub fn mark_shown(&mut self) {
        if self.shown_at.is_none() {
            self.shown_at = Some(Utc::now());
        }
    }

    /// Marks the notification as dismissed.
    pub fn mark_dismissed(&mut self) {
        if self.dismissed_at.is_none() {
            self.dismissed_at = Some(Utc::now());
        }
    }
}

impl std::fmt::Display for Notification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(body) = &self.body {
            write!(f, "{}: {}", self.title, body)
        } else {
            write!(f, "{}", self.title)
        }
    }
}
