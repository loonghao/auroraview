//! Utility function tests

use auroraview_core::utils::{escape_js_string, parse_size};

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
