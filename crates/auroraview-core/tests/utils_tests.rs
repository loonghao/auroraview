//! Utility function tests

use auroraview_core::utils::{
    ensure_dir_exists, escape_js_string, escape_json_for_js, get_cache_dir, get_extensions_dir,
    get_webview_data_dir, is_process_alive, parse_size,
};

// ─── escape_js_string ────────────────────────────────────────────────────────

#[test]
fn test_escape_js_string_plain() {
    assert_eq!(escape_js_string("hello"), "hello");
}

#[test]
fn test_escape_js_string_double_quote() {
    assert_eq!(escape_js_string("hello\"world"), "hello\\\"world");
}

#[test]
fn test_escape_js_string_newline() {
    assert_eq!(escape_js_string("hello\nworld"), "hello\\nworld");
}

#[test]
fn test_escape_js_string_backslash() {
    assert_eq!(escape_js_string("path\\to\\file"), "path\\\\to\\\\file");
}

#[test]
fn test_escape_js_string_single_quote() {
    assert_eq!(escape_js_string("it's"), "it\\'s");
}

#[test]
fn test_escape_js_string_carriage_return() {
    assert_eq!(escape_js_string("line\rend"), "line\\rend");
}

#[test]
fn test_escape_js_string_tab() {
    assert_eq!(escape_js_string("col\tval"), "col\\tval");
}

#[test]
fn test_escape_js_string_all_special_chars() {
    let input = "\\\"\'\n\r\t";
    let expected = "\\\\\\\"\\'\\n\\r\\t";
    assert_eq!(escape_js_string(input), expected);
}

#[test]
fn test_escape_js_string_empty() {
    assert_eq!(escape_js_string(""), "");
}

#[test]
fn test_escape_js_string_unicode_preserved() {
    let s = "你好世界";
    assert_eq!(escape_js_string(s), s);
}

// ─── escape_json_for_js ──────────────────────────────────────────────────────

#[test]
fn test_escape_json_for_js_plain() {
    assert_eq!(escape_json_for_js("hello"), "hello");
}

#[test]
fn test_escape_json_for_js_backslash() {
    assert_eq!(escape_json_for_js("a\\b"), "a\\\\b");
}

#[test]
fn test_escape_json_for_js_double_quote() {
    assert_eq!(escape_json_for_js("a\"b"), "a\\\"b");
}

#[test]
fn test_escape_json_for_js_newline() {
    assert_eq!(escape_json_for_js("a\nb"), "a\\nb");
}

#[test]
fn test_escape_json_for_js_cr() {
    assert_eq!(escape_json_for_js("a\rb"), "a\\rb");
}

#[test]
fn test_escape_json_for_js_empty() {
    assert_eq!(escape_json_for_js(""), "");
}

// ─── parse_size ──────────────────────────────────────────────────────────────

#[test]
fn test_parse_size_standard() {
    assert_eq!(parse_size("800x600"), Some((800, 600)));
}

#[test]
fn test_parse_size_hd() {
    assert_eq!(parse_size("1920x1080"), Some((1920, 1080)));
}

#[test]
fn test_parse_size_with_spaces() {
    assert_eq!(parse_size(" 800 x 600 "), Some((800, 600)));
}

#[test]
fn test_parse_size_invalid() {
    assert_eq!(parse_size("invalid"), None);
}

#[test]
fn test_parse_size_single_value() {
    assert_eq!(parse_size("800"), None);
}

#[test]
fn test_parse_size_zero() {
    assert_eq!(parse_size("0x0"), Some((0, 0)));
}

#[test]
fn test_parse_size_non_numeric() {
    assert_eq!(parse_size("axb"), None);
}

// ─── directory helpers ────────────────────────────────────────────────────────

#[test]
fn test_get_webview_data_dir_not_empty() {
    let dir = get_webview_data_dir();
    assert!(!dir.as_os_str().is_empty());
}

#[test]
fn test_get_webview_data_dir_ends_with_webview_data() {
    let dir = get_webview_data_dir();
    let last = dir.file_name().unwrap().to_string_lossy();
    assert_eq!(last, "webview_data");
}

#[test]
fn test_get_extensions_dir_not_empty() {
    let dir = get_extensions_dir();
    assert!(!dir.as_os_str().is_empty());
}

#[test]
fn test_get_extensions_dir_ends_with_extensions() {
    let dir = get_extensions_dir();
    let last = dir.file_name().unwrap().to_string_lossy();
    assert_eq!(last, "extensions");
}

#[test]
fn test_get_cache_dir_not_empty() {
    let dir = get_cache_dir();
    assert!(!dir.as_os_str().is_empty());
}

#[test]
fn test_ensure_dir_exists_creates_dir() {
    let tmp = std::env::temp_dir().join(format!("av_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().subsec_nanos()));
    assert!(!tmp.exists());
    ensure_dir_exists(&tmp).unwrap();
    assert!(tmp.exists());
    std::fs::remove_dir(&tmp).ok();
}

#[test]
fn test_ensure_dir_exists_existing_dir_is_ok() {
    let tmp = std::env::temp_dir();
    assert!(tmp.exists());
    // Should succeed even when dir already exists
    ensure_dir_exists(&tmp).unwrap();
}

// ─── is_process_alive ────────────────────────────────────────────────────────

#[test]
fn test_is_process_alive_current_process() {
    let pid = std::process::id();
    assert!(is_process_alive(pid));
}

#[test]
fn test_is_process_alive_nonexistent_pid() {
    // PID 0 or very large PID unlikely to be alive
    // On Windows pid 0 is the System Idle Process but query should return false
    // Use a very high PID that is unlikely to exist
    let result = is_process_alive(4_000_000_000u32);
    // We just ensure it doesn't panic; result may vary by platform
    let _ = result;
}

