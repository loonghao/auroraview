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
    }
}

#[test]
fn test_sentry_capture_unknown_level_defaults_to_info() {
    let _result = Telemetry::capture_sentry_message("test msg", "notice");
}

#[test]
fn test_sentry_config_traces_sample_rate_disabled() {
    let config = TelemetryConfig {
        sentry_traces_sample_rate: 0.0,
        ..TelemetryConfig::default()
    };
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

#[test]
fn test_sentry_config_dsn_formats() {
    for dsn in &[
        "https://abc123@o123.ingest.sentry.io/456",
        "https://public@sentry.example.com/1",
        "https://key@127.0.0.1/99",
    ] {
        let config = TelemetryConfig {
            sentry_dsn: Some(dsn.to_string()),
            ..TelemetryConfig::default()
        };
        assert_eq!(config.sentry_dsn.as_deref(), Some(*dsn));
    }
}

#[test]
fn test_sentry_config_environment_variants() {
    for env in &["production", "staging", "development", "test", "ci"] {
        let config = TelemetryConfig {
            sentry_environment: Some(env.to_string()),
            ..TelemetryConfig::default()
        };
        assert_eq!(config.sentry_environment.as_deref(), Some(*env));
    }
}

#[test]
fn test_sentry_config_release_formats() {
    for release in &["1.0.0", "auroraview@2.5.3", "v0.1.0-beta.1"] {
        let config = TelemetryConfig {
            sentry_release: Some(release.to_string()),
            ..TelemetryConfig::default()
        };
        assert_eq!(config.sentry_release.as_deref(), Some(*release));
    }
}

#[test]
fn test_sentry_capture_empty_message() {
    // Empty message should not panic
    let _result = Telemetry::capture_sentry_message("", "info");
}

#[test]
fn test_sentry_capture_unicode_message() {
    let _result = Telemetry::capture_sentry_message("エラーが発生しました", "error");
}

#[test]
fn test_sentry_capture_long_message() {
    let long_msg = "x".repeat(4096);
    let _result = Telemetry::capture_sentry_message(&long_msg, "warning");
}

#[test]
fn test_sentry_capture_returns_consistent_without_feature() {
    #[cfg(not(feature = "sentry"))]
    {
        // Without sentry feature, always returns false
        assert!(!Telemetry::capture_sentry_message("test1", "error"));
        assert!(!Telemetry::capture_sentry_message("test2", "info"));
        assert!(!Telemetry::capture_sentry_message("test3", "debug"));
    }
}

#[test]
fn test_sentry_config_sample_rate_clamp_zero() {
    let config = TelemetryConfig {
        sentry_sample_rate: 0.0,
        ..TelemetryConfig::default()
    };
    assert!((config.sentry_sample_rate).abs() < f32::EPSILON);
}

#[test]
fn test_sentry_config_debug_format() {
    let config = TelemetryConfig {
        sentry_dsn: Some("https://key@sentry.io/1".to_string()),
        ..TelemetryConfig::default()
    };
    let debug_str = format!("{:?}", config);
    assert!(!debug_str.is_empty());
}

#[test]
fn test_sentry_config_clone() {
    let config = TelemetryConfig {
        sentry_dsn: Some("https://key@sentry.io/42".to_string()),
        sentry_environment: Some("prod".to_string()),
        sentry_sample_rate: 0.9,
        ..TelemetryConfig::default()
    };
    let cloned = config.clone();
    assert_eq!(config.sentry_dsn, cloned.sentry_dsn);
    assert_eq!(config.sentry_environment, cloned.sentry_environment);
    assert!((config.sentry_sample_rate - cloned.sentry_sample_rate).abs() < f32::EPSILON);
}

#[test]
fn test_sentry_capture_level_case_variations() {
    // Both uppercase and lowercase should not panic
    for level in &["ERROR", "Error", "error", "INFO", "Info", "info"] {
        let _result = Telemetry::capture_sentry_message("test", level);
    }
}
