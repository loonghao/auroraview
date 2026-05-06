//! Browser.* CDP methods.
//!
//! This module provides methods for interacting with the Browser domain.

use serde_json::json;
use std::time::Duration;

use super::{BrowserVersion, CdpClient, CdpError};

impl CdpClient {
    /// `Browser.getVersion` — lightweight liveness probe.
    ///
    /// # Errors
    ///
    /// Returns [`CdpError`] if the CDP call fails (see [`Self::call_with_retry`] for details).
    #[tracing::instrument(skip(self, timeout), fields(timeout_ms = ?timeout.as_millis()))]
    pub async fn get_version(&self, timeout: Duration) -> Result<BrowserVersion, CdpError> {
        // Use retry logic for this idempotent probe
        let result = self
            .call_with_retry(
                "Browser.getVersion",
                json!({}),
                timeout,
                3,                          // max_retries
                Duration::from_millis(100), // initial_delay
                Duration::from_secs(5),     // max_delay
            )
            .await?;
        Ok(BrowserVersion {
            product: result
                .get("product")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_owned(),
            protocol_version: result
                .get("protocolVersion")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_owned(),
        })
    }
}
