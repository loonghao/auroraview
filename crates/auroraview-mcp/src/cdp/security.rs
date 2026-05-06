//! Security.* CDP methods.
//!
//! This module provides security-related CDP methods:
//! - `set_ignore_certificate_errors`

use std::time::Duration;

use serde_json::json;
use tracing::debug;

use super::CdpClient;
use crate::cdp::CdpError;

impl CdpClient {
    /// `Security.setIgnoreCertificateErrors` — ignore SSL certificate errors.
    ///
    /// **WARNING**: This should only be used in development/testing.
    /// When `ignore` is `true`, all certificate errors are ignored.
    ///
    /// # Errors
    ///
    /// Returns [`CdpError`] if the CDP call fails.
    #[tracing::instrument(skip(self, timeout), fields(ignore, timeout_ms = ?timeout.as_millis()))]
    pub async fn set_ignore_certificate_errors(
        &self,
        ignore: bool,
        timeout: Duration,
    ) -> Result<(), CdpError> {
        let params = json!({ "ignore": ignore });
        self.call("Security.setIgnoreCertificateErrors", params, timeout)
            .await?;
        debug!(%ignore, "Security.setIgnoreCertificateErrors succeeded");
        Ok(())
    }
}
