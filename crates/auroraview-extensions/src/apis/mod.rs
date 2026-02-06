//! Chrome Extension API Implementations
//!
//! This module contains implementations of Chrome Extension APIs
//! that are compatible with AuroraView.
//!
//! ## Supported APIs
//!
//! | API | Status | Description |
//! |-----|--------|-------------|
//! | `chrome.runtime` | ✅ Full | Extension lifecycle and messaging |
//! | `chrome.storage` | ✅ Full | Local/sync/session storage |
//! | `chrome.tabs` | ✅ Basic | Tab management (single-tab mode) |
//! | `chrome.sidePanel` | ✅ Full | Side panel API |
//! | `chrome.action` | ✅ Full | Extension action (toolbar button) |
//! | `chrome.scripting` | ✅ Basic | Script injection |
//! | `chrome.webRequest` | ✅ Basic | Request interception |
//! | `chrome.contextMenus` | ✅ Full | Context menus |
//! | `chrome.notifications` | ✅ Full | System notifications |
//! | `chrome.alarms` | ✅ Full | Scheduled tasks |
//! | `chrome.bookmarks` | ✅ Full | Bookmark management |
//! | `chrome.history` | ✅ Full | Browsing history |
//! | `chrome.downloads` | ✅ Full | Download management |
//! | `chrome.cookies` | ✅ Full | Cookie management |
//! | `chrome.topSites` | ✅ Full | Most visited sites |
//! | `chrome.omnibox` | ✅ Full | Address bar integration |
//! | `chrome.search` | ✅ Full | Search functionality |
//! | `chrome.sessions` | ✅ Full | Session management |
//! | `chrome.tts` | ✅ Full | Text-to-speech |
//! | `chrome.browsingData` | ✅ Full | Browsing data removal |
//! | `chrome.idle` | ✅ Full | Idle state detection |
//! | `chrome.power` | ✅ Full | Power management |
//! | `chrome.tabGroups` | ✅ Full | Tab group management |
//! | `chrome.management` | ✅ Full | Extension management |
//! | `chrome.fontSettings` | ✅ Full | Font settings |

pub mod action;
pub mod alarms;
pub mod bookmarks;
pub mod browsing_data;
pub mod context_menus;
pub mod cookies;
pub mod downloads;
pub mod font_settings;
pub mod history;
pub mod idle;
pub mod management;
pub mod notifications;
pub mod omnibox;
pub mod power;
pub mod runtime;
pub mod scripting;
pub mod search;
pub mod sessions;
pub mod side_panel;
pub mod storage;
pub mod tab_groups;
pub mod tabs;
pub mod top_sites;
pub mod tts;
pub mod web_request;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::ExtensionResult;
use crate::ExtensionId;

/// API call request from JavaScript
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiCallRequest {
    /// Extension ID making the call
    pub extension_id: ExtensionId,
    /// API namespace (e.g., "storage", "tabs")
    pub api: String,
    /// Method name (e.g., "get", "set")
    pub method: String,
    /// Method parameters
    pub params: Value,
}

/// API call response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiCallResponse {
    /// Whether the call succeeded
    pub success: bool,
    /// Result data (if success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ApiCallResponse {
    /// Create a success response
    pub fn success(result: Value) -> Self {
        Self {
            success: true,
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            result: None,
            error: Some(message.into()),
        }
    }
}

/// API handler trait
pub trait ApiHandler: Send + Sync {
    /// Get the API namespace
    fn namespace(&self) -> &str;

    /// Handle an API call
    fn handle(&self, method: &str, params: Value, extension_id: &str) -> ExtensionResult<Value>;

    /// Get available methods
    fn methods(&self) -> Vec<&str>;
}

/// API router - routes API calls to appropriate handlers
pub struct ApiRouter {
    handlers: std::collections::HashMap<String, Box<dyn ApiHandler>>,
}

impl ApiRouter {
    /// Create a new API router
    pub fn new() -> Self {
        Self {
            handlers: std::collections::HashMap::new(),
        }
    }

    /// Register an API handler
    pub fn register<H: ApiHandler + 'static>(&mut self, handler: H) {
        let namespace = handler.namespace().to_string();
        self.handlers.insert(namespace, Box::new(handler));
    }

    /// Route an API call
    pub fn route(&self, request: &ApiCallRequest) -> ApiCallResponse {
        match self.handlers.get(&request.api) {
            Some(handler) => {
                match handler.handle(
                    &request.method,
                    request.params.clone(),
                    &request.extension_id,
                ) {
                    Ok(result) => ApiCallResponse::success(result),
                    Err(e) => ApiCallResponse::error(e.to_string()),
                }
            }
            None => ApiCallResponse::error(format!("Unknown API: {}", request.api)),
        }
    }

    /// Get all registered namespaces
    pub fn namespaces(&self) -> Vec<&str> {
        self.handlers.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for ApiRouter {
    fn default() -> Self {
        Self::new()
    }
}
