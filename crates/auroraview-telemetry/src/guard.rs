//! Telemetry guard and global state.

use std::sync::atomic::{AtomicBool, Ordering};

use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;

static ENABLED: AtomicBool = AtomicBool::new(false);
static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Check if telemetry is currently enabled.
pub fn is_enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// Set the enabled state.
pub fn set_enabled(enabled: bool) {
    ENABLED.store(enabled, Ordering::Relaxed);
}

/// Check if telemetry has been initialized.
pub fn is_initialized() -> bool {
    INITIALIZED.load(Ordering::Relaxed)
}

/// Mark telemetry as initialized (called once during init).
pub(crate) fn mark_initialized() {
    INITIALIZED.store(true, Ordering::Relaxed);
}

/// Guard that shuts down telemetry providers when dropped.
///
/// Hold this for the lifetime of your application. When the guard
/// is dropped, all pending telemetry data is flushed and providers
/// are shut down gracefully.
pub struct TelemetryGuard {
    meter_provider: Option<SdkMeterProvider>,
    tracer_provider: Option<SdkTracerProvider>,
}

impl TelemetryGuard {
    pub(crate) fn new(
        meter_provider: Option<SdkMeterProvider>,
        tracer_provider: Option<SdkTracerProvider>,
    ) -> Self {
        ENABLED.store(true, Ordering::Relaxed);
        Self {
            meter_provider,
            tracer_provider,
        }
    }
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        tracing::debug!("Shutting down telemetry providers");
        ENABLED.store(false, Ordering::Relaxed);
        INITIALIZED.store(false, Ordering::Relaxed);

        if let Some(ref provider) = self.meter_provider {
            if let Err(e) = provider.shutdown() {
                eprintln!("Failed to shutdown meter provider: {e}");
            }
        }

        if let Some(ref provider) = self.tracer_provider {
            if let Err(e) = provider.shutdown() {
                eprintln!("Failed to shutdown tracer provider: {e}");
            }
        }
    }
}
