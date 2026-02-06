//! chrome.webRequest API handler
//!
//! Provides web request interception and modification capabilities.
//! This is a key API for ad blockers, privacy extensions, etc.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

use crate::apis::ApiHandler;
use crate::error::{ExtensionError, ExtensionResult};
use crate::ExtensionId;

/// Request filter for matching URLs
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestFilter {
    /// URL patterns to match
    #[serde(default)]
    pub urls: Vec<String>,
    /// Resource types to match
    #[serde(default)]
    pub types: Vec<ResourceType>,
    /// Tab ID to match (-1 for all)
    #[serde(default)]
    pub tab_id: Option<i32>,
    /// Window ID to match (-1 for all)
    #[serde(default)]
    pub window_id: Option<i32>,
}

/// HTTP resource types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    MainFrame,
    SubFrame,
    Stylesheet,
    Script,
    Image,
    Font,
    Object,
    Xmlhttprequest,
    Ping,
    CspReport,
    Media,
    Websocket,
    Webbundle,
    Other,
}

/// Request details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestDetails {
    /// Request ID
    pub request_id: String,
    /// Request URL
    pub url: String,
    /// HTTP method
    pub method: String,
    /// Frame ID
    pub frame_id: i32,
    /// Parent frame ID
    pub parent_frame_id: i32,
    /// Tab ID
    pub tab_id: i32,
    /// Resource type
    #[serde(rename = "type")]
    pub resource_type: ResourceType,
    /// Request timestamp
    pub time_stamp: f64,
    /// Initiator URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initiator: Option<String>,
}

/// HTTP header
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpHeader {
    /// Header name
    pub name: String,
    /// Header value (string)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Header value (binary)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary_value: Option<Vec<u8>>,
}

/// Blocking response for modifying requests
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BlockingResponse {
    /// Cancel the request
    #[serde(default)]
    pub cancel: bool,
    /// Redirect to this URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_url: Option<String>,
    /// Modified request headers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_headers: Option<Vec<HttpHeader>>,
    /// Modified response headers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_headers: Option<Vec<HttpHeader>>,
    /// Authentication credentials
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_credentials: Option<AuthCredentials>,
}

/// Authentication credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthCredentials {
    /// Username
    pub username: String,
    /// Password
    pub password: String,
}

/// Listener registration
#[derive(Debug, Clone)]
pub struct ListenerRegistration {
    /// Extension ID
    pub extension_id: ExtensionId,
    /// Request filter
    pub filter: RequestFilter,
    /// Extra info to include
    pub extra_info_spec: Vec<String>,
}

/// WebRequest manager
pub struct WebRequestManager {
    /// Listeners for onBeforeRequest
    before_request_listeners: RwLock<Vec<ListenerRegistration>>,
    /// Listeners for onBeforeSendHeaders
    before_send_headers_listeners: RwLock<Vec<ListenerRegistration>>,
    /// Listeners for onSendHeaders
    send_headers_listeners: RwLock<Vec<ListenerRegistration>>,
    /// Listeners for onHeadersReceived
    headers_received_listeners: RwLock<Vec<ListenerRegistration>>,
    /// Listeners for onResponseStarted
    response_started_listeners: RwLock<Vec<ListenerRegistration>>,
    /// Listeners for onCompleted
    completed_listeners: RwLock<Vec<ListenerRegistration>>,
    /// Listeners for onErrorOccurred
    error_listeners: RwLock<Vec<ListenerRegistration>>,
    /// Request ID counter
    next_request_id: RwLock<u64>,
}

impl WebRequestManager {
    /// Create a new web request manager
    pub fn new() -> Self {
        Self {
            before_request_listeners: RwLock::new(Vec::new()),
            before_send_headers_listeners: RwLock::new(Vec::new()),
            send_headers_listeners: RwLock::new(Vec::new()),
            headers_received_listeners: RwLock::new(Vec::new()),
            response_started_listeners: RwLock::new(Vec::new()),
            completed_listeners: RwLock::new(Vec::new()),
            error_listeners: RwLock::new(Vec::new()),
            next_request_id: RwLock::new(1),
        }
    }

    /// Generate a new request ID
    pub fn next_request_id(&self) -> String {
        let mut id = self.next_request_id.write();
        let current = *id;
        *id += 1;
        current.to_string()
    }

    /// Add a listener
    pub fn add_listener(
        &self,
        event: &str,
        registration: ListenerRegistration,
    ) -> ExtensionResult<()> {
        match event {
            "onBeforeRequest" => self.before_request_listeners.write().push(registration),
            "onBeforeSendHeaders" => self
                .before_send_headers_listeners
                .write()
                .push(registration),
            "onSendHeaders" => self.send_headers_listeners.write().push(registration),
            "onHeadersReceived" => self.headers_received_listeners.write().push(registration),
            "onResponseStarted" => self.response_started_listeners.write().push(registration),
            "onCompleted" => self.completed_listeners.write().push(registration),
            "onErrorOccurred" => self.error_listeners.write().push(registration),
            _ => {
                return Err(ExtensionError::InvalidArgument(format!(
                    "Unknown event: {}",
                    event
                )))
            }
        }
        Ok(())
    }

    /// Remove listeners for an extension
    pub fn remove_listeners(&self, extension_id: &str) {
        let remove_for = |listeners: &RwLock<Vec<ListenerRegistration>>| {
            listeners.write().retain(|l| l.extension_id != extension_id);
        };

        remove_for(&self.before_request_listeners);
        remove_for(&self.before_send_headers_listeners);
        remove_for(&self.send_headers_listeners);
        remove_for(&self.headers_received_listeners);
        remove_for(&self.response_started_listeners);
        remove_for(&self.completed_listeners);
        remove_for(&self.error_listeners);
    }

    /// Check if URL matches filter
    pub fn matches_filter(url: &str, filter: &RequestFilter) -> bool {
        if filter.urls.is_empty() {
            return true;
        }

        for pattern in &filter.urls {
            if Self::match_url_pattern(url, pattern) {
                return true;
            }
        }

        false
    }

    /// Match URL against a pattern (simplified glob matching)
    fn match_url_pattern(url: &str, pattern: &str) -> bool {
        // Handle <all_urls>
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

impl Default for WebRequestManager {
    fn default() -> Self {
        Self::new()
    }
}

/// WebRequest API handler
pub struct WebRequestApiHandler {
    manager: Arc<WebRequestManager>,
}

impl WebRequestApiHandler {
    /// Create a new web request API handler
    pub fn new(manager: Arc<WebRequestManager>) -> Self {
        Self { manager }
    }
}

impl ApiHandler for WebRequestApiHandler {
    fn namespace(&self) -> &str {
        "webRequest"
    }

    fn handle(&self, method: &str, params: Value, extension_id: &str) -> ExtensionResult<Value> {
        match method {
            "addListener" => {
                let event = params
                    .get("event")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ExtensionError::InvalidArgument("event is required".to_string())
                    })?;

                let filter: RequestFilter = params
                    .get("filter")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();

                let extra_info_spec: Vec<String> = params
                    .get("extraInfoSpec")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();

                let registration = ListenerRegistration {
                    extension_id: extension_id.to_string(),
                    filter,
                    extra_info_spec,
                };

                self.manager.add_listener(event, registration)?;
                Ok(serde_json::json!({"success": true}))
            }
            "removeListener" => {
                // Remove all listeners for this extension
                self.manager.remove_listeners(extension_id);
                Ok(serde_json::json!({"success": true}))
            }
            "handlerBehaviorChanged" => {
                // Notify that handler behavior has changed
                // This is a hint to flush caches
                Ok(serde_json::json!({}))
            }
            _ => Err(ExtensionError::ApiNotSupported(format!(
                "webRequest.{} is not supported",
                method
            ))),
        }
    }

    fn methods(&self) -> Vec<&str> {
        vec!["addListener", "removeListener", "handlerBehaviorChanged"]
    }
}
