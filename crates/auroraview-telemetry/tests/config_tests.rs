//! Tests for auroraview-telemetry config module.

use auroraview_telemetry::TelemetryConfig;

#[test]
fn test_default_config() {
    let config = TelemetryConfig::default();
    assert!(config.enabled);
    assert_eq!(config.service_name, "auroraview");
    assert!(config.log_to_stdout);
    assert!(!config.log_json);
    assert!(config.otlp_endpoint.is_none());
    assert!(config.metrics_enabled);
    assert!(config.traces_enabled);
    assert!((config.trace_sample_ratio - 1.0).abs() < f64::EPSILON);
    assert_eq!(config.metrics_interval_secs, 60);
    assert!(config.sentry_dsn.is_none());
    assert!(config.sentry_environment.is_none());
    assert!(config.sentry_release.is_none());
    assert!((config.sentry_sample_rate - 1.0).abs() < f32::EPSILON);
    assert!((config.sentry_traces_sample_rate - 0.0).abs() < f32::EPSILON);
}

#[test]
fn test_testing_config() {
    let config = TelemetryConfig::for_testing();
    assert!(config.enabled);
    assert_eq!(config.service_name, "auroraview-test");
    assert_eq!(config.log_level, "debug");
    assert_eq!(config.metrics_interval_secs, 5);
}

#[cfg(feature = "otlp")]
#[test]
fn test_otlp_config() {
    let config = TelemetryConfig::with_otlp("http://localhost:4317");
    assert_eq!(
        config.otlp_endpoint,
        Some("http://localhost:4317".to_string())
    );
    assert!(config.enabled);
}

#[test]
fn test_config_serde() {
    let config = TelemetryConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: TelemetryConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.service_name, config.service_name);
    assert_eq!(deserialized.enabled, config.enabled);
    assert_eq!(deserialized.log_level, config.log_level);
}

#[test]
fn test_config_clone() {
    let config = TelemetryConfig::default();
    let cloned = config.clone();
    assert_eq!(cloned.service_name, config.service_name);
    assert_eq!(cloned.enabled, config.enabled);
    assert_eq!(cloned.log_level, config.log_level);
    assert_eq!(cloned.metrics_enabled, config.metrics_enabled);
    assert_eq!(cloned.traces_enabled, config.traces_enabled);
}

#[test]
fn test_config_disabled() {
    let config = TelemetryConfig {
        enabled: false,
        ..TelemetryConfig::default()
    };
    assert!(!config.enabled);
    assert_eq!(config.service_name, "auroraview");
}

#[test]
fn test_config_metrics_disabled() {
    let config = TelemetryConfig {
        metrics_enabled: false,
        ..TelemetryConfig::default()
    };
    assert!(!config.metrics_enabled);
    assert!(config.traces_enabled);
}

#[test]
fn test_config_traces_disabled() {
    let config = TelemetryConfig {
        traces_enabled: false,
        ..TelemetryConfig::default()
    };
    assert!(!config.traces_enabled);
    assert!(config.metrics_enabled);
}

#[test]
fn test_config_both_disabled() {
    let config = TelemetryConfig {
        metrics_enabled: false,
        traces_enabled: false,
        ..TelemetryConfig::default()
    };
    assert!(!config.metrics_enabled);
    assert!(!config.traces_enabled);
}

#[test]
fn test_config_log_json() {
    let config = TelemetryConfig {
        log_json: true,
        ..TelemetryConfig::default()
    };
    assert!(config.log_json);
}

#[test]
fn test_config_no_stdout() {
    let config = TelemetryConfig {
        log_to_stdout: false,
        ..TelemetryConfig::default()
    };
    assert!(!config.log_to_stdout);
}

#[test]
fn test_config_custom_service_name() {
    let config = TelemetryConfig {
        service_name: "my-dcc-tool".to_string(),
        service_version: "2.0.0".to_string(),
        ..TelemetryConfig::default()
    };
    assert_eq!(config.service_name, "my-dcc-tool");
    assert_eq!(config.service_version, "2.0.0");
}

#[test]
fn test_config_custom_log_level() {
    let config = TelemetryConfig {
        log_level: "auroraview=debug,warn".to_string(),
        ..TelemetryConfig::default()
    };
    assert_eq!(config.log_level, "auroraview=debug,warn");
}

#[test]
fn test_config_trace_sample_ratio_zero() {
    let config = TelemetryConfig {
        trace_sample_ratio: 0.0,
        ..TelemetryConfig::default()
    };
    assert!((config.trace_sample_ratio - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_config_trace_sample_ratio_half() {
    let config = TelemetryConfig {
        trace_sample_ratio: 0.5,
        ..TelemetryConfig::default()
    };
    assert!((config.trace_sample_ratio - 0.5).abs() < f64::EPSILON);
}

#[test]
fn test_config_metrics_interval_custom() {
    let config = TelemetryConfig {
        metrics_interval_secs: 30,
        ..TelemetryConfig::default()
    };
    assert_eq!(config.metrics_interval_secs, 30);
}

#[test]
fn test_config_otlp_endpoint() {
    let config = TelemetryConfig {
        otlp_endpoint: Some("http://otlp.example.com:4317".to_string()),
        ..TelemetryConfig::default()
    };
    assert_eq!(
        config.otlp_endpoint.as_deref(),
        Some("http://otlp.example.com:4317")
    );
}

#[test]
fn test_config_serde_roundtrip_all_fields() {
    let original = TelemetryConfig {
        enabled: false,
        service_name: "test-svc".to_string(),
        service_version: "3.1.4".to_string(),
        log_level: "debug".to_string(),
        log_to_stdout: false,
        log_json: true,
        otlp_endpoint: Some("http://localhost:4317".to_string()),
        metrics_enabled: false,
        metrics_interval_secs: 10,
        traces_enabled: false,
        trace_sample_ratio: 0.25,
        sentry_dsn: Some("https://key@sentry.io/123".to_string()),
        sentry_environment: Some("staging".to_string()),
        sentry_release: Some("v1.2.3".to_string()),
        sentry_sample_rate: 0.5,
        sentry_traces_sample_rate: 0.1,
    };
    let json = serde_json::to_string(&original).unwrap();
    let restored: TelemetryConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.enabled, original.enabled);
    assert_eq!(restored.service_name, original.service_name);
    assert_eq!(restored.service_version, original.service_version);
    assert_eq!(restored.log_level, original.log_level);
    assert_eq!(restored.log_to_stdout, original.log_to_stdout);
    assert_eq!(restored.log_json, original.log_json);
    assert_eq!(restored.otlp_endpoint, original.otlp_endpoint);
    assert_eq!(restored.metrics_enabled, original.metrics_enabled);
    assert_eq!(
        restored.metrics_interval_secs,
        original.metrics_interval_secs
    );
    assert_eq!(restored.traces_enabled, original.traces_enabled);
    assert!((restored.trace_sample_ratio - original.trace_sample_ratio).abs() < f64::EPSILON);
    assert_eq!(restored.sentry_dsn, original.sentry_dsn);
    assert_eq!(restored.sentry_environment, original.sentry_environment);
    assert_eq!(restored.sentry_release, original.sentry_release);
    assert!((restored.sentry_sample_rate - original.sentry_sample_rate).abs() < f32::EPSILON);
    assert!(
        (restored.sentry_traces_sample_rate - original.sentry_traces_sample_rate).abs()
            < f32::EPSILON
    );
}

#[test]
fn test_config_debug_format() {
    let config = TelemetryConfig::default();
    let debug_str = format!("{config:?}");
    assert!(debug_str.contains("TelemetryConfig"));
    assert!(debug_str.contains("enabled"));
}

#[test]
fn test_testing_config_no_otlp() {
    let config = TelemetryConfig::for_testing();
    assert!(config.otlp_endpoint.is_none());
}

#[test]
fn test_testing_config_sentry_disabled() {
    let config = TelemetryConfig::for_testing();
    assert!(config.sentry_dsn.is_none());
}
