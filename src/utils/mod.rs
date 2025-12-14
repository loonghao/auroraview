//! Utility functions and helpers
//!
//! This module provides logging initialization, URL normalization,
//! and re-exports protocol utilities from auroraview-core.
//!
//! NOTE: Functions in this module are used by PyO3 bindings (conditionally compiled).
//! The dead_code warnings are expected when compiling without python-bindings feature.
//!
//! ## AuroraView Protocol URL Format
//!
//! Local files are accessed through the custom protocol with type prefixes:
//!
//! - `type:file` - Converted from file:// URLs
//!   - `file:///C:/path/to/file.ext` → `https://auroraview.localhost/type:file/C:/path/to/file.ext`
//!
//! - `type:local` - Converted from local file paths
//!   - `C:/path/to/file.ext` → `https://auroraview.localhost/type:local/C:/path/to/file.ext`
//!   - `/path/to/file.ext` → `https://auroraview.localhost/type:local/path/to/file.ext`
//!
//! The type prefix helps distinguish the source of the path for debugging and logging.

use tracing_subscriber::{fmt, EnvFilter};

// Re-export IdGenerator from core for backward compatibility
#[allow(unused_imports)]
pub use auroraview_core::id_generator::IdGenerator;

// Re-export protocol utilities from auroraview-core
pub use auroraview_core::protocol::{
    file_url_to_auroraview, local_path_to_auroraview, PROTOCOL_TYPE_FILE, PROTOCOL_TYPE_LOCAL,
};
// Re-export additional utilities that may be used by external consumers
#[allow(unused_imports)]
pub use auroraview_core::protocol::{is_auroraview_url, strip_protocol_type, AURORAVIEW_HOST};

/// Normalize a URL string to ensure it has a valid scheme.
///
/// This function handles common URL input patterns:
/// - `baidu.com` -> `https://baidu.com`
/// - `www.example.com` -> `https://www.example.com`
/// - `http://example.com` -> `http://example.com` (unchanged)
/// - `https://example.com` -> `https://example.com` (unchanged)
/// - `file:///path/to/file.html` -> `https://auroraview.localhost/type:file/path/to/file.html`
/// - `/path/to/file.html` -> `https://auroraview.localhost/type:local/path/to/file.html`
/// - `C:\path\to\file.html` -> `https://auroraview.localhost/type:local/C:/path/to/file.html`
///
/// # Arguments
/// * `url` - The URL string to normalize
///
/// # Returns
/// A normalized URL string with a valid scheme
#[allow(dead_code)] // Used by webview/core/main.rs
pub fn normalize_url(url: &str) -> String {
    let trimmed = url.trim();

    // Empty URL
    if trimmed.is_empty() {
        return String::new();
    }

    // Check for Windows absolute path first (e.g., C:\path or D:/path)
    // This must be checked before url::Url::parse() because "C:" would be parsed as a scheme
    #[cfg(target_os = "windows")]
    {
        if trimmed.len() >= 2 {
            let chars: Vec<char> = trimmed.chars().take(3).collect();
            if chars.len() >= 2
                && chars[0].is_ascii_alphabetic()
                && (chars[1] == ':')
                && (chars.len() < 3 || chars[2] == '\\' || chars[2] == '/')
            {
                // Convert Windows path to auroraview protocol URL with type:local prefix
                return local_path_to_auroraview(trimmed);
            }
        }
    }

    // Check if it already has a valid web scheme
    // We only accept specific schemes, not arbitrary ones like "C:" or "localhost"
    if let Ok(parsed) = url::Url::parse(trimmed) {
        let scheme = parsed.scheme();
        // Convert file:// to auroraview protocol with type:file prefix
        if scheme == "file" {
            return file_url_to_auroraview(trimmed);
        }
        // Only accept known web schemes (keep as-is)
        if matches!(
            scheme,
            "http" | "https" | "data" | "about" | "blob" | "javascript"
        ) {
            return trimmed.to_string();
        }
    }

    // Unix absolute path - use type:local prefix
    if trimmed.starts_with('/') {
        return local_path_to_auroraview(trimmed);
    }

    // Looks like a domain or localhost
    // - Contains a dot (e.g., baidu.com, www.example.com)
    // - Starts with "localhost" (e.g., localhost, localhost:8080)
    // - Does not contain spaces
    // - Does not start with a dot
    if !trimmed.starts_with('.')
        && !trimmed.contains(' ')
        && (trimmed.contains('.') || trimmed.starts_with("localhost"))
    {
        return format!("https://{}", trimmed);
    }

    // Default: assume it's a relative path or invalid, return as-is
    trimmed.to_string()
}

/// Initialize logging for the library
#[allow(dead_code)] // Used by lib.rs when python-bindings feature is enabled
pub fn init_logging() {
    // Only initialize once
    static INIT: std::sync::Once = std::sync::Once::new();

    INIT.call_once(|| {
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

        fmt()
            .with_env_filter(filter)
            .with_target(false)
            .with_thread_ids(true)
            .with_line_number(true)
            .with_writer(std::io::stderr) // Write to stderr to avoid interfering with stdout
            .init();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_init() {
        // Test that logging can be initialized
        init_logging();
        // Call again to ensure it's idempotent
        init_logging();
        // If we get here without panicking, the test passes
    }

    #[test]
    fn test_id_generator_reexport() {
        // Test that the re-exported IdGenerator works
        let gen = IdGenerator::new();
        let id1 = gen.next();
        let id2 = gen.next();
        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
    }

    #[test]
    fn test_normalize_url_with_scheme() {
        // URLs with scheme should remain unchanged
        assert_eq!(normalize_url("https://example.com"), "https://example.com");
        assert_eq!(normalize_url("http://example.com"), "http://example.com");
        assert_eq!(
            normalize_url("https://www.baidu.com/search?q=test"),
            "https://www.baidu.com/search?q=test"
        );
    }

    #[test]
    fn test_normalize_url_domain_only() {
        // Domain without scheme should get https://
        assert_eq!(normalize_url("baidu.com"), "https://baidu.com");
        assert_eq!(normalize_url("www.example.com"), "https://www.example.com");
        assert_eq!(
            normalize_url("example.com/path"),
            "https://example.com/path"
        );
        assert_eq!(normalize_url("localhost"), "https://localhost");
        assert_eq!(normalize_url("localhost:8080"), "https://localhost:8080");
    }

    #[test]
    fn test_normalize_url_file_protocol() {
        // file:// URLs should be converted to auroraview protocol with type:file prefix
        assert_eq!(
            normalize_url("file:///path/to/file.html"),
            "https://auroraview.localhost/type:file/path/to/file.html"
        );
    }

    #[test]
    fn test_normalize_url_unix_path() {
        // Unix absolute paths should be converted to auroraview protocol with type:local prefix
        assert_eq!(
            normalize_url("/path/to/file.html"),
            "https://auroraview.localhost/type:local/path/to/file.html"
        );
        assert_eq!(
            normalize_url("/home/user/index.html"),
            "https://auroraview.localhost/type:local/home/user/index.html"
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_normalize_url_windows_path() {
        // Windows paths should be converted to auroraview protocol with type:local prefix
        assert_eq!(
            normalize_url("C:\\Users\\test\\file.html"),
            "https://auroraview.localhost/type:local/C:/Users/test/file.html"
        );
        assert_eq!(
            normalize_url("D:/path/to/file.html"),
            "https://auroraview.localhost/type:local/D:/path/to/file.html"
        );
    }

    #[test]
    fn test_normalize_url_empty() {
        assert_eq!(normalize_url(""), "");
        assert_eq!(normalize_url("   "), "");
    }

    #[test]
    fn test_normalize_url_whitespace() {
        // Whitespace should be trimmed
        assert_eq!(normalize_url("  baidu.com  "), "https://baidu.com");
        assert_eq!(
            normalize_url("  https://example.com  "),
            "https://example.com"
        );
    }

    #[test]
    fn test_normalize_url_special_schemes() {
        // Special schemes should remain unchanged
        assert_eq!(normalize_url("about:blank"), "about:blank");
        assert_eq!(
            normalize_url("data:text/html,<h1>Hello</h1>"),
            "data:text/html,<h1>Hello</h1>"
        );
    }

    #[test]
    fn test_reexported_protocol_functions() {
        // Verify re-exported functions work correctly
        assert_eq!(
            file_url_to_auroraview("file:///C:/path/to/file.html"),
            "https://auroraview.localhost/type:file/C:/path/to/file.html"
        );
        assert_eq!(
            local_path_to_auroraview("C:/path/to/file.html"),
            "https://auroraview.localhost/type:local/C:/path/to/file.html"
        );
        assert!(is_auroraview_url(
            "https://auroraview.localhost/type:file/path"
        ));
        assert!(!is_auroraview_url("https://example.com"));
    }

    #[test]
    fn test_reexported_constants() {
        // Verify re-exported constants
        assert_eq!(AURORAVIEW_HOST, "auroraview.localhost");
        assert_eq!(PROTOCOL_TYPE_FILE, "type:file");
        assert_eq!(PROTOCOL_TYPE_LOCAL, "type:local");
    }
}
