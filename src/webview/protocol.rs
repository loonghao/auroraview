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
