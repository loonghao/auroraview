//! Chrome Tab Groups API Implementation
//!
//! Provides functionality to interact with tab groups.
//!
//! ## Features
//! - Query tab groups
//! - Update tab group properties
//! - Move tab groups
//! - Event notifications

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::error::{ExtensionError, ExtensionResult};

/// Tab group color
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum Color {
    #[default]
    Grey,
    Blue,
    Red,
    Yellow,
    Green,
    Pink,
    Purple,
    Cyan,
    Orange,
}

/// Tab group
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TabGroup {
    /// Group ID
    pub id: i32,
    /// Whether collapsed
    pub collapsed: bool,
    /// Group color
    pub color: Color,
    /// Group title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Window ID
    pub window_id: i32,
}

/// Query info for tab groups
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct QueryInfo {
    /// Filter by collapsed state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collapsed: Option<bool>,
    /// Filter by color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
    /// Filter by title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Filter by window ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_id: Option<i32>,
}

/// Update properties
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProperties {
    /// New collapsed state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collapsed: Option<bool>,
    /// New color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
    /// New title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Move properties
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveProperties {
    /// New window ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_id: Option<i32>,
    /// New index
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<i32>,
}

/// Tab Groups API handler
pub struct TabGroupsApi {
    /// In-memory tab groups storage
    groups: Arc<RwLock<HashMap<i32, TabGroup>>>,
    /// Next group ID
    next_id: Arc<RwLock<i32>>,
}

impl Default for TabGroupsApi {
    fn default() -> Self {
        Self::new()
    }
}

impl TabGroupsApi {
    /// Create a new TabGroupsApi instance
    pub fn new() -> Self {
        Self {
            groups: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(1)),
        }
    }

    /// Generate next group ID
    fn next_id(&self) -> i32 {
        let mut id = self.next_id.write().unwrap();
        let current = *id;
        *id += 1;
        current
    }

    /// Get a tab group by ID
    pub fn get(&self, group_id: i32) -> ExtensionResult<Value> {
        let groups = self.groups.read().unwrap();
        let group = groups
            .get(&group_id)
            .ok_or_else(|| ExtensionError::NotFound(format!("Tab group {} not found", group_id)))?;
        Ok(serde_json::to_value(group)?)
    }

    /// Query tab groups
    pub fn query(&self, query_info: QueryInfo) -> ExtensionResult<Value> {
        let groups = self.groups.read().unwrap();

        let results: Vec<&TabGroup> = groups
            .values()
            .filter(|group| {
                if let Some(collapsed) = query_info.collapsed {
                    if group.collapsed != collapsed {
                        return false;
                    }
                }
                if let Some(ref color) = query_info.color {
                    if &group.color != color {
                        return false;
                    }
                }
                if let Some(ref title) = query_info.title {
                    if group.title.as_ref() != Some(title) {
                        return false;
                    }
                }
                if let Some(window_id) = query_info.window_id {
                    if group.window_id != window_id {
                        return false;
                    }
                }
                true
            })
            .collect();

        Ok(serde_json::to_value(results)?)
    }

    /// Update a tab group
    pub fn update(
        &self,
        group_id: i32,
        update_properties: UpdateProperties,
    ) -> ExtensionResult<Value> {
        let mut groups = self.groups.write().unwrap();

        let group = groups
            .get_mut(&group_id)
            .ok_or_else(|| ExtensionError::NotFound(format!("Tab group {} not found", group_id)))?;

        if let Some(collapsed) = update_properties.collapsed {
            group.collapsed = collapsed;
        }
        if let Some(color) = update_properties.color {
            group.color = color;
        }
        if let Some(title) = update_properties.title {
            group.title = Some(title);
        }

        Ok(serde_json::to_value(group.clone())?)
    }

    /// Move a tab group
    pub fn move_group(
        &self,
        group_id: i32,
        move_properties: MoveProperties,
    ) -> ExtensionResult<Value> {
        let mut groups = self.groups.write().unwrap();

        let group = groups
            .get_mut(&group_id)
            .ok_or_else(|| ExtensionError::NotFound(format!("Tab group {} not found", group_id)))?;

        if let Some(window_id) = move_properties.window_id {
            group.window_id = window_id;
        }
        // Index handling would require more complex logic in a real implementation

        Ok(serde_json::to_value(group.clone())?)
    }

    /// Create a tab group (internal use)
    pub fn create_group(&self, window_id: i32, title: Option<String>, color: Option<Color>) -> i32 {
        let id = self.next_id();
        let group = TabGroup {
            id,
            collapsed: false,
            color: color.unwrap_or_default(),
            title,
            window_id,
        };

        let mut groups = self.groups.write().unwrap();
        groups.insert(id, group);
        id
    }

    /// Handle API call
    pub fn handle(&self, method: &str, params: Value) -> ExtensionResult<Value> {
        match method {
            "get" => {
                let group_id = params
                    .get("groupId")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| ExtensionError::InvalidParams("Missing groupId".into()))?
                    as i32;
                self.get(group_id)
            }
            "query" => {
                let query_info: QueryInfo = serde_json::from_value(params).unwrap_or_default();
                self.query(query_info)
            }
            "update" => {
                let group_id = params
                    .get("groupId")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| ExtensionError::InvalidParams("Missing groupId".into()))?
                    as i32;
                let update_properties: UpdateProperties = params
                    .get("updateProperties")
                    .cloned()
                    .map(|v| serde_json::from_value(v).unwrap_or_default())
                    .unwrap_or_default();
                self.update(group_id, update_properties)
            }
            "move" => {
                let group_id = params
                    .get("groupId")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| ExtensionError::InvalidParams("Missing groupId".into()))?
                    as i32;
                let move_properties: MoveProperties = serde_json::from_value(
                    params.get("moveProperties").cloned().unwrap_or_default(),
                )?;
                self.move_group(group_id, move_properties)
            }
            _ => Err(ExtensionError::UnknownMethod(method.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_get_group() {
        let api = TabGroupsApi::new();
        let id = api.create_group(1, Some("Test Group".to_string()), Some(Color::Blue));

        let result = api.get(id).unwrap();
        let group: TabGroup = serde_json::from_value(result).unwrap();
        assert_eq!(group.title, Some("Test Group".to_string()));
        assert_eq!(group.color, Color::Blue);
    }

    #[test]
    fn test_update_group() {
        let api = TabGroupsApi::new();
        let id = api.create_group(1, None, None);

        api.update(
            id,
            UpdateProperties {
                title: Some("Updated".to_string()),
                color: Some(Color::Red),
                collapsed: Some(true),
            },
        )
        .unwrap();

        let result = api.get(id).unwrap();
        let group: TabGroup = serde_json::from_value(result).unwrap();
        assert_eq!(group.title, Some("Updated".to_string()));
        assert_eq!(group.color, Color::Red);
        assert!(group.collapsed);
    }

    #[test]
    fn test_query_groups() {
        let api = TabGroupsApi::new();
        api.create_group(1, Some("Group 1".to_string()), Some(Color::Blue));
        api.create_group(1, Some("Group 2".to_string()), Some(Color::Red));

        let result = api
            .query(QueryInfo {
                color: Some(Color::Blue),
                ..Default::default()
            })
            .unwrap();
        let groups: Vec<TabGroup> = serde_json::from_value(result).unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].color, Color::Blue);
    }
}
