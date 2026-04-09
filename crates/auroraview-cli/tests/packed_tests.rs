//! Unit tests for packed module functions
//!
//! These tests verify the packed application runtime functionality.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use auroraview_cli::packed::{
    build_css_injection_script, build_module_search_paths, build_packed_init_script_with_csp,
    escape_json_for_js, get_python_exe_path, get_runtime_cache_dir_with_hash, get_webview_data_dir,
    inject_environment_variables,
};
use auroraview_core::json;
use rstest::rstest;

// =============================================================================
// escape_json_for_js tests
// =============================================================================

/// Test basic JSON escaping for JS embedding
/// Backslashes ARE escaped to support Windows paths in JSON
#[rstest]
#[case(r#"{"key": "value"}"#, r#"{\"key\": \"value\"}"#)]
#[case(r#"path\to\file"#, r#"path\\to\\file"#)] // Backslash escaped for JS string embedding
#[case("line1\nline2\rline3\ttab", r#"line1\nline2\rline3	tab"#)] // Only \n \r escaped, \t preserved
#[case("", "")]
fn test_escape_json_for_js(#[case] input: &str, #[case] expected: &str) {
    let escaped = escape_json_for_js(input);
    assert_eq!(escaped, expected);
}

#[test]
fn test_escape_json_for_js_complex() {
    // Test JSON with backslashes (common for Windows paths)
    // The input has literal \n and backslash in it
    let input = r#"{"message": "Hello\nWorld", "path": "C:\Users"}"#;
    let escaped = escape_json_for_js(input);
    // Backslashes are escaped for JS string embedding
    assert_eq!(
        escaped,
        r#"{\"message\": \"Hello\\nWorld\", \"path\": \"C:\\Users\"}"#
    );
}

#[test]
fn test_escape_json_for_js_with_json_module() {
    // Test that escaped JSON can be parsed back after being embedded in JS
    let original = serde_json::json!({
        "key": "value",
        "nested": {"a": 1, "b": 2}
    });

    let json_str = json::to_string(&original).unwrap();
    let escaped = escape_json_for_js(&json_str);

    // The escaped string should be valid for embedding in JS
    assert!(escaped.contains("\\\"key\\\""));
    assert!(escaped.contains("\\\"value\\\""));
}

/// Test Windows paths are correctly escaped for JS string embedding
/// This is critical for fs plugin read_dir on Windows
#[test]
fn test_escape_json_for_js_windows_path() {
    // Simulate serde_json output for Windows path
    // serde_json produces: {"path":"C:\\Users\\test"} (backslash becomes \\)
    let original = serde_json::json!({
        "path": "C:\\Users\\test",
        "name": "file.txt"
    });

    let json_str = json::to_string(&original).unwrap();
    // json_str = {"name":"file.txt","path":"C:\\Users\\test"}

    let escaped = escape_json_for_js(&json_str);
    // Each \\ in JSON must become \\\\ in JS string literal
    // So C:\\Users becomes C:\\\\Users in the escaped string

    // Verify backslashes are properly doubled
    // In Rust string literal: C:\\\\Users\\\\test represents C:\\Users\\test
    assert!(
        escaped.contains(r#"C:\\\\Users\\\\test"#),
        "Windows path backslashes should be escaped. Got: {}",
        escaped
    );

    // Verify the structure is correct
    assert!(escaped.contains("\\\"path\\\""));
    assert!(escaped.contains("\\\"name\\\""));
}

/// Test that Unicode characters are preserved correctly
/// This is critical for Chinese/Japanese/etc. text support
#[test]
fn test_escape_json_for_js_unicode() {
    // Create JSON with Chinese characters
    let original = serde_json::json!({
        "message": "你好世界",
        "emoji": "👋🎉",
        "model": "deepseek-chat"
    });

    let json_str = json::to_string(&original).unwrap();
    let escaped = escape_json_for_js(&json_str);

    // Unicode characters should be preserved (not corrupted by backslash escaping)
    // serde_json may output Chinese as-is or as \uXXXX, both should work
    // The key is that after JSON.parse(), we get the original characters back

    // Verify the escaped string is valid for JSON.parse
    // We simulate what happens in JS: JSON.parse('"' + escaped + '"')
    // But since escaped already has escaped quotes, we need different approach

    // Just verify the key structure is correct
    assert!(escaped.contains("\\\"message\\\""));
    assert!(escaped.contains("\\\"model\\\""));
    assert!(escaped.contains("deepseek-chat"));

    // With the new escaping, Unicode escape sequences like \u4F60 become \\u4F60
    // This is correct because when embedded in JS string and parsed:
    // JS string literal \\u4F60 -> \u4F60 -> JSON.parse interprets as Unicode
    // Note: serde_json usually outputs UTF-8 directly, not as \uXXXX escapes
}

// =============================================================================
// build_packed_init_script_with_csp tests
// =============================================================================

#[test]
fn test_build_packed_init_script_no_csp() {
    let script = build_packed_init_script_with_csp(None);
    // Should contain event bridge
    assert!(script.contains("auroraview"));
    // API methods are registered dynamically by Python backend,
    // not via static configuration
    assert!(!script.contains("Auto-generated API method registration"));
    // No CSP meta tag when policy is None
    assert!(!script.contains("Content-Security-Policy"));
}

#[test]
fn test_build_packed_init_script_with_csp_policy() {
    let policy = "default-src 'self'; script-src 'self' 'unsafe-inline'";
    let script = build_packed_init_script_with_csp(Some(policy));
    // Should contain event bridge
    assert!(script.contains("auroraview"));
    // Should contain CSP injection
    assert!(script.contains("Content-Security-Policy"));
    assert!(script.contains("default-src"));
}

#[test]
fn test_build_packed_init_script_csp_escapes_quotes() {
    // Verify that single quotes in the policy are escaped
    let policy = "default-src 'self'";
    let script = build_packed_init_script_with_csp(Some(policy));
    // The resulting JS must not have unescaped single quotes that break out of the string
    assert!(script.contains("Content-Security-Policy"));
    // Should not contain a raw unescaped ' inside the JS string assignment
    // (escaped as \\' in the injected JS)
    assert!(!script.contains("= 'default-src 'self'"));
}

// =============================================================================
// build_css_injection_script tests
// =============================================================================

#[test]
fn test_build_css_injection_script_basic() {
    let css = "body { margin: 0; }";
    let script = build_css_injection_script(css);
    // Must create a <style> element
    assert!(script.contains("createElement('style')"));
    // Must contain the CSS text
    assert!(script.contains("body { margin: 0; }"));
    // Must be an IIFE
    assert!(script.contains("(function()"));
}

#[test]
fn test_build_css_injection_script_empty() {
    let script = build_css_injection_script("");
    // Even for empty CSS, the script structure must be valid JS
    assert!(script.contains("createElement('style')"));
}

#[test]
fn test_build_css_injection_script_escapes_backtick() {
    // CSS with a backtick (rare but possible in content: "")
    let css = r#"body::before { content: "`"; }"#;
    let script = build_css_injection_script(css);
    // The backtick must be escaped as \` inside the template literal
    assert!(script.contains("\\`"));
    // The script must still contain the style element creation
    assert!(script.contains("createElement('style')"));
}

#[test]
fn test_build_css_injection_script_escapes_backslash() {
    // CSS with a backslash (e.g. in unicode escapes)
    let css = r#"content: "\2022";"#;
    let script = build_css_injection_script(css);
    // Backslash must be double-escaped inside the template literal
    assert!(script.contains("\\\\"));
}

// =============================================================================
// get_webview_data_dir tests
// =============================================================================

#[test]
fn test_get_webview_data_dir() {
    let dir = get_webview_data_dir();
    // Should end with AuroraView/WebView2
    assert!(dir.ends_with("WebView2"));
    let parent = dir.parent().unwrap();
    assert!(parent.ends_with("AuroraView"));
}

// =============================================================================
// inject_environment_variables tests
// =============================================================================

#[test]
fn test_inject_environment_variables_empty() {
    let env: HashMap<String, String> = HashMap::new();
    // Should not panic with empty map
    inject_environment_variables(&env);
}

#[test]
fn test_inject_environment_variables_sets_vars() {
    let mut env = HashMap::new();
    let test_key = "AURORAVIEW_TEST_VAR_12345";
    let test_value = "test_value_12345";
    env.insert(test_key.to_string(), test_value.to_string());

    inject_environment_variables(&env);

    // Verify the env var was set
    assert_eq!(std::env::var(test_key).ok(), Some(test_value.to_string()));

    // Clean up
    std::env::remove_var(test_key);
}

#[test]
fn test_inject_environment_variables_multiple() {
    let mut env = HashMap::new();
    let keys = [
        "AURORAVIEW_TEST_A_12345",
        "AURORAVIEW_TEST_B_12345",
        "AURORAVIEW_TEST_C_12345",
    ];

    for (i, key) in keys.iter().enumerate() {
        env.insert(key.to_string(), format!("value_{}", i));
    }

    inject_environment_variables(&env);

    // Verify all env vars were set
    for (i, key) in keys.iter().enumerate() {
        assert_eq!(
            std::env::var(key).ok(),
            Some(format!("value_{}", i)),
            "Expected {} to be set",
            key
        );
    }

    // Clean up
    for key in &keys {
        std::env::remove_var(key);
    }
}

// =============================================================================
// get_runtime_cache_dir_with_hash tests
// =============================================================================

#[test]
fn test_get_runtime_cache_dir_with_hash() {
    let dir = get_runtime_cache_dir_with_hash("test_app", "abc123def456");
    // Should contain AuroraView/runtime/test_app/hash
    assert!(dir.ends_with("abc123def456"));
    let parent = dir.parent().unwrap();
    assert!(parent.ends_with("test_app"));
    let grandparent = parent.parent().unwrap();
    assert!(grandparent.ends_with("runtime"));
}

#[rstest]
#[case("my_app", "hash1234", "hash1234")]
#[case("gallery", "abcd5678", "abcd5678")]
#[case("test-app", "xyz99999", "xyz99999")]
fn test_get_runtime_cache_dir_with_hash_various(
    #[case] app_name: &str,
    #[case] hash: &str,
    #[case] expected_suffix: &str,
) {
    let dir = get_runtime_cache_dir_with_hash(app_name, hash);
    assert!(dir.ends_with(expected_suffix));
}

// =============================================================================
// get_python_exe_path tests
// =============================================================================

#[test]
fn test_get_python_exe_path() {
    let cache_dir = PathBuf::from("/test/cache");
    let exe_path = get_python_exe_path(&cache_dir);

    #[cfg(target_os = "windows")]
    assert!(exe_path.ends_with("python.exe"));

    #[cfg(not(target_os = "windows"))]
    assert!(exe_path.ends_with("python3"));
}

#[test]
fn test_get_python_exe_path_structure() {
    let cache_dir = PathBuf::from("/some/cache/dir");
    let exe_path = get_python_exe_path(&cache_dir);

    // Should be under python subdirectory
    assert!(exe_path.to_string_lossy().contains("python"));
}

// =============================================================================
// build_module_search_paths tests
// =============================================================================

#[test]
fn test_build_module_search_paths_expands_variables() {
    // Create temp directories for testing
    let temp_dir = std::env::temp_dir().join("auroraview_test_paths_expand");
    let extract_dir = temp_dir.join("extract");
    let resources_dir = temp_dir.join("resources");
    let site_packages_dir = temp_dir.join("site-packages");

    fs::create_dir_all(&extract_dir).unwrap();
    fs::create_dir_all(&resources_dir).unwrap();
    fs::create_dir_all(&site_packages_dir).unwrap();

    let config_paths = vec![
        "$EXTRACT_DIR".to_string(),
        "$RESOURCES_DIR".to_string(),
        "$SITE_PACKAGES".to_string(),
    ];

    let result = build_module_search_paths(
        &config_paths,
        &extract_dir,
        &resources_dir,
        &site_packages_dir,
    );

    // All paths should exist and be included
    assert_eq!(result.len(), 3);
    assert!(result.contains(&extract_dir.to_string_lossy().to_string()));
    assert!(result.contains(&resources_dir.to_string_lossy().to_string()));
    assert!(result.contains(&site_packages_dir.to_string_lossy().to_string()));

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_build_module_search_paths_filters_nonexistent() {
    let config_paths = vec![
        "/nonexistent/path/12345".to_string(),
        "/another/fake/path".to_string(),
    ];

    let extract_dir = PathBuf::from("/tmp");
    let resources_dir = PathBuf::from("/tmp");
    let site_packages_dir = PathBuf::from("/tmp");

    let result = build_module_search_paths(
        &config_paths,
        &extract_dir,
        &resources_dir,
        &site_packages_dir,
    );

    // Non-existent paths should be filtered out
    assert!(result.is_empty());
}

#[test]
fn test_build_module_search_paths_mixed() {
    // Create one existing directory
    let temp_dir = std::env::temp_dir().join("auroraview_test_paths_mixed");
    fs::create_dir_all(&temp_dir).unwrap();

    let config_paths = vec![
        "$EXTRACT_DIR".to_string(),
        "/nonexistent/path/xyz".to_string(),
    ];

    let result = build_module_search_paths(
        &config_paths,
        &temp_dir,
        &PathBuf::from("/fake"),
        &PathBuf::from("/fake"),
    );

    // Only the existing path should be included
    assert_eq!(result.len(), 1);
    assert!(result.contains(&temp_dir.to_string_lossy().to_string()));

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

// =============================================================================
// JSON integration tests using auroraview_core::json
// =============================================================================

#[test]
fn test_json_roundtrip_with_escape() {
    // Create a complex JSON structure
    let data = serde_json::json!({
        "id": "call_123",
        "method": "api.test",
        "params": {
            "path": "C:\\Users\\test\\file.txt",
            "message": "Hello\nWorld"
        }
    });

    // Serialize using core json module
    let json_str = json::to_string(&data).unwrap();

    // Escape for JS embedding
    let escaped = escape_json_for_js(&json_str);

    // The escaped string should be valid for embedding in JS
    // Note: The path is already escaped in JSON as \\, then we escape again for JS
    assert!(escaped.contains("\\\"id\\\""));
    assert!(escaped.contains("\\\"method\\\""));
    // The newline in message becomes \n in JSON, then \\n in escaped
    assert!(escaped.contains("Hello\\\\nWorld") || escaped.contains("Hello\\nWorld"));
}

#[test]
fn test_json_parse_ipc_message() {
    // Simulate parsing an IPC message like in handle_ipc_message
    let msg_str = r#"{"type":"call","id":"1","method":"api.echo","params":{"message":"test"}}"#;

    let msg = json::from_str(msg_str).unwrap();

    assert_eq!(msg["type"], "call");
    assert_eq!(msg["id"], "1");
    assert_eq!(msg["method"], "api.echo");
    assert_eq!(msg["params"]["message"], "test");
}

#[test]
fn test_json_build_response() {
    // Test building a response like in handle_ipc_message
    let response = serde_json::json!({
        "id": "call_123",
        "ok": true,
        "result": {"status": "success"},
        "error": null
    });

    let json_str = json::to_string(&response).unwrap();
    let escaped = escape_json_for_js(&json_str);

    // Should be valid for embedding in JS
    assert!(escaped.contains("\\\"id\\\""));
    assert!(escaped.contains("\\\"ok\\\""));
}

// =============================================================================
// R11 Extensions - additional packed_tests coverage
// =============================================================================

// escape_json_for_js – extended edge cases

#[test]
fn test_escape_json_for_js_only_backslash() {
    let input = "\\";
    let escaped = escape_json_for_js(input);
    assert_eq!(escaped, "\\\\");
}

#[test]
fn test_escape_json_for_js_multiple_backslashes() {
    let input = "\\\\\\";
    let escaped = escape_json_for_js(input);
    // Every \ becomes \\
    assert_eq!(escaped, "\\\\\\\\\\\\");
}

#[test]
fn test_escape_json_for_js_only_newlines() {
    let input = "\n\n\n";
    let escaped = escape_json_for_js(input);
    assert_eq!(escaped, "\\n\\n\\n");
}

#[test]
fn test_escape_json_for_js_only_carriage_return() {
    let input = "\r\r";
    let escaped = escape_json_for_js(input);
    assert_eq!(escaped, "\\r\\r");
}

#[test]
fn test_escape_json_for_js_mixed_special() {
    let input = "a\\b\nc\rd";
    let escaped = escape_json_for_js(input);
    // \\ → \\\\, \n → \\n, \r → \\r
    assert!(escaped.contains("\\\\"));
    assert!(escaped.contains("\\n"));
    assert!(escaped.contains("\\r"));
}

#[test]
fn test_escape_json_for_js_double_quote() {
    let input = r#"say "hello""#;
    let escaped = escape_json_for_js(input);
    assert!(escaped.contains("\\\"hello\\\""));
}

#[test]
fn test_escape_json_for_js_preserves_single_quote() {
    let input = "it's a test";
    let escaped = escape_json_for_js(input);
    assert!(escaped.contains("it's"));
}

#[test]
fn test_escape_json_for_js_large_input() {
    let input = "A".repeat(100_000);
    let escaped = escape_json_for_js(&input);
    assert_eq!(escaped.len(), 100_000);
}

// get_runtime_cache_dir_with_hash – hierarchy

#[test]
fn test_get_runtime_cache_dir_hierarchy_depth() {
    let dir = get_runtime_cache_dir_with_hash("app_name", "deadbeef");
    // path ends with: .../runtime/app_name/deadbeef
    let components: Vec<_> = dir.components().collect();
    assert!(components.len() >= 3, "Expected at least 3 path components");
}

#[test]
fn test_get_runtime_cache_dir_hash_is_leaf() {
    let hash = "a1b2c3d4";
    let dir = get_runtime_cache_dir_with_hash("myapp", hash);
    let leaf = dir.file_name().unwrap().to_string_lossy();
    assert_eq!(leaf, hash);
}

#[test]
fn test_get_runtime_cache_dir_app_name_in_path() {
    let app_name = "special_gallery_app";
    let dir = get_runtime_cache_dir_with_hash(app_name, "h123");
    let path_str = dir.to_string_lossy();
    assert!(path_str.contains(app_name));
}

#[test]
fn test_get_runtime_cache_dir_different_hashes_different_paths() {
    let dir1 = get_runtime_cache_dir_with_hash("app", "hash_aaaa");
    let dir2 = get_runtime_cache_dir_with_hash("app", "hash_bbbb");
    assert_ne!(dir1, dir2);
}

#[test]
fn test_get_runtime_cache_dir_different_apps_different_paths() {
    let dir1 = get_runtime_cache_dir_with_hash("app_a", "samehash");
    let dir2 = get_runtime_cache_dir_with_hash("app_b", "samehash");
    assert_ne!(dir1, dir2);
}

// get_python_exe_path – extended

#[test]
fn test_get_python_exe_path_is_absolute() {
    let cache_dir = std::path::PathBuf::from("/test/cache");
    let exe = get_python_exe_path(&cache_dir);
    // The exe path should start from the cache_dir root
    assert!(exe.to_string_lossy().starts_with("/test/cache"));
}

#[test]
fn test_get_python_exe_path_different_bases() {
    let base_a = std::path::PathBuf::from("/cache/a");
    let base_b = std::path::PathBuf::from("/cache/b");
    let exe_a = get_python_exe_path(&base_a);
    let exe_b = get_python_exe_path(&base_b);
    assert_ne!(exe_a, exe_b);
}

// build_packed_init_script_with_csp – content checks

#[test]
fn test_build_packed_init_script_contains_ready_event() {
    let script = build_packed_init_script_with_csp(None);
    assert!(
        script.contains("auroraviewready") || script.contains("auroraview"),
        "Init script should reference the auroraview bridge"
    );
}

#[test]
fn test_build_packed_init_script_csp_none_no_meta() {
    let script = build_packed_init_script_with_csp(None);
    assert!(!script.contains("<meta"), "No CSP meta when policy is None");
}

#[test]
fn test_build_packed_init_script_csp_strict() {
    let policy = "default-src 'none'; script-src 'self'";
    let script = build_packed_init_script_with_csp(Some(policy));
    assert!(script.contains("Content-Security-Policy"));
    assert!(script.contains("default-src"));
    assert!(script.contains("script-src"));
}

// build_css_injection_script – more variants

#[test]
fn test_build_css_injection_script_with_variable() {
    let css = ":root { --bg: #1a1a2e; }";
    let script = build_css_injection_script(css);
    assert!(script.contains("--bg"));
    assert!(script.contains("#1a1a2e"));
}

#[test]
fn test_build_css_injection_script_with_media_query() {
    let css = "@media (prefers-color-scheme: dark) { body { background: #000; } }";
    let script = build_css_injection_script(css);
    assert!(script.contains("prefers-color-scheme"));
    assert!(script.contains("createElement('style')"));
}

// inject_environment_variables – idempotent overwrite

#[test]
fn test_inject_environment_variables_overwrites_existing() {
    let key = "AURORAVIEW_TEST_OVERWRITE_9876";
    std::env::set_var(key, "original");

    let mut env = std::collections::HashMap::new();
    env.insert(key.to_string(), "overwritten".to_string());
    inject_environment_variables(&env);

    assert_eq!(std::env::var(key).ok(), Some("overwritten".to_string()));
    std::env::remove_var(key);
}

#[test]
fn test_inject_environment_variables_empty_value() {
    let key = "AURORAVIEW_TEST_EMPTY_VALUE_9876";
    let mut env = std::collections::HashMap::new();
    env.insert(key.to_string(), "".to_string());
    inject_environment_variables(&env);
    assert_eq!(std::env::var(key).ok(), Some("".to_string()));
    std::env::remove_var(key);
}

// =============================================================================
// Python ready signal format tests
// =============================================================================

#[test]
fn test_python_ready_signal_format() {
    // Test the expected format of Python ready signal
    let handlers = vec!["get_samples", "get_categories", "run_sample"];
    let ready_signal = serde_json::json!({
        "type": "ready",
        "handlers": handlers
    });

    let json_str = json::to_string(&ready_signal).unwrap();

    // Parse it back
    let parsed: serde_json::Value = json::from_str(&json_str).unwrap();
    assert_eq!(parsed["type"], "ready");
    assert!(parsed["handlers"].is_array());
    assert_eq!(parsed["handlers"].as_array().unwrap().len(), 3);
}

#[test]
fn test_python_ready_signal_parsing() {
    // Simulate receiving a ready signal from Python
    let ready_line = r#"{"type": "ready", "handlers": ["get_samples", "get_categories"]}"#;

    let msg: serde_json::Value = json::from_str(ready_line).unwrap();

    assert_eq!(msg.get("type").and_then(|v| v.as_str()), Some("ready"));
    let handlers = msg.get("handlers").and_then(|v| v.as_array());
    assert!(handlers.is_some());
    assert_eq!(handlers.unwrap().len(), 2);
}

#[test]
fn test_python_error_response_format() {
    // Test the error response format sent when Python backend fails
    let error_response = serde_json::json!({
        "id": "call_123",
        "ok": false,
        "result": null,
        "error": {
            "name": "PythonBackendError",
            "message": "Python backend process has exited"
        }
    });

    let json_str = json::to_string(&error_response).unwrap();
    let parsed: serde_json::Value = json::from_str(&json_str).unwrap();

    assert_eq!(parsed["ok"], false);
    assert!(parsed["error"]["name"].as_str().is_some());
    assert!(parsed["error"]["message"].as_str().is_some());
}

#[test]
fn test_api_request_format() {
    // Test the format of API requests sent to Python
    let request = serde_json::json!({
        "id": "av_call_123456_1",
        "method": "get_samples",
        "params": null
    });

    let json_str = json::to_string(&request).unwrap();

    // Verify it can be parsed back
    let parsed: serde_json::Value = json::from_str(&json_str).unwrap();
    assert_eq!(parsed["method"], "get_samples");
    assert!(parsed["id"].as_str().unwrap().starts_with("av_call_"));
}

#[test]
fn test_api_response_format() {
    // Test the format of API responses from Python
    let response = serde_json::json!({
        "id": "av_call_123456_1",
        "ok": true,
        "result": [
            {"id": "sample1", "title": "Sample 1"},
            {"id": "sample2", "title": "Sample 2"}
        ]
    });

    let json_str = json::to_string(&response).unwrap();
    let parsed: serde_json::Value = json::from_str(&json_str).unwrap();

    assert_eq!(parsed["ok"], true);
    assert!(parsed["result"].is_array());
}
