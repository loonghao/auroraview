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
        let mut handlers = self.handlers.lock().unwrap();
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

        let handlers = self.handlers.lock().unwrap();

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
        let mut handlers = self.handlers.lock().unwrap();
        handlers.remove(scheme);
        tracing::info!("Unregistered protocol: {}", scheme);
    }

    /// Clear all protocol handlers
    #[allow(dead_code)]
    pub fn clear(&self) {
        let mut handlers = self.handlers.lock().unwrap();
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

    #[test]
    fn test_protocol_handler_creation() {
        let handler = ProtocolHandler::new();
        assert!(handler.handle("unknown://test").is_none());
    }

    #[test]
    fn test_protocol_handler_default() {
        let handler = ProtocolHandler::default();
        assert!(handler.handle("test://resource").is_none());
    }

    #[test]
    fn test_register_and_handle_protocol() {
        let handler = ProtocolHandler::new();

        handler.register("dcc", |uri| {
            if uri.contains("test") {
                Some(ProtocolResponse::text("Test response"))
            } else {
                None
            }
        });

        let response = handler.handle("dcc://test/resource");
        assert!(response.is_some());
        let resp = response.unwrap();
        assert_eq!(resp.status, 200);
        assert_eq!(resp.mime_type, "text/plain");
        assert_eq!(String::from_utf8(resp.data).unwrap(), "Test response");
    }

    #[test]
    fn test_handle_unregistered_protocol() {
        let handler = ProtocolHandler::new();
        let response = handler.handle("unknown://resource");
        assert!(response.is_none());
    }

    #[test]
    fn test_handle_invalid_uri() {
        let handler = ProtocolHandler::new();
        let response = handler.handle("invalid_uri_without_scheme");
        assert!(response.is_none());
    }

    #[test]
    fn test_unregister_protocol() {
        let handler = ProtocolHandler::new();

        handler.register("test", |_| Some(ProtocolResponse::text("data")));
        assert!(handler.handle("test://resource").is_some());

        handler.unregister("test");
        assert!(handler.handle("test://resource").is_none());
    }

    #[test]
    fn test_clear_all_protocols() {
        let handler = ProtocolHandler::new();

        handler.register("proto1", |_| Some(ProtocolResponse::text("1")));
        handler.register("proto2", |_| Some(ProtocolResponse::text("2")));

        handler.clear();

        assert!(handler.handle("proto1://test").is_none());
        assert!(handler.handle("proto2://test").is_none());
    }

    #[test]
    fn test_protocol_response_new() {
        let data = b"test data".to_vec();
        let response = ProtocolResponse::new(data.clone(), "text/plain");

        assert_eq!(response.data, data);
        assert_eq!(response.mime_type, "text/plain");
        assert_eq!(response.status, 200);
    }

    #[test]
    fn test_protocol_response_with_status() {
        let response = ProtocolResponse::text("error").with_status(500);
        assert_eq!(response.status, 500);
    }

    #[test]
    fn test_protocol_response_text() {
        let response = ProtocolResponse::text("Hello, World!");
        assert_eq!(response.mime_type, "text/plain");
        assert_eq!(String::from_utf8(response.data).unwrap(), "Hello, World!");
        assert_eq!(response.status, 200);
    }

    #[test]
    fn test_protocol_response_html() {
        let response = ProtocolResponse::html("<h1>Title</h1>");
        assert_eq!(response.mime_type, "text/html");
        assert_eq!(String::from_utf8(response.data).unwrap(), "<h1>Title</h1>");
        assert_eq!(response.status, 200);
    }

    #[test]
    fn test_protocol_response_json() {
        let value = serde_json::json!({"key": "value", "number": 42});
        let response = ProtocolResponse::json(&value);

        assert_eq!(response.mime_type, "application/json");
        assert_eq!(response.status, 200);

        let parsed: serde_json::Value = serde_json::from_slice(&response.data).unwrap();
        assert_eq!(parsed, value);
    }

    #[test]
    fn test_protocol_response_not_found() {
        let response = ProtocolResponse::not_found();
        assert_eq!(response.status, 404);
        assert_eq!(response.mime_type, "text/plain");
        assert_eq!(String::from_utf8(response.data).unwrap(), "Not Found");
    }

    #[test]
    fn test_multiple_protocols() {
        let handler = ProtocolHandler::new();

        handler.register("asset", |uri| {
            Some(ProtocolResponse::text(format!("Asset: {}", uri)))
        });

        handler.register("dcc", |uri| {
            Some(ProtocolResponse::json(&serde_json::json!({"uri": uri})))
        });

        let asset_resp = handler.handle("asset://texture.png").unwrap();
        assert_eq!(asset_resp.mime_type, "text/plain");

        let dcc_resp = handler.handle("dcc://command").unwrap();
        assert_eq!(dcc_resp.mime_type, "application/json");
    }

    #[test]
    fn test_protocol_handler_thread_safety() {
        use std::sync::Arc;
        use std::thread;

        let handler = Arc::new(ProtocolHandler::new());
        handler.register("test", |_| Some(ProtocolResponse::text("data")));

        let handler_clone = Arc::clone(&handler);
        let handle = thread::spawn(move || handler_clone.handle("test://resource"));

        let result = handle.join().unwrap();
        assert!(result.is_some());
    }
}
