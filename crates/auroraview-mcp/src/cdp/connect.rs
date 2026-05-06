//! CDP connection and request/response handling.
//!
//! This module provides `connect()`, `call()`, and `call_with_retry()`
//! for the `CdpClient`.

use std::sync::Arc;
use std::time::Duration;

use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use serde_json::{json, Value};
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{debug, error, warn};

use super::{CdpClient, CdpClientInner, CdpError};

impl CdpClient {
    /// Connect to a CDP endpoint on `http://host:port`.
    ///
    /// Performs `GET /json/version` to discover the browser-level WebSocket
    /// debugger URL, then opens a WebSocket to it.
    ///
    /// # Errors
    ///
    /// Returns [`CdpError`] if:
    /// - HTTP request to `/json/version` fails ([`CdpError::Http`])
    /// - WebSocket handshake fails ([`CdpError::WebSocket`])
    /// - JSON response is malformed ([`CdpError::Json`])
    #[tracing::instrument(fields(%http_endpoint))]
    pub async fn connect(http_endpoint: &str) -> Result<Self, CdpError> {
        let url = format!("{}/json/version", http_endpoint.trim_end_matches('/'));
        let info: super::VersionInfo = reqwest::get(&url).await?.error_for_status()?.json().await?;
        debug!(
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

    /// Send a `CDP` command and wait for its matching response.
    ///
    /// Any events received while waiting are dropped — the skeleton adapter
    /// is request/response only.
    ///
    /// # Errors
    ///
    /// Returns [`CdpError`] if:
    /// - WebSocket send fails ([`CdpError::WebSocket`])
    /// - Connection is closed before response ([`CdpError::ConnectionClosed`])
    /// - Response times out ([`CdpError::Timeout`])
    /// - JSON parsing fails ([`CdpError::Json`])
    /// - CDP returns an error ([`CdpError::Remote`])
    /// - Response is malformed ([`CdpError::MalformedResponse`])
    #[tracing::instrument(skip(self, params), fields(method = %method))]
    pub async fn call(
        &self,
        method: &str,
        params: Value,
        timeout: Duration,
    ) -> Result<Value, CdpError> {
        let mut inner = self.inner.lock().await;

        let id = inner.next_id;
        inner.next_id += 1;

        debug!(%method, %id, "CDP call");

        let request = json!({
            "id": id,
            "method": method,
            "params": params,
        });
        inner
            .ws
            .send(Message::Text(request.to_string()))
            .await
            .map_err(|e| {
                warn!(%method, error = %e, "CDP send failed");
                CdpError::WebSocket(e)
            })?;

        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                warn!(%method, ?timeout, "CDP call timed out");
                return Err(CdpError::Timeout(method.to_string(), timeout));
            }
            let msg = match tokio::time::timeout(remaining, inner.ws.next()).await {
                Ok(Some(m)) => m.map_err(|e| {
                    warn!(%method, error = %e, "CDP WebSocket error");
                    CdpError::WebSocket(e)
                })?,
                Ok(None) => {
                    warn!(%method, "CDP connection closed");
                    return Err(CdpError::ConnectionClosed(method.to_string()));
                }
                Err(_) => {
                    warn!(%method, ?timeout, "CDP timeout waiting for response");
                    return Err(CdpError::Timeout(method.to_string(), timeout));
                }
            };

            let text = match msg {
                Message::Text(t) => t,
                #[allow(clippy::needless_continue)]
                Message::Binary(_) | Message::Ping(_) | Message::Pong(_) | Message::Frame(_) => {
                    continue
                }
                Message::Close(_) => {
                    warn!(%method, "CDP connection closed by peer");
                    return Err(CdpError::ConnectionClosed(method.to_string()));
                }
            };

            let value: Value = serde_json::from_str(&text).map_err(|e| {
                warn!(%method, error = %e, "CDP JSON parse failed");
                CdpError::Json(e)
            })?;

            // Events have no `id` — skip them in this minimal client.
            match value.get("id").and_then(Value::as_u64) {
                Some(resp_id) if resp_id == id => {
                    if let Some(err) = value.get("error") {
                        // Log the full error JSON for debugging
                        let error_json = serde_json::to_string(err).unwrap_or_default();
                        warn!(
                            %method,
                            error = %error_json,
                            "CDP returned error"
                        );
                        return Err(CdpError::Remote(method.to_string(), error_json));
                    }
                    let result = value.get("result").cloned().ok_or_else(|| {
                        warn!(%method, "CDP response missing 'result' field");
                        CdpError::MalformedResponse(method.to_string(), "result")
                    })?;
                    debug!(%method, %id, "CDP call succeeded");
                    return Ok(result);
                }
                #[allow(clippy::needless_continue)]
                _ => continue,
            }
        }
    }

    /// Send a CDP command with retry logic and exponential backoff.
    ///
    /// Retries up to `max_retries` times, waiting `initial_delay * 2^attempt`
    /// between retries (capped at `max_delay`).
    ///
    /// # Errors
    ///
    /// Returns [`CdpError`] if all retries are exhausted.
    /// The error is the last [`CdpError`] encountered.
    #[tracing::instrument(skip(self, params), fields(method = %method, max_retries = %max_retries))]
    pub async fn call_with_retry(
        &self,
        method: &str,
        params: Value,
        timeout: Duration,
        max_retries: u32,
        initial_delay: Duration,
        max_delay: Duration,
    ) -> Result<Value, CdpError> {
        let mut attempt = 0;
        let mut delay = initial_delay;

        loop {
            match self.call(method, params.clone(), timeout).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempt += 1;
                    if attempt > max_retries {
                        error!(
                            %method,
                            error = %e,
                            attempts = attempt,
                            "CDP call failed after all retries"
                        );
                        return Err(e);
                    }

                    warn!(
                        %method,
                        error = %e,
                        attempt,
                        max_retries,
                        ?delay,
                        "CDP call failed, retrying..."
                    );

                    tokio::time::sleep(delay).await;

                    // Exponential backoff with cap
                    delay = std::cmp::min(delay * 2, max_delay);
                }
            }
        }
    }
}
