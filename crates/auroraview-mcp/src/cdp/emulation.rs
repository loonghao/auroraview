//! Emulation.* CDP methods.
//!
//! This module provides emulation-related CDP methods:
//! - `set_device_metrics_override`

use std::time::Duration;

use serde_json::json;
use tracing::debug;

use super::CdpClient;
use crate::cdp::CdpError;

impl CdpClient {
    /// `Emulation.setDeviceMetricsOverride` — override device metrics.
    ///
    /// Simulates different screen sizes, pixel ratios, etc.
    /// Set all parameters to `0` or `None` to clear the override.
    ///
    /// # Errors
    ///
    /// Returns [`CdpError`] if the CDP call fails.
    #[tracing::instrument(skip(self, timeout), fields(timeout_ms = ?timeout.as_millis()))]
    pub async fn set_device_metrics_override(
        &self,
        width: i64,
        height: i64,
        device_scale_factor: f64,
        mobile: bool,
        timeout: Duration,
    ) -> Result<(), CdpError> {
        let params = json!({
            "width": width,
            "height": height,
            "deviceScaleFactor": device_scale_factor,
            "mobile": mobile,
        });
        self.call("Emulation.setDeviceMetricsOverride", params, timeout)
            .await?;
        debug!(%width, %height, %device_scale_factor, %mobile, "Emulation.setDeviceMetricsOverride succeeded");
        Ok(())
    }
}
