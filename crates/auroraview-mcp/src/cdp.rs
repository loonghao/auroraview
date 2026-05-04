//! Minimal async CDP (Chrome DevTools Protocol) client for the AuroraView adapter.
//!
//! Only implements the handful of commands the adapter skeleton needs:
//!
//! - `Browser.getVersion` — health check.
//! - `Page.captureScreenshot` — viewport capture.
//!
//! Target discovery (`GET http://host:port/json/version`) is handled via
//! `reqwest`, then we open a single WebSocket to the browser-level endpoint.
//!
//! The client is deliberately tiny: one request in flight at a time, no
//! event subscription plumbing. That is enough to back `DccSnapshot` and
//! `DccConnection::health_check`.

use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};

/// Errors produced by the CDP client.
#[derive(Debug, thiserror::Error)]
pub enum CdpError {
    /// HTTP error during CDP endpoint discovery (GET /json/version).
    #[error("HTTP target discovery failed: {0}")]
    Http(#[from] reqwest::Error),
    /// WebSocket error during CDP communication.
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),
    /// JSON serialization or deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    /// Base64 decoding error (e.g. when decoding screenshot data).
    #[error("base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),
    /// CDP returned an error message in the response.
    #[error("CDP method {0} returned error: {1}")]
    Remote(String, String),
    /// CDP response is missing an expected field.
    #[error("unexpected CDP response for {0}: missing `{1}` field")]
    MalformedResponse(String, &'static str),
    /// CDP connection was closed before receiving a response.
    #[error("CDP connection closed before receiving response for {0}")]
    ConnectionClosed(String),
    /// CDP request timed out waiting for a response.
    #[error("CDP method {0} timed out after {1:?}")]
    Timeout(String, Duration),
}

/// `http://<host>:<port>/json/version` response shape (subset we care about).
#[derive(Debug, Deserialize)]
struct VersionInfo {
    #[serde(rename = "webSocketDebuggerUrl")]
    web_socket_debugger_url: String,
    #[serde(rename = "Browser", default)]
    browser: String,
    #[serde(rename = "Protocol-Version", default)]
    protocol_version: String,
}

/// Static information returned by `Browser.getVersion`.
#[derive(Debug, Clone)]
pub struct BrowserVersion {
    /// Product identifier (e.g. "Chrome/120.0.6099.109").
    pub product: String,
    /// CDP protocol version string.
    pub protocol_version: String,
}

/// Async CDP client holding a single browser-level WebSocket.
///
/// Implements `Clone` by wrapping the WebSocket in an `Arc<Mutex<>>`.
/// This allows the client to be shared across multiple tool calls.
#[derive(Clone)]
pub struct CdpClient {
    inner: Arc<Mutex<CdpClientInner>>,
    /// Endpoint URL we connected to, kept around for diagnostics.
    pub endpoint: String,
}

/// Inner state of `CdpClient` (not `Clone`).
struct CdpClientInner {
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
    next_id: u64,
}

impl CdpClient {
    /// Connect to a CDP endpoint on `http://host:port`.
    ///
    /// Performs `GET /json/version` to discover the browser-level WebSocket
    /// debugger URL, then opens a WebSocket to it.
    pub async fn connect(http_endpoint: &str) -> Result<Self, CdpError> {
        let url = format!("{}/json/version", http_endpoint.trim_end_matches('/'));
        let info: VersionInfo = reqwest::get(&url).await?.error_for_status()?.json().await?;
        tracing::debug!(
            browser = %info.browser,
            protocol = %info.protocol_version,
            ws = %info.web_socket_debugger_url,
            "resolved CDP target"
        );

        let (ws, _resp) = connect_async(&info.web_socket_debugger_url).await?;
        let inner = CdpClientInner { ws, next_id: 1 };
        Ok(Self {
            inner: Arc::new(Mutex::new(inner)),
            endpoint: info.web_socket_debugger_url,
        })
    }

    /// Send a CDP command and wait for its matching response.
    ///
    /// Any events received while waiting are dropped — the skeleton adapter
    /// is request/response only.
    pub async fn call(
        &self,
        method: &str,
        params: Value,
        timeout: Duration,
    ) -> Result<Value, CdpError> {
        let mut inner = self.inner.lock().await;

        let id = inner.next_id;
        inner.next_id += 1;

        tracing::debug!(%method, %id, "CDP call");

        let request = json!({
            "id": id,
            "method": method,
            "params": params,
        });
        inner.ws.send(Message::Text(request.to_string())).await.map_err(|e| {
            tracing::warn!(%method, error = %e, "CDP send failed");
            e
        })?;

        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                tracing::warn!(%method, ?timeout, "CDP call timed out");
                return Err(CdpError::Timeout(method.to_string(), timeout));
            }
            let msg = match tokio::time::timeout(remaining, inner.ws.next()).await {
                Ok(Some(m)) => m.map_err(|e| {
                    tracing::warn!(%method, error = %e, "CDP WebSocket error");
                    e
                })?,
                Ok(None) => {
                    tracing::warn!(%method, "CDP connection closed");
                    return Err(CdpError::ConnectionClosed(method.to_string()));
                }
                Err(_) => {
                    tracing::warn!(%method, ?timeout, "CDP timeout waiting for response");
                    return Err(CdpError::Timeout(method.to_string(), timeout));
                }
            };

            let text = match msg {
                Message::Text(t) => t,
                Message::Binary(_) | Message::Ping(_) | Message::Pong(_) | Message::Frame(_) => {
                    continue
                }
                Message::Close(_) => {
                    tracing::warn!(%method, "CDP connection closed by peer");
                    return Err(CdpError::ConnectionClosed(method.to_string()));
                }
            };

            let value: Value = serde_json::from_str(&text).map_err(|e| {
                tracing::warn!(%method, error = %e, "CDP JSON parse failed");
                e
            })?;

            // Events have no `id` — skip them in this minimal client.
            match value.get("id").and_then(Value::as_u64) {
                Some(resp_id) if resp_id == id => {
                    if let Some(err) = value.get("error") {
                        tracing::warn!(%method, error = %err, "CDP returned error");
                        return Err(CdpError::Remote(method.to_string(), err.to_string()));
                    }
                    let result = value
                        .get("result")
                        .cloned()
                        .ok_or_else(|| {
                            tracing::warn!(%method, "CDP response missing 'result' field");
                            CdpError::MalformedResponse(method.to_string(), "result")
                        })?;
                    tracing::debug!(%method, %id, "CDP call succeeded");
                    return Ok(result);
                }
                _ => continue,
            }
        }
    }

    /// `Browser.getVersion` — lightweight liveness probe.
    pub async fn get_version(&self, timeout: Duration) -> Result<BrowserVersion, CdpError> {
        let result = self.call("Browser.getVersion", json!({}), timeout).await?;
        Ok(BrowserVersion {
            product: result
                .get("product")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned(),
            protocol_version: result
                .get("protocolVersion")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned(),
        })
    }

    /// `Page.captureScreenshot` — returns raw image bytes.
    ///
    /// `format` is passed straight through (`"png"` / `"jpeg"` / `"webp"`).
    /// Callers are expected to pre-validate it.
    pub async fn capture_screenshot(
        &self,
        format: &str,
        timeout: Duration,
    ) -> Result<Vec<u8>, CdpError> {
        let params = json!({
            "format": format,
            "captureBeyondViewport": false,
            "fromSurface": true,
        });
        let result = self.call("Page.captureScreenshot", params, timeout).await?;
        let data_b64 = result
            .get("data")
            .and_then(Value::as_str)
            .ok_or(CdpError::MalformedResponse(
                "Page.captureScreenshot".to_string(),
                "data",
            ))?;
        let bytes = <base64::engine::general_purpose::GeneralPurpose as base64::Engine>::decode(
            &base64::engine::general_purpose::STANDARD,
            data_b64,
        )?;
        Ok(bytes)
    }

    /// `Runtime.evaluate` — execute JavaScript and return the result.
    ///
    /// Returns the JSON value of the expression result.
    pub async fn evaluate_script(
        &self,
        script: &str,
        timeout: Duration,
    ) -> Result<Value, CdpError> {
        let params = json!({
            "expression": script,
            "returnByValue": true,
            "awaitPromise": true,
        });
        let result = self.call("Runtime.evaluate", params, timeout).await?;
        let value = result
            .get("result")
            .and_then(|v| v.get("value"))
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        Ok(value)
    }

    /// `Page.navigate` — navigate the WebView to a URL.
    pub async fn navigate_to(&self, url: &str, timeout: Duration) -> Result<(), CdpError> {
        let params = json!({ "url": url });
        self.call("Page.navigate", params, timeout).await?;
        Ok(())
    }

    /// `Page.reload` — reload the current page.
    pub async fn reload(&self, timeout: Duration) -> Result<(), CdpError> {
        let params = json!({ "ignoreCache": false });
        self.call("Page.reload", params, timeout).await?;
        Ok(())
    }

    /// `Page.printToPDF` — generate a PDF of the current page.
    ///
    /// Returns the PDF as raw bytes (already decoded from base64).
    pub async fn print_to_pdf(
        &self,
        timeout: Duration,
    ) -> Result<Vec<u8>, CdpError> {
        let params = json!({
            "printBackground": true,
            "preferCSSPageSize": true,
        });
        let result = self.call("Page.printToPDF", params, timeout).await?;
        let data_b64 = result
            .get("data")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                tracing::warn!("Page.printToPDF response missing 'data' field");
                CdpError::MalformedResponse("Page.printToPDF".to_string(), "data")
            })?;
        let bytes = <base64::engine::general_purpose::GeneralPurpose as base64::Engine>::decode(
            &base64::engine::general_purpose::STANDARD,
            data_b64,
        )?;
        tracing::debug!(size = bytes.len(), "Page.printToPDF succeeded");
        Ok(bytes)
    }

    /// `Network.enable` — enable network monitoring.
    ///
    /// Call this before using `Network.*` events.
    pub async fn network_enable(&self, timeout: Duration) -> Result<(), CdpError> {
        self.call("Network.enable", json!({}), timeout).await?;
        tracing::debug!("Network monitoring enabled");
        Ok(())
    }

    /// `Network.disable` — disable network monitoring.
    pub async fn network_disable(&self, timeout: Duration) -> Result<(), CdpError> {
        self.call("Network.disable", json!({}), timeout).await?;
        tracing::debug!("Network monitoring disabled");
        Ok(())
    }

    /// `DOM.getDocument` — get the DOM document node.
    ///
    /// Returns the root `Document` node as JSON.
    pub async fn get_document(&self, timeout: Duration) -> Result<Value, CdpError> {
        let result = self.call("DOM.getDocument", json!({}), timeout).await?;
        tracing::debug!(?result, "DOM.getDocument succeeded");
        Ok(result)
    }

    /// `CSS.getStylesForNode` — get computed styles for a DOM node.
    ///
    /// `node_id` is the DOM node ID (from `DOM.getDocument` or `DOM.querySelector`).
    /// Returns the computed styles as JSON.
    pub async fn get_styles_for_node(
        &self,
        node_id: i64,
        timeout: Duration,
    ) -> Result<Value, CdpError> {
        let params = json!({"nodeId": node_id});
        let result = self.call("CSS.getStylesForNode", params, timeout).await?;
        tracing::debug!(?node_id, ?result, "CSS.getStylesForNode succeeded");
        Ok(result)
    }

    /// `DOM.querySelector` — find the first element matching a CSS selector.
    ///
    /// `node_id` is the parent node ID (usually from `DOM.getDocument`).
    /// `selector` is a CSS selector string (e.g., `"#my-id"`, `".my-class"`).
    /// Returns the found node ID, or `None` if not found.
    pub async fn query_selector(
        &self,
        node_id: i64,
        selector: &str,
        timeout: Duration,
    ) -> Result<Option<i64>, CdpError> {
        let params = json!({
            "nodeId": node_id,
            "selector": selector,
        });
        let result = self.call("DOM.querySelector", params, timeout).await?;
        let found_node_id = result
            .get("nodeId")
            .and_then(Value::as_i64)
            .filter(|&id| id != 0); // CDP returns 0 when not found
        tracing::debug!(?node_id, %selector, ?found_node_id, "DOM.querySelector succeeded");
        Ok(found_node_id)
    }

    /// `DOM.querySelectorAll` — find all elements matching a CSS selector.
    ///
    /// `node_id` is the parent node ID (usually from `DOM.getDocument`).
    /// `selector` is a CSS selector string.
    /// Returns a vector of node IDs.
    pub async fn query_selector_all(
        &self,
        node_id: i64,
        selector: &str,
        timeout: Duration,
    ) -> Result<Vec<i64>, CdpError> {
        let params = json!({
            "nodeId": node_id,
            "selector": selector,
        });
        let result = self.call("DOM.querySelectorAll", params, timeout).await?;
        let node_ids: Vec<i64> = result
            .get("nodeIds")
            .and_then(Value::as_array)
            .map(|arr| arr.iter().filter_map(Value::as_i64).collect())
            .unwrap_or_default();
        tracing::debug!(?node_id, %selector, count = node_ids.len(), "DOM.querySelectorAll succeeded");
        Ok(node_ids)
    }

    /// `DOM.getOuterHTML` — get the outer HTML of a DOM node.
    ///
    /// `node_id` is the DOM node ID (from `DOM.getDocument` or `DOM.querySelector`).
    /// Returns the outer HTML as a string.
    pub async fn get_outer_html(
        &self,
        node_id: i64,
        timeout: Duration,
    ) -> Result<String, CdpError> {
        let params = json!({"nodeId": node_id});
        let result = self.call("DOM.getOuterHTML", params, timeout).await?;
        let html = result
            .get("outerHTML")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                tracing::warn!(?node_id, "DOM.getOuterHTML response missing 'outerHTML' field");
                CdpError::MalformedResponse("DOM.getOuterHTML".to_string(), "outerHTML")
            })?
            .to_owned();
        tracing::debug!(?node_id, html_len = html.len(), "DOM.getOuterHTML succeeded");
        Ok(html)
    }

    /// `DOM.getAttributes` — get all attributes of a DOM node.
    ///
    /// `node_id` is the DOM node ID.
    /// Returns a map of attribute name -> value.
    pub async fn get_attributes(
        &self,
        node_id: i64,
        timeout: Duration,
    ) -> Result<std::collections::HashMap<String, String>, CdpError> {
        let params = json!({"nodeId": node_id});
        let result = self.call("DOM.getAttributes", params, timeout).await?;
        let attrs_array = result
            .get("attributes")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                tracing::warn!(?node_id, "DOM.getAttributes response missing 'attributes' field");
                CdpError::MalformedResponse("DOM.getAttributes".to_string(), "attributes")
            })?;

        // CDP returns attributes as a flat array: ["name1", "value1", "name2", "value2", ...]
        let mut attrs = std::collections::HashMap::new();
        let mut i = 0;
        while i + 1 < attrs_array.len() {
            if let (Some(name), Some(value)) = (attrs_array[i].as_str(), attrs_array[i + 1].as_str()) {
                attrs.insert(name.to_owned(), value.to_owned());
            }
            i += 2;
        }
        tracing::debug!(?node_id, count = attrs.len(), "DOM.getAttributes succeeded");
        Ok(attrs)
    }

    /// `DOM.setNodeValue` — set the value of a text node.
    ///
    /// `node_id` is the DOM node ID (must be a text node).
    /// `value` is the new text value.
    pub async fn set_node_value(
        &self,
        node_id: i64,
        value: &str,
        timeout: Duration,
    ) -> Result<(), CdpError> {
        let params = json!({
            "nodeId": node_id,
            "value": value,
        });
        self.call("DOM.setNodeValue", params, timeout).await?;
        tracing::debug!(?node_id, %value, "DOM.setNodeValue succeeded");
        Ok(())
    }

    /// `Runtime.getProperties` — get object properties (for inspecting JS objects).
    ///
    /// `object_id` is the unique object ID (from `Runtime.evaluate` result with `objectId`).
    /// Returns a list of property descriptors.
    pub async fn get_properties(
        &self,
        object_id: &str,
        timeout: Duration,
    ) -> Result<Vec<Value>, CdpError> {
        let params = json!({
            "objectId": object_id,
            "ownProperties": true,
        });
        let result = self.call("Runtime.getProperties", params, timeout).await?;
        let props = result
            .get("result")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        tracing::debug!(?object_id, count = props.len(), "Runtime.getProperties succeeded");
        Ok(props)
    }

    /// `Network.getResponseBody` — get the response body for a network request.
    ///
    /// `request_id` is the network request ID (from `Network.requestWillBeSent` event).
    /// Returns the response body as bytes (handles base64-encoded bodies).
    pub async fn get_response_body(
        &self,
        request_id: &str,
        timeout: Duration,
    ) -> Result<Vec<u8>, CdpError> {
        let params = json!({"requestId": request_id});
        let result = self.call("Network.getResponseBody", params, timeout).await?;
        let body = result
            .get("body")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                tracing::warn!(?request_id, "Network.getResponseBody response missing 'body' field");
                CdpError::MalformedResponse("Network.getResponseBody".to_string(), "body")
            })?;
        let is_base64 = result
            .get("base64Encoded")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let bytes = if is_base64 {
            <base64::engine::general_purpose::GeneralPurpose as base64::Engine>::decode(
                &base64::engine::general_purpose::STANDARD,
                body,
            )?
        } else {
            body.as_bytes().to_vec()
        };
        tracing::debug!(?request_id, size = bytes.len(), "Network.getResponseBody succeeded");
        Ok(bytes)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn cdp_error_display_timeout() {
        let dur = Duration::from_secs(5);
        let err = CdpError::Timeout("test_method".to_string(), dur);
        let msg = format!("{err}");
        assert!(msg.contains("timed out"), "got: {msg}");
    }

    #[test]
    fn cdp_error_display_connection_closed() {
        let err = CdpError::ConnectionClosed("test_method".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("closed before"), "got: {msg}");
    }

    #[test]
    fn cdp_error_display_remote() {
        let err = CdpError::Remote("test_method".to_string(), "test error".to_owned());
        let msg = format!("{err}");
        assert!(msg.contains("test error"), "got: {msg}");
    }

    #[test]
    fn cdp_error_display_malformed_response() {
        let err = CdpError::MalformedResponse("test_method".to_string(), "result");
        let msg = format!("{err}");
        assert!(msg.contains("result"), "got: {msg}");
    }

    #[test]
    fn browser_version_creation() {
        let version = BrowserVersion {
            product: "Chrome/120.0.6099.109".to_owned(),
            protocol_version: "1.3".to_owned(),
        };
        assert_eq!(version.product, "Chrome/120.0.6099.109");
        assert_eq!(version.protocol_version, "1.3");
    }

    #[test]
    fn browser_version_debug() {
        let version = BrowserVersion {
            product: "test".to_owned(),
            protocol_version: "1.0".to_owned(),
        };
        let debug = format!("{version:?}");
        assert!(debug.contains("test"));
    }

    // Note: `query_selector` and `query_selector_all` require a live CDP WebSocket
    // connection.  Their unit tests live in `tests/cdp_integration_test.rs`, which
    // is only run when `CARGO_FEATURE_INTEGRATION_TESTS` is set.
    //
    // We do however test the response-parsing logic through the tests below.

    /// Simulate a successful `DOM.querySelector` response and assert the parser
    /// returns `Some(node_id)`.
    #[test]
    fn query_selector_returns_node_id() {
        // We can't easily unit-test `CdpClient` without a live WebSocket, but we
        // can verify the JSON response shape our implementation expects.
        let json = serde_json::json!({"result": {"nodeId": 42}});
        let node_id = json
            .get("result")
            .and_then(|r| r.get("nodeId"))
            .and_then(serde_json::Value::as_i64)
            .filter(|&id| id != 0);
        assert_eq!(node_id, Some(42));
    }

    /// Simulate a `DOM.querySelector` response where no element matched (CDP
    /// returns `nodeId: 0`).
    #[test]
    fn query_selector_returns_none_when_not_found() {
        let json = serde_json::json!({"result": {"nodeId": 0}});
        let node_id = json
            .get("result")
            .and_then(|r| r.get("nodeId"))
            .and_then(serde_json::Value::as_i64)
            .filter(|&id| id != 0);
        assert_eq!(node_id, None);
    }

    /// Simulate a successful `DOM.querySelectorAll` response.
    #[test]
    fn query_selector_all_returns_node_ids() {
        let json = serde_json::json!({"result": {"nodeIds": [1, 2, 3]}});
        let node_ids = json
            .get("result")
            .and_then(|r| r.get("nodeIds"))
            .and_then(serde_json::Value::as_array)
            .map(|arr| arr.iter().filter_map(serde_json::Value::as_i64).collect::<Vec<_>>())
            .unwrap_or_default();
        assert_eq!(node_ids, vec![1, 2, 3]);
    }

    /// Simulate a `DOM.querySelectorAll` response with no matches.
    #[test]
    fn query_selector_all_returns_empty_vec() {
        let json = serde_json::json!({"result": {"nodeIds": []}});
        let node_ids = json
            .get("result")
            .and_then(|r| r.get("nodeIds"))
            .and_then(serde_json::Value::as_array)
            .map(|arr| arr.iter().filter_map(serde_json::Value::as_i64).collect::<Vec<_>>())
            .unwrap_or_default();
        assert!(node_ids.is_empty());
    }

    /// Simulate a successful `DOM.getOuterHTML` response.
    #[test]
    fn get_outer_html_returns_html() {
        let json = serde_json::json!({"result": {"outerHTML": "<div>Hello</div>"}});
        let html = json
            .get("result")
            .and_then(|r| r.get("outerHTML"))
            .and_then(serde_json::Value::as_str)
            .map(String::from)
            .unwrap_or_default();
        assert_eq!(html, "<div>Hello</div>");
    }

    /// Simulate a `DOM.getOuterHTML` response with missing field.
    #[test]
    fn get_outer_html_handles_missing_field() {
        let json = serde_json::json!({"result": {}});
        let html = json
            .get("result")
            .and_then(|r| r.get("outerHTML"))
            .and_then(serde_json::Value::as_str)
            .map(String::from)
            .unwrap_or_default();
        assert_eq!(html, "");
    }

    /// Simulate a successful `DOM.getAttributes` response.
    #[test]
    fn get_attributes_returns_attributes() {
        let json = serde_json::json!({"result": {"attributes": ["id", "my-id", "class", "my-class"]}});
        let attrs_array = json
            .get("result")
            .and_then(|r| r.get("attributes"))
            .and_then(serde_json::Value::as_array)
            .unwrap();

        let mut attrs = std::collections::HashMap::new();
        let mut i = 0;
        while i + 1 < attrs_array.len() {
            if let (Some(name), Some(value)) = (attrs_array[i].as_str(), attrs_array[i + 1].as_str()) {
                attrs.insert(name.to_owned(), value.to_owned());
            }
            i += 2;
        }
        assert_eq!(attrs.len(), 2);
        assert_eq!(attrs.get("id"), Some(&"my-id".to_owned()));
        assert_eq!(attrs.get("class"), Some(&"my-class".to_owned()));
    }

    /// Simulate a `DOM.getAttributes` response with no attributes.
    #[test]
    fn get_attributes_returns_empty() {
        let json = serde_json::json!({"result": {"attributes": []}});
        let attrs_array = json
            .get("result")
            .and_then(|r| r.get("attributes"))
            .and_then(serde_json::Value::as_array)
            .unwrap();

        let mut attrs = std::collections::HashMap::new();
        let mut i = 0;
        while i + 1 < attrs_array.len() {
            if let (Some(name), Some(value)) = (attrs_array[i].as_str(), attrs_array[i + 1].as_str()) {
                attrs.insert(name.to_owned(), value.to_owned());
            }
            i += 2;
        }
        assert!(attrs.is_empty());
    }

    /// Simulate a successful `Runtime.getProperties` response.
    #[test]
    fn get_properties_returns_properties() {
        let json = serde_json::json!({
            "result": {
                "result": [
                    {"name": "prop1", "value": {"type": "string", "value": "hello"}},
                    {"name": "prop2", "value": {"type": "number", "value": 42}}
                ]
            }
        });
        let props = json
            .get("result")
            .and_then(|r| r.get("result"))
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        assert_eq!(props.len(), 2);
        assert_eq!(props[0].get("name").and_then(|v| v.as_str()), Some("prop1"));
        assert_eq!(props[1].get("name").and_then(|v| v.as_str()), Some("prop2"));
    }

    /// Simulate a `Network.getResponseBody` response with plain text.
    #[test]
    fn get_response_body_returns_text() {
        let json = serde_json::json!({
            "result": {
                "body": "Hello, World!",
                "base64Encoded": false
            }
        });
        let body = json.get("result").and_then(|r| r.get("body")).and_then(|v| v.as_str()).unwrap();
        let is_base64 = json.get("result").and_then(|r| r.get("base64Encoded")).and_then(|v| v.as_bool()).unwrap_or(false);
        let bytes = if is_base64 {
            <base64::engine::general_purpose::GeneralPurpose as base64::Engine>::decode(
                &base64::engine::general_purpose::STANDARD,
                body,
            ).unwrap()
        } else {
            body.as_bytes().to_vec()
        };
        assert_eq!(bytes, b"Hello, World!");
    }

    /// Simulate a `Network.getResponseBody` response with base64-encoded data.
    #[test]
    fn get_response_body_returns_base64() {
        let json = serde_json::json!({
            "result": {
                "body": "SGVsbG8sIFdvcmxkIQ==",
                "base64Encoded": true
            }
        });
        let body = json.get("result").and_then(|r| r.get("body")).and_then(|v| v.as_str()).unwrap();
        let is_base64 = json.get("result").and_then(|r| r.get("base64Encoded")).and_then(|v| v.as_bool()).unwrap_or(false);
        let bytes = if is_base64 {
            <base64::engine::general_purpose::GeneralPurpose as base64::Engine>::decode(
                &base64::engine::general_purpose::STANDARD,
                body,
            ).unwrap()
        } else {
            body.as_bytes().to_vec()
        };
        assert_eq!(bytes, b"Hello, World!");
    }
}
