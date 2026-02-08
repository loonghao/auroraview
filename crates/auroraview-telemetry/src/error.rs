//! Telemetry error types.

/// Errors that can occur during telemetry operations.
#[derive(Debug, thiserror::Error)]
pub enum TelemetryError {
    /// Failed to initialize the tracing subscriber.
    #[error("failed to initialize tracing subscriber: {0}")]
    TracingInit(String),

    /// Failed to initialize the metrics provider.
    #[error("failed to initialize metrics provider: {0}")]
    MetricsInit(String),

    /// Failed to initialize the trace provider.
    #[error("failed to initialize trace provider: {0}")]
    TraceInit(String),

    /// OTLP export configuration error.
    #[error("OTLP configuration error: {0}")]
    OtlpConfig(String),

    /// Telemetry already initialized.
    #[error("telemetry already initialized")]
    AlreadyInitialized,
}
