//! Chrome Search API Implementation
//!
//! Provides search functionality for extensions.
//!
//! ## Features
//! - Perform searches using default search engine
//! - Open search results in new tab

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::{ExtensionError, ExtensionResult};

/// Search disposition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(Default)]
pub enum Disposition {
    #[default]
    CurrentTab,
    NewTab,
    NewWindow,
}

/// Search query options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryInfo {
    /// Search text
    pub text: String,
    /// Where to display results
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disposition: Option<Disposition>,
    /// Tab ID to use (for CURRENT_TAB)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tab_id: Option<i32>,
}

/// Search API handler
pub struct SearchApi;

impl Default for SearchApi {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchApi {
    /// Create a new SearchApi instance
    pub fn new() -> Self {
        Self
    }

    /// Perform a search
    pub fn query(&self, info: QueryInfo) -> ExtensionResult<Value> {
        // In a real implementation, this would:
        // 1. Get the default search engine URL template
        // 2. Substitute the search text
        // 3. Navigate to the search URL in the appropriate tab/window

        let search_url = format!(
            "https://www.google.com/search?q={}",
            urlencoding::encode(&info.text)
        );

        // Return the search URL for the caller to handle
        Ok(json!({
            "url": search_url,
            "disposition": info.disposition.unwrap_or_default(),
            "tabId": info.tab_id
        }))
    }

    /// Handle API call
    pub fn handle(&self, method: &str, params: Value) -> ExtensionResult<Value> {
        match method {
            "query" => {
                let info: QueryInfo = serde_json::from_value(params)?;
                self.query(info)
            }
            _ => Err(ExtensionError::UnknownMethod(method.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query() {
        let api = SearchApi::new();
        let result = api
            .query(QueryInfo {
                text: "hello world".to_string(),
                disposition: None,
                tab_id: None,
            })
            .unwrap();

        let url = result.get("url").and_then(|v| v.as_str()).unwrap();
        assert!(url.contains("google.com/search"));
        assert!(url.contains("hello"));
    }
}
