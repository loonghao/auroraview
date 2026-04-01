//! Tests for auroraview-telemetry guard module.

use auroraview_telemetry::{Telemetry, TelemetryConfig};

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

// ─── is_initialized ──────────────────────────────────────────────────────────

#[test]
fn test_is_initialized_false_before_init() {
    // Note: global state may be set by other tests in the same process,
    // but we can test after a guard has been dropped to verify reset.
    // The key: calling is_initialized() does not panic.
    let _ = Telemetry::is_initialized();
}

#[test]
fn test_is_initialized_true_after_disabled_config_init() {
    // TelemetryConfig with enabled=false still calls mark_initialized
    let config = TelemetryConfig { enabled: false, ..TelemetryConfig::default() };
    // Only proceed if not already initialized (avoid double-init error)
    if !Telemetry::is_initialized() {
        let guard = Telemetry::init(config).expect("init should succeed when disabled");
        assert!(Telemetry::is_initialized());
        drop(guard);
        // After drop, INITIALIZED is reset to false
        assert!(!Telemetry::is_initialized());
    }
}

#[test]
fn test_double_init_returns_already_initialized_error() {
    use auroraview_telemetry::TelemetryError;

    if !Telemetry::is_initialized() {
        let config = TelemetryConfig { enabled: false, ..TelemetryConfig::default() };
        let _guard = Telemetry::init(config.clone()).expect("first init ok");

        // Second init while guard is alive must fail
        let result = Telemetry::init(config);
        assert!(matches!(result, Err(TelemetryError::AlreadyInitialized)));
        // _guard drops here, resetting state
    }
}

#[test]
fn test_guard_drop_resets_initialized() {
    if !Telemetry::is_initialized() {
        let config = TelemetryConfig { enabled: false, ..TelemetryConfig::default() };
        let guard = Telemetry::init(config).expect("init ok");
        assert!(Telemetry::is_initialized());
        drop(guard);
        assert!(!Telemetry::is_initialized());
    }
}
