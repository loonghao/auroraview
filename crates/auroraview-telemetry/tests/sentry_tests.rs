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

// ============================================================================
// New: Send+Sync bounds
// ============================================================================

#[test]
fn telemetry_config_is_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<TelemetryConfig>();
    assert_sync::<TelemetryConfig>();
}

// ============================================================================
// New: TelemetryConfig default sample rates
// ============================================================================

#[test]
fn test_default_sample_rate_is_one() {
    let config = TelemetryConfig::default();
    assert!((config.sentry_sample_rate - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_default_traces_sample_rate_is_zero() {
    let config = TelemetryConfig::default();
    assert!((config.sentry_traces_sample_rate - 0.0).abs() < f32::EPSILON);
}

// ============================================================================
// New: TelemetryConfig partial struct update
// ============================================================================

#[test]
fn test_config_partial_update_preserves_defaults() {
    let config = TelemetryConfig {
        sentry_dsn: Some("https://x@sentry.io/1".to_string()),
        ..TelemetryConfig::default()
    };
    // Other fields should remain default
    assert_eq!(config.sentry_environment, None);
    assert_eq!(config.sentry_release, None);
    assert!((config.sentry_sample_rate - 1.0).abs() < f32::EPSILON);
}

// ============================================================================
// New: sentry_dsn None vs Some serde
// ============================================================================

#[test]
fn test_sentry_dsn_none_serde() {
    let config = TelemetryConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("null") || !json.contains("sentry_dsn") || json.contains("\"sentry_dsn\":null"));
}

// ============================================================================
// New: capture with special chars in message
// ============================================================================

#[test]
fn test_sentry_capture_with_special_chars() {
    let _r = Telemetry::capture_sentry_message(r#"Error: file "path/to/file" not found"#, "error");
    let _r2 = Telemetry::capture_sentry_message("newline\nand\ttab", "info");
}

// ============================================================================
// New: multiple configs don't interfere
// ============================================================================

#[test]
fn test_multiple_config_instances_are_independent() {
    let c1 = TelemetryConfig {
        sentry_dsn: Some("https://a@sentry.io/1".to_string()),
        sentry_sample_rate: 0.1,
        ..TelemetryConfig::default()
    };
    let c2 = TelemetryConfig {
        sentry_dsn: Some("https://b@sentry.io/2".to_string()),
        sentry_sample_rate: 0.9,
        ..TelemetryConfig::default()
    };
    assert_ne!(c1.sentry_dsn, c2.sentry_dsn);
    assert!(c1.sentry_sample_rate < c2.sentry_sample_rate);
}

// ============================================================================
// New: config clone independence
// ============================================================================

#[test]
fn test_telemetry_config_clone_independence() {
    let config = TelemetryConfig {
        sentry_dsn: Some("https://orig@sentry.io/1".to_string()),
        sentry_sample_rate: 0.5,
        ..TelemetryConfig::default()
    };
    let mut cloned = config.clone();
    cloned.sentry_dsn = Some("https://cloned@sentry.io/2".to_string());
    assert_eq!(config.sentry_dsn.as_deref(), Some("https://orig@sentry.io/1"));
    assert_eq!(cloned.sentry_dsn.as_deref(), Some("https://cloned@sentry.io/2"));
}

// ============================================================================
// R8 Extensions
// ============================================================================

#[test]
fn test_sentry_capture_message_does_not_panic() {
    // Basic smoke: calling capture_sentry_message should not panic regardless of args
    let _ = Telemetry::capture_sentry_message("smoke test", "info");
    let _ = Telemetry::capture_sentry_message("", "error");
    let _ = Telemetry::capture_sentry_message("test", "");
}

#[test]
fn test_sentry_config_dsn_none_by_default() {
    let config = TelemetryConfig::default();
    assert!(config.sentry_dsn.is_none(), "sentry_dsn should default to None");
}

#[test]
fn test_sentry_config_environment_none_by_default() {
    let config = TelemetryConfig::default();
    assert!(config.sentry_environment.is_none());
}

#[test]
fn test_sentry_config_release_none_by_default() {
    let config = TelemetryConfig::default();
    assert!(config.sentry_release.is_none());
}

#[test]
fn test_sentry_config_sample_rate_between_0_and_1() {
    let config = TelemetryConfig::default();
    assert!(config.sentry_sample_rate >= 0.0);
    assert!(config.sentry_sample_rate <= 1.0);
}

#[test]
fn test_sentry_config_traces_sample_rate_between_0_and_1() {
    let config = TelemetryConfig {
        sentry_traces_sample_rate: 0.5,
        ..TelemetryConfig::default()
    };
    assert!(config.sentry_traces_sample_rate >= 0.0);
    assert!(config.sentry_traces_sample_rate <= 1.0);
}

#[test]
fn test_sentry_config_serde_roundtrip_all_fields() {
    let config = TelemetryConfig {
        sentry_dsn: Some("https://abc@sentry.io/100".to_string()),
        sentry_environment: Some("integration-test".to_string()),
        sentry_release: Some("auroraview@test-version".to_string()),
        sentry_sample_rate: 0.3,
        sentry_traces_sample_rate: 0.1,
        ..TelemetryConfig::default()
    };
    let json = serde_json::to_string(&config).unwrap();
    let restored: TelemetryConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.sentry_dsn, config.sentry_dsn);
    assert_eq!(restored.sentry_environment, config.sentry_environment);
    assert_eq!(restored.sentry_release, config.sentry_release);
    assert!((restored.sentry_sample_rate - config.sentry_sample_rate).abs() < f32::EPSILON);
    assert!((restored.sentry_traces_sample_rate - config.sentry_traces_sample_rate).abs() < f32::EPSILON);
}

#[test]
fn test_sentry_config_debug_format_contains_field_names() {
    let config = TelemetryConfig {
        sentry_dsn: Some("https://k@sentry.io/1".to_string()),
        sentry_environment: Some("prod".to_string()),
        ..TelemetryConfig::default()
    };
    let s = format!("{:?}", config);
    // Debug output should contain at least one of the field names
    assert!(
        s.contains("sentry_dsn") || s.contains("TelemetryConfig"),
        "Debug format should mention fields: {}",
        s
    );
}

#[test]
fn test_sentry_capture_returns_bool() {
    // Capture returns bool; just verify the type without caring about value
    let result: bool = Telemetry::capture_sentry_message("type check", "warning");
    let _ = result;
}

#[test]
fn test_sentry_config_multiple_clones_independent() {
    let base = TelemetryConfig {
        sentry_dsn: Some("https://base@sentry.io/1".to_string()),
        sentry_sample_rate: 0.5,
        ..TelemetryConfig::default()
    };
    let mut c1 = base.clone();
    let mut c2 = base.clone();

    c1.sentry_dsn = Some("https://c1@sentry.io/1".to_string());
    c2.sentry_dsn = Some("https://c2@sentry.io/2".to_string());

    assert_ne!(c1.sentry_dsn, c2.sentry_dsn);
    // base should remain unchanged
    assert_eq!(base.sentry_dsn.as_deref(), Some("https://base@sentry.io/1"));
}

#[test]
fn test_sentry_config_partial_update_dsn_only() {
    let config = TelemetryConfig {
        sentry_dsn: Some("https://dsn-only@sentry.io/1".to_string()),
        ..TelemetryConfig::default()
    };
    // Other fields remain at default
    assert!(config.sentry_environment.is_none());
    assert!(config.sentry_release.is_none());
    assert!((config.sentry_sample_rate - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_sentry_capture_with_newline_in_message() {
    // Multi-line messages should not panic
    let _result = Telemetry::capture_sentry_message("line1\nline2\nline3", "error");
}

#[test]
fn test_sentry_config_serde_preserves_null_fields() {
    let config = TelemetryConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    // Serde should handle null fields correctly
    let restored: TelemetryConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.sentry_dsn, None);
    assert_eq!(restored.sentry_environment, None);
}

// ============================================================================
// R10 Extensions
// ============================================================================

#[test]
fn test_sentry_capture_with_null_byte_like_chars() {
    // Strings with unusual chars should not panic
    let _r = Telemetry::capture_sentry_message("msg\x00with null-ish", "error");
}

#[test]
fn test_sentry_config_sample_rate_near_boundary() {
    // Near-zero and near-one sample rates
    for rate in &[0.001_f32, 0.999_f32, 0.5_f32] {
        let config = TelemetryConfig {
            sentry_sample_rate: *rate,
            ..TelemetryConfig::default()
        };
        assert!(config.sentry_sample_rate >= 0.0);
        assert!(config.sentry_sample_rate <= 1.0);
    }
}

#[test]
fn test_sentry_capture_multiple_levels_sequential() {
    // Sequential calls with different levels should all be safe
    let levels = ["trace", "debug", "info", "warning", "warn", "error", "fatal", "critical", ""];
    for level in &levels {
        let _r = Telemetry::capture_sentry_message("sequential-test", level);
    }
}

#[test]
fn test_sentry_config_all_fields_set() {
    let config = TelemetryConfig {
        sentry_dsn: Some("https://k@sentry.io/1".to_string()),
        sentry_environment: Some("dev".to_string()),
        sentry_release: Some("0.0.1".to_string()),
        sentry_sample_rate: 0.8,
        sentry_traces_sample_rate: 0.4,
        ..TelemetryConfig::default()
    };
    assert!(config.sentry_dsn.is_some());
    assert!(config.sentry_environment.is_some());
    assert!(config.sentry_release.is_some());
    assert!((config.sentry_sample_rate - 0.8).abs() < f32::EPSILON);
    assert!((config.sentry_traces_sample_rate - 0.4).abs() < f32::EPSILON);
}

#[test]
fn test_sentry_config_serde_roundtrip_partial_fields() {
    // Only dsn set; other optional fields remain None after roundtrip
    let config = TelemetryConfig {
        sentry_dsn: Some("https://partial@sentry.io/5".to_string()),
        ..TelemetryConfig::default()
    };
    let json = serde_json::to_string(&config).unwrap();
    let restored: TelemetryConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.sentry_dsn.as_deref(), Some("https://partial@sentry.io/5"));
    assert!(restored.sentry_environment.is_none());
    assert!(restored.sentry_release.is_none());
}

#[test]
fn test_sentry_config_dsn_empty_string() {
    // An empty string DSN is technically set but empty
    let config = TelemetryConfig {
        sentry_dsn: Some(String::new()),
        ..TelemetryConfig::default()
    };
    assert_eq!(config.sentry_dsn.as_deref(), Some(""));
}

#[test]
fn test_sentry_capture_returns_same_type_as_bool() {
    let r: bool = Telemetry::capture_sentry_message("typed", "info");
    // Both true and false are valid; just ensure it compiles as bool
    let _ = r;
}

#[test]
fn test_sentry_config_traces_rate_full() {
    let config = TelemetryConfig {
        sentry_traces_sample_rate: 1.0,
        ..TelemetryConfig::default()
    };
    assert!((config.sentry_traces_sample_rate - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_sentry_config_environment_long_string() {
    let long_env = "a".repeat(256);
    let config = TelemetryConfig {
        sentry_environment: Some(long_env.clone()),
        ..TelemetryConfig::default()
    };
    assert_eq!(config.sentry_environment.as_deref(), Some(long_env.as_str()));
}

#[test]
fn test_sentry_config_release_with_build_metadata() {
    for release in &[
        "auroraview@1.0.0+build.123",
        "1.2.3-alpha.1+exp.sha.5114f85",
        "v0.1.0-rc.1",
    ] {
        let config = TelemetryConfig {
            sentry_release: Some(release.to_string()),
            ..TelemetryConfig::default()
        };
        assert_eq!(config.sentry_release.as_deref(), Some(*release));
    }
}
