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
