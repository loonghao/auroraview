//! General utilities
//!
//! Common utility functions used across AuroraView components.

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Initialize logging for the library
///
/// Sets up tracing with environment-based filtering via RUST_LOG
pub fn init_logging() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("warn,auroraview=info"));

        tracing_subscriber::registry()
            .with(fmt::layer().with_target(true).with_thread_ids(true))
            .with(filter)
            .try_init()
            .ok();
    });
}

/// Escape a string for use in JavaScript
pub fn escape_js_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Parse a size string like "800x600" into (width, height)
pub fn parse_size(s: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = s.split('x').collect();
    if parts.len() == 2 {
        let width = parts[0].trim().parse().ok()?;
        let height = parts[1].trim().parse().ok()?;
        Some((width, height))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_js_string() {
        assert_eq!(escape_js_string("hello"), "hello");
        assert_eq!(escape_js_string("hello\"world"), "hello\\\"world");
        assert_eq!(escape_js_string("hello\nworld"), "hello\\nworld");
        assert_eq!(escape_js_string("path\\to\\file"), "path\\\\to\\\\file");
    }

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("800x600"), Some((800, 600)));
        assert_eq!(parse_size("1920x1080"), Some((1920, 1080)));
        assert_eq!(parse_size(" 800 x 600 "), Some((800, 600)));
        assert_eq!(parse_size("invalid"), None);
        assert_eq!(parse_size("800"), None);
    }
}
