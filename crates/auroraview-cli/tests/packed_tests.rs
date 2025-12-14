//! Unit tests for packed module functions
//!
//! These tests verify the packed application runtime functionality.

use auroraview_cli::packed::{
    build_module_search_paths, build_packed_init_script, escape_json_for_js, get_python_exe_path,
    get_runtime_cache_dir, get_webview_data_dir, inject_environment_variables,
};
use auroraview_core::json;
use rstest::rstest;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// =============================================================================
// escape_json_for_js tests
// =============================================================================

#[rstest]
#[case(r#"{"key": "value"}"#, r#"{\"key\": \"value\"}"#)]
#[case(r#"path\to\file"#, r#"path\\to\\file"#)]
#[case("line1\nline2\rline3\ttab", r#"line1\nline2\rline3\ttab"#)]
#[case("", "")]
fn test_escape_json_for_js(#[case] input: &str, #[case] expected: &str) {
    let escaped = escape_json_for_js(input);
    assert_eq!(escaped, expected);
}

#[test]
fn test_escape_json_for_js_complex() {
    let input = r#"{"message": "Hello\nWorld", "path": "C:\Users"}"#;
    let escaped = escape_json_for_js(input);
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

// =============================================================================
// build_packed_init_script tests
// =============================================================================

#[test]
fn test_build_packed_init_script_frontend_mode() {
    let script = build_packed_init_script(false);
    // Frontend mode should NOT contain Gallery API registration
    assert!(!script.contains("Register Gallery API methods"));
    assert!(!script.contains("'get_samples'"));
}

#[test]
fn test_build_packed_init_script_fullstack_mode() {
    let script = build_packed_init_script(true);
    // Should contain API registration for fullstack mode
    assert!(script.contains("Register Gallery API methods"));
    assert!(script.contains("get_samples"));
    assert!(script.contains("run_sample"));
}

#[rstest]
#[case(true, &["Register Gallery API methods", "get_samples", "run_sample", "kill_process"])]
#[case(false, &[])]
fn test_build_packed_init_script_api_methods(
    #[case] is_fullstack: bool,
    #[case] expected_methods: &[&str],
) {
    let script = build_packed_init_script(is_fullstack);

    for method in expected_methods {
        assert!(
            script.contains(method),
            "Expected script to contain '{}' for fullstack={}",
            method,
            is_fullstack
        );
    }

    if !is_fullstack {
        // Frontend mode should not have Gallery-specific API registration
        assert!(!script.contains("Register Gallery API methods"));
    }
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
// get_runtime_cache_dir tests
// =============================================================================

#[test]
fn test_get_runtime_cache_dir() {
    let dir = get_runtime_cache_dir("test_app");
    // Should contain AuroraView/runtime/test_app
    assert!(dir.ends_with("test_app"));
    let parent = dir.parent().unwrap();
    assert!(parent.ends_with("runtime"));
}

#[rstest]
#[case("my_app", "my_app")]
#[case("gallery", "gallery")]
#[case("test-app", "test-app")]
fn test_get_runtime_cache_dir_various_names(#[case] app_name: &str, #[case] expected_suffix: &str) {
    let dir = get_runtime_cache_dir(app_name);
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
