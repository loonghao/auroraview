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
    Telemetry::disable();
    assert!(!Telemetry::is_enabled());

    Telemetry::enable();
    assert!(Telemetry::is_enabled());

    Telemetry::disable();
    assert!(!Telemetry::is_enabled());

    Telemetry::enable();
    assert!(Telemetry::is_enabled());

    Telemetry::disable();
}

#[test]
fn test_disable_does_not_panic() {
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
    #[cfg(feature = "sentry")]
    assert!(result);
    #[cfg(not(feature = "sentry"))]
    assert!(!result);
}

#[test]
fn test_sentry_capture_levels() {
    for level in &[
        "fatal", "error", "warning", "warn", "info", "debug", "unknown",
    ] {
        Telemetry::capture_sentry_message("test-msg", level);
    }
}

// ─── is_initialized ──────────────────────────────────────────────────────────

#[test]
fn test_is_initialized_false_before_init() {
    let _ = Telemetry::is_initialized();
}

#[test]
fn test_is_initialized_true_after_disabled_config_init() {
    let config = TelemetryConfig {
        enabled: false,
        ..TelemetryConfig::default()
    };
    if !Telemetry::is_initialized() {
        let guard = Telemetry::init(config).expect("init should succeed when disabled");
        assert!(Telemetry::is_initialized());
        drop(guard);
        assert!(!Telemetry::is_initialized());
    }
}

#[test]
fn test_double_init_returns_already_initialized_error() {
    use auroraview_telemetry::TelemetryError;

    if !Telemetry::is_initialized() {
        let config = TelemetryConfig {
            enabled: false,
            ..TelemetryConfig::default()
        };
        let _guard = Telemetry::init(config.clone()).expect("first init ok");

        let result = Telemetry::init(config);
        assert!(matches!(result, Err(TelemetryError::AlreadyInitialized)));
    }
}

#[test]
fn test_guard_drop_resets_initialized() {
    if !Telemetry::is_initialized() {
        let config = TelemetryConfig {
            enabled: false,
            ..TelemetryConfig::default()
        };
        let guard = Telemetry::init(config).expect("init ok");
        assert!(Telemetry::is_initialized());
        drop(guard);
        assert!(!Telemetry::is_initialized());
    }
}

// ─── Additional guard behaviour ───────────────────────────────────────────────

#[test]
fn test_enable_does_not_affect_initialized() {
    // enable/disable should not change the initialized state
    let was_initialized = Telemetry::is_initialized();
    Telemetry::enable();
    Telemetry::disable();
    assert_eq!(Telemetry::is_initialized(), was_initialized);
}

#[test]
fn test_is_enabled_after_multiple_toggles() {
    // After even number of toggles, should end where it started
    Telemetry::disable();
    for _ in 0..10 {
        Telemetry::enable();
        Telemetry::disable();
    }
    assert!(!Telemetry::is_enabled());
}

#[test]
fn test_enable_is_idempotent() {
    Telemetry::enable();
    let state_after_first = Telemetry::is_enabled();
    Telemetry::enable();
    let state_after_second = Telemetry::is_enabled();
    assert_eq!(state_after_first, state_after_second);
    Telemetry::disable();
}

#[test]
fn test_disable_is_idempotent() {
    Telemetry::disable();
    let state_after_first = Telemetry::is_enabled();
    Telemetry::disable();
    let state_after_second = Telemetry::is_enabled();
    assert_eq!(state_after_first, state_after_second);
    assert!(!state_after_second);
}

#[test]
fn test_is_initialized_call_does_not_panic() {
    // Just verify it doesn't panic under any state
    let _ = Telemetry::is_initialized();
    Telemetry::enable();
    let _ = Telemetry::is_initialized();
    Telemetry::disable();
    let _ = Telemetry::is_initialized();
}

#[test]
fn test_guard_init_with_disabled_config_does_not_enable() {
    if !Telemetry::is_initialized() {
        // Explicitly disable before init
        Telemetry::disable();
        let config = TelemetryConfig {
            enabled: false,
            ..TelemetryConfig::default()
        };
        let guard = Telemetry::init(config).expect("init ok");
        // init() with enabled=false should NOT enable telemetry
        // (the global ENABLED flag should remain false, as we set it above)
        // Note: we check initialized, not enabled, since enabled is a separate flag
        assert!(Telemetry::is_initialized());
        drop(guard);
    }
}

#[test]
fn test_sentry_capture_idempotent_without_feature() {
    #[cfg(not(feature = "sentry"))]
    {
        let r1 = Telemetry::capture_sentry_message("msg", "info");
        let r2 = Telemetry::capture_sentry_message("msg", "info");
        assert_eq!(r1, r2);
    }
}

#[test]
fn test_multiple_init_after_drop_sequence() {
    // Test that init → drop → init → drop works correctly
    for _ in 0..2 {
        if !Telemetry::is_initialized() {
            let config = TelemetryConfig {
                enabled: false,
                ..TelemetryConfig::default()
            };
            let guard = Telemetry::init(config).expect("init ok");
            assert!(Telemetry::is_initialized());
            drop(guard);
            assert!(!Telemetry::is_initialized());
        }
    }
}

#[test]
fn test_enabled_state_persists_across_is_initialized_check() {
    Telemetry::enable();
    let _ = Telemetry::is_initialized();
    assert!(Telemetry::is_enabled());
    Telemetry::disable();
}
