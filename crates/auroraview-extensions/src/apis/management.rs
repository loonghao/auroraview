//! Chrome Management API Implementation
//!
//! Provides functionality to manage installed extensions and apps.
//!
//! ## Features
//! - Get list of installed extensions
//! - Enable/disable extensions
//! - Uninstall extensions
//! - Event notifications

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::error::{ExtensionError, ExtensionResult};

/// Extension type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum ExtensionType {
    #[default]
    Extension,
    HostedApp,
    PackagedApp,
    LegacyPackagedApp,
    Theme,
    LoginScreenExtension,
}

/// Install type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum InstallType {
    Admin,
    Development,
    #[default]
    Normal,
    Sideload,
    Other,
}

/// Launch type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LaunchType {
    OpenAsRegularTab,
    OpenAsPinnedTab,
    OpenAsWindow,
    OpenFullScreen,
}

/// Extension info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionInfo {
    /// Extension ID
    pub id: String,
    /// Extension name
    pub name: String,
    /// Short name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_name: Option<String>,
    /// Description
    pub description: String,
    /// Version
    pub version: String,
    /// Version name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_name: Option<String>,
    /// May disable
    pub may_disable: bool,
    /// May enable
    pub may_enable: bool,
    /// Enabled
    pub enabled: bool,
    /// Disabled reason
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled_reason: Option<String>,
    /// Is app
    pub is_app: bool,
    /// Type
    #[serde(rename = "type")]
    pub extension_type: ExtensionType,
    /// App launch URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_launch_url: Option<String>,
    /// Homepage URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage_url: Option<String>,
    /// Update URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_url: Option<String>,
    /// Offline enabled
    pub offline_enabled: bool,
    /// Options URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options_url: Option<String>,
    /// Icons
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icons: Option<Vec<IconInfo>>,
    /// Permissions
    pub permissions: Vec<String>,
    /// Host permissions
    pub host_permissions: Vec<String>,
    /// Install type
    pub install_type: InstallType,
    /// Launch type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub launch_type: Option<LaunchType>,
    /// Available launch types
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_launch_types: Option<Vec<LaunchType>>,
}

/// Icon info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconInfo {
    /// Icon size
    pub size: i32,
    /// Icon URL
    pub url: String,
}

/// Uninstall options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UninstallOptions {
    /// Show confirm dialog
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_confirm_dialog: Option<bool>,
}

/// Management API handler
pub struct ManagementApi {
    /// In-memory extension storage
    extensions: Arc<RwLock<HashMap<String, ExtensionInfo>>>,
}

impl Default for ManagementApi {
    fn default() -> Self {
        Self::new()
    }
}

impl ManagementApi {
    /// Create a new ManagementApi instance
    pub fn new() -> Self {
        let api = Self {
            extensions: Arc::new(RwLock::new(HashMap::new())),
        };
        api.init_self_extension();
        api
    }

    /// Initialize self extension info
    fn init_self_extension(&self) {
        let mut extensions = self.extensions.write().unwrap();
        extensions.insert(
            "auroraview-host".to_string(),
            ExtensionInfo {
                id: "auroraview-host".to_string(),
                name: "AuroraView Host".to_string(),
                short_name: Some("AuroraView".to_string()),
                description: "AuroraView extension host environment".to_string(),
                version: "1.0.0".to_string(),
                version_name: Some("1.0.0".to_string()),
                may_disable: false,
                may_enable: true,
                enabled: true,
                disabled_reason: None,
                is_app: false,
                extension_type: ExtensionType::Extension,
                app_launch_url: None,
                homepage_url: Some("https://github.com/AuroraView".to_string()),
                update_url: None,
                offline_enabled: true,
                options_url: None,
                icons: None,
                permissions: vec!["storage".to_string(), "tabs".to_string()],
                host_permissions: vec!["<all_urls>".to_string()],
                install_type: InstallType::Development,
                launch_type: None,
                available_launch_types: None,
            },
        );
    }

    /// Get all extensions
    pub fn get_all(&self) -> ExtensionResult<Value> {
        let extensions = self.extensions.read().unwrap();
        let list: Vec<&ExtensionInfo> = extensions.values().collect();
        Ok(serde_json::to_value(list)?)
    }

    /// Get extension by ID
    pub fn get(&self, id: &str) -> ExtensionResult<Value> {
        let extensions = self.extensions.read().unwrap();
        let info = extensions
            .get(id)
            .ok_or_else(|| ExtensionError::NotFound(format!("Extension {} not found", id)))?;
        Ok(serde_json::to_value(info)?)
    }

    /// Get self (the calling extension)
    pub fn get_self(&self, extension_id: &str) -> ExtensionResult<Value> {
        self.get(extension_id)
    }

    /// Get permission warnings by ID
    pub fn get_permission_warnings_by_id(&self, id: &str) -> ExtensionResult<Value> {
        let extensions = self.extensions.read().unwrap();
        let info = extensions
            .get(id)
            .ok_or_else(|| ExtensionError::NotFound(format!("Extension {} not found", id)))?;

        // Generate permission warnings based on permissions
        let warnings: Vec<String> = info
            .permissions
            .iter()
            .filter_map(|p| match p.as_str() {
                "tabs" => Some("Read your browsing history".to_string()),
                "history" => Some("Read and change your browsing history".to_string()),
                "downloads" => Some("Manage your downloads".to_string()),
                "bookmarks" => Some("Read and change your bookmarks".to_string()),
                "cookies" => Some("Read and change all your data on websites".to_string()),
                _ => None,
            })
            .collect();

        Ok(serde_json::to_value(warnings)?)
    }

    /// Set enabled state
    pub fn set_enabled(&self, id: &str, enabled: bool) -> ExtensionResult<Value> {
        let mut extensions = self.extensions.write().unwrap();
        let info = extensions
            .get_mut(id)
            .ok_or_else(|| ExtensionError::NotFound(format!("Extension {} not found", id)))?;

        if !info.may_disable && !enabled {
            return Err(ExtensionError::PermissionDenied(
                "Extension cannot be disabled".into(),
            ));
        }

        info.enabled = enabled;
        Ok(json!(null))
    }

    /// Uninstall extension
    pub fn uninstall(&self, id: &str, _options: UninstallOptions) -> ExtensionResult<Value> {
        let mut extensions = self.extensions.write().unwrap();

        if !extensions.contains_key(id) {
            return Err(ExtensionError::NotFound(format!(
                "Extension {} not found",
                id
            )));
        }

        extensions.remove(id);
        Ok(json!(null))
    }

    /// Uninstall self
    pub fn uninstall_self(
        &self,
        extension_id: &str,
        _options: UninstallOptions,
    ) -> ExtensionResult<Value> {
        let mut extensions = self.extensions.write().unwrap();
        extensions.remove(extension_id);
        Ok(json!(null))
    }

    /// Launch app
    pub fn launch_app(&self, id: &str) -> ExtensionResult<Value> {
        let extensions = self.extensions.read().unwrap();
        let info = extensions
            .get(id)
            .ok_or_else(|| ExtensionError::NotFound(format!("Extension {} not found", id)))?;

        if !info.is_app {
            return Err(ExtensionError::InvalidParams("Not an app".into()));
        }

        // In a real implementation, this would launch the app
        Ok(json!(null))
    }

    /// Create app shortcut
    pub fn create_app_shortcut(&self, id: &str) -> ExtensionResult<Value> {
        let extensions = self.extensions.read().unwrap();
        let info = extensions
            .get(id)
            .ok_or_else(|| ExtensionError::NotFound(format!("Extension {} not found", id)))?;

        if !info.is_app {
            return Err(ExtensionError::InvalidParams("Not an app".into()));
        }

        // In a real implementation, this would create a shortcut
        Ok(json!(null))
    }

    /// Set launch type
    pub fn set_launch_type(&self, id: &str, launch_type: LaunchType) -> ExtensionResult<Value> {
        let mut extensions = self.extensions.write().unwrap();
        let info = extensions
            .get_mut(id)
            .ok_or_else(|| ExtensionError::NotFound(format!("Extension {} not found", id)))?;

        if !info.is_app {
            return Err(ExtensionError::InvalidParams("Not an app".into()));
        }

        info.launch_type = Some(launch_type);
        Ok(json!(null))
    }

    /// Generate temporary ID
    pub fn generate_app_for_link(&self, _url: &str, _title: &str) -> ExtensionResult<Value> {
        // This would create a temporary app for a URL
        Ok(json!({
            "id": format!("generated-app-{}", uuid::Uuid::new_v4()),
            "isApp": true
        }))
    }

    /// Register extension (internal use)
    pub fn register_extension(&self, info: ExtensionInfo) {
        let mut extensions = self.extensions.write().unwrap();
        extensions.insert(info.id.clone(), info);
    }

    /// Handle API call
    pub fn handle(
        &self,
        method: &str,
        params: Value,
        extension_id: &str,
    ) -> ExtensionResult<Value> {
        match method {
            "getAll" => self.get_all(),
            "get" => {
                let id = params
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ExtensionError::InvalidParams("Missing id".into()))?;
                self.get(id)
            }
            "getSelf" => self.get_self(extension_id),
            "getPermissionWarningsById" => {
                let id = params
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ExtensionError::InvalidParams("Missing id".into()))?;
                self.get_permission_warnings_by_id(id)
            }
            "setEnabled" => {
                let id = params
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ExtensionError::InvalidParams("Missing id".into()))?;
                let enabled = params
                    .get("enabled")
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| ExtensionError::InvalidParams("Missing enabled".into()))?;
                self.set_enabled(id, enabled)
            }
            "uninstall" => {
                let id = params
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ExtensionError::InvalidParams("Missing id".into()))?;
                let options: UninstallOptions = params
                    .get("options")
                    .cloned()
                    .map(|v| serde_json::from_value(v).unwrap_or_default())
                    .unwrap_or_default();
                self.uninstall(id, options)
            }
            "uninstallSelf" => {
                let options: UninstallOptions = params
                    .get("options")
                    .cloned()
                    .map(|v| serde_json::from_value(v).unwrap_or_default())
                    .unwrap_or_default();
                self.uninstall_self(extension_id, options)
            }
            "launchApp" => {
                let id = params
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ExtensionError::InvalidParams("Missing id".into()))?;
                self.launch_app(id)
            }
            "createAppShortcut" => {
                let id = params
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ExtensionError::InvalidParams("Missing id".into()))?;
                self.create_app_shortcut(id)
            }
            "setLaunchType" => {
                let id = params
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ExtensionError::InvalidParams("Missing id".into()))?;
                let launch_type: LaunchType = params
                    .get("launchType")
                    .cloned()
                    .map(|v| serde_json::from_value(v).unwrap_or(LaunchType::OpenAsRegularTab))
                    .unwrap_or(LaunchType::OpenAsRegularTab);
                self.set_launch_type(id, launch_type)
            }
            "generateAppForLink" => {
                let url = params
                    .get("url")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ExtensionError::InvalidParams("Missing url".into()))?;
                let title = params
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("App");
                self.generate_app_for_link(url, title)
            }
            _ => Err(ExtensionError::UnknownMethod(method.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all() {
        let api = ManagementApi::new();
        let result = api.get_all().unwrap();
        let extensions: Vec<ExtensionInfo> = serde_json::from_value(result).unwrap();
        assert!(!extensions.is_empty());
    }

    #[test]
    fn test_get_self() {
        let api = ManagementApi::new();
        let result = api.get_self("auroraview-host").unwrap();
        let info: ExtensionInfo = serde_json::from_value(result).unwrap();
        assert_eq!(info.id, "auroraview-host");
    }

    #[test]
    fn test_set_enabled() {
        let api = ManagementApi::new();

        // Register a test extension that can be disabled
        api.register_extension(ExtensionInfo {
            id: "test-ext".to_string(),
            name: "Test".to_string(),
            short_name: None,
            description: "Test extension".to_string(),
            version: "1.0.0".to_string(),
            version_name: None,
            may_disable: true,
            may_enable: true,
            enabled: true,
            disabled_reason: None,
            is_app: false,
            extension_type: ExtensionType::Extension,
            app_launch_url: None,
            homepage_url: None,
            update_url: None,
            offline_enabled: true,
            options_url: None,
            icons: None,
            permissions: vec![],
            host_permissions: vec![],
            install_type: InstallType::Development,
            launch_type: None,
            available_launch_types: None,
        });

        api.set_enabled("test-ext", false).unwrap();

        let result = api.get("test-ext").unwrap();
        let info: ExtensionInfo = serde_json::from_value(result).unwrap();
        assert!(!info.enabled);
    }
}
