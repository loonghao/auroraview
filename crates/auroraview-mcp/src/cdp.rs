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
}
