//! Page.* CDP methods.
//!
//! This module provides page-related CDP methods:
//! - `capture_screenshot`
//! - `navigate_to`
//! - `reload`
//! - `print_to_pdf`
//! - `set_download_behavior`

use std::time::Duration;

use serde_json::{json, Value};
use tracing::debug;

use super::CdpClient;
use crate::cdp::CdpError;

impl CdpClient {
    /// `Page.captureScreenshot` — returns raw image bytes.
    ///
    /// `format` is passed straight through (`"png"` / `"jpeg"` / `"webp"`).
    /// Callers are expected to pre-validate it.
    ///
    /// # Errors
    ///
    /// Returns [`CdpError`] if:
    /// - CDP call fails ([`CdpError::WebSocket`], [`CdpError::Timeout`], etc.)
    /// - Response is malformed ([`CdpError::MalformedResponse`])
    /// - Base64 decoding fails ([`CdpError::Base64`])
    #[tracing::instrument(skip(self, timeout), fields(%format, timeout_ms = ?timeout.as_millis()))]
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
        let data_b64 =
            result
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

    /// `Page.navigate` — navigate the `WebView` to a URL.
    ///
    /// # Errors
    ///
    /// Returns [`CdpError`] if the CDP call fails.
    #[tracing::instrument(skip(self, timeout), fields(%url, timeout_ms = ?timeout.as_millis()))]
    pub async fn navigate_to(&self, url: &str, timeout: Duration) -> Result<(), CdpError> {
        let params = json!({ "url": url });
        self.call("Page.navigate", params, timeout).await?;
        Ok(())
    }

    /// `Page.reload` — reload the current page.
    ///
    /// # Errors
    ///
    /// Returns [`CdpError`] if the CDP call fails.
    #[tracing::instrument(skip(self, timeout), fields(timeout_ms = ?timeout.as_millis()))]
    pub async fn reload(&self, timeout: Duration) -> Result<(), CdpError> {
        let params = json!({ "ignoreCache": false });
        self.call("Page.reload", params, timeout).await?;
        Ok(())
    }

    /// `Page.printToPDF` — generate a PDF of the current page.
    ///
    /// Returns the PDF as raw bytes (already decoded from base64).
    ///
    /// # Errors
    ///
    /// Returns [`CdpError`] if:
    /// - CDP call fails ([`CdpError::WebSocket`], [`CdpError::Timeout`], etc.)
    /// - Response is malformed ([`CdpError::MalformedResponse`])
    /// - Base64 decoding fails ([`CdpError::Base64`])
    #[tracing::instrument(skip(self, timeout), fields(timeout_ms = ?timeout.as_millis()))]
    pub async fn print_to_pdf(&self, timeout: Duration) -> Result<Vec<u8>, CdpError> {
        let params = json!({
            "printBackground": true,
            "preferCSSPageSize": true,
        });
        let result = self.call("Page.printToPDF", params, timeout).await?;
        let data_b64 = result.get("data").and_then(Value::as_str).ok_or_else(|| {
            tracing::warn!("Page.printToPDF response missing 'data' field");
            CdpError::MalformedResponse("Page.printToPDF".to_string(), "data")
        })?;
        let bytes = <base64::engine::general_purpose::GeneralPurpose as base64::Engine>::decode(
            &base64::engine::general_purpose::STANDARD,
            data_b64,
        )?;
        debug!(size = bytes.len(), "Page.printToPDF succeeded");
        Ok(bytes)
    }

    /// `Page.setDownloadBehavior` — control how downloads are handled.
    ///
    /// `behavior` can be:
    /// - `"deny"`: prevent downloads
    /// - `"allow"`: allow downloads (default)
    /// - `"default"`: use browser default
    ///
    /// `download_path` is required when `behavior` is `"allow"`.
    ///
    /// # Errors
    ///
    /// Returns [`CdpError`] if the CDP call fails.
    #[tracing::instrument(skip(self, timeout), fields(%behavior, timeout_ms = ?timeout.as_millis()))]
    pub async fn set_download_behavior(
        &self,
        behavior: &str,
        download_path: Option<&str>,
        timeout: Duration,
    ) -> Result<(), CdpError> {
        let mut params = json!({ "behavior": behavior });
        if let Some(path) = download_path {
            params["downloadPath"] = serde_json::json!(path);
        }
        self.call("Page.setDownloadBehavior", params, timeout)
            .await?;
        debug!(%behavior, ?download_path, "Page.setDownloadBehavior succeeded");
        Ok(())
    }
}
