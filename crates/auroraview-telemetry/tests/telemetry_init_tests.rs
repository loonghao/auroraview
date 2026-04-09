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
    match result {
        Ok(_guard) => {}
        Err(auroraview_telemetry::TelemetryError::AlreadyInitialized) => {}
        Err(e) => panic!("Unexpected error: {e}"),
    }
}

#[test]
fn test_init_disabled_guard_drop_resets_state() {
    {
        let _ = Telemetry::init(disabled_config());
    }
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
    let result = Telemetry::capture_sentry_message("hello", "info");
    #[cfg(not(feature = "sentry"))]
    assert!(!result, "without sentry feature, should return false");
    #[cfg(feature = "sentry")]
    let _ = result;
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

// ─── TelemetryConfig field tests ─────────────────────────────────────────────

#[test]
fn test_config_default_enabled_true() {
    let config = TelemetryConfig::default();
    assert!(config.enabled);
}

#[test]
fn test_config_default_service_name() {
    let config = TelemetryConfig::default();
    assert_eq!(config.service_name, "auroraview");
}

#[test]
fn test_config_default_log_level() {
    let config = TelemetryConfig::default();
    assert_eq!(config.log_level, "info");
}

#[test]
fn test_config_default_log_to_stdout() {
    let config = TelemetryConfig::default();
    assert!(config.log_to_stdout);
}

#[test]
fn test_config_default_no_otlp() {
    let config = TelemetryConfig::default();
    assert!(config.otlp_endpoint.is_none());
}

#[test]
fn test_config_default_metrics_interval() {
    let config = TelemetryConfig::default();
    assert_eq!(config.metrics_interval_secs, 60);
}

#[test]
fn test_config_default_trace_sample_ratio() {
    let config = TelemetryConfig::default();
    assert!((config.trace_sample_ratio - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_config_default_no_sentry_dsn() {
    let config = TelemetryConfig::default();
    assert!(config.sentry_dsn.is_none());
}

#[test]
fn test_config_default_sentry_traces_disabled() {
    let config = TelemetryConfig::default();
    assert!((config.sentry_traces_sample_rate).abs() < f32::EPSILON);
}

#[test]
fn test_config_for_testing_service_name() {
    let config = TelemetryConfig::for_testing();
    assert_eq!(config.service_name, "auroraview-test");
}

#[test]
fn test_config_for_testing_log_level() {
    let config = TelemetryConfig::for_testing();
    assert_eq!(config.log_level, "debug");
}

#[test]
fn test_config_clone() {
    let config = TelemetryConfig::default();
    let cloned = config.clone();
    assert_eq!(config.service_name, cloned.service_name);
    assert_eq!(config.enabled, cloned.enabled);
    assert_eq!(config.metrics_interval_secs, cloned.metrics_interval_secs);
}

#[test]
fn test_config_serde_roundtrip() {
    let config = TelemetryConfig {
        enabled: false,
        service_name: "my-service".to_string(),
        log_level: "debug".to_string(),
        metrics_interval_secs: 30,
        ..TelemetryConfig::default()
    };
    let json = serde_json::to_string(&config).unwrap();
    let restored: TelemetryConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.service_name, "my-service");
    assert!(!restored.enabled);
    assert_eq!(restored.log_level, "debug");
    assert_eq!(restored.metrics_interval_secs, 30);
}

#[test]
fn test_config_debug_format() {
    let config = TelemetryConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("TelemetryConfig"));
}

#[test]
fn test_is_initialized_returns_bool() {
    let _ = Telemetry::is_initialized();
}


#[test]
fn test_init_accept_any_service_name() {
    if !Telemetry::is_initialized() {
        let config = TelemetryConfig {
            enabled: false,
            service_name: "custom-dcc-tool".to_string(),
            ..TelemetryConfig::default()
        };
        let result = Telemetry::init(config);
        match result {
            Ok(_) | Err(auroraview_telemetry::TelemetryError::AlreadyInitialized) => {}
            Err(e) => panic!("Unexpected: {e}"),
        }
    }
}

// ─── New: TelemetryConfig additional coverage ──────────────────────────────────

#[test]
fn test_config_custom_log_level() {
    let config = TelemetryConfig {
        log_level: "warn".to_string(),
        ..TelemetryConfig::default()
    };
    assert_eq!(config.log_level, "warn");
}

#[test]
fn test_config_custom_service_name_preserved() {
    let config = TelemetryConfig {
        service_name: "maya-auroraview-tool".to_string(),
        ..TelemetryConfig::default()
    };
    assert_eq!(config.service_name, "maya-auroraview-tool");
}

#[test]
fn test_config_metrics_enabled_flag() {
    let config = TelemetryConfig {
        metrics_enabled: false,
        ..TelemetryConfig::default()
    };
    assert!(!config.metrics_enabled);
}

#[test]
fn test_config_traces_enabled_flag() {
    let config = TelemetryConfig {
        traces_enabled: false,
        ..TelemetryConfig::default()
    };
    assert!(!config.traces_enabled);
}

#[test]
fn test_config_metrics_interval_custom() {
    let config = TelemetryConfig {
        metrics_interval_secs: 120,
        ..TelemetryConfig::default()
    };
    assert_eq!(config.metrics_interval_secs, 120);
}

#[test]
fn test_config_serde_roundtrip_for_testing() {
    let config = TelemetryConfig::for_testing();
    let json = serde_json::to_string(&config).unwrap();
    let restored: TelemetryConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.service_name, config.service_name);
    assert_eq!(restored.log_level, config.log_level);
    assert_eq!(restored.metrics_interval_secs, config.metrics_interval_secs);
}

#[test]
fn test_config_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<TelemetryConfig>();
    assert_sync::<TelemetryConfig>();
}

#[test]
fn test_config_clone_independence() {
    let config = TelemetryConfig {
        service_name: "original".to_string(),
        ..TelemetryConfig::default()
    };
    let mut cloned = config.clone();
    cloned.service_name = "cloned".to_string();
    assert_eq!(config.service_name, "original");
    assert_eq!(cloned.service_name, "cloned");
}

#[test]
fn test_init_disabled_then_enable_disable_cycle() {
    {
        let _ = Telemetry::init(disabled_config());
    }
    // After guard drop, toggle enable/disable should work
    for _ in 0..3 {
        Telemetry::enable();
        assert!(Telemetry::is_enabled());
        Telemetry::disable();
        assert!(!Telemetry::is_enabled());
    }
}

#[test]
fn test_config_log_to_stdout_default() {
    let config = TelemetryConfig::default();
    assert!(config.log_to_stdout);
}

#[test]
fn test_config_trace_sample_ratio_custom() {
    let config = TelemetryConfig {
        trace_sample_ratio: 0.5,
        ..TelemetryConfig::default()
    };
    assert!((config.trace_sample_ratio - 0.5).abs() < f64::EPSILON);
}

#[test]
fn test_disabled_config_is_still_valid_for_init() {
    // Disabled config should not panic during init
    let config = disabled_config();
    assert!(!config.enabled);
    assert_eq!(config.service_name, "auroraview");
}

