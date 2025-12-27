//! Chrome Bookmarks API Implementation
//!
//! Provides bookmark management functionality for extensions.
//!
//! ## Features
//! - Create, update, delete bookmarks and folders
//! - Search and query bookmarks
//! - Tree structure navigation
//! - Event notifications for changes

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::error::{ExtensionError, ExtensionResult};

/// Bookmark tree node
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkTreeNode {
    /// Unique identifier for the node
    pub id: String,
    /// ID of the parent folder (None for root nodes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    /// Position within parent folder
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<usize>,
    /// URL of the bookmark (None for folders)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Title of the bookmark or folder
    pub title: String,
    /// When the bookmark was created (ms since epoch)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_added: Option<u64>,
    /// When the folder's contents were last changed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_group_modified: Option<u64>,
    /// When the bookmark was last opened
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_last_used: Option<u64>,
    /// Children of this node (for folders)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<BookmarkTreeNode>>,
    /// Whether this bookmark is unmodifiable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unmodifiable: Option<String>,
}

impl BookmarkTreeNode {
    /// Check if this node is a folder
    pub fn is_folder(&self) -> bool {
        self.url.is_none()
    }
}

/// Create bookmark details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDetails {
    /// Parent folder ID (defaults to "Other Bookmarks")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    /// Position within parent folder
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<usize>,
    /// Title of the bookmark or folder
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// URL of the bookmark (omit for folders)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Move destination
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveDestination {
    /// New parent folder ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    /// New index within parent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<usize>,
}

/// Update changes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateChanges {
    /// New title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// New URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Search query
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchQuery {
    /// Search query string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    /// URL to search for
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Title to search for
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Bookmarks API handler
pub struct BookmarksApi {
    /// In-memory bookmark storage
    bookmarks: Arc<RwLock<HashMap<String, BookmarkTreeNode>>>,
    /// Next ID counter
    next_id: Arc<RwLock<u64>>,
}

impl Default for BookmarksApi {
    fn default() -> Self {
        Self::new()
    }
}

impl BookmarksApi {
    /// Create a new BookmarksApi instance
    pub fn new() -> Self {
        let api = Self {
            bookmarks: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(1)),
        };
        api.init_default_folders();
        api
    }

    /// Initialize default bookmark folders
    fn init_default_folders(&self) {
        let mut bookmarks = self.bookmarks.write().unwrap();

        // Root node
        bookmarks.insert(
            "0".to_string(),
            BookmarkTreeNode {
                id: "0".to_string(),
                parent_id: None,
                index: None,
                url: None,
                title: "".to_string(),
                date_added: Some(now_ms()),
                date_group_modified: Some(now_ms()),
                date_last_used: None,
                children: Some(vec![]),
                unmodifiable: Some("managed".to_string()),
            },
        );

        // Bookmarks Bar
        bookmarks.insert(
            "1".to_string(),
            BookmarkTreeNode {
                id: "1".to_string(),
                parent_id: Some("0".to_string()),
                index: Some(0),
                url: None,
                title: "Bookmarks Bar".to_string(),
                date_added: Some(now_ms()),
                date_group_modified: Some(now_ms()),
                date_last_used: None,
                children: Some(vec![]),
                unmodifiable: Some("managed".to_string()),
            },
        );

        // Other Bookmarks
        bookmarks.insert(
            "2".to_string(),
            BookmarkTreeNode {
                id: "2".to_string(),
                parent_id: Some("0".to_string()),
                index: Some(1),
                url: None,
                title: "Other Bookmarks".to_string(),
                date_added: Some(now_ms()),
                date_group_modified: Some(now_ms()),
                date_last_used: None,
                children: Some(vec![]),
                unmodifiable: Some("managed".to_string()),
            },
        );
    }

    /// Generate next ID
    fn next_id(&self) -> String {
        let mut id = self.next_id.write().unwrap();
        let current = *id;
        *id += 1;
        // Start from 100 to avoid conflicts with default folders
        format!("{}", current + 100)
    }

    /// Get bookmark by ID
    pub fn get(&self, id_or_ids: Value) -> ExtensionResult<Value> {
        let bookmarks = self.bookmarks.read().unwrap();
        let ids: Vec<String> = match id_or_ids {
            Value::String(s) => vec![s],
            Value::Array(arr) => arr
                .into_iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect(),
            _ => return Err(ExtensionError::InvalidParams("Invalid id parameter".into())),
        };

        let results: Vec<&BookmarkTreeNode> =
            ids.iter().filter_map(|id| bookmarks.get(id)).collect();

        Ok(serde_json::to_value(results)?)
    }

    /// Get children of a bookmark folder
    pub fn get_children(&self, id: &str) -> ExtensionResult<Value> {
        let bookmarks = self.bookmarks.read().unwrap();

        let children: Vec<&BookmarkTreeNode> = bookmarks
            .values()
            .filter(|b| b.parent_id.as_deref() == Some(id))
            .collect();

        Ok(serde_json::to_value(children)?)
    }

    /// Get recent bookmarks
    pub fn get_recent(&self, number_of_items: usize) -> ExtensionResult<Value> {
        let bookmarks = self.bookmarks.read().unwrap();

        let mut recent: Vec<&BookmarkTreeNode> = bookmarks
            .values()
            .filter(|b| b.url.is_some()) // Only actual bookmarks, not folders
            .collect();

        // Sort by date_added descending
        recent.sort_by(|a, b| b.date_added.unwrap_or(0).cmp(&a.date_added.unwrap_or(0)));

        recent.truncate(number_of_items);
        Ok(serde_json::to_value(recent)?)
    }

    /// Get subtree starting from a node
    pub fn get_sub_tree(&self, id: &str) -> ExtensionResult<Value> {
        let bookmarks = self.bookmarks.read().unwrap();

        let node = bookmarks
            .get(id)
            .ok_or_else(|| ExtensionError::NotFound(format!("Bookmark {} not found", id)))?;

        let tree = self.build_tree(node, &bookmarks);
        Ok(serde_json::to_value(vec![tree])?)
    }

    /// Get entire bookmark tree
    pub fn get_tree(&self) -> ExtensionResult<Value> {
        self.get_sub_tree("0")
    }

    /// Build tree structure recursively
    #[allow(clippy::only_used_in_recursion)]
    fn build_tree(
        &self,
        node: &BookmarkTreeNode,
        bookmarks: &HashMap<String, BookmarkTreeNode>,
    ) -> BookmarkTreeNode {
        let mut result = node.clone();

        if node.is_folder() {
            let children: Vec<BookmarkTreeNode> = bookmarks
                .values()
                .filter(|b| b.parent_id.as_deref() == Some(&node.id))
                .map(|b| self.build_tree(b, bookmarks))
                .collect();
            result.children = Some(children);
        }

        result
    }

    /// Create a bookmark or folder
    pub fn create(&self, details: CreateDetails) -> ExtensionResult<Value> {
        let id = self.next_id();
        let parent_id = details.parent_id.unwrap_or_else(|| "2".to_string()); // Default to "Other Bookmarks"
        let is_folder = details.url.is_none();

        let node = BookmarkTreeNode {
            id: id.clone(),
            parent_id: Some(parent_id),
            index: details.index,
            url: details.url,
            title: details.title.unwrap_or_default(),
            date_added: Some(now_ms()),
            date_group_modified: if is_folder { Some(now_ms()) } else { None },
            date_last_used: None,
            children: if is_folder { Some(vec![]) } else { None },
            unmodifiable: None,
        };

        let mut bookmarks = self.bookmarks.write().unwrap();
        bookmarks.insert(id, node.clone());

        Ok(serde_json::to_value(node)?)
    }

    /// Move a bookmark
    pub fn move_bookmark(&self, id: &str, destination: MoveDestination) -> ExtensionResult<Value> {
        let mut bookmarks = self.bookmarks.write().unwrap();

        let node = bookmarks
            .get_mut(id)
            .ok_or_else(|| ExtensionError::NotFound(format!("Bookmark {} not found", id)))?;

        if node.unmodifiable.is_some() {
            return Err(ExtensionError::PermissionDenied(
                "Cannot modify managed bookmark".into(),
            ));
        }

        if let Some(parent_id) = destination.parent_id {
            node.parent_id = Some(parent_id);
        }
        if let Some(index) = destination.index {
            node.index = Some(index);
        }

        Ok(serde_json::to_value(node.clone())?)
    }

    /// Update a bookmark
    pub fn update(&self, id: &str, changes: UpdateChanges) -> ExtensionResult<Value> {
        let mut bookmarks = self.bookmarks.write().unwrap();

        let node = bookmarks
            .get_mut(id)
            .ok_or_else(|| ExtensionError::NotFound(format!("Bookmark {} not found", id)))?;

        if node.unmodifiable.is_some() {
            return Err(ExtensionError::PermissionDenied(
                "Cannot modify managed bookmark".into(),
            ));
        }

        if let Some(title) = changes.title {
            node.title = title;
        }
        if let Some(url) = changes.url {
            node.url = Some(url);
        }

        Ok(serde_json::to_value(node.clone())?)
    }

    /// Remove a bookmark
    pub fn remove(&self, id: &str) -> ExtensionResult<Value> {
        let mut bookmarks = self.bookmarks.write().unwrap();

        let node = bookmarks
            .get(id)
            .ok_or_else(|| ExtensionError::NotFound(format!("Bookmark {} not found", id)))?;

        if node.unmodifiable.is_some() {
            return Err(ExtensionError::PermissionDenied(
                "Cannot remove managed bookmark".into(),
            ));
        }

        // Check if folder has children
        if node.is_folder() {
            let has_children = bookmarks
                .values()
                .any(|b| b.parent_id.as_deref() == Some(id));
            if has_children {
                return Err(ExtensionError::InvalidParams(
                    "Cannot remove non-empty folder. Use removeTree instead.".into(),
                ));
            }
        }

        bookmarks.remove(id);
        Ok(json!(null))
    }

    /// Remove a bookmark tree
    pub fn remove_tree(&self, id: &str) -> ExtensionResult<Value> {
        let mut bookmarks = self.bookmarks.write().unwrap();

        let node = bookmarks
            .get(id)
            .ok_or_else(|| ExtensionError::NotFound(format!("Bookmark {} not found", id)))?;

        if node.unmodifiable.is_some() {
            return Err(ExtensionError::PermissionDenied(
                "Cannot remove managed bookmark".into(),
            ));
        }

        // Collect all IDs to remove
        let mut ids_to_remove = vec![id.to_string()];
        self.collect_children_ids(id, &bookmarks, &mut ids_to_remove);

        for id in ids_to_remove {
            bookmarks.remove(&id);
        }

        Ok(json!(null))
    }

    /// Recursively collect child IDs
    #[allow(clippy::only_used_in_recursion)]
    fn collect_children_ids(
        &self,
        parent_id: &str,
        bookmarks: &HashMap<String, BookmarkTreeNode>,
        ids: &mut Vec<String>,
    ) {
        for (id, node) in bookmarks {
            if node.parent_id.as_deref() == Some(parent_id) {
                ids.push(id.clone());
                self.collect_children_ids(id, bookmarks, ids);
            }
        }
    }

    /// Search bookmarks
    pub fn search(&self, query: Value) -> ExtensionResult<Value> {
        let bookmarks = self.bookmarks.read().unwrap();

        let search_query: SearchQuery = if query.is_string() {
            SearchQuery {
                query: query.as_str().map(|s| s.to_string()),
                url: None,
                title: None,
            }
        } else {
            serde_json::from_value(query)?
        };

        let results: Vec<&BookmarkTreeNode> = bookmarks
            .values()
            .filter(|b| {
                // Only search actual bookmarks, not folders
                if b.url.is_none() {
                    return false;
                }

                if let Some(ref q) = search_query.query {
                    let q_lower = q.to_lowercase();
                    let title_match = b.title.to_lowercase().contains(&q_lower);
                    let url_match = b
                        .url
                        .as_ref()
                        .map(|u| u.to_lowercase().contains(&q_lower))
                        .unwrap_or(false);
                    if !title_match && !url_match {
                        return false;
                    }
                }

                if let Some(ref url) = search_query.url {
                    if b.url.as_ref() != Some(url) {
                        return false;
                    }
                }

                if let Some(ref title) = search_query.title {
                    if &b.title != title {
                        return false;
                    }
                }

                true
            })
            .collect();

        Ok(serde_json::to_value(results)?)
    }

    /// Handle API call
    pub fn handle(&self, method: &str, params: Value) -> ExtensionResult<Value> {
        match method {
            "get" => {
                let id = params
                    .get("idOrIdList")
                    .or_else(|| params.get("id"))
                    .cloned()
                    .unwrap_or(Value::Null);
                self.get(id)
            }
            "getChildren" => {
                let id = params
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ExtensionError::InvalidParams("Missing id".into()))?;
                self.get_children(id)
            }
            "getRecent" => {
                let number = params
                    .get("numberOfItems")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(10) as usize;
                self.get_recent(number)
            }
            "getSubTree" => {
                let id = params
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ExtensionError::InvalidParams("Missing id".into()))?;
                self.get_sub_tree(id)
            }
            "getTree" => self.get_tree(),
            "create" => {
                let details: CreateDetails = serde_json::from_value(params)?;
                self.create(details)
            }
            "move" => {
                let id = params
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ExtensionError::InvalidParams("Missing id".into()))?;
                let destination: MoveDestination =
                    serde_json::from_value(params.get("destination").cloned().unwrap_or_default())?;
                self.move_bookmark(id, destination)
            }
            "update" => {
                let id = params
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ExtensionError::InvalidParams("Missing id".into()))?;
                let changes: UpdateChanges =
                    serde_json::from_value(params.get("changes").cloned().unwrap_or_default())?;
                self.update(id, changes)
            }
            "remove" => {
                let id = params
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ExtensionError::InvalidParams("Missing id".into()))?;
                self.remove(id)
            }
            "removeTree" => {
                let id = params
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ExtensionError::InvalidParams("Missing id".into()))?;
                self.remove_tree(id)
            }
            "search" => {
                let query = params
                    .get("query")
                    .cloned()
                    .unwrap_or(Value::String(String::new()));
                self.search(query)
            }
            _ => Err(ExtensionError::UnknownMethod(method.to_string())),
        }
    }
}

/// Get current time in milliseconds
fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_bookmark() {
        let api = BookmarksApi::new();
        let result = api
            .create(CreateDetails {
                parent_id: Some("1".to_string()),
                index: None,
                title: Some("Test Bookmark".to_string()),
                url: Some("https://example.com".to_string()),
            })
            .unwrap();

        let node: BookmarkTreeNode = serde_json::from_value(result).unwrap();
        assert_eq!(node.title, "Test Bookmark");
        assert_eq!(node.url, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_get_tree() {
        let api = BookmarksApi::new();
        let result = api.get_tree().unwrap();
        let tree: Vec<BookmarkTreeNode> = serde_json::from_value(result).unwrap();
        assert!(!tree.is_empty());
    }
}
