//! Span extension helpers for AuroraView-specific attributes.

use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

/// Extension trait for adding AuroraView-specific attributes to tracing spans.
pub trait SpanExt {
    /// Tag the span with a WebView ID.
    fn set_webview_id(&self, id: &str);

    /// Tag the span with an app name.
    fn set_app_name(&self, name: &str);

    /// Tag the span with an operation type.
    fn set_operation(&self, op: &str);

    /// Tag the span with an error.
    fn set_error(&self, err: &dyn std::error::Error);
}

impl SpanExt for Span {
    fn set_webview_id(&self, id: &str) {
        self.set_attribute("auroraview.webview_id", id.to_string());
    }

    fn set_app_name(&self, name: &str) {
        self.set_attribute("auroraview.app_name", name.to_string());
    }

    fn set_operation(&self, op: &str) {
        self.set_attribute("auroraview.operation", op.to_string());
    }

    fn set_error(&self, err: &dyn std::error::Error) {
        self.set_attribute("error", true);
        self.set_attribute("error.message", err.to_string());
    }
}
