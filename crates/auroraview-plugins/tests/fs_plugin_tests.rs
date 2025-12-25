//! Unit tests for file system plugin
//!
//! Tests for FsPlugin commands and operations.

use auroraview_plugins::fs::FsPlugin;
use auroraview_plugins::{
    create_router_with_scope, PathScope, PluginHandler, PluginRequest, ScopeConfig,
};
use tempfile::tempdir;

#[test]
fn test_fs_plugin_commands() {
    let plugin = FsPlugin::new();
    let commands = plugin.commands();
    assert!(commands.contains(&"read_file"));
    assert!(commands.contains(&"read_file_binary"));
    assert!(commands.contains(&"write_file"));
    assert!(commands.contains(&"write_file_binary"));
    assert!(commands.contains(&"read_dir"));
    assert!(commands.contains(&"create_dir"));
    assert!(commands.contains(&"remove"));
    assert!(commands.contains(&"copy"));
    assert!(commands.contains(&"rename"));
    assert!(commands.contains(&"exists"));
    assert!(commands.contains(&"stat"));
}

#[test]
fn test_fs_plugin_name() {
    let plugin = FsPlugin::new();
    assert_eq!(plugin.name(), "fs");
}

#[test]
fn test_fs_plugin_default() {
    let plugin = FsPlugin::default();
    assert_eq!(plugin.name(), "fs");
}

#[test]
fn test_write_and_read_file() {
    let temp = tempdir().unwrap();
    let scope = ScopeConfig::new().with_fs_scope(PathScope::new().allow(temp.path()));
    let router = create_router_with_scope(scope);

    let file_path = temp.path().join("test.txt");
    let file_path_str = file_path.to_string_lossy().to_string();

    // Write file
    let write_req = PluginRequest::new(
        "fs",
        "write_file",
        serde_json::json!({
            "path": file_path_str,
            "contents": "Hello, AuroraView!"
        }),
    );
    let write_resp = router.handle(write_req);
    assert!(write_resp.success, "Write failed: {:?}", write_resp.error);

    // Read file
    let read_req = PluginRequest::new(
        "fs",
        "read_file",
        serde_json::json!({ "path": file_path_str }),
    );
    let read_resp = router.handle(read_req);
    assert!(read_resp.success, "Read failed: {:?}", read_resp.error);
    assert_eq!(read_resp.data.unwrap(), "Hello, AuroraView!");
}

#[test]
fn test_exists_command() {
    let temp = tempdir().unwrap();
    let scope = ScopeConfig::new().with_fs_scope(PathScope::new().allow(temp.path()));
    let router = create_router_with_scope(scope);

    // Create a file
    let file_path = temp.path().join("exists_test.txt");
    std::fs::write(&file_path, "test").unwrap();

    // Check exists
    let req = PluginRequest::new(
        "fs",
        "exists",
        serde_json::json!({ "path": file_path.to_string_lossy() }),
    );
    let resp = router.handle(req);
    assert!(resp.success);
    let data = resp.data.unwrap();
    assert_eq!(data["exists"], true);
}

#[test]
fn test_exists_nonexistent() {
    let temp = tempdir().unwrap();
    let scope = ScopeConfig::new().with_fs_scope(PathScope::new().allow(temp.path()));
    let router = create_router_with_scope(scope);

    let file_path = temp.path().join("nonexistent.txt");

    let req = PluginRequest::new(
        "fs",
        "exists",
        serde_json::json!({ "path": file_path.to_string_lossy() }),
    );
    let resp = router.handle(req);
    assert!(resp.success);
    let data = resp.data.unwrap();
    assert_eq!(data["exists"], false);
}

#[test]
fn test_scope_violation() {
    let temp = tempdir().unwrap();
    let scope = ScopeConfig::new().with_fs_scope(PathScope::new().allow(temp.path()));
    let router = create_router_with_scope(scope);

    // Try to read outside scope (should fail)
    let req = PluginRequest::new(
        "fs",
        "read_file",
        serde_json::json!({ "path": "C:\\Windows\\System32\\config.sys" }),
    );
    let resp = router.handle(req);
    assert!(!resp.success);
    assert_eq!(resp.code, Some("SCOPE_VIOLATION".to_string()));
}

#[test]
fn test_create_and_read_dir() {
    let temp = tempdir().unwrap();
    let scope = ScopeConfig::new().with_fs_scope(PathScope::new().allow(temp.path()));
    let router = create_router_with_scope(scope);

    // Create directory
    let dir_path = temp.path().join("new_dir");
    let req = PluginRequest::new(
        "fs",
        "create_dir",
        serde_json::json!({ "path": dir_path.to_string_lossy() }),
    );
    let resp = router.handle(req);
    assert!(resp.success);

    // Create a file in the directory
    let file_path = dir_path.join("test.txt");
    std::fs::write(&file_path, "test").unwrap();

    // Read directory
    let req = PluginRequest::new(
        "fs",
        "read_dir",
        serde_json::json!({ "path": dir_path.to_string_lossy() }),
    );
    let resp = router.handle(req);
    assert!(resp.success);
    let data = resp.data.unwrap();
    assert!(data.is_array());
    assert!(!data.as_array().unwrap().is_empty());
}

#[test]
fn test_stat_file() {
    let temp = tempdir().unwrap();
    let scope = ScopeConfig::new().with_fs_scope(PathScope::new().allow(temp.path()));
    let router = create_router_with_scope(scope);

    // Create a file
    let file_path = temp.path().join("stat_test.txt");
    std::fs::write(&file_path, "test content").unwrap();

    // Get stat
    let req = PluginRequest::new(
        "fs",
        "stat",
        serde_json::json!({ "path": file_path.to_string_lossy() }),
    );
    let resp = router.handle(req);
    assert!(resp.success);
    let data = resp.data.unwrap();
    assert!(data["isFile"].as_bool().unwrap());
    assert!(!data["isDirectory"].as_bool().unwrap());
    assert!(data["size"].as_u64().unwrap() > 0);
}

#[test]
fn test_copy_file() {
    let temp = tempdir().unwrap();
    let scope = ScopeConfig::new().with_fs_scope(PathScope::new().allow(temp.path()));
    let router = create_router_with_scope(scope);

    // Create source file
    let src_path = temp.path().join("source.txt");
    std::fs::write(&src_path, "copy me").unwrap();

    let dst_path = temp.path().join("dest.txt");

    // Copy file
    let req = PluginRequest::new(
        "fs",
        "copy",
        serde_json::json!({
            "from": src_path.to_string_lossy(),
            "to": dst_path.to_string_lossy()
        }),
    );
    let resp = router.handle(req);
    assert!(resp.success);

    // Verify copy
    assert!(dst_path.exists());
    assert_eq!(std::fs::read_to_string(&dst_path).unwrap(), "copy me");
}

#[test]
fn test_rename_file() {
    let temp = tempdir().unwrap();
    let scope = ScopeConfig::new().with_fs_scope(PathScope::new().allow(temp.path()));
    let router = create_router_with_scope(scope);

    // Create source file
    let src_path = temp.path().join("old_name.txt");
    std::fs::write(&src_path, "rename me").unwrap();

    let dst_path = temp.path().join("new_name.txt");

    // Rename file
    let req = PluginRequest::new(
        "fs",
        "rename",
        serde_json::json!({
            "from": src_path.to_string_lossy(),
            "to": dst_path.to_string_lossy()
        }),
    );
    let resp = router.handle(req);
    assert!(resp.success);

    // Verify rename
    assert!(!src_path.exists());
    assert!(dst_path.exists());
    assert_eq!(std::fs::read_to_string(&dst_path).unwrap(), "rename me");
}

#[test]
fn test_remove_file() {
    let temp = tempdir().unwrap();
    let scope = ScopeConfig::new().with_fs_scope(PathScope::new().allow(temp.path()));
    let router = create_router_with_scope(scope);

    // Create file
    let file_path = temp.path().join("to_remove.txt");
    std::fs::write(&file_path, "delete me").unwrap();
    assert!(file_path.exists());

    // Remove file
    let req = PluginRequest::new(
        "fs",
        "remove",
        serde_json::json!({ "path": file_path.to_string_lossy() }),
    );
    let resp = router.handle(req);
    assert!(resp.success);

    // Verify removal
    assert!(!file_path.exists());
}

#[test]
fn test_command_not_found() {
    let plugin = FsPlugin::new();
    let scope = ScopeConfig::permissive();

    let result = plugin.handle("unknown_command", serde_json::json!({}), &scope);
    assert!(result.is_err());
}

#[test]
fn test_read_file_invalid_args() {
    let plugin = FsPlugin::new();
    let scope = ScopeConfig::permissive();

    let result = plugin.handle(
        "read_file",
        serde_json::json!({ "invalid": "args" }),
        &scope,
    );
    assert!(result.is_err());
}

#[test]
fn test_write_file_invalid_args() {
    let plugin = FsPlugin::new();
    let scope = ScopeConfig::permissive();

    let result = plugin.handle(
        "write_file",
        serde_json::json!({ "path": "/test" }), // Missing contents
        &scope,
    );
    assert!(result.is_err());
}
