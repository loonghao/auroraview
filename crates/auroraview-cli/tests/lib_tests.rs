//! Unit tests for auroraview-cli lib module
//!
//! These tests verify the core library functions.

use auroraview_cli::{load_window_icon, load_window_icon_from_bytes, normalize_url, ICON_PNG_BYTES};
use rstest::rstest;

// =============================================================================
// ICON_PNG_BYTES tests
// =============================================================================

#[test]
fn test_icon_png_bytes_not_empty() {
    assert!(!ICON_PNG_BYTES.is_empty());
    // PNG magic bytes: 0x89 0x50 0x4E 0x47
    assert_eq!(&ICON_PNG_BYTES[0..4], &[0x89, 0x50, 0x4E, 0x47]);
}

#[test]
fn test_icon_png_bytes_have_png_signature() {
    // Full 8-byte PNG signature: \x89PNG\r\n\x1a\n
    assert_eq!(
        &ICON_PNG_BYTES[0..8],
        &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
    );
}

#[test]
fn test_icon_png_bytes_minimum_size() {
    // A valid minimal PNG is at least 67 bytes (signature + IHDR + IDAT + IEND)
    assert!(ICON_PNG_BYTES.len() >= 67);
}

// =============================================================================
// load_window_icon tests
// =============================================================================

#[test]
fn test_load_window_icon_succeeds() {
    let icon = load_window_icon();
    assert!(icon.is_some());
}

#[test]
fn test_load_window_icon_from_bytes_valid_png() {
    let icon = load_window_icon_from_bytes(ICON_PNG_BYTES);
    assert!(icon.is_some(), "Should load valid PNG bytes");
}

#[test]
fn test_load_window_icon_from_bytes_invalid_returns_none() {
    let icon = load_window_icon_from_bytes(b"not a png");
    assert!(icon.is_none(), "Should return None for invalid PNG data");
}

#[test]
fn test_load_window_icon_from_bytes_empty_returns_none() {
    let icon = load_window_icon_from_bytes(b"");
    assert!(icon.is_none(), "Should return None for empty bytes");
}

#[test]
fn test_load_window_icon_from_bytes_truncated_returns_none() {
    // Provide only PNG header bytes (incomplete PNG)
    let truncated = &ICON_PNG_BYTES[..8];
    let icon = load_window_icon_from_bytes(truncated);
    assert!(icon.is_none(), "Should return None for truncated PNG");
}

// =============================================================================
// normalize_url tests
// =============================================================================

#[rstest]
#[case("https://example.com", "https://example.com/")]
#[case("http://example.com", "http://example.com/")]
#[case("example.com", "https://example.com/")]
#[case("www.baidu.com", "https://www.baidu.com/")]
#[case("example.com/path", "https://example.com/path")]
#[case("example.com:8080", "https://example.com:8080/")]
fn test_normalize_url_valid(#[case] input: &str, #[case] expected: &str) {
    let result = normalize_url(input).unwrap();
    assert_eq!(result, expected);
}

#[rstest]
#[case("://invalid")]
#[case("not a valid url with spaces")]
fn test_normalize_url_invalid(#[case] input: &str) {
    let result = normalize_url(input);
    assert!(result.is_err());
}

#[test]
fn test_normalize_url_preserves_path_and_query() {
    let result = normalize_url("example.com/path?query=value&foo=bar").unwrap();
    assert!(result.contains("/path"));
    assert!(result.contains("query=value"));
    assert!(result.contains("foo=bar"));
}

#[test]
fn test_normalize_url_preserves_fragment() {
    let result = normalize_url("example.com/page#section").unwrap();
    assert!(result.contains("#section"));
}

#[test]
fn test_normalize_url_https_scheme_preserved() {
    let result = normalize_url("https://secure.example.com").unwrap();
    assert!(result.starts_with("https://"));
}

#[test]
fn test_normalize_url_http_scheme_preserved() {
    let result = normalize_url("http://insecure.example.com").unwrap();
    assert!(result.starts_with("http://"));
}

#[test]
fn test_normalize_url_adds_trailing_slash_for_bare_host() {
    let result = normalize_url("example.com").unwrap();
    assert!(result.ends_with('/'));
}

#[test]
fn test_normalize_url_with_port() {
    let result = normalize_url("localhost:3000").unwrap();
    assert!(result.contains("3000"));
}

#[test]
fn test_normalize_url_deeply_nested_path() {
    let result = normalize_url("example.com/a/b/c/d").unwrap();
    assert!(result.contains("/a/b/c/d"));
}


#[test]
fn test_normalize_url_returns_string_result() {
    let result: anyhow::Result<String> = normalize_url("example.com");
    assert!(result.is_ok());
    let s = result.unwrap();
    assert!(!s.is_empty());
}

// =============================================================================
// New: ICON_PNG_BYTES additional tests
// =============================================================================

#[test]
fn test_icon_png_bytes_is_static() {
    let b: &'static [u8] = ICON_PNG_BYTES;
    assert!(!b.is_empty());
}

#[test]
fn test_icon_png_bytes_len_reasonable() {
    // A typical app icon should be between 100 bytes and 1 MB
    assert!(ICON_PNG_BYTES.len() >= 100);
    assert!(ICON_PNG_BYTES.len() <= 1_048_576);
}

// =============================================================================
// New: load_window_icon additional tests
// =============================================================================

#[test]
fn test_load_window_icon_is_idempotent() {
    let icon1 = load_window_icon();
    let icon2 = load_window_icon();
    assert_eq!(icon1.is_some(), icon2.is_some());
}

#[test]
fn test_load_window_icon_from_bytes_only_png_header() {
    // 8 bytes is signature only, not a complete PNG
    let just_sig = &ICON_PNG_BYTES[..8];
    let icon = load_window_icon_from_bytes(just_sig);
    assert!(icon.is_none());
}

#[test]
fn test_load_window_icon_from_bytes_full_png_succeeds() {
    // The full built-in PNG should always succeed
    let icon = load_window_icon_from_bytes(ICON_PNG_BYTES);
    assert!(icon.is_some());
}

// =============================================================================
// New: normalize_url additional edge cases
// =============================================================================

#[rstest]
#[case("localhost:5173/")]
#[case("localhost:8080/api/v1")]
#[case("127.0.0.1:3000")]
fn test_normalize_url_localhost_variants(#[case] url: &str) {
    let result = normalize_url(url);
    assert!(result.is_ok(), "Expected Ok for url={}, got {:?}", url, result);
}

#[test]
fn test_normalize_url_result_starts_with_http() {
    let result = normalize_url("example.com").unwrap();
    assert!(result.starts_with("http://") || result.starts_with("https://"));
}

#[test]
fn test_normalize_url_no_double_slash_in_path() {
    let result = normalize_url("example.com/path").unwrap();
    // Should not have double slashes in path portion
    let after_scheme = result.split("://").nth(1).unwrap_or("");
    assert!(!after_scheme.contains("//"), "No double slashes expected: {}", result);
}

#[rstest]
#[case("https://a.com")]
#[case("https://b.example.org/path")]
#[case("http://c.internal.corp/tool")]
fn test_normalize_url_absolute_url_unchanged_scheme(#[case] url: &str) {
    let result = normalize_url(url).unwrap();
    // Scheme should be preserved
    let expected_scheme = url.split("://").next().unwrap();
    assert!(result.starts_with(expected_scheme), "Scheme mismatch: expected {} in {}", expected_scheme, result);
}

#[test]
fn test_normalize_url_unicode_path() {
    // URLs with unicode in path should succeed or fail gracefully
    let result = normalize_url("example.com/日本語パス");
    // Either succeeds or fails cleanly (no panic)
    let _ = result;
}

