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
