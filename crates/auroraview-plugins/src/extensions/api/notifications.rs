use serde_json::Value;

use crate::extensions::ExtensionsPlugin;
use crate::extensions::NotificationInfo;
use auroraview_plugin_core::{PluginError, PluginResult};

impl ExtensionsPlugin {
    /// Handle notifications API calls
    pub fn handle_notifications_api(
        &self,
        extension_id: &str,
        method: &str,
        params: &Value,
    ) -> PluginResult<Value> {
        match method {
            "create" => {
                let id = params
                    .get("notificationId")
                    .and_then(|v| v.as_str())
                    .map(String::from)
                    .unwrap_or_else(|| {
                        format!(
                            "notif_{}",
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_nanos()
                        )
                    });

                let options = params.get("options").cloned().unwrap_or(params.clone());
                let title = options
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let message = options
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let icon_url = options
                    .get("iconUrl")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                let notification_type = options
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("basic")
                    .to_string();
                let created_at = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as i64;

                let notification = NotificationInfo {
                    id: id.clone(),
                    title,
                    message,
                    icon_url,
                    notification_type,
                    created_at,
                };

                // Show system notification via callback
                let cbs = self.callbacks.read();
                if let Some(ref notif_cb) = cbs.on_notification {
                    notif_cb(&notification);
                } else {
                    tracing::info!("Creating notification: {:?}", notification);
                }

                let mut state = self.state.write();
                let ext_notifs = state
                    .notifications
                    .entry(extension_id.to_string())
                    .or_default();

                ext_notifs.insert(id.clone(), notification);

                Ok(serde_json::json!(id))
            }
            "update" => {
                let id = params
                    .get("notificationId")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| PluginError::invalid_args("notificationId is required"))?;

                let state = self.state.read();
                let exists = state
                    .notifications
                    .get(extension_id)
                    .map(|n| n.contains_key(id))
                    .unwrap_or(false);

                Ok(serde_json::json!(exists))
            }
            "clear" => {
                let id = params
                    .get("notificationId")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| PluginError::invalid_args("notificationId is required"))?;

                let mut state = self.state.write();
                if let Some(notifs) = state.notifications.get_mut(extension_id) {
                    let cleared = notifs.remove(id).is_some();
                    Ok(serde_json::json!(cleared))
                } else {
                    Ok(serde_json::json!(false))
                }
            }
            "getAll" => {
                let state = self.state.read();
                let notifs: std::collections::HashMap<String, bool> = state
                    .notifications
                    .get(extension_id)
                    .map(|n| n.keys().map(|k| (k.clone(), true)).collect())
                    .unwrap_or_default();
                serde_json::to_value(notifs).map_err(PluginError::serialization_error)
            }
            "getPermissionLevel" => Ok(serde_json::json!("granted")),
            _ => Err(PluginError::command_not_found(&format!(
                "notifications.{}",
                method
            ))),
        }
    }
}
