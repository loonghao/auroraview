//! Tests for Telemetry::init and lifecycle (disabled-config path).
//!
//! Note: `Telemetry::init` uses process-global state (INITIALIZED AtomicBool),
//! so we only test paths that don't conflict with each other. The `enabled=false`
//! path is safe to test inline because it still marks initialized=true then drops
//! the guard which resets to false.

use auroraview_telemetry::{Telemetry, TelemetryConfig};

/// Helper: build a disabled config to avoid touching the global subscriber.
fn disabled_config() -> TelemetryConfig {
    TelemetryConfig {
        enabled: false,
        ..TelemetryConfig::default()
    }
}

#[test]
fn test_init_disabled_config_ok() {
    let result = Telemetry::init(disabled_config());
    // May fail if another test already holds an initialized guard, so accept both
    match result {
        Ok(_guard) => {
            // Guard drop resets state
        }
        Err(auroraview_telemetry::TelemetryError::AlreadyInitialized) => {
            // Another test in the process initialized first - acceptable
        }
        Err(e) => panic!("Unexpected error: {e}"),
    }
}

#[test]
fn test_init_disabled_guard_drop_resets_state() {
    // Drop the guard and verify enable/disable still work
    {
        let _ = Telemetry::init(disabled_config());
    }
    // After drop, we should be able to call these without panic
    Telemetry::enable();
    assert!(Telemetry::is_enabled());
    Telemetry::disable();
    assert!(!Telemetry::is_enabled());
}

#[test]
fn test_telemetry_is_enabled_after_manual_enable() {
    Telemetry::enable();
    assert!(Telemetry::is_enabled());
    Telemetry::disable();
}

#[test]
fn test_telemetry_is_disabled_after_manual_disable() {
    Telemetry::enable();
    Telemetry::disable();
    assert!(!Telemetry::is_enabled());
}

#[test]
fn test_disabled_config_fields() {
    let config = disabled_config();
    assert!(!config.enabled);
    assert_eq!(config.service_name, "auroraview");
}

#[test]
fn test_telemetry_capture_sentry_message_returns_expected() {
    // Returns false (no sentry feature) or true (with sentry feature)
    let result = Telemetry::capture_sentry_message("hello", "info");
    #[cfg(not(feature = "sentry"))]
    assert!(!result, "without sentry feature, should return false");
    #[cfg(feature = "sentry")]
    let _ = result; // with sentry feature, either value is acceptable here
}

#[test]
fn test_already_initialized_error_display() {
    use auroraview_telemetry::TelemetryError;
    let err = TelemetryError::AlreadyInitialized;
    assert!(!err.to_string().is_empty());
}

#[test]
fn test_config_for_testing_metrics_interval() {
    let config = TelemetryConfig::for_testing();
    assert_eq!(config.metrics_interval_secs, 5);
    assert!(config.metrics_enabled);
    assert!(config.traces_enabled);
}

#[test]
fn test_config_no_traces_no_metrics() {
    let config = TelemetryConfig {
        enabled: false,
        metrics_enabled: false,
        traces_enabled: false,
        ..TelemetryConfig::default()
    };
    let result = Telemetry::init(config);
    match result {
        Ok(_) | Err(auroraview_telemetry::TelemetryError::AlreadyInitialized) => {}
        Err(e) => panic!("Unexpected error: {e}"),
    }
}
