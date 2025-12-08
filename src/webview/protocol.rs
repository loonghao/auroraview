//! Custom protocol handler for loading resources

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Protocol response
pub struct ProtocolResponse {
    /// Response data
    #[allow(dead_code)]
    pub data: Vec<u8>,

    /// MIME type
    #[allow(dead_code)]
    pub mime_type: String,

    /// HTTP status code
    pub status: u16,
}

/// Protocol handler callback type
pub type ProtocolCallback = Arc<dyn Fn(&str) -> Option<ProtocolResponse> + Send + Sync>;

/// Custom protocol handler for WebView
pub struct ProtocolHandler {
    /// Registered protocol handlers
    handlers: Arc<Mutex<HashMap<String, ProtocolCallback>>>,
}

impl ProtocolHandler {
    /// Create a new protocol handler
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a custom protocol
    ///
    /// # Arguments
    /// * `scheme` - Protocol scheme (e.g., "dcc", "asset")
    /// * `handler` - Callback function to handle requests
    #[allow(dead_code)]
    pub fn register<F>(&self, scheme: &str, handler: F)
    where
        F: Fn(&str) -> Option<ProtocolResponse> + Send + Sync + 'static,
    {
        // Use ok() and into_inner() to handle poisoned mutex gracefully
        let mut handlers = match self.handlers.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        handlers.insert(scheme.to_string(), Arc::new(handler));
        tracing::info!("Registered custom protocol: {}", scheme);
    }

    /// Handle a protocol request
    ///
    /// # Arguments
    /// * `uri` - Full URI (e.g., "dcc://assets/texture.png")
    #[allow(dead_code)]
    pub fn handle(&self, uri: &str) -> Option<ProtocolResponse> {
        // Parse scheme from URI
        let scheme = uri.split("://").next()?;

        // Use ok() and into_inner() to handle poisoned mutex gracefully
        let handlers = match self.handlers.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        if let Some(handler) = handlers.get(scheme) {
            tracing::debug!("Handling protocol request: {}", uri);
            return handler(uri);
        }

        tracing::warn!("No handler registered for scheme: {}", scheme);
        None
    }

    /// Unregister a protocol
    #[allow(dead_code)]
    pub fn unregister(&self, scheme: &str) {
        // Use ok() and into_inner() to handle poisoned mutex gracefully
        let mut handlers = match self.handlers.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        handlers.remove(scheme);
        tracing::info!("Unregistered protocol: {}", scheme);
    }

    /// Clear all protocol handlers
    #[allow(dead_code)]
    pub fn clear(&self) {
        // Use ok() and into_inner() to handle poisoned mutex gracefully
        let mut handlers = match self.handlers.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        handlers.clear();
    }
}

impl Default for ProtocolHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ProtocolResponse {
    /// Create a new protocol response
    pub fn new(data: Vec<u8>, mime_type: impl Into<String>) -> Self {
        Self {
            data,
            mime_type: mime_type.into(),
            status: 200,
        }
    }

    /// Create a response with custom status code
    #[allow(dead_code)]
    pub fn with_status(mut self, status: u16) -> Self {
        self.status = status;
        self
    }

    /// Create a text response
    pub fn text(content: impl Into<String>) -> Self {
        Self::new(content.into().into_bytes(), "text/plain")
    }

    /// Create an HTML response
    #[allow(dead_code)]
    pub fn html(content: impl Into<String>) -> Self {
        Self::new(content.into().into_bytes(), "text/html")
    }

    /// Create a JSON response
    #[allow(dead_code)]
    pub fn json(value: &serde_json::Value) -> Self {
        let data = serde_json::to_vec(value).unwrap_or_default();
        Self::new(data, "application/json")
    }

    /// Create a 404 Not Found response
    #[allow(dead_code)]
    pub fn not_found() -> Self {
        Self::text("Not Found").with_status(404)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Unit test: Verify ProtocolHandler creation
    #[test]
    fn test_protocol_handler_creation() {
        let handler = ProtocolHandler::new();
        assert!(handler.handle("unknown://test").is_none());
    }

    /// Unit test: Verify ProtocolHandler default implementation
    #[test]
    fn test_protocol_handler_default() {
        let handler = ProtocolHandler::default();
        assert!(handler.handle("test://resource").is_none());
    }

    /// Unit test: Verify handling unregistered protocol
    #[test]
    fn test_handle_unregistered_protocol() {
        let handler = ProtocolHandler::new();
        let response = handler.handle("unknown://resource");
        assert!(response.is_none());
    }

    /// Unit test: Verify handling invalid URI
    #[test]
    fn test_handle_invalid_uri() {
        let handler = ProtocolHandler::new();
        let response = handler.handle("invalid_uri_without_scheme");
        assert!(response.is_none());
    }

    /// Unit test: Verify ProtocolResponse creation
    #[test]
    fn test_protocol_response_new() {
        let data = b"test data".to_vec();
        let response = ProtocolResponse::new(data.clone(), "text/plain");

        assert_eq!(response.data, data);
        assert_eq!(response.mime_type, "text/plain");
        assert_eq!(response.status, 200);
    }

    /// Unit test: Verify ProtocolResponse with custom status
    #[test]
    fn test_protocol_response_with_status() {
        let response = ProtocolResponse::text("error").with_status(500);
        assert_eq!(response.status, 500);
    }

    /// Unit test: Verify ProtocolResponse::text()
    #[test]
    fn test_protocol_response_text() {
        let response = ProtocolResponse::text("Hello, World!");
        assert_eq!(response.mime_type, "text/plain");
        assert_eq!(String::from_utf8(response.data).unwrap(), "Hello, World!");
        assert_eq!(response.status, 200);
    }

    /// Unit test: Verify ProtocolResponse::html()
    #[test]
    fn test_protocol_response_html() {
        let response = ProtocolResponse::html("<h1>Title</h1>");
        assert_eq!(response.mime_type, "text/html");
        assert_eq!(String::from_utf8(response.data).unwrap(), "<h1>Title</h1>");
        assert_eq!(response.status, 200);
    }

    /// Unit test: Verify ProtocolResponse::json()
    #[test]
    fn test_protocol_response_json() {
        let value = serde_json::json!({"key": "value", "number": 42});
        let response = ProtocolResponse::json(&value);

        assert_eq!(response.mime_type, "application/json");
        assert_eq!(response.status, 200);

        let parsed: serde_json::Value = serde_json::from_slice(&response.data).unwrap();
        assert_eq!(parsed, value);
    }

    /// Unit test: Verify ProtocolResponse::not_found()
    #[test]
    fn test_protocol_response_not_found() {
        let response = ProtocolResponse::not_found();
        assert_eq!(response.status, 404);
        assert_eq!(response.mime_type, "text/plain");
        assert_eq!(String::from_utf8(response.data).unwrap(), "Not Found");
    }
}

// Note: Integration tests have been moved to tests/protocol_integration_tests.rs
// This includes tests for:
// - Protocol registration and handling workflow
// - Unregistering protocols
// - Clearing all protocols
// - Multiple protocol registrations
// - Concurrent access to protocol handler
