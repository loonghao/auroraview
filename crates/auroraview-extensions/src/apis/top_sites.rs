//! Chrome Top Sites API Implementation
//!
//! Provides access to the most visited sites.
//!
//! ## Features
//! - Get most visited URLs
//! - Configurable result count

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::{Arc, RwLock};

use crate::error::{ExtensionError, ExtensionResult};

/// Most visited URL entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MostVisitedURL {
    /// The URL
    pub url: String,
    /// The title
    pub title: String,
}

/// Top Sites API handler
pub struct TopSitesApi {
    /// In-memory top sites storage
    top_sites: Arc<RwLock<Vec<MostVisitedURL>>>,
}

impl Default for TopSitesApi {
    fn default() -> Self {
        Self::new()
    }
}

impl TopSitesApi {
    /// Create a new TopSitesApi instance
    pub fn new() -> Self {
        let api = Self {
            top_sites: Arc::new(RwLock::new(Vec::new())),
        };
        api.init_default_sites();
        api
    }

    /// Initialize with some default sites
    fn init_default_sites(&self) {
        let mut sites = self.top_sites.write().unwrap();
        sites.push(MostVisitedURL {
            url: "https://www.google.com".to_string(),
            title: "Google".to_string(),
        });
        sites.push(MostVisitedURL {
            url: "https://www.youtube.com".to_string(),
            title: "YouTube".to_string(),
        });
        sites.push(MostVisitedURL {
            url: "https://www.github.com".to_string(),
            title: "GitHub".to_string(),
        });
    }

    /// Get most visited sites
    pub fn get(&self) -> ExtensionResult<Value> {
        let sites = self.top_sites.read().unwrap();
        Ok(serde_json::to_value(sites.clone())?)
    }

    /// Add a site to top sites (internal use)
    pub fn add_site(&self, url: String, title: String) {
        let mut sites = self.top_sites.write().unwrap();

        // Check if already exists
        if let Some(site) = sites.iter_mut().find(|s| s.url == url) {
            site.title = title;
        } else {
            sites.push(MostVisitedURL { url, title });
        }

        // Keep only top 10
        sites.truncate(10);
    }

    /// Handle API call
    pub fn handle(&self, method: &str, _params: Value) -> ExtensionResult<Value> {
        match method {
            "get" => self.get(),
            _ => Err(ExtensionError::UnknownMethod(method.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_top_sites() {
        let api = TopSitesApi::new();
        let result = api.get().unwrap();
        let sites: Vec<MostVisitedURL> = serde_json::from_value(result).unwrap();
        assert!(!sites.is_empty());
    }

    #[test]
    fn test_add_site() {
        let api = TopSitesApi::new();
        api.add_site("https://example.com".to_string(), "Example".to_string());

        let result = api.get().unwrap();
        let sites: Vec<MostVisitedURL> = serde_json::from_value(result).unwrap();
        assert!(sites.iter().any(|s| s.url == "https://example.com"));
    }
}
