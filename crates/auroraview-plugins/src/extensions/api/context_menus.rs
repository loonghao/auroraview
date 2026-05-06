use serde_json::Value;

use crate::extensions::ExtensionsPlugin;
use crate::extensions::MenuItemInfo;
use auroraview_plugin_core::{PluginError, PluginResult};

impl ExtensionsPlugin {
    /// Handle contextMenus API calls
    pub fn handle_context_menus_api(
        &self,
        extension_id: &str,
        method: &str,
        params: &Value,
    ) -> PluginResult<Value> {
        match method {
            "create" => {
                let id = params
                    .get("id")
                    .and_then(|v| v.as_str())
                    .map(String::from)
                    .unwrap_or_else(|| {
                        format!(
                            "menu_{}",
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_nanos()
                        )
                    });

                let menu_item = MenuItemInfo {
                    id: id.clone(),
                    title: params
                        .get("title")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    item_type: params
                        .get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("normal")
                        .to_string(),
                    contexts: params
                        .get("contexts")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or_else(|| vec!["page".to_string()]),
                    parent_id: params
                        .get("parentId")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    enabled: params
                        .get("enabled")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true),
                    visible: params
                        .get("visible")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true),
                };

                let mut state = self.state.write();
                let ext_menus = state
                    .context_menus
                    .entry(extension_id.to_string())
                    .or_default();

                ext_menus.insert(id.clone(), menu_item);

                Ok(serde_json::json!(id))
            }
            "update" => {
                let id = params
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| PluginError::invalid_args("id is required"))?;

                let mut state = self.state.write();
                if let Some(ext_menus) = state.context_menus.get_mut(extension_id) {
                    if let Some(menu) = ext_menus.get_mut(id) {
                        if let Some(updates) = params.get("updateProperties") {
                            // Update menu item properties
                            if let Some(title) = updates.get("title").and_then(|v| v.as_str()) {
                                menu.title = Some(title.to_string());
                            }
                        }
                    }
                }

                Ok(serde_json::json!({}))
            }
            "remove" => {
                let id = params
                    .get("menuItemId")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| PluginError::invalid_args("menuItemId is required"))?;

                let mut state = self.state.write();
                if let Some(ext_menus) = state.context_menus.get_mut(extension_id) {
                    ext_menus.remove(id);
                }

                Ok(serde_json::json!({}))
            }
            "removeAll" => {
                let mut state = self.state.write();
                if let Some(ext_menus) = state.context_menus.get_mut(extension_id) {
                    ext_menus.clear();
                }

                Ok(serde_json::json!({}))
            }
            _ => Err(PluginError::command_not_found(&format!(
                "contextMenus.{}",
                method
            ))),
        }
    }
}
