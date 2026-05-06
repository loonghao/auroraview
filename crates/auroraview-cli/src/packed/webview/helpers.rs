//! Helper functions for WebView module
//!
//! This module contains utility functions for API registration,
//! timing conversions, and telemetry capture.

use std::collections::HashMap;

/// Regex pattern for valid handler names: alphanumeric, underscore, dot, colon, hyphen
/// This prevents injection attacks via malicious handler names
static VALID_HANDLER_PATTERN: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r"^[A-Za-z0-9_\.:\-]+$").expect("valid regex"));

/// Build JavaScript to register API methods in frontend
///
/// Groups handlers by namespace (e.g., "api.get_samples" -> namespace "api", method "get_samples")
/// and generates a script that calls `window.auroraview._registerApiMethods()` for each namespace.
///
/// Security: Uses serde_json for proper escaping and validates handler names against a whitelist pattern.
pub fn build_api_registration_script(handlers: &[String]) -> String {
    if handlers.is_empty() {
        return String::new();
    }

    // Group handlers by namespace, validating each handler name
    let mut namespaces: HashMap<String, Vec<String>> = HashMap::new();
    for handler in handlers {
        // Validate handler name against whitelist pattern
        if !VALID_HANDLER_PATTERN.is_match(handler) {
            tracing::warn!(
                "[Rust] Skipping invalid handler name (must match [A-Za-z0-9_.:-]+): {}",
                handler
            );
            continue;
        }

        if let Some(dot_pos) = handler.find('.') {
            let namespace = &handler[..dot_pos];
            let method = &handler[dot_pos + 1..];
            namespaces
                .entry(namespace.to_string())
                .or_default()
                .push(method.to_string());
        }
    }

    if namespaces.is_empty() {
        return String::new();
    }

    // Generate registration script using serde_json for safe escaping
    let mut script = String::from(
        "(function() {\n\
        if (!window.auroraview || !window.auroraview._registerApiMethods) {\n\
            console.warn('[AuroraView] Event bridge not ready for API registration');\n\
            return;\n\
        }\n",
    );

    for (namespace, methods) in &namespaces {
        // Use serde_json for proper JS string escaping (handles quotes, backslashes, unicode, etc.)
        let namespace_json =
            serde_json::to_string(namespace).unwrap_or_else(|_| "\"\"".to_string());
        let methods_json = serde_json::to_string(&methods).unwrap_or_else(|_| "[]".to_string());

        script.push_str(&format!(
            "window.auroraview._registerApiMethods({}, {});\n",
            namespace_json, methods_json
        ));
        tracing::debug!(
            "[Rust] Registering {} methods in namespace '{}'",
            methods.len(),
            namespace
        );
    }

    script.push_str("})()");
    script
}

/// Convert Option<Duration> to milliseconds as f64
pub fn duration_to_ms(duration: Option<std::time::Duration>) -> Option<f64> {
    duration.map(|d| d.as_secs_f64() * 1000.0)
}

/// Capture a sentry message for packed application telemetry
pub fn capture_packed_sentry(level: &str, message: &str) {
    let _ = auroraview_telemetry::Telemetry::capture_sentry_message(message, level);
}
