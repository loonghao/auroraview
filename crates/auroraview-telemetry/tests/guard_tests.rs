//! Tests for auroraview-telemetry guard module.

use auroraview_telemetry::Telemetry;

#[test]
fn test_is_enabled_default() {
    // Before init, should be disabled
    assert!(!Telemetry::is_enabled());
}

#[test]
fn test_enable_disable() {
    Telemetry::enable();
    assert!(Telemetry::is_enabled());
    Telemetry::disable();
    assert!(!Telemetry::is_enabled());
}

#[test]
fn test_enable_twice() {
    Telemetry::enable();
    Telemetry::enable();
    assert!(Telemetry::is_enabled());
    Telemetry::disable();
}

#[test]
fn test_disable_twice() {
    Telemetry::disable();
    Telemetry::disable();
    assert!(!Telemetry::is_enabled());
}

#[test]
fn test_enable_disable_toggle_sequence() {
    // Start from known state
    Telemetry::disable();
    assert!(!Telemetry::is_enabled());

    Telemetry::enable();
    assert!(Telemetry::is_enabled());

    Telemetry::disable();
    assert!(!Telemetry::is_enabled());

    Telemetry::enable();
    assert!(Telemetry::is_enabled());

    // Clean up
    Telemetry::disable();
}

#[test]
fn test_disable_does_not_panic() {
    // Should be safe to call even without init
    Telemetry::disable();
}

#[test]
fn test_enable_does_not_panic() {
    Telemetry::enable();
    Telemetry::disable();
}

#[test]
fn test_sentry_capture_without_sentry_feature() {
    let result = Telemetry::capture_sentry_message("test", "info");
    // Without sentry feature, returns false; with feature it returns true
    #[cfg(feature = "sentry")]
    assert!(result);
    #[cfg(not(feature = "sentry"))]
    assert!(!result);
}

#[test]
fn test_sentry_capture_levels() {
    // All levels should not panic
    for level in &["fatal", "error", "warning", "warn", "info", "debug", "unknown"] {
        Telemetry::capture_sentry_message("test-msg", level);
    }
}
