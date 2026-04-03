//! Tests for auroraview-pack license module

use auroraview_pack::{get_machine_id, LicenseConfig, LicenseReason, LicenseStatus, LicenseValidator};

#[test]
fn no_license_required() {
    let config = LicenseConfig::default();
    let validator = LicenseValidator::new(config);
    let status = validator.validate(None);

    assert!(status.valid);
    assert_eq!(status.reason, LicenseReason::NoLicenseRequired);
}

#[test]
fn token_required() {
    let config = LicenseConfig::token_required();
    let validator = LicenseValidator::new(config);

    // Without token
    let status = validator.validate(None);
    assert!(!status.valid);
    assert_eq!(status.reason, LicenseReason::TokenRequired);

    // With token
    let status = validator.validate(Some("valid-token-12345"));
    assert!(status.valid);
}

#[test]
fn expiration() {
    // Future date
    let config = LicenseConfig::time_limited("2099-12-31");
    let validator = LicenseValidator::new(config);
    let status = validator.validate(None);
    assert!(status.valid);
    assert!(status.days_remaining.unwrap() > 0);

    // Past date
    let config = LicenseConfig::time_limited("2020-01-01");
    let validator = LicenseValidator::new(config);
    let status = validator.validate(None);
    assert!(!status.valid);
    assert_eq!(status.reason, LicenseReason::Expired);
}

#[test]
fn grace_period() {
    // Create a config with grace period
    let mut config = LicenseConfig::time_limited("2020-01-01");
    config.grace_period_days = 36500; // 100 years grace period for testing

    let validator = LicenseValidator::new(config);
    let status = validator.validate(None);

    assert!(status.valid);
    assert_eq!(status.reason, LicenseReason::GracePeriod);
    assert!(status.in_grace_period);
}

#[test]
fn machine_id() {
    let id = get_machine_id();
    assert!(!id.is_empty());
}

#[test]
fn token_too_short_is_invalid() {
    let config = LicenseConfig::token_required();
    let validator = LicenseValidator::new(config);

    // Token with fewer than 8 chars should fail format check
    let status = validator.validate(Some("short"));
    assert!(!status.valid);
    assert_eq!(status.reason, LicenseReason::InvalidToken);
}

#[test]
fn empty_token_is_missing() {
    let config = LicenseConfig::token_required();
    let validator = LicenseValidator::new(config);

    // Empty string is treated as no token by the format check
    let status = validator.validate(Some(""));
    assert!(!status.valid);
    // Either TokenRequired or InvalidToken is acceptable
    assert!(matches!(
        status.reason,
        LicenseReason::TokenRequired | LicenseReason::InvalidToken
    ));
}

#[test]
fn embedded_token_used_when_no_provided() {
    let mut config = LicenseConfig::token_required();
    config.embedded_token = Some("embedded-token-xyz".to_string());
    let validator = LicenseValidator::new(config);

    // No token provided but embedded token should satisfy requirement
    let status = validator.validate(None);
    assert!(status.valid);
}

#[test]
fn full_config_requires_token_and_expiry() {
    let config = LicenseConfig::full("2099-12-31");
    let validator = LicenseValidator::new(config);

    // No token: should fail
    let status = validator.validate(None);
    assert!(!status.valid);
    assert_eq!(status.reason, LicenseReason::TokenRequired);

    // With valid token and future expiry: should pass
    let config2 = LicenseConfig::full("2099-12-31");
    let validator2 = LicenseValidator::new(config2);
    let status2 = validator2.validate(Some("my-valid-token-abc123"));
    assert!(status2.valid);
    assert!(matches!(
        status2.reason,
        LicenseReason::Valid | LicenseReason::GracePeriod
    ));
}

#[test]
fn is_active_with_expiry() {
    let config = LicenseConfig::time_limited("2099-01-01");
    assert!(config.is_active());
}

#[test]
fn is_active_with_token() {
    let config = LicenseConfig::token_required();
    assert!(config.is_active());
}

#[test]
fn is_not_active_when_disabled() {
    let config = LicenseConfig::default();
    assert!(!config.is_active());
}

#[test]
fn invalid_date_format_returns_config_error() {
    let config = LicenseConfig::time_limited("not-a-date");
    let validator = LicenseValidator::new(config);
    let status = validator.validate(None);
    assert!(!status.valid);
    assert_eq!(status.reason, LicenseReason::ConfigError);
}

#[test]
fn custom_expiration_message() {
    let mut config = LicenseConfig::time_limited("2020-01-01");
    config.expiration_message = Some("Custom expired message".to_string());
    let validator = LicenseValidator::new(config);
    let status = validator.validate(None);
    assert!(!status.valid);
    assert_eq!(
        status.message.as_deref(),
        Some("Custom expired message")
    );
}

#[test]
fn machine_id_is_consistent() {
    let id1 = get_machine_id();
    let id2 = get_machine_id();
    assert_eq!(id1, id2, "machine ID should be consistent across calls");
}

// ============================================================================
// Extended coverage tests
// ============================================================================

#[test]
fn license_status_valid_serde_roundtrip() {
    let config = LicenseConfig::default();
    let validator = LicenseValidator::new(config);
    let status = validator.validate(None);

    let json = serde_json::to_string(&status).unwrap();
    let parsed: LicenseStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.valid, status.valid);
    assert_eq!(parsed.reason, status.reason);
}

#[test]
fn license_status_clone() {
    let config = LicenseConfig::default();
    let validator = LicenseValidator::new(config);
    let status = validator.validate(None);
    let cloned = status.clone();
    assert_eq!(cloned.valid, status.valid);
    assert_eq!(cloned.reason, status.reason);
}

#[test]
fn license_reason_debug() {
    let reason = LicenseReason::Valid;
    let s = format!("{reason:?}");
    assert!(s.contains("Valid"));
}

#[test]
fn license_reason_all_variants_debug() {
    let variants = [
        LicenseReason::Valid,
        LicenseReason::NoLicenseRequired,
        LicenseReason::Expired,
        LicenseReason::GracePeriod,
        LicenseReason::TokenRequired,
        LicenseReason::InvalidToken,
        LicenseReason::MachineNotAllowed,
        LicenseReason::ValidationFailed,
        LicenseReason::ConfigError,
    ];
    for v in &variants {
        let s = format!("{v:?}");
        assert!(!s.is_empty());
    }
}

#[test]
fn days_remaining_positive_for_future_date() {
    let config = LicenseConfig::time_limited("2099-01-01");
    let validator = LicenseValidator::new(config);
    let status = validator.validate(None);
    assert!(status.valid);
    assert!(status.days_remaining.unwrap() > 0);
}

#[test]
fn days_remaining_none_when_no_expiry() {
    let config = LicenseConfig::default();
    let validator = LicenseValidator::new(config);
    let status = validator.validate(None);
    assert!(status.days_remaining.is_none());
}

#[test]
fn token_exactly_eight_chars_is_valid() {
    let config = LicenseConfig::token_required();
    let validator = LicenseValidator::new(config);
    let status = validator.validate(Some("12345678"));
    assert!(status.valid);
}

#[test]
fn token_exactly_seven_chars_is_invalid() {
    let config = LicenseConfig::token_required();
    let validator = LicenseValidator::new(config);
    let status = validator.validate(Some("1234567"));
    assert!(!status.valid);
    assert_eq!(status.reason, LicenseReason::InvalidToken);
}

#[test]
fn license_config_serde_roundtrip() {
    let config = LicenseConfig::full("2099-06-15");
    let json = serde_json::to_string(&config).unwrap();
    let parsed: LicenseConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.enabled, config.enabled);
    assert_eq!(parsed.require_token, config.require_token);
    assert_eq!(parsed.expires_at, config.expires_at);
}

#[test]
fn license_config_clone() {
    let config = LicenseConfig::token_required();
    let cloned = config.clone();
    assert_eq!(cloned.enabled, config.enabled);
    assert_eq!(cloned.require_token, config.require_token);
}

#[test]
fn allowed_machines_blocks_unknown_machine() {
    let mut config = LicenseConfig::default();
    config.enabled = true;
    config.allowed_machines = vec!["non-existent-machine-id-xyz-9999".to_string()];
    let validator = LicenseValidator::new(config);
    let status = validator.validate(None);
    assert!(!status.valid);
    assert_eq!(status.reason, LicenseReason::MachineNotAllowed);
}

#[test]
fn allowed_machines_allows_current_machine() {
    let machine_id = get_machine_id();
    let mut config = LicenseConfig::default();
    config.enabled = true;
    config.allowed_machines = vec![machine_id];
    let validator = LicenseValidator::new(config);
    let status = validator.validate(None);
    // Current machine is in allowed list → should pass machine check and be valid
    assert!(status.valid);
}

#[test]
fn invalid_date_format_with_letters() {
    let config = LicenseConfig::time_limited("YYYY-MM-DD");
    let validator = LicenseValidator::new(config);
    let status = validator.validate(None);
    assert!(!status.valid);
    assert_eq!(status.reason, LicenseReason::ConfigError);
}

#[test]
fn invalid_date_format_too_few_parts() {
    let config = LicenseConfig::time_limited("2099-12");
    let validator = LicenseValidator::new(config);
    let status = validator.validate(None);
    assert!(!status.valid);
    assert_eq!(status.reason, LicenseReason::ConfigError);
}

#[test]
fn grace_period_zero_means_no_grace() {
    let mut config = LicenseConfig::time_limited("2020-01-01");
    config.grace_period_days = 0;
    let validator = LicenseValidator::new(config);
    let status = validator.validate(None);
    assert!(!status.valid);
    assert_eq!(status.reason, LicenseReason::Expired);
}
