//! chrome.contextMenus API handler
//!
//! Provides context menu (right-click menu) functionality for extensions.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::apis::ApiHandler;
use crate::error::{ExtensionError, ExtensionResult};
use crate::ExtensionId;

/// Context menu item type
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ItemType {
    /// Normal menu item
    #[default]
    Normal,
    /// Checkbox item
    Checkbox,
    /// Radio button item
    Radio,
    /// Separator
    Separator,
}

/// Contexts where menu item appears
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ContextType {
    /// All contexts
    All,
    /// Page context
    Page,
    /// Frame context
    Frame,
    /// Selection context
    Selection,
    /// Link context
    Link,
    /// Editable element context
    Editable,
    /// Image context
    Image,
    /// Video context
    Video,
    /// Audio context
    Audio,
    /// Browser action context
    BrowserAction,
    /// Page action context
    PageAction,
    /// Action context (MV3)
    Action,
}

/// Context menu item creation properties
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProperties {
    /// Item type
    #[serde(rename = "type", default)]
    pub item_type: ItemType,
    /// Item ID (auto-generated if not provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Display title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Checkbox/radio checked state
    #[serde(default)]
    pub checked: bool,
    /// Contexts where item appears
    #[serde(default)]
    pub contexts: Vec<ContextType>,
    /// Whether item is visible
    #[serde(default = "default_true")]
    pub visible: bool,
    /// Whether item is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Parent item ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    /// URL patterns to match
    #[serde(default)]
    pub document_url_patterns: Vec<String>,
    /// Target URL patterns (for links, images, etc.)
    #[serde(default)]
    pub target_url_patterns: Vec<String>,
}

fn default_true() -> bool {
    true
}

/// Context menu item (stored)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuItem {
    /// Item ID
    pub id: String,
    /// Extension ID
    pub extension_id: ExtensionId,
    /// Item properties
    pub properties: CreateProperties,
}

/// Click info passed to onclick handler
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnClickData {
    /// Menu item ID
    pub menu_item_id: String,
    /// Parent menu item ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_menu_item_id: Option<String>,
    /// Media type (image, video, audio)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    /// Link URL (if context is link)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_url: Option<String>,
    /// Source URL (for media elements)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src_url: Option<String>,
    /// Page URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_url: Option<String>,
    /// Frame URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame_url: Option<String>,
    /// Frame ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame_id: Option<i32>,
    /// Selected text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selection_text: Option<String>,
    /// Whether element is editable
    #[serde(default)]
    pub editable: bool,
    /// Checkbox/radio checked state (before click)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub was_checked: Option<bool>,
    /// Checkbox/radio checked state (after click)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checked: Option<bool>,
}

/// Click handler type alias
type ClickHandler = Box<dyn Fn(OnClickData) + Send + Sync>;

/// Context menus manager
pub struct ContextMenusManager {
    /// Menu items by extension
    items: RwLock<HashMap<ExtensionId, Vec<MenuItem>>>,
    /// Item ID counter
    next_id: RwLock<u64>,
    /// Click handlers (extension_id -> callback)
    #[allow(dead_code)]
    click_handlers: RwLock<HashMap<ExtensionId, Vec<ClickHandler>>>,
}

impl ContextMenusManager {
    /// Create a new context menus manager
    pub fn new() -> Self {
        Self {
            items: RwLock::new(HashMap::new()),
            next_id: RwLock::new(1),
            click_handlers: RwLock::new(HashMap::new()),
        }
    }

    /// Generate a new item ID
    fn generate_id(&self) -> String {
        let mut id = self.next_id.write();
        let current = *id;
        *id += 1;
        format!("menu_{}", current)
    }

    /// Create a menu item
    pub fn create(
        &self,
        extension_id: &str,
        mut properties: CreateProperties,
    ) -> ExtensionResult<String> {
        let id = properties.id.take().unwrap_or_else(|| self.generate_id());

        let item = MenuItem {
            id: id.clone(),
            extension_id: extension_id.to_string(),
            properties,
        };

        let mut items = self.items.write();
        let ext_items = items.entry(extension_id.to_string()).or_default();
        ext_items.push(item);

        Ok(id)
    }

    /// Update a menu item
    pub fn update(
        &self,
        extension_id: &str,
        id: &str,
        properties: CreateProperties,
    ) -> ExtensionResult<()> {
        let mut items = self.items.write();
        if let Some(ext_items) = items.get_mut(extension_id) {
            for item in ext_items.iter_mut() {
                if item.id == id {
                    item.properties = properties;
                    return Ok(());
                }
            }
        }
        Err(ExtensionError::NotFound(format!(
            "Menu item {} not found",
            id
        )))
    }

    /// Remove a menu item
    pub fn remove(&self, extension_id: &str, id: &str) -> ExtensionResult<()> {
        let mut items = self.items.write();
        if let Some(ext_items) = items.get_mut(extension_id) {
            let len_before = ext_items.len();
            ext_items.retain(|item| item.id != id);
            if ext_items.len() < len_before {
                return Ok(());
            }
        }
        Err(ExtensionError::NotFound(format!(
            "Menu item {} not found",
            id
        )))
    }

    /// Remove all menu items for an extension
    pub fn remove_all(&self, extension_id: &str) {
        let mut items = self.items.write();
        items.remove(extension_id);
    }

    /// Get all menu items for an extension
    pub fn get_items(&self, extension_id: &str) -> Vec<MenuItem> {
        let items = self.items.read();
        items.get(extension_id).cloned().unwrap_or_default()
    }

    /// Get all menu items for a context
    pub fn get_items_for_context(&self, context: &ContextType, url: Option<&str>) -> Vec<MenuItem> {
        let items = self.items.read();
        let mut result = Vec::new();

        for ext_items in items.values() {
            for item in ext_items {
                // Check if context matches
                let contexts = &item.properties.contexts;
                if contexts.is_empty()
                    || contexts.contains(&ContextType::All)
                    || contexts.contains(context)
                {
                    // Check URL pattern if specified
                    if let Some(url) = url {
                        if !item.properties.document_url_patterns.is_empty() {
                            let matches = item
                                .properties
                                .document_url_patterns
                                .iter()
                                .any(|pattern| Self::match_url_pattern(url, pattern));
                            if !matches {
                                continue;
                            }
                        }
                    }

                    if item.properties.visible && item.properties.enabled {
                        result.push(item.clone());
                    }
                }
            }
        }

        result
    }

    /// Match URL against pattern (simplified)
    fn match_url_pattern(url: &str, pattern: &str) -> bool {
        if pattern == "<all_urls>" {
            return true;
        }
        // Simple wildcard matching
        let pattern = pattern.replace("*", ".*");
        if let Ok(regex) = regex::Regex::new(&format!("^{}$", pattern)) {
            return regex.is_match(url);
        }
        false
    }
}

impl Default for ContextMenusManager {
    fn default() -> Self {
        Self::new()
    }
}

/// ContextMenus API handler
pub struct ContextMenusApiHandler {
    manager: Arc<ContextMenusManager>,
}

impl ContextMenusApiHandler {
    /// Create a new context menus API handler
    pub fn new(manager: Arc<ContextMenusManager>) -> Self {
        Self { manager }
    }
}

impl ApiHandler for ContextMenusApiHandler {
    fn namespace(&self) -> &str {
        "contextMenus"
    }

    fn handle(&self, method: &str, params: Value, extension_id: &str) -> ExtensionResult<Value> {
        match method {
            "create" => {
                let properties: CreateProperties = serde_json::from_value(params)
                    .map_err(|e| ExtensionError::InvalidArgument(e.to_string()))?;

                let id = self.manager.create(extension_id, properties)?;
                Ok(serde_json::json!(id))
            }
            "update" => {
                let id = params
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ExtensionError::InvalidArgument("id is required".to_string()))?;

                let properties: CreateProperties = params
                    .get("updateProperties")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .ok_or_else(|| {
                        ExtensionError::InvalidArgument("updateProperties is required".to_string())
                    })?;

                self.manager.update(extension_id, id, properties)?;
                Ok(serde_json::json!({}))
            }
            "remove" => {
                let id = params
                    .get("menuItemId")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ExtensionError::InvalidArgument("menuItemId is required".to_string())
                    })?;

                self.manager.remove(extension_id, id)?;
                Ok(serde_json::json!({}))
            }
            "removeAll" => {
                self.manager.remove_all(extension_id);
                Ok(serde_json::json!({}))
            }
            _ => Err(ExtensionError::ApiNotSupported(format!(
                "contextMenus.{} is not supported",
                method
            ))),
        }
    }

    fn methods(&self) -> Vec<&str> {
        vec!["create", "update", "remove", "removeAll"]
    }
}
