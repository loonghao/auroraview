//! Chrome History API Implementation
//!
//! Provides browsing history management for extensions.
//!
//! ## Features
//! - Search history entries
//! - Add/delete URLs from history
//! - Get visit details
//! - Event notifications

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::error::{ExtensionError, ExtensionResult};

/// History item representing a URL in history
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryItem {
    /// Unique identifier for this history item
    pub id: String,
    /// The URL
    pub url: String,
    /// The title of the page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// When the page was last visited (ms since epoch)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_visit_time: Option<f64>,
    /// Number of times the user has visited this page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visit_count: Option<i32>,
    /// Number of times the user has typed this URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typed_count: Option<i32>,
}

/// Visit item representing a single visit to a URL
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisitItem {
    /// Unique identifier for this visit
    pub id: String,
    /// Visit ID
    pub visit_id: String,
    /// When the visit occurred (ms since epoch)
    pub visit_time: f64,
    /// ID of the referring visit (0 if none)
    pub referring_visit_id: String,
    /// Transition type
    pub transition: TransitionType,
    /// Whether this visit originated on this device
    pub is_local: bool,
}

/// Transition types for history visits
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum TransitionType {
    #[default]
    Link,
    Typed,
    AutoBookmark,
    AutoSubframe,
    ManualSubframe,
    Generated,
    AutoToplevel,
    FormSubmit,
    Reload,
    Keyword,
    KeywordGenerated,
}

/// Search query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchQuery {
    /// Free-text query
    pub text: String,
    /// Maximum results to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<i32>,
    /// Start time filter (ms since epoch)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<f64>,
    /// End time filter (ms since epoch)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<f64>,
}

/// Delete range parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteRange {
    /// Start time (ms since epoch)
    pub start_time: f64,
    /// End time (ms since epoch)
    pub end_time: f64,
}

/// URL details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UrlDetails {
    /// The URL
    pub url: String,
}

/// History API handler
pub struct HistoryApi {
    /// In-memory history storage
    history: Arc<RwLock<HashMap<String, HistoryItem>>>,
    /// Visit records
    visits: Arc<RwLock<Vec<VisitItem>>>,
    /// Next ID counter
    next_id: Arc<RwLock<u64>>,
}

impl Default for HistoryApi {
    fn default() -> Self {
        Self::new()
    }
}

impl HistoryApi {
    /// Create a new HistoryApi instance
    pub fn new() -> Self {
        Self {
            history: Arc::new(RwLock::new(HashMap::new())),
            visits: Arc::new(RwLock::new(Vec::new())),
            next_id: Arc::new(RwLock::new(1)),
        }
    }

    /// Generate next ID
    fn next_id(&self) -> String {
        let mut id = self.next_id.write().unwrap();
        let current = *id;
        *id += 1;
        format!("{}", current)
    }

    /// Search history
    pub fn search(&self, query: SearchQuery) -> ExtensionResult<Value> {
        let history = self.history.read().unwrap();
        let text_lower = query.text.to_lowercase();
        let max_results = query.max_results.unwrap_or(100) as usize;

        let mut results: Vec<&HistoryItem> = history
            .values()
            .filter(|item| {
                // Text filter
                if !text_lower.is_empty() {
                    let url_match = item.url.to_lowercase().contains(&text_lower);
                    let title_match = item
                        .title
                        .as_ref()
                        .map(|t| t.to_lowercase().contains(&text_lower))
                        .unwrap_or(false);
                    if !url_match && !title_match {
                        return false;
                    }
                }

                // Time filters
                if let Some(start_time) = query.start_time {
                    if item.last_visit_time.unwrap_or(0.0) < start_time {
                        return false;
                    }
                }
                if let Some(end_time) = query.end_time {
                    if item.last_visit_time.unwrap_or(0.0) > end_time {
                        return false;
                    }
                }

                true
            })
            .collect();

        // Sort by last visit time descending
        results.sort_by(|a, b| {
            b.last_visit_time
                .unwrap_or(0.0)
                .partial_cmp(&a.last_visit_time.unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results.truncate(max_results);
        Ok(serde_json::to_value(results)?)
    }

    /// Get visits for a URL
    pub fn get_visits(&self, details: UrlDetails) -> ExtensionResult<Value> {
        let history = self.history.read().unwrap();
        let visits = self.visits.read().unwrap();

        // Find the history item
        let item = history.values().find(|h| h.url == details.url);

        if let Some(item) = item {
            let item_visits: Vec<&VisitItem> = visits.iter().filter(|v| v.id == item.id).collect();
            Ok(serde_json::to_value(item_visits)?)
        } else {
            Ok(json!([]))
        }
    }

    /// Add URL to history
    pub fn add_url(&self, details: UrlDetails) -> ExtensionResult<Value> {
        let mut history = self.history.write().unwrap();
        let mut visits = self.visits.write().unwrap();

        let now = now_ms();

        // Check if URL already exists
        let existing = history.values_mut().find(|h| h.url == details.url);

        if let Some(item) = existing {
            item.visit_count = Some(item.visit_count.unwrap_or(0) + 1);
            item.last_visit_time = Some(now);

            // Add visit record
            let visit_id = self.next_id();
            visits.push(VisitItem {
                id: item.id.clone(),
                visit_id,
                visit_time: now,
                referring_visit_id: "0".to_string(),
                transition: TransitionType::Link,
                is_local: true,
            });
        } else {
            let id = self.next_id();
            let visit_id = self.next_id();

            history.insert(
                id.clone(),
                HistoryItem {
                    id: id.clone(),
                    url: details.url,
                    title: None,
                    last_visit_time: Some(now),
                    visit_count: Some(1),
                    typed_count: Some(0),
                },
            );

            visits.push(VisitItem {
                id,
                visit_id,
                visit_time: now,
                referring_visit_id: "0".to_string(),
                transition: TransitionType::Link,
                is_local: true,
            });
        }

        Ok(json!(null))
    }

    /// Delete URL from history
    pub fn delete_url(&self, details: UrlDetails) -> ExtensionResult<Value> {
        let mut history = self.history.write().unwrap();
        let mut visits = self.visits.write().unwrap();

        // Find and remove the history item
        let id_to_remove: Option<String> = history
            .iter()
            .find(|(_, h)| h.url == details.url)
            .map(|(id, _)| id.clone());

        if let Some(id) = id_to_remove {
            history.remove(&id);
            visits.retain(|v| v.id != id);
        }

        Ok(json!(null))
    }

    /// Delete all history
    pub fn delete_all(&self) -> ExtensionResult<Value> {
        let mut history = self.history.write().unwrap();
        let mut visits = self.visits.write().unwrap();

        history.clear();
        visits.clear();

        Ok(json!(null))
    }

    /// Delete history in range
    pub fn delete_range(&self, range: DeleteRange) -> ExtensionResult<Value> {
        let mut history = self.history.write().unwrap();
        let mut visits = self.visits.write().unwrap();

        // Remove visits in range
        visits.retain(|v| v.visit_time < range.start_time || v.visit_time > range.end_time);

        // Remove history items with no visits
        let visit_ids: std::collections::HashSet<String> =
            visits.iter().map(|v| v.id.clone()).collect();
        history.retain(|id, _| visit_ids.contains(id));

        Ok(json!(null))
    }

    /// Handle API call
    pub fn handle(&self, method: &str, params: Value) -> ExtensionResult<Value> {
        match method {
            "search" => {
                let query: SearchQuery = serde_json::from_value(params)?;
                self.search(query)
            }
            "getVisits" => {
                let details: UrlDetails = serde_json::from_value(params)?;
                self.get_visits(details)
            }
            "addUrl" => {
                let details: UrlDetails = serde_json::from_value(params)?;
                self.add_url(details)
            }
            "deleteUrl" => {
                let details: UrlDetails = serde_json::from_value(params)?;
                self.delete_url(details)
            }
            "deleteAll" => self.delete_all(),
            "deleteRange" => {
                let range: DeleteRange = serde_json::from_value(params)?;
                self.delete_range(range)
            }
            _ => Err(ExtensionError::UnknownMethod(method.to_string())),
        }
    }
}

/// Get current time in milliseconds
fn now_ms() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as f64)
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_search() {
        let api = HistoryApi::new();

        // Add URL
        api.add_url(UrlDetails {
            url: "https://example.com".to_string(),
        })
        .unwrap();

        // Search
        let result = api
            .search(SearchQuery {
                text: "example".to_string(),
                max_results: None,
                start_time: None,
                end_time: None,
            })
            .unwrap();

        let items: Vec<HistoryItem> = serde_json::from_value(result).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].url, "https://example.com");
    }

    #[test]
    fn test_delete_url() {
        let api = HistoryApi::new();

        api.add_url(UrlDetails {
            url: "https://example.com".to_string(),
        })
        .unwrap();

        api.delete_url(UrlDetails {
            url: "https://example.com".to_string(),
        })
        .unwrap();

        let result = api
            .search(SearchQuery {
                text: "".to_string(),
                max_results: None,
                start_time: None,
                end_time: None,
            })
            .unwrap();

        let items: Vec<HistoryItem> = serde_json::from_value(result).unwrap();
        assert!(items.is_empty());
    }
}
