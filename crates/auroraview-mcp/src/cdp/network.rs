//! Network.* CDP methods.
//!
//! This module provides network-related CDP methods:
//! - `network_enable`
//! - `network_disable`
//! - `get_response_body`
//! - `clear_browser_cache`
//! - `set_cache_disabled`

use std::time::Duration;

use serde_json::{json, Value};
use tracing::debug;

use super::CdpClient;
use crate::cdp::CdpError;

impl CdpClient {
    /// `Network.enable` — enable network monitoring.
    ///
    /// Call this before using `Network.*` events.
    ///
    /// # Errors
    ///
    /// Returns [`CdpError`] if the CDP call fails.
    pub async fn network_enable(&self, timeout: Duration) -> Result<(), CdpError> {
        self.call("Network.enable", json!({}), timeout).await?;
        debug!("Network monitoring enabled");
        Ok(())
    }

    /// `Network.disable` — disable network monitoring.
    ///
    /// # Errors
    ///
    /// Returns [`CdpError`] if the CDP call fails.
    pub async fn network_disable(&self, timeout: Duration) -> Result<(), CdpError> {
        self.call("Network.disable", json!({}), timeout).await?;
        debug!("Network monitoring disabled");
        Ok(())
    }

    /// `Network.getResponseBody` — get the response body for a network request.
    ///
    /// `request_id` is the network request ID (from `Network.requestWillBeSent` event).
    /// Returns the response body as bytes (handles base64-encoded bodies).
    ///
    /// # Errors
    ///
    /// Returns [`CdpError`] if:
    /// - CDP call fails ([`CdpError::WebSocket`], [`CdpError::Timeout`], etc.)
    /// - Response is malformed ([`CdpError::MalformedResponse`])
    /// - Base64 decoding fails ([`CdpError::Base64`])
    pub async fn get_response_body(
        &self,
        request_id: &str,
        timeout: Duration,
    ) -> Result<Vec<u8>, CdpError> {
        let params = json!({"requestId": request_id});
        // Use retry logic for this idempotent method
        let result = self
            .call_with_retry(
                "Network.getResponseBody",
                params,
                timeout,
                3,
                Duration::from_millis(100),
                Duration::from_secs(5),
            )
            .await?;
        let body = result.get("body").and_then(Value::as_str).ok_or_else(|| {
            tracing::warn!(
                ?request_id,
                "Network.getResponseBody response missing 'body' field"
            );
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
        tracing::debug!(
            ?request_id,
            size = bytes.len(),
            "Network.getResponseBody succeeded"
        );
        Ok(bytes)
    }

    /// `Network.clearBrowserCache` — clear the browser cache.
    ///
    /// # Errors
    ///
    /// Returns [`CdpError`] if the CDP call fails.
    pub async fn clear_browser_cache(&self, timeout: Duration) -> Result<(), CdpError> {
        self.call("Network.clearBrowserCache", json!({}), timeout)
            .await?;
        tracing::debug!("Network.clearBrowserCache succeeded");
        Ok(())
    }

    /// `Network.setCacheDisabled` — disable or enable browser cache.
    ///
    /// When `disabled` is `true`, the browser will not use the cache.
    /// When `false`, normal cache behavior is restored.
    ///
    /// # Errors
    ///
    /// Returns [`CdpError`] if the CDP call fails.
    pub async fn set_cache_disabled(
        &self,
        disabled: bool,
        timeout: Duration,
    ) -> Result<(), CdpError> {
        let params = json!({ "cacheDisabled": disabled });
        self.call("Network.setCacheDisabled", params, timeout)
            .await?;
        tracing::debug!(%disabled, "Network.setCacheDisabled succeeded");
        Ok(())
    }
}
