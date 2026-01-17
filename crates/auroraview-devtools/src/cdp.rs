//! CDP (Chrome DevTools Protocol) types and utilities

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// CDP session info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CdpSessionInfo {
    /// WebSocket URL for DevTools connection
    pub websocket_debugger_url: String,
    /// DevTools frontend URL
    pub devtools_frontend_url: String,
    /// Browser version
    pub browser: String,
    /// Protocol version
    pub protocol_version: String,
    /// User agent
    pub user_agent: String,
    /// V8 version
    #[serde(default)]
    pub v8_version: String,
    /// WebKit version
    #[serde(default)]
    pub webkit_version: String,
}

/// CDP command request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdpRequest {
    /// Command ID
    pub id: u64,
    /// Method name (e.g., "Page.navigate", "Runtime.evaluate")
    pub method: String,
    /// Parameters
    #[serde(default)]
    pub params: Value,
}

/// CDP command response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdpResponse {
    /// Command ID (matches request)
    pub id: u64,
    /// Result (if success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<CdpError>,
}

/// CDP error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdpError {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
    /// Additional data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}

/// CDP event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdpEvent {
    /// Event method (e.g., "Page.loadEventFired", "Console.messageAdded")
    pub method: String,
    /// Event parameters
    #[serde(default)]
    pub params: Value,
}

/// Common CDP domains
pub mod domains {
    /// Page domain - Page navigation and lifecycle
    pub const PAGE: &str = "Page";
    /// Runtime domain - JavaScript runtime
    pub const RUNTIME: &str = "Runtime";
    /// Network domain - Network activity
    pub const NETWORK: &str = "Network";
    /// DOM domain - DOM tree access
    pub const DOM: &str = "DOM";
    /// Console domain - Console messages
    pub const CONSOLE: &str = "Console";
    /// Debugger domain - JavaScript debugging
    pub const DEBUGGER: &str = "Debugger";
    /// Profiler domain - Performance profiling
    pub const PROFILER: &str = "Profiler";
    /// Target domain - Target/session management
    pub const TARGET: &str = "Target";
    /// Browser domain - Browser-level operations
    pub const BROWSER: &str = "Browser";
}

/// Common CDP methods
pub mod methods {
    // Page domain
    pub const PAGE_NAVIGATE: &str = "Page.navigate";
    pub const PAGE_RELOAD: &str = "Page.reload";
    pub const PAGE_GET_FRAME_TREE: &str = "Page.getFrameTree";
    pub const PAGE_CAPTURE_SCREENSHOT: &str = "Page.captureScreenshot";
    pub const PAGE_ENABLE: &str = "Page.enable";
    pub const PAGE_DISABLE: &str = "Page.disable";

    // Runtime domain
    pub const RUNTIME_EVALUATE: &str = "Runtime.evaluate";
    pub const RUNTIME_CALL_FUNCTION_ON: &str = "Runtime.callFunctionOn";
    pub const RUNTIME_GET_PROPERTIES: &str = "Runtime.getProperties";
    pub const RUNTIME_ENABLE: &str = "Runtime.enable";
    pub const RUNTIME_DISABLE: &str = "Runtime.disable";

    // Network domain
    pub const NETWORK_ENABLE: &str = "Network.enable";
    pub const NETWORK_DISABLE: &str = "Network.disable";
    pub const NETWORK_SET_EXTRA_HEADERS: &str = "Network.setExtraHTTPHeaders";
    pub const NETWORK_GET_RESPONSE_BODY: &str = "Network.getResponseBody";

    // DOM domain
    pub const DOM_GET_DOCUMENT: &str = "DOM.getDocument";
    pub const DOM_QUERY_SELECTOR: &str = "DOM.querySelector";
    pub const DOM_QUERY_SELECTOR_ALL: &str = "DOM.querySelectorAll";
    pub const DOM_ENABLE: &str = "DOM.enable";
    pub const DOM_DISABLE: &str = "DOM.disable";

    // Console domain
    pub const CONSOLE_ENABLE: &str = "Console.enable";
    pub const CONSOLE_DISABLE: &str = "Console.disable";
    pub const CONSOLE_CLEAR_MESSAGES: &str = "Console.clearMessages";

    // Target domain
    pub const TARGET_GET_TARGETS: &str = "Target.getTargets";
    pub const TARGET_CREATE_TARGET: &str = "Target.createTarget";
    pub const TARGET_CLOSE_TARGET: &str = "Target.closeTarget";
    pub const TARGET_ATTACH_TO_TARGET: &str = "Target.attachToTarget";

    // Browser domain
    pub const BROWSER_GET_VERSION: &str = "Browser.getVersion";
    pub const BROWSER_CLOSE: &str = "Browser.close";
}

impl CdpRequest {
    /// Create a new CDP request
    pub fn new(id: u64, method: impl Into<String>, params: Value) -> Self {
        Self {
            id,
            method: method.into(),
            params,
        }
    }

    /// Create a simple request without parameters
    pub fn simple(id: u64, method: impl Into<String>) -> Self {
        Self::new(id, method, Value::Null)
    }

    /// Create Page.navigate request
    pub fn page_navigate(id: u64, url: &str) -> Self {
        Self::new(
            id,
            methods::PAGE_NAVIGATE,
            serde_json::json!({ "url": url }),
        )
    }

    /// Create Page.reload request
    pub fn page_reload(id: u64) -> Self {
        Self::simple(id, methods::PAGE_RELOAD)
    }

    /// Create Runtime.evaluate request
    pub fn runtime_evaluate(id: u64, expression: &str) -> Self {
        Self::new(
            id,
            methods::RUNTIME_EVALUATE,
            serde_json::json!({ "expression": expression }),
        )
    }

    /// Create Page.captureScreenshot request
    pub fn capture_screenshot(id: u64, format: &str) -> Self {
        Self::new(
            id,
            methods::PAGE_CAPTURE_SCREENSHOT,
            serde_json::json!({ "format": format }),
        )
    }
}

impl CdpResponse {
    /// Create a success response
    pub fn success(id: u64, result: Value) -> Self {
        Self {
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response
    pub fn error(id: u64, code: i32, message: impl Into<String>) -> Self {
        Self {
            id,
            result: None,
            error: Some(CdpError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }

    /// Check if response is success
    pub fn is_success(&self) -> bool {
        self.error.is_none()
    }

    /// Get result or error message
    pub fn into_result(self) -> Result<Value, String> {
        if let Some(error) = self.error {
            Err(error.message)
        } else {
            Ok(self.result.unwrap_or(Value::Null))
        }
    }
}

impl CdpEvent {
    /// Create a new CDP event
    pub fn new(method: impl Into<String>, params: Value) -> Self {
        Self {
            method: method.into(),
            params,
        }
    }

    /// Check if this is a Page event
    pub fn is_page_event(&self) -> bool {
        self.method.starts_with("Page.")
    }

    /// Check if this is a Network event
    pub fn is_network_event(&self) -> bool {
        self.method.starts_with("Network.")
    }

    /// Check if this is a Console event
    pub fn is_console_event(&self) -> bool {
        self.method.starts_with("Console.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_cdp_request() {
        let req = CdpRequest::new(1, "Page.navigate", json!({"url": "https://example.com"}));
        assert_eq!(req.id, 1);
        assert_eq!(req.method, "Page.navigate");
    }

    #[test]
    fn test_cdp_request_helpers() {
        let nav = CdpRequest::page_navigate(1, "https://example.com");
        assert_eq!(nav.method, methods::PAGE_NAVIGATE);

        let eval = CdpRequest::runtime_evaluate(2, "1 + 1");
        assert_eq!(eval.method, methods::RUNTIME_EVALUATE);
    }

    #[test]
    fn test_cdp_response() {
        let success = CdpResponse::success(1, json!({"frameId": "123"}));
        assert!(success.is_success());

        let error = CdpResponse::error(2, -32601, "Method not found");
        assert!(!error.is_success());
    }

    #[test]
    fn test_cdp_response_into_result() {
        let success = CdpResponse::success(1, json!({"value": 42}));
        assert!(success.into_result().is_ok());

        let error = CdpResponse::error(2, -32601, "Method not found");
        assert!(error.into_result().is_err());
    }

    #[test]
    fn test_cdp_event() {
        let page_event = CdpEvent::new("Page.loadEventFired", json!({}));
        assert!(page_event.is_page_event());
        assert!(!page_event.is_network_event());

        let network_event = CdpEvent::new("Network.requestWillBeSent", json!({}));
        assert!(network_event.is_network_event());
    }
}
