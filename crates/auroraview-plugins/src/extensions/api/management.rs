use serde_json::Value;

use crate::extensions::ExtensionsPlugin;
use auroraview_plugin_core::{PluginError, PluginResult};

impl ExtensionsPlugin {
    /// Handle management API calls
    pub fn handle_management_api(
        &self,
        extension_id: &str,
        method: &str,
        params: &Value,
    ) -> PluginResult<Value> {
        let state = self.state.read();

        match method {
            "getAll" => {
                // Return all extensions
                let extensions: Vec<serde_json::Value> = state
                    .extensions
                    .values()
                    .map(|ext| {
                        serde_json::json!({
                            "id": ext.id,
                            "name": ext.name,
                            "version": ext.version,
                            "description": ext.description,
                            "enabled": ext.enabled,
                            "mayDisable": true,
                            "mayEnable": true,
                            "isApp": false,
                            "type": "extension",
                            "offlineEnabled": true,
                            "permissions": ext.permissions,
                            "hostPermissions": ext.host_permissions,
                            "installType": "development"
                        })
                    })
                    .collect();
                serde_json::to_value(extensions).map_err(PluginError::serialization_error)
            }
            "get" => {
                let id = params
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| PluginError::invalid_args("Missing id"))?;

                match state.extensions.get(id) {
                    Some(ext) => Ok(serde_json::json!({
                        "id": ext.id,
                        "name": ext.name,
                        "version": ext.version,
                        "description": ext.description,
                        "enabled": ext.enabled,
                        "mayDisable": true,
                        "mayEnable": true,
                        "isApp": false,
                        "type": "extension",
                        "offlineEnabled": true,
                        "permissions": ext.permissions,
                        "hostPermissions": ext.host_permissions,
                        "installType": "development"
                    })),
                    None => Err(PluginError::invalid_args(format!(
                        "Extension not found: {}",
                        id
                    ))),
                }
            }
            "getSelf" => match state.extensions.get(extension_id) {
                Some(ext) => Ok(serde_json::json!({
                    "id": ext.id,
                    "name": ext.name,
                    "version": ext.version,
                    "description": ext.description,
                    "enabled": ext.enabled,
                    "mayDisable": true,
                    "mayEnable": true,
                    "isApp": false,
                    "type": "extension",
                    "offlineEnabled": true,
                    "permissions": ext.permissions,
                    "hostPermissions": ext.host_permissions,
                    "installType": "development"
                })),
                None => Err(PluginError::invalid_args(format!(
                    "Extension not found: {}",
                    extension_id
                ))),
            },
            "setEnabled" => {
                let id = params
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| PluginError::invalid_args("Missing id"))?;
                let enabled = params
                    .get("enabled")
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| PluginError::invalid_args("Missing enabled"))?;

                drop(state);
                let mut state = self.state.write();

                // Update if extension exists in our state, otherwise just return success
                // (the extension might be managed by WebView2 directly)
                if let Some(ext) = state.extensions.get_mut(id) {
                    ext.enabled = enabled;
                }

                // Always return success - the actual enabled state is managed by the frontend
                Ok(serde_json::json!(null))
            }
            "getPermissionWarningsById" => {
                let id = params
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| PluginError::invalid_args("Missing id"))?;

                match state.extensions.get(id) {
                    Some(ext) => {
                        // Generate permission warnings based on permissions
                        let warnings: Vec<String> = ext
                            .permissions
                            .iter()
                            .filter_map(|p| match p.as_str() {
                                "tabs" => Some("Read your browsing history".to_string()),
                                "history" => {
                                    Some("Read and change your browsing history".to_string())
                                }
                                "downloads" => Some("Manage your downloads".to_string()),
                                "bookmarks" => Some("Read and change your bookmarks".to_string()),
                                "cookies" => {
                                    Some("Read and change all your data on websites".to_string())
                                }
                                "storage" => Some("Store data in this application".to_string()),
                                _ => None,
                            })
                            .collect();
                        serde_json::to_value(warnings).map_err(PluginError::serialization_error)
                    }
                    // Return empty array if extension not found in our state
                    // (it might be managed by WebView2 directly)
                    None => Ok(serde_json::json!([])),
                }
            }
            "uninstall" => {
                let id = params
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| PluginError::invalid_args("Missing id"))?;

                drop(state);
                let mut state = self.state.write();

                if state.extensions.remove(id).is_some() {
                    Ok(serde_json::json!(null))
                } else {
                    Err(PluginError::invalid_args(format!(
                        "Extension not found: {}",
                        id
                    )))
                }
            }
            "uninstallSelf" => {
                drop(state);
                let mut state = self.state.write();
                state.extensions.remove(extension_id);
                Ok(serde_json::json!(null))
            }
            _ => Err(PluginError::command_not_found(&format!(
                "management.{}",
                method
            ))),
        }
    }
}
