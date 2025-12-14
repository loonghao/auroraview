//! Unit tests for auroraview-cli lib module
//!
//! These tests verify the core library functions.

use auroraview_cli::{load_window_icon, normalize_url, ICON_PNG_BYTES};
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
fn test_load_window_icon_succeeds() {
    let icon = load_window_icon();
    assert!(icon.is_some());
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
