//! Optional Sentry integration helpers.

use crate::config::TelemetryConfig;
use crate::error::TelemetryError;

#[cfg(feature = "sentry")]
pub type SentryGuard = sentry::ClientInitGuard;

#[cfg(not(feature = "sentry"))]
pub struct SentryGuard;

pub fn init(config: &TelemetryConfig) -> Result<Option<SentryGuard>, TelemetryError> {
    #[cfg(feature = "sentry")]
    {
        if let Some(dsn) = config.sentry_dsn.as_deref() {
            let parsed_dsn = dsn
                .parse()
                .map_err(|e| TelemetryError::SentryConfig(format!("invalid DSN: {e}")))?;

            let options = sentry::ClientOptions {
                dsn: Some(parsed_dsn),
                environment: config
                    .sentry_environment
                    .clone()
                    .map(std::borrow::Cow::Owned),
                release: config.sentry_release.clone().map(std::borrow::Cow::Owned),
                sample_rate: config.sentry_sample_rate,
                traces_sample_rate: config.sentry_traces_sample_rate,
                ..Default::default()
            };

            let guard = sentry::init(options);
            return Ok(Some(guard));
        }

        Ok(None)
    }

    #[cfg(not(feature = "sentry"))]
    {
        let _ = config;
        Ok(None)
    }
}

pub fn capture_message(message: &str, level: &str) -> bool {
    #[cfg(feature = "sentry")]
    {
        let sentry_level = match level.to_ascii_lowercase().as_str() {
            "fatal" => sentry::Level::Fatal,
            "error" => sentry::Level::Error,
            "warning" | "warn" => sentry::Level::Warning,
            "debug" => sentry::Level::Debug,
            _ => sentry::Level::Info,
        };
        sentry::capture_message(message, sentry_level);
        true
    }

    #[cfg(not(feature = "sentry"))]
    {
        let _ = message;
        let _ = level;
        false
    }
}
