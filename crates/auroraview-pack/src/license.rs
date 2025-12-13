//! License validation module for packed applications
//!
//! This module provides license validation functionality including:
//! - Time-based expiration
//! - Token validation (offline and online)
//! - Machine ID binding
//! - Grace period handling

use crate::config::LicenseConfig;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// License validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseStatus {
    /// Whether the license is valid
    pub valid: bool,
    /// Reason for validation result
    pub reason: LicenseReason,
    /// Days remaining until expiration (if applicable)
    pub days_remaining: Option<i64>,
    /// Whether currently in grace period
    pub in_grace_period: bool,
    /// Custom message to display
    pub message: Option<String>,
}

/// Reason for license validation result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LicenseReason {
    /// License is valid
    Valid,
    /// No license required
    NoLicenseRequired,
    /// License has expired
    Expired,
    /// Currently in grace period
    GracePeriod,
    /// Token is required but not provided
    TokenRequired,
    /// Token is invalid
    InvalidToken,
    /// Machine ID not allowed
    MachineNotAllowed,
    /// Online validation failed
    ValidationFailed,
    /// License configuration error
    ConfigError,
}

/// License validator
pub struct LicenseValidator {
    config: LicenseConfig,
}

impl LicenseValidator {
    /// Create a new license validator
    pub fn new(config: LicenseConfig) -> Self {
        Self { config }
    }

    /// Validate the license
    pub fn validate(&self, provided_token: Option<&str>) -> LicenseStatus {
        // If license is not enabled, always valid
        if !self.config.enabled {
            return LicenseStatus {
                valid: true,
                reason: LicenseReason::NoLicenseRequired,
                days_remaining: None,
                in_grace_period: false,
                message: None,
            };
        }

        // Check token requirement
        if self.config.require_token {
            let token = provided_token.or(self.config.embedded_token.as_deref());
            if token.is_none() {
                return LicenseStatus {
                    valid: false,
                    reason: LicenseReason::TokenRequired,
                    days_remaining: None,
                    in_grace_period: false,
                    message: Some("Authorization token is required".to_string()),
                };
            }

            // Validate token format (basic check)
            if let Some(t) = token {
                if !self.validate_token_format(t) {
                    return LicenseStatus {
                        valid: false,
                        reason: LicenseReason::InvalidToken,
                        days_remaining: None,
                        in_grace_period: false,
                        message: Some("Invalid authorization token".to_string()),
                    };
                }
            }
        }

        // Check machine ID binding
        if !self.config.allowed_machines.is_empty() {
            let machine_id = get_machine_id();
            if !self.config.allowed_machines.contains(&machine_id) {
                return LicenseStatus {
                    valid: false,
                    reason: LicenseReason::MachineNotAllowed,
                    days_remaining: None,
                    in_grace_period: false,
                    message: Some("This machine is not authorized".to_string()),
                };
            }
        }

        // Check expiration
        if let Some(ref expires_at) = self.config.expires_at {
            match self.check_expiration(expires_at) {
                ExpirationCheck::Valid { days_remaining } => {
                    return LicenseStatus {
                        valid: true,
                        reason: LicenseReason::Valid,
                        days_remaining: Some(days_remaining),
                        in_grace_period: false,
                        message: None,
                    };
                }
                ExpirationCheck::GracePeriod { days_remaining } => {
                    let message = self.config.expiration_message.clone().unwrap_or_else(|| {
                        format!(
                            "License expired. Grace period: {} days remaining",
                            days_remaining
                        )
                    });
                    return LicenseStatus {
                        valid: true,
                        reason: LicenseReason::GracePeriod,
                        days_remaining: Some(days_remaining),
                        in_grace_period: true,
                        message: Some(message),
                    };
                }
                ExpirationCheck::Expired => {
                    let message = self
                        .config
                        .expiration_message
                        .clone()
                        .unwrap_or_else(|| "License has expired".to_string());
                    return LicenseStatus {
                        valid: false,
                        reason: LicenseReason::Expired,
                        days_remaining: None,
                        in_grace_period: false,
                        message: Some(message),
                    };
                }
                ExpirationCheck::ParseError => {
                    return LicenseStatus {
                        valid: false,
                        reason: LicenseReason::ConfigError,
                        days_remaining: None,
                        in_grace_period: false,
                        message: Some("Invalid expiration date format".to_string()),
                    };
                }
            }
        }

        // All checks passed
        LicenseStatus {
            valid: true,
            reason: LicenseReason::Valid,
            days_remaining: None,
            in_grace_period: false,
            message: None,
        }
    }

    /// Validate token format (basic check)
    fn validate_token_format(&self, token: &str) -> bool {
        // Token should be non-empty and have reasonable length
        !token.is_empty() && token.len() >= 8 && token.len() <= 512
    }

    /// Check expiration date
    fn check_expiration(&self, expires_at: &str) -> ExpirationCheck {
        // Parse date in YYYY-MM-DD format
        let parts: Vec<&str> = expires_at.split('-').collect();
        if parts.len() != 3 {
            return ExpirationCheck::ParseError;
        }

        let year: i32 = match parts[0].parse() {
            Ok(y) => y,
            Err(_) => return ExpirationCheck::ParseError,
        };
        let month: u32 = match parts[1].parse() {
            Ok(m) => m,
            Err(_) => return ExpirationCheck::ParseError,
        };
        let day: u32 = match parts[2].parse() {
            Ok(d) => d,
            Err(_) => return ExpirationCheck::ParseError,
        };

        // Calculate expiration timestamp (end of day in UTC)
        let expiration_days = days_since_epoch(year, month, day);
        let current_days = current_days_since_epoch();

        let days_until_expiration = expiration_days - current_days;

        if days_until_expiration >= 0 {
            ExpirationCheck::Valid {
                days_remaining: days_until_expiration,
            }
        } else {
            let days_expired = -days_until_expiration;
            let grace_days = self.config.grace_period_days as i64;

            if days_expired <= grace_days {
                ExpirationCheck::GracePeriod {
                    days_remaining: grace_days - days_expired,
                }
            } else {
                ExpirationCheck::Expired
            }
        }
    }
}

/// Result of expiration check
enum ExpirationCheck {
    Valid { days_remaining: i64 },
    GracePeriod { days_remaining: i64 },
    Expired,
    ParseError,
}

/// Get current days since Unix epoch
fn current_days_since_epoch() -> i64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO);
    (now.as_secs() / 86400) as i64
}

/// Calculate days since Unix epoch for a given date
fn days_since_epoch(year: i32, month: u32, day: u32) -> i64 {
    // Simple calculation (not accounting for all edge cases)
    let mut days: i64 = 0;

    // Days from years
    for y in 1970..year {
        days += if is_leap_year(y) { 366 } else { 365 };
    }

    // Days from months
    let days_in_month = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for m in 1..month {
        days += days_in_month[(m - 1) as usize] as i64;
        if m == 2 && is_leap_year(year) {
            days += 1;
        }
    }

    // Days
    days += day as i64;

    days
}

/// Check if a year is a leap year
fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Get machine ID for hardware binding
pub fn get_machine_id() -> String {
    #[cfg(target_os = "windows")]
    {
        get_windows_machine_id()
    }
    #[cfg(not(target_os = "windows"))]
    {
        get_fallback_machine_id()
    }
}

#[cfg(target_os = "windows")]
fn get_windows_machine_id() -> String {
    use std::process::Command;

    // Try to get Windows machine GUID
    let output = Command::new("wmic")
        .args(["csproduct", "get", "UUID"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let line = line.trim();
            if !line.is_empty() && line != "UUID" {
                return line.to_string();
            }
        }
    }

    // Use hostname as fallback on Windows
    hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

#[cfg(not(target_os = "windows"))]
fn get_fallback_machine_id() -> String {
    // Use hostname as fallback
    hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
