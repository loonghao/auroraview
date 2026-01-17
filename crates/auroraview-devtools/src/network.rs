//! Network inspection types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Network request info for DevTools
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkRequestInfo {
    /// Request ID
    pub request_id: String,
    /// Request URL
    pub url: String,
    /// HTTP method
    pub method: String,
    /// Request headers
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// Post data (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_data: Option<String>,
    /// Resource type
    pub resource_type: String,
    /// Timestamp (seconds since epoch)
    pub timestamp: f64,
}

impl NetworkRequestInfo {
    /// Create a new request info
    pub fn new(
        request_id: impl Into<String>,
        url: impl Into<String>,
        method: impl Into<String>,
    ) -> Self {
        Self {
            request_id: request_id.into(),
            url: url.into(),
            method: method.into(),
            headers: HashMap::new(),
            post_data: None,
            resource_type: "Other".to_string(),
            timestamp: current_timestamp(),
        }
    }

    /// Add a header
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Set headers
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }

    /// Set post data
    pub fn with_post_data(mut self, data: impl Into<String>) -> Self {
        self.post_data = Some(data.into());
        self
    }

    /// Set resource type
    pub fn with_resource_type(mut self, resource_type: impl Into<String>) -> Self {
        self.resource_type = resource_type.into();
        self
    }

    /// Get the domain from URL
    pub fn domain(&self) -> Option<&str> {
        self.url
            .strip_prefix("https://")
            .or_else(|| self.url.strip_prefix("http://"))
            .and_then(|s| s.split('/').next())
    }
}

/// Network response info for DevTools
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkResponseInfo {
    /// Request ID
    pub request_id: String,
    /// Status code
    pub status: u16,
    /// Status text
    pub status_text: String,
    /// Response headers
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// MIME type
    pub mime_type: String,
    /// Content length
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_length: Option<u64>,
    /// Whether response came from cache
    #[serde(default)]
    pub from_cache: bool,
    /// Timestamp (seconds since epoch)
    pub timestamp: f64,
}

impl NetworkResponseInfo {
    /// Create a new response info
    pub fn new(request_id: impl Into<String>, status: u16, status_text: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            status,
            status_text: status_text.into(),
            headers: HashMap::new(),
            mime_type: "text/plain".to_string(),
            content_length: None,
            from_cache: false,
            timestamp: current_timestamp(),
        }
    }

    /// Add a header
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Set headers
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }

    /// Set MIME type
    pub fn with_mime_type(mut self, mime_type: impl Into<String>) -> Self {
        self.mime_type = mime_type.into();
        self
    }

    /// Set content length
    pub fn with_content_length(mut self, length: u64) -> Self {
        self.content_length = Some(length);
        self
    }

    /// Set from cache
    pub fn with_from_cache(mut self, from_cache: bool) -> Self {
        self.from_cache = from_cache;
        self
    }

    /// Check if response is successful (2xx)
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// Check if response is redirect (3xx)
    pub fn is_redirect(&self) -> bool {
        (300..400).contains(&self.status)
    }

    /// Check if response is client error (4xx)
    pub fn is_client_error(&self) -> bool {
        (400..500).contains(&self.status)
    }

    /// Check if response is server error (5xx)
    pub fn is_server_error(&self) -> bool {
        (500..600).contains(&self.status)
    }
}

/// Get current timestamp in seconds
fn current_timestamp() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_info() {
        let request = NetworkRequestInfo::new("req-1", "https://example.com/api", "GET")
            .with_header("Accept", "application/json");

        assert_eq!(request.request_id, "req-1");
        assert_eq!(request.method, "GET");
        assert_eq!(request.domain(), Some("example.com"));
    }

    #[test]
    fn test_response_info() {
        let response = NetworkResponseInfo::new("req-1", 200, "OK")
            .with_mime_type("application/json")
            .with_content_length(1024);

        assert!(response.is_success());
        assert!(!response.is_redirect());
        assert_eq!(response.content_length, Some(1024));
    }

    #[test]
    fn test_response_status_categories() {
        assert!(NetworkResponseInfo::new("1", 200, "OK").is_success());
        assert!(NetworkResponseInfo::new("2", 301, "Moved").is_redirect());
        assert!(NetworkResponseInfo::new("3", 404, "Not Found").is_client_error());
        assert!(NetworkResponseInfo::new("4", 500, "Internal Server Error").is_server_error());
    }
}
