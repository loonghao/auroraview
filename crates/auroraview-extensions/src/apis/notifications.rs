//! chrome.notifications API handler
//!
//! Provides system notification functionality for extensions.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::apis::ApiHandler;
use crate::error::{ExtensionError, ExtensionResult};
use crate::ExtensionId;

/// Notification template type
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TemplateType {
    /// Basic notification with icon, title, and message
    #[default]
    Basic,
    /// Notification with image
    Image,
    /// Notification with list items
    List,
    /// Notification with progress bar
    Progress,
}

/// Notification button
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationButton {
    /// Button title
    pub title: String,
    /// Button icon URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
}

/// Notification item (for list type)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationItem {
    /// Item title
    pub title: String,
    /// Item message
    pub message: String,
}

/// Notification creation options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationOptions {
    /// Template type
    #[serde(rename = "type")]
    pub template_type: TemplateType,
    /// Icon URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    /// App icon mask URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_icon_mask_url: Option<String>,
    /// Notification title
    pub title: String,
    /// Notification message
    pub message: String,
    /// Context message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_message: Option<String>,
    /// Priority (-2 to 2)
    #[serde(default)]
    pub priority: i32,
    /// Event timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_time: Option<f64>,
    /// Buttons (max 2)
    #[serde(default)]
    pub buttons: Vec<NotificationButton>,
    /// Image URL (for image type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    /// List items (for list type)
    #[serde(default)]
    pub items: Vec<NotificationItem>,
    /// Progress (0-100, for progress type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<i32>,
    /// Whether notification requires interaction
    #[serde(default)]
    pub require_interaction: bool,
    /// Silent notification
    #[serde(default)]
    pub silent: bool,
}

/// Stored notification
#[derive(Debug, Clone)]
pub struct Notification {
    /// Notification ID
    pub id: String,
    /// Extension ID
    pub extension_id: ExtensionId,
    /// Notification options
    pub options: NotificationOptions,
    /// Creation timestamp
    pub created_at: f64,
}

/// Notifications manager
pub struct NotificationsManager {
    /// Active notifications
    notifications: RwLock<HashMap<String, Notification>>,
    /// Notification ID counter
    next_id: RwLock<u64>,
}

impl NotificationsManager {
    /// Create a new notifications manager
    pub fn new() -> Self {
        Self {
            notifications: RwLock::new(HashMap::new()),
            next_id: RwLock::new(1),
        }
    }

    /// Generate a notification ID
    fn generate_id(&self) -> String {
        let mut id = self.next_id.write();
        let current = *id;
        *id += 1;
        format!("notif_{}", current)
    }

    /// Create a notification
    pub fn create(
        &self,
        extension_id: &str,
        id: Option<String>,
        options: NotificationOptions,
    ) -> ExtensionResult<String> {
        let id = id.unwrap_or_else(|| self.generate_id());

        let notification = Notification {
            id: id.clone(),
            extension_id: extension_id.to_string(),
            options,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64(),
        };

        // TODO: Actually show system notification using native APIs
        // For now, just store it
        self.notifications.write().insert(id.clone(), notification);

        Ok(id)
    }

    /// Update a notification
    pub fn update(
        &self,
        extension_id: &str,
        id: &str,
        options: NotificationOptions,
    ) -> ExtensionResult<bool> {
        let mut notifications = self.notifications.write();
        if let Some(notification) = notifications.get_mut(id) {
            if notification.extension_id == extension_id {
                notification.options = options;
                // TODO: Actually update system notification
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Clear a notification
    pub fn clear(&self, extension_id: &str, id: &str) -> ExtensionResult<bool> {
        let mut notifications = self.notifications.write();
        if let Some(notification) = notifications.get(id) {
            if notification.extension_id == extension_id {
                notifications.remove(id);
                // TODO: Actually dismiss system notification
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Get all notifications for an extension
    pub fn get_all(&self, extension_id: &str) -> HashMap<String, NotificationOptions> {
        let notifications = self.notifications.read();
        notifications
            .iter()
            .filter(|(_, n)| n.extension_id == extension_id)
            .map(|(id, n)| (id.clone(), n.options.clone()))
            .collect()
    }

    /// Get permission level
    pub fn get_permission_level(&self) -> &'static str {
        // In AuroraView, we always grant notification permission
        "granted"
    }
}

impl Default for NotificationsManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Notifications API handler
pub struct NotificationsApiHandler {
    manager: Arc<NotificationsManager>,
}

impl NotificationsApiHandler {
    /// Create a new notifications API handler
    pub fn new(manager: Arc<NotificationsManager>) -> Self {
        Self { manager }
    }
}

impl ApiHandler for NotificationsApiHandler {
    fn namespace(&self) -> &str {
        "notifications"
    }

    fn handle(&self, method: &str, params: Value, extension_id: &str) -> ExtensionResult<Value> {
        match method {
            "create" => {
                let id = params
                    .get("notificationId")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let options: NotificationOptions = params
                    .get("options")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .ok_or_else(|| {
                        ExtensionError::InvalidArgument("options is required".to_string())
                    })?;

                let id = self.manager.create(extension_id, id, options)?;
                Ok(serde_json::json!(id))
            }
            "update" => {
                let id = params
                    .get("notificationId")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ExtensionError::InvalidArgument("notificationId is required".to_string())
                    })?;

                let options: NotificationOptions = params
                    .get("options")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .ok_or_else(|| {
                        ExtensionError::InvalidArgument("options is required".to_string())
                    })?;

                let was_updated = self.manager.update(extension_id, id, options)?;
                Ok(serde_json::json!(was_updated))
            }
            "clear" => {
                let id = params
                    .get("notificationId")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ExtensionError::InvalidArgument("notificationId is required".to_string())
                    })?;

                let was_cleared = self.manager.clear(extension_id, id)?;
                Ok(serde_json::json!(was_cleared))
            }
            "getAll" => {
                let notifications = self.manager.get_all(extension_id);
                Ok(serde_json::to_value(notifications)?)
            }
            "getPermissionLevel" => {
                let level = self.manager.get_permission_level();
                Ok(serde_json::json!(level))
            }
            _ => Err(ExtensionError::ApiNotSupported(format!(
                "notifications.{} is not supported",
                method
            ))),
        }
    }

    fn methods(&self) -> Vec<&str> {
        vec!["create", "update", "clear", "getAll", "getPermissionLevel"]
    }
}
