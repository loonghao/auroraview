//! Tests for auroraview-pack license module

use auroraview_pack::{get_machine_id, LicenseConfig, LicenseReason, LicenseValidator};

#[test]
fn test_no_license_required() {
    let config = LicenseConfig::default();
    let validator = LicenseValidator::new(config);
    let status = validator.validate(None);

    assert!(status.valid);
    assert_eq!(status.reason, LicenseReason::NoLicenseRequired);
}

#[test]
fn test_token_required() {
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
fn test_expiration() {
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
fn test_grace_period() {
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
fn test_machine_id() {
    let id = get_machine_id();
    assert!(!id.is_empty());
}
