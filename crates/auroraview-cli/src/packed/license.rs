//! License validation module

use anyhow::Result;
use auroraview_pack::{LicenseConfig, LicenseReason, LicenseValidator};
use std::io::{self, Write};

/// Validate license and handle token input if needed
pub fn validate_license(license_config: &LicenseConfig) -> Result<bool> {
    let validator = LicenseValidator::new(license_config.clone());

    // First try without token
    let mut status = validator.validate(None);

    // If token is required and not embedded, prompt for it
    if status.reason == LicenseReason::TokenRequired {
        tracing::info!("Authorization token required");

        // Try to get token from environment variable first
        let env_token = std::env::var("AURORAVIEW_TOKEN").ok();

        if let Some(ref token) = env_token {
            tracing::debug!("Using token from AURORAVIEW_TOKEN environment variable");
            status = validator.validate(Some(token));
        } else {
            // Prompt user for token
            print!("Enter authorization token: ");
            io::stdout().flush()?;

            let mut token = String::new();
            io::stdin().read_line(&mut token)?;
            let token = token.trim();

            if token.is_empty() {
                eprintln!("Error: No token provided");
                return Ok(false);
            }

            status = validator.validate(Some(token));
        }
    }

    // Handle validation result
    if status.valid {
        if status.in_grace_period {
            if let Some(ref msg) = status.message {
                eprintln!("Warning: {}", msg);
            }
            if let Some(days) = status.days_remaining {
                eprintln!("Grace period: {} days remaining", days);
            }
        } else if let Some(days) = status.days_remaining {
            tracing::info!("License valid for {} more days", days);
        }
        Ok(true)
    } else {
        let error_msg = status
            .message
            .unwrap_or_else(|| format!("License validation failed: {:?}", status.reason));
        eprintln!("Error: {}", error_msg);
        Ok(false)
    }
}
