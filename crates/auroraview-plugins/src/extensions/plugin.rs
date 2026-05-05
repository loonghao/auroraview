//! PluginHandler implementation for ExtensionsPlugin
//!
//! This module contains the PluginHandler trait implementation
//! that routes commands to the appropriate handlers.

use serde_json::Value;

use crate::extensions::{ApiCallRequest, ExtensionIdRequest, ViewIdRequest};
use crate::extensions::{CreateViewRequest, EventDispatchRequest};
use auroraview_extensions::ExtensionViewManager;
use auroraview_plugin_core::{PluginError, PluginHandler, PluginResult, ScopeConfig};

impl Default for crate::extensions::ExtensionsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginHandler for crate::extensions::ExtensionsPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn handle(&self, command: &str, args: Value, _scope: &ScopeConfig) -> PluginResult<Value> {
        match command {
            "api_call" => {
                let req: ApiCallRequest = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                // Route to appropriate API handler
                match req.api.as_str() {
                    "storage" => {
                        self.handle_storage_api(&req.extension_id, &req.method, &req.params)
                    }
                    "tabs" => self.handle_tabs_api(&req.extension_id, &req.method, &req.params),
                    "sidePanel" => {
                        self.handle_side_panel_api(&req.extension_id, &req.method, &req.params)
                    }
                    "runtime" => {
                        self.handle_runtime_api(&req.extension_id, &req.method, &req.params)
                    }
                    "action" => self.handle_action_api(&req.extension_id, &req.method, &req.params),
                    "scripting" => {
                        self.handle_scripting_api(&req.extension_id, &req.method, &req.params)
                    }
                    "alarms" => self.handle_alarms_api(&req.extension_id, &req.method, &req.params),
                    "notifications" => {
                        self.handle_notifications_api(&req.extension_id, &req.method, &req.params)
                    }
                    "contextMenus" => {
                        self.handle_context_menus_api(&req.extension_id, &req.method, &req.params)
                    }
                    "windows" => {
                        self.handle_windows_api(&req.extension_id, &req.method, &req.params)
                    }
                    "commands" => {
                        self.handle_commands_api(&req.extension_id, &req.method, &req.params)
                    }
                    "permissions" => {
                        self.handle_permissions_api(&req.extension_id, &req.method, &req.params)
                    }
                    "identity" => {
                        self.handle_identity_api(&req.extension_id, &req.method, &req.params)
                    }
                    "webRequest" => {
                        self.handle_web_request_api(&req.extension_id, &req.method, &req.params)
                    }
                    "declarativeNetRequest" => self.handle_declarative_net_request_api(
                        &req.extension_id,
                        &req.method,
                        &req.params,
                    ),
                    "offscreen" => {
                        self.handle_offscreen_api(&req.extension_id, &req.method, &req.params)
                    }
                    "management" => {
                        self.handle_management_api(&req.extension_id, &req.method, &req.params)
                    }
                    _ => Err(PluginError::command_not_found(&format!(
                        "Unknown API: {}",
                        req.api
                    ))),
                }
            }
            "list_extensions" => {
                let state = self.state.read();
                let extensions: Vec<&crate::extensions::ExtensionInfo> =
                    state.extensions.values().collect();
                serde_json::to_value(extensions).map_err(PluginError::serialization_error)
            }
            "get_extension" => {
                let req: ExtensionIdRequest = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let state = self.state.read();
                match state.extensions.get(&req.extension_id) {
                    Some(ext) => {
                        serde_json::to_value(ext).map_err(PluginError::serialization_error)
                    }
                    None => Err(PluginError::invalid_args(format!(
                        "Extension not found: {}",
                        req.extension_id
                    ))),
                }
            }
            "get_side_panel" => {
                let req: ExtensionIdRequest = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let state = self.state.read();
                match state.extensions.get(&req.extension_id) {
                    Some(ext) => {
                        if let Some(path) = &ext.side_panel_path {
                            let full_path = std::path::PathBuf::from(&ext.root_dir).join(path);
                            match std::fs::read_to_string(&full_path) {
                                Ok(html) => Ok(serde_json::json!({
                                    "html": html,
                                    "path": full_path.to_string_lossy()
                                })),
                                Err(e) => Err(PluginError::invalid_args(format!(
                                    "Failed to read side panel: {}",
                                    e
                                ))),
                            }
                        } else {
                            Err(PluginError::invalid_args("Extension has no side panel"))
                        }
                    }
                    None => Err(PluginError::invalid_args(format!(
                        "Extension not found: {}",
                        req.extension_id
                    ))),
                }
            }
            "open_side_panel" => {
                let req: ExtensionIdRequest = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let mut state = self.state.write();
                let panel = state
                    .side_panels
                    .entry(req.extension_id.clone())
                    .or_default();

                panel.is_open = true;

                Ok(serde_json::json!({ "success": true }))
            }
            "close_side_panel" => {
                let req: ExtensionIdRequest = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let mut state = self.state.write();
                if let Some(panel) = state.side_panels.get_mut(&req.extension_id) {
                    panel.is_open = false;
                }

                Ok(serde_json::json!({ "success": true }))
            }
            "get_side_panel_state" => {
                let req: ExtensionIdRequest = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let state = self.state.read();
                let panel_state = state
                    .side_panels
                    .get(&req.extension_id)
                    .cloned()
                    .unwrap_or_default();

                serde_json::to_value(panel_state).map_err(PluginError::serialization_error)
            }
            "get_polyfill" => {
                let req: ExtensionIdRequest = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                // Get extension info for path and manifest
                let state = self.state.read();
                let (extension_path, manifest) = state
                    .extensions
                    .get(&req.extension_id)
                    .map(|ext| (ext.root_dir.clone(), ext.manifest.clone()))
                    .unwrap_or_else(|| (String::new(), None));
                drop(state);

                // Attempt to load _locales messages from extension directory
                let messages = if !extension_path.is_empty() {
                    let locales_path =
                        std::path::PathBuf::from(&extension_path).join("_locales/en/messages.json");
                    if locales_path.exists() {
                        std::fs::read_to_string(&locales_path)
                            .ok()
                            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Generate the polyfill script using SDK
                let polyfill = auroraview_extensions::generate_polyfill_from_sdk(
                    &req.extension_id,
                    &extension_path,
                    manifest.as_ref(),
                    messages.as_ref(),
                );
                let wxt_shim = auroraview_extensions::generate_wxt_shim();

                Ok(serde_json::json!({
                    "polyfill": polyfill,
                    "wxtShim": wxt_shim
                }))
            }
            "dispatch_event" => {
                let req: EventDispatchRequest = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                // Dispatch event to extension via callback
                let cbs = self.callbacks.read();
                if let Some(ref dispatch_cb) = cbs.on_event_dispatch {
                    dispatch_cb(&req.extension_id, &req.api, &req.event, &req.args);
                } else {
                    tracing::info!(
                        "Dispatching event {}.{} to {} (no callback registered)",
                        req.api,
                        req.event,
                        req.extension_id
                    );
                }

                Ok(serde_json::json!({ "success": true }))
            }
            // ============================================================
            // Extension View Management (Chrome DevTools-like)
            // ============================================================
            "create_view" => {
                let req: CreateViewRequest = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let view_manager = ExtensionViewManager::global();
                let config = auroraview_extensions::ExtensionViewConfig {
                    extension_id: req.extension_id,
                    view_type: req.view_type.into(),
                    html_path: req.html_path.unwrap_or_default(),
                    title: req.title.unwrap_or_else(|| "Extension View".to_string()),
                    width: req.width.unwrap_or(400),
                    height: req.height.unwrap_or(600),
                    dev_tools: req.dev_tools.unwrap_or(true),
                    debug_port: req.debug_port,
                    visible: req.visible.unwrap_or(true),
                    parent_hwnd: req.parent_hwnd,
                };

                match view_manager.create_view(config) {
                    Ok(info) => {
                        serde_json::to_value(info).map_err(PluginError::serialization_error)
                    }
                    Err(e) => Err(PluginError::from_plugin("extensions", e)),
                }
            }
            "get_view" => {
                let req: ViewIdRequest = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let view_manager = ExtensionViewManager::global();
                match view_manager.get_view(&req.view_id) {
                    Some(info) => {
                        serde_json::to_value(info).map_err(PluginError::serialization_error)
                    }
                    None => Err(PluginError::invalid_args(format!(
                        "View not found: {}",
                        req.view_id
                    ))),
                }
            }
            "get_extension_views" => {
                let req: ExtensionIdRequest = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let view_manager = ExtensionViewManager::global();
                let views = view_manager.get_extension_views(&req.extension_id);
                serde_json::to_value(views).map_err(PluginError::serialization_error)
            }
            "get_all_views" => {
                let view_manager = ExtensionViewManager::global();
                let views = view_manager.get_all_views();
                serde_json::to_value(views).map_err(PluginError::serialization_error)
            }
            "open_devtools" => {
                let req: ViewIdRequest = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let view_manager = ExtensionViewManager::global();
                match view_manager.open_devtools(&req.view_id) {
                    Ok(()) => Ok(serde_json::json!({ "success": true })),
                    Err(e) => Err(PluginError::from_plugin("extensions", e)),
                }
            }
            "close_devtools" => {
                let req: ViewIdRequest = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let view_manager = ExtensionViewManager::global();
                match view_manager.close_devtools(&req.view_id) {
                    Ok(()) => Ok(serde_json::json!({ "success": true })),
                    Err(e) => Err(PluginError::from_plugin("extensions", e)),
                }
            }
            "show_view" => {
                let req: ViewIdRequest = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let view_manager = ExtensionViewManager::global();
                match view_manager.show_view(&req.view_id) {
                    Ok(()) => Ok(serde_json::json!({ "success": true })),
                    Err(e) => Err(PluginError::from_plugin("extensions", e)),
                }
            }
            "hide_view" => {
                let req: ViewIdRequest = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let view_manager = ExtensionViewManager::global();
                match view_manager.hide_view(&req.view_id) {
                    Ok(()) => Ok(serde_json::json!({ "success": true })),
                    Err(e) => Err(PluginError::from_plugin("extensions", e)),
                }
            }
            "destroy_view" => {
                let req: ViewIdRequest = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let view_manager = ExtensionViewManager::global();
                match view_manager.destroy_view(&req.view_id) {
                    Ok(()) => Ok(serde_json::json!({ "success": true })),
                    Err(e) => Err(PluginError::from_plugin("extensions", e)),
                }
            }
            "get_cdp_info" => {
                let req: ViewIdRequest = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let view_manager = ExtensionViewManager::global();
                match view_manager.get_cdp_info(&req.view_id) {
                    Some(info) => {
                        serde_json::to_value(info).map_err(PluginError::serialization_error)
                    }
                    None => Err(PluginError::invalid_args(format!(
                        "View not found: {}",
                        req.view_id
                    ))),
                }
            }
            "get_all_cdp_connections" => {
                let view_manager = ExtensionViewManager::global();
                let connections = view_manager.get_all_cdp_connections();
                serde_json::to_value(connections).map_err(PluginError::serialization_error)
            }
            _ => Err(PluginError::command_not_found(command)),
        }
    }

    fn commands(&self) -> Vec<&str> {
        vec![
            "api_call",
            "list_extensions",
            "get_extension",
            "get_side_panel",
            "open_side_panel",
            "close_side_panel",
            "get_side_panel_state",
            "get_polyfill",
            "dispatch_event",
            // Extension View Management (Chrome DevTools-like)
            "create_view",
            "get_view",
            "get_extension_views",
            "get_all_views",
            "open_devtools",
            "close_devtools",
            "show_view",
            "hide_view",
            "destroy_view",
            "get_cdp_info",
            "get_all_cdp_connections",
        ]
    }
}
