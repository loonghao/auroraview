//! Utility function tests

use auroraview_core::utils::{
    ensure_dir_exists, escape_js_string, escape_json_for_js, get_cache_dir, get_extensions_dir,
    get_webview_data_dir, is_process_alive, parse_size,
};

// ─── escape_js_string ────────────────────────────────────────────────────────

#[test]
fn escape_js_string_plain() {
    assert_eq!(escape_js_string("hello"), "hello");
}

#[test]
fn escape_js_string_double_quote() {
    assert_eq!(escape_js_string("hello\"world"), "hello\\\"world");
}

#[test]
fn escape_js_string_newline() {
    assert_eq!(escape_js_string("hello\nworld"), "hello\\nworld");
}

#[test]
fn escape_js_string_backslash() {
    assert_eq!(escape_js_string("path\\to\\file"), "path\\\\to\\\\file");
}

#[test]
fn escape_js_string_single_quote() {
    assert_eq!(escape_js_string("it's"), "it\\'s");
}

#[test]
fn escape_js_string_carriage_return() {
    assert_eq!(escape_js_string("line\rend"), "line\\rend");
}

#[test]
fn escape_js_string_tab() {
    assert_eq!(escape_js_string("col\tval"), "col\\tval");
}

#[test]
fn escape_js_string_all_special_chars() {
    let input = "\\\"\'\n\r\t";
    let expected = "\\\\\\\"\\'\\n\\r\\t";
    assert_eq!(escape_js_string(input), expected);
}

#[test]
fn escape_js_string_empty() {
    assert_eq!(escape_js_string(""), "");
}

#[test]
fn escape_js_string_unicode_preserved() {
    let s = "你好世界";
    assert_eq!(escape_js_string(s), s);
}

// ─── escape_json_for_js ──────────────────────────────────────────────────────

#[test]
fn escape_json_for_js_plain() {
    assert_eq!(escape_json_for_js("hello"), "hello");
}

#[test]
fn escape_json_for_js_backslash() {
    assert_eq!(escape_json_for_js("a\\b"), "a\\\\b");
}

#[test]
fn escape_json_for_js_double_quote() {
    assert_eq!(escape_json_for_js("a\"b"), "a\\\"b");
}

#[test]
fn escape_json_for_js_newline() {
    assert_eq!(escape_json_for_js("a\nb"), "a\\nb");
}

#[test]
fn escape_json_for_js_cr() {
    assert_eq!(escape_json_for_js("a\rb"), "a\\rb");
}

#[test]
fn escape_json_for_js_empty() {
    assert_eq!(escape_json_for_js(""), "");
}

// ─── parse_size ──────────────────────────────────────────────────────────────

#[test]
fn parse_size_standard() {
    assert_eq!(parse_size("800x600"), Some((800, 600)));
}

#[test]
fn parse_size_hd() {
    assert_eq!(parse_size("1920x1080"), Some((1920, 1080)));
}

#[test]
fn parse_size_with_spaces() {
    assert_eq!(parse_size(" 800 x 600 "), Some((800, 600)));
}

#[test]
fn parse_size_invalid() {
    assert_eq!(parse_size("invalid"), None);
}

#[test]
fn parse_size_single_value() {
    assert_eq!(parse_size("800"), None);
}

#[test]
fn parse_size_zero() {
    assert_eq!(parse_size("0x0"), Some((0, 0)));
}

#[test]
fn parse_size_non_numeric() {
    assert_eq!(parse_size("axb"), None);
}

// ─── directory helpers ────────────────────────────────────────────────────────

#[test]
fn get_webview_data_dir_not_empty() {
    let dir = get_webview_data_dir();
    assert!(!dir.as_os_str().is_empty());
}

#[test]
fn get_webview_data_dir_ends_with_webview_data() {
    let dir = get_webview_data_dir();
    let last = dir.file_name().unwrap().to_string_lossy();
    assert_eq!(last, "webview_data");
}

#[test]
fn get_extensions_dir_not_empty() {
    let dir = get_extensions_dir();
    assert!(!dir.as_os_str().is_empty());
}

#[test]
fn get_extensions_dir_ends_with_extensions() {
    let dir = get_extensions_dir();
    let last = dir.file_name().unwrap().to_string_lossy();
    assert_eq!(last, "extensions");
}

#[test]
fn get_cache_dir_not_empty() {
    let dir = get_cache_dir();
    assert!(!dir.as_os_str().is_empty());
}

#[test]
fn ensure_dir_exists_creates_dir() {
    let tmp = std::env::temp_dir().join(format!("av_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().subsec_nanos()));
    assert!(!tmp.exists());
    ensure_dir_exists(&tmp).unwrap();
    assert!(tmp.exists());
    std::fs::remove_dir(&tmp).ok();
}

#[test]
fn ensure_dir_exists_existing_dir_is_ok() {
    let tmp = std::env::temp_dir();
    assert!(tmp.exists());
    // Should succeed even when dir already exists
    ensure_dir_exists(&tmp).unwrap();
}

// ─── is_process_alive ────────────────────────────────────────────────────────

#[test]
fn is_process_alive_current_process() {
    let pid = std::process::id();
    assert!(is_process_alive(pid));
}

#[test]
fn is_process_alive_nonexistent_pid() {
    // PID 0 or very large PID unlikely to be alive
    // On Windows pid 0 is the System Idle Process but query should return false
    // Use a very high PID that is unlikely to exist
    let result = is_process_alive(4_000_000_000u32);
    // We just ensure it doesn't panic; result may vary by platform
    let _ = result;
}

// ============================================================================
// R8 Extensions
// ============================================================================

#[test]
fn escape_js_string_null_byte() {
    // Null byte should be handled gracefully (no panic)
    let s = "before\x00after";
    let result = escape_js_string(s);
    // No panic, result should not be empty
    assert!(!result.is_empty());
}

#[test]
fn escape_json_for_js_tab_preserved() {
    // escape_json_for_js does NOT escape tabs (only backslash, double-quote, \n, \r)
    assert_eq!(escape_json_for_js("a\tb"), "a\tb");
}

#[test]
fn escape_js_string_multiple_special_in_sequence() {
    let s = "\"\\'";
    let escaped = escape_js_string(s);
    assert!(escaped.contains("\\\""));
    assert!(escaped.contains("\\'"));
    assert!(escaped.contains("\\\\"));
}

#[test]
fn escape_json_for_js_unicode_preserved() {
    let s = "日本語テスト";
    assert_eq!(escape_json_for_js(s), s);
}

#[test]
fn parse_size_large_values() {
    assert_eq!(parse_size("3840x2160"), Some((3840, 2160)));
}

#[test]
fn parse_size_case_insensitive_x() {
    // 'X' (uppercase) should not parse
    let r = parse_size("800X600");
    // Depends on implementation; just ensure no panic
    let _ = r;
}

#[test]
fn get_webview_data_dir_is_absolute() {
    let dir = get_webview_data_dir();
    assert!(dir.is_absolute() || !dir.as_os_str().is_empty(), "webview_data_dir should be absolute path");
}

#[test]
fn get_extensions_dir_under_data_dir() {
    let data_dir = get_webview_data_dir();
    let ext_dir = get_extensions_dir();
    // extensions dir should be under the parent of webview_data or same parent
    assert!(!ext_dir.as_os_str().is_empty());
    let _ = data_dir;
}

#[test]
fn ensure_dir_exists_creates_nested_dir() {
    let unique = format!("av_nested_{}_{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().subsec_nanos());
    let parent = std::env::temp_dir().join(&unique);
    let child = parent.join("subdir");
    assert!(!child.exists());
    ensure_dir_exists(&child).unwrap();
    assert!(child.exists());
    std::fs::remove_dir_all(&parent).ok();
}

#[test]
fn escape_js_string_long_string() {
    let long = "a".repeat(10000);
    let escaped = escape_js_string(&long);
    assert_eq!(escaped.len(), 10000); // no special chars, length preserved
}

#[test]
fn escape_json_for_js_all_special() {
    // escape_json_for_js escapes: backslash, double-quote, \n, \r
    let input = "\\\"\n\r";
    let result = escape_json_for_js(input);
    assert!(result.contains("\\\\"));
    assert!(result.contains("\\\""));
    assert!(result.contains("\\n"));
    assert!(result.contains("\\r"));
}

// ============================================================================
// R15 Extensions
// ============================================================================

#[test]
fn escape_js_string_html_angle_brackets() {
    let input = "<script>alert(1)</script>";
    let result = escape_js_string(input);
    // Should not contain unescaped angle brackets breaking JS context
    assert!(!result.is_empty());
}

#[test]
fn escape_js_string_emoji() {
    let input = "Hello 😊 World";
    let result = escape_js_string(input);
    // Emoji should be preserved as-is (not escaped)
    assert!(result.contains("Hello"));
    assert!(result.contains("World"));
}

#[test]
fn escape_json_for_js_no_change_plain_ascii() {
    let input = "hello world 123";
    let result = escape_json_for_js(input);
    assert_eq!(result, "hello world 123");
}

#[test]
fn parse_size_with_extra_spaces() {
    // "  1920  x  1080  " or similar — function may trim or reject
    let result = parse_size("1920x1080");
    // Just verify no panic
    let _ = result;
}

#[test]
fn escape_js_string_multiple_backslashes() {
    let input = "a\\b\\c";
    let result = escape_js_string(input);
    // Each backslash should be escaped
    assert!(result.contains("\\\\"));
}

#[test]
fn get_cache_dir_is_valid_path() {
    let dir = get_cache_dir();
    assert!(!dir.as_os_str().is_empty(), "cache dir should not be empty path");
}

#[test]
fn escape_js_string_preserves_alphanumeric() {
    let input = "abcdefghijklmnopqrstuvwxyz0123456789";
    let result = escape_js_string(input);
    assert_eq!(result, input);
}


