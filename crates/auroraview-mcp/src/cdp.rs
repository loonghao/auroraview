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

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};

/// Errors produced by the CDP client.
#[derive(Debug, thiserror::Error)]
pub enum CdpError {
    #[error("HTTP target discovery failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("CDP returned an error response: {0}")]
    Remote(String),
    #[error("unexpected CDP response: missing `{0}` field")]
    MalformedResponse(&'static str),
    #[error("CDP connection closed before a response was received")]
    ConnectionClosed,
    #[error("CDP request timed out after {0:?}")]
    Timeout(Duration),
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
    pub product: String,
    pub protocol_version: String,
}

/// Async CDP client holding a single browser-level WebSocket.
pub struct CdpClient {
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
    next_id: u64,
    /// Endpoint URL we connected to, kept around for diagnostics.
    pub endpoint: String,
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
        Ok(Self {
            ws,
            next_id: 1,
            endpoint: info.web_socket_debugger_url,
        })
    }

    /// Send a CDP command and wait for its matching response.
    ///
    /// Any events received while waiting are dropped — the skeleton adapter
    /// is request/response only.
    pub async fn call(
        &mut self,
        method: &str,
        params: Value,
        timeout: Duration,
    ) -> Result<Value, CdpError> {
        let id = self.next_id;
        self.next_id += 1;

        let request = json!({
            "id": id,
            "method": method,
            "params": params,
        });
        self.ws.send(Message::Text(request.to_string())).await?;

        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                return Err(CdpError::Timeout(timeout));
            }
            let msg = match tokio::time::timeout(remaining, self.ws.next()).await {
                Ok(Some(m)) => m?,
                Ok(None) => return Err(CdpError::ConnectionClosed),
                Err(_) => return Err(CdpError::Timeout(timeout)),
            };

            let text = match msg {
                Message::Text(t) => t,
                Message::Binary(_) | Message::Ping(_) | Message::Pong(_) | Message::Frame(_) => {
                    continue
                }
                Message::Close(_) => return Err(CdpError::ConnectionClosed),
            };

            let value: Value = serde_json::from_str(&text)?;
            // Events have no `id` — skip them in this minimal client.
            match value.get("id").and_then(Value::as_u64) {
                Some(resp_id) if resp_id == id => {
                    if let Some(err) = value.get("error") {
                        return Err(CdpError::Remote(err.to_string()));
                    }
                    return value
                        .get("result")
                        .cloned()
                        .ok_or(CdpError::MalformedResponse("result"));
                }
                _ => continue,
            }
        }
    }

    /// `Browser.getVersion` — lightweight liveness probe.
    pub async fn get_version(&mut self, timeout: Duration) -> Result<BrowserVersion, CdpError> {
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
        &mut self,
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
            .ok_or(CdpError::MalformedResponse("data"))?;
        let bytes = <base64::engine::general_purpose::GeneralPurpose as base64::Engine>::decode(
            &base64::engine::general_purpose::STANDARD,
            data_b64,
        )?;
        Ok(bytes)
    }
}
