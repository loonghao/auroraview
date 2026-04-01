//! Tests for optional Sentry integration in auroraview-telemetry.

use auroraview_telemetry::{Telemetry, TelemetryConfig};

#[test]
fn test_sentry_capture_without_init() {
    let captured = Telemetry::capture_sentry_message("test-message", "error");

    #[cfg(feature = "sentry")]
    assert!(captured);

    #[cfg(not(feature = "sentry"))]
    assert!(!captured);
}

#[test]
fn test_sentry_config_fields_roundtrip() {
    let config = TelemetryConfig {
        sentry_dsn: Some("https://public@example.com/1".to_string()),
        sentry_environment: Some("test".to_string()),
        sentry_release: Some("0.1.0".to_string()),
        sentry_sample_rate: 0.7,
        sentry_traces_sample_rate: 0.2,
        ..TelemetryConfig::default()
    };

    let json = serde_json::to_string(&config).unwrap();
    let restored: TelemetryConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(
        restored.sentry_dsn.as_deref(),
        Some("https://public@example.com/1")
    );
    assert_eq!(restored.sentry_environment.as_deref(), Some("test"));
    assert_eq!(restored.sentry_release.as_deref(), Some("0.1.0"));
    assert!((restored.sentry_sample_rate - 0.7).abs() < f32::EPSILON);
    assert!((restored.sentry_traces_sample_rate - 0.2).abs() < f32::EPSILON);
}

#[test]
fn test_sentry_config_production() {
    let config = TelemetryConfig {
        sentry_dsn: Some("https://key@sentry.io/12345".to_string()),
        sentry_environment: Some("production".to_string()),
        sentry_release: Some("auroraview@1.0.0".to_string()),
        sentry_sample_rate: 1.0,
        sentry_traces_sample_rate: 0.1,
        ..TelemetryConfig::default()
    };
    assert_eq!(config.sentry_environment.as_deref(), Some("production"));
    assert!((config.sentry_traces_sample_rate - 0.1).abs() < f32::EPSILON);
}

#[test]
fn test_sentry_config_staging() {
    let config = TelemetryConfig {
        sentry_dsn: Some("https://key@sentry.io/99".to_string()),
        sentry_environment: Some("staging".to_string()),
        sentry_sample_rate: 0.5,
        ..TelemetryConfig::default()
    };
    assert_eq!(config.sentry_environment.as_deref(), Some("staging"));
    assert!((config.sentry_sample_rate - 0.5).abs() < f32::EPSILON);
}

#[test]
fn test_sentry_no_dsn_default() {
    let config = TelemetryConfig::default();
    assert!(config.sentry_dsn.is_none());
    assert!(config.sentry_environment.is_none());
    assert!(config.sentry_release.is_none());
}

#[test]
fn test_sentry_capture_all_log_levels() {
    for level in &["fatal", "error", "warning", "warn", "info", "debug"] {
        let _result = Telemetry::capture_sentry_message("level test", level);
        // Should not panic regardless of feature flag
    }
}

#[test]
fn test_sentry_capture_unknown_level_defaults_to_info() {
    // "unknown" level should not panic - defaults to info
    let _result = Telemetry::capture_sentry_message("test msg", "notice");
}

#[test]
fn test_sentry_config_traces_sample_rate_disabled() {
    let config = TelemetryConfig {
        sentry_traces_sample_rate: 0.0,
        ..TelemetryConfig::default()
    };
    // 0.0 means disabled
    assert!((config.sentry_traces_sample_rate - 0.0).abs() < f32::EPSILON);
}

#[test]
fn test_sentry_config_full_sample_rates() {
    let config = TelemetryConfig {
        sentry_sample_rate: 1.0,
        sentry_traces_sample_rate: 1.0,
        ..TelemetryConfig::default()
    };
    assert!((config.sentry_sample_rate - 1.0).abs() < f32::EPSILON);
    assert!((config.sentry_traces_sample_rate - 1.0).abs() < f32::EPSILON);
}
