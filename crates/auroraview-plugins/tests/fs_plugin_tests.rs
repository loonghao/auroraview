//! Unit tests for file system plugin
//!
//! Tests for FsPlugin commands and operations.

use std::sync::Arc;
use std::thread;

use auroraview_plugins::fs::FsPlugin;
use auroraview_plugins::{
    create_router_with_scope, PathScope, PluginHandler, PluginRequest, ScopeConfig,
};
use tempfile::tempdir;

#[test]
fn fs_plugin_commands() {
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
fn fs_plugin_name() {
    let plugin = FsPlugin::new();
    assert_eq!(plugin.name(), "fs");
}

#[test]
fn fs_plugin_default() {
    let plugin = FsPlugin::default();
    assert_eq!(plugin.name(), "fs");
}

#[test]
fn write_and_read_file() {
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
fn exists_command() {
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
fn exists_nonexistent() {
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
fn scope_violation() {
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
fn create_and_read_dir() {
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
fn stat_file() {
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
fn copy_file() {
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
fn rename_file() {
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
fn remove_file() {
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
fn command_not_found() {
    let plugin = FsPlugin::new();
    let scope = ScopeConfig::permissive();

    let result = plugin.handle("unknown_command", serde_json::json!({}), &scope);
    assert!(result.is_err());
}

#[test]
fn read_file_invalid_args() {
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
fn write_file_invalid_args() {
    let plugin = FsPlugin::new();
    let scope = ScopeConfig::permissive();

    let result = plugin.handle(
        "write_file",
        serde_json::json!({ "path": "/test" }), // Missing contents
        &scope,
    );
    assert!(result.is_err());
}

// === Extended edge case tests ===

#[test]
fn write_and_read_binary_file() {
    let temp = tempdir().unwrap();
    let scope = ScopeConfig::new().with_fs_scope(PathScope::new().allow(temp.path()));
    let router = create_router_with_scope(scope);

    let file_path = temp.path().join("binary.bin");
    let file_path_str = file_path.to_string_lossy().to_string();

    // write_file_binary expects contents as a byte array (Vec<u8>)
    let binary_data: Vec<u8> = vec![0x00, 0xFF, 0x42, 0xDE, 0xAD, 0xBE, 0xEF];

    let write_req = PluginRequest::new(
        "fs",
        "write_file_binary",
        serde_json::json!({
            "path": file_path_str,
            "contents": binary_data
        }),
    );
    let write_resp = router.handle(write_req);
    assert!(write_resp.success, "Binary write failed: {:?}", write_resp.error);

    // read_file_binary returns base64-encoded string
    let read_req = PluginRequest::new(
        "fs",
        "read_file_binary",
        serde_json::json!({ "path": file_path_str }),
    );
    let read_resp = router.handle(read_req);
    assert!(read_resp.success, "Binary read failed: {:?}", read_resp.error);

    // Verify the returned base64 decodes back to the original bytes
    let returned = read_resp.data.unwrap();
    let b64_str = returned.as_str().unwrap();
    let decoded = base64_decode(b64_str);
    assert_eq!(decoded, binary_data);
}

#[test]
fn write_and_read_empty_file() {
    let temp = tempdir().unwrap();
    let scope = ScopeConfig::new().with_fs_scope(PathScope::new().allow(temp.path()));
    let router = create_router_with_scope(scope);

    let file_path = temp.path().join("empty.txt");
    let file_path_str = file_path.to_string_lossy().to_string();

    let write_req = PluginRequest::new(
        "fs",
        "write_file",
        serde_json::json!({ "path": file_path_str, "contents": "" }),
    );
    let write_resp = router.handle(write_req);
    assert!(write_resp.success);

    let read_req = PluginRequest::new(
        "fs",
        "read_file",
        serde_json::json!({ "path": file_path_str }),
    );
    let read_resp = router.handle(read_req);
    assert!(read_resp.success);
    assert_eq!(read_resp.data.unwrap(), "");
}

#[test]
fn deep_path_create_and_read() {
    let temp = tempdir().unwrap();
    let scope = ScopeConfig::new().with_fs_scope(PathScope::new().allow(temp.path()));
    let router = create_router_with_scope(scope);

    // Create deep directory
    let deep_dir = temp.path().join("a").join("b").join("c").join("d");
    std::fs::create_dir_all(&deep_dir).unwrap();

    let file_path = deep_dir.join("deep.txt");
    let file_path_str = file_path.to_string_lossy().to_string();

    let write_req = PluginRequest::new(
        "fs",
        "write_file",
        serde_json::json!({ "path": file_path_str, "contents": "deep file" }),
    );
    assert!(router.handle(write_req).success);

    let read_req = PluginRequest::new(
        "fs",
        "read_file",
        serde_json::json!({ "path": file_path_str }),
    );
    let resp = router.handle(read_req);
    assert!(resp.success);
    assert_eq!(resp.data.unwrap(), "deep file");
}

#[test]
fn stat_directory() {
    let temp = tempdir().unwrap();
    let scope = ScopeConfig::new().with_fs_scope(PathScope::new().allow(temp.path()));
    let router = create_router_with_scope(scope);

    let dir_path = temp.path().join("stat_dir");
    std::fs::create_dir(&dir_path).unwrap();

    let req = PluginRequest::new(
        "fs",
        "stat",
        serde_json::json!({ "path": dir_path.to_string_lossy() }),
    );
    let resp = router.handle(req);
    assert!(resp.success);
    let data = resp.data.unwrap();
    assert!(!data["isFile"].as_bool().unwrap());
    assert!(data["isDirectory"].as_bool().unwrap());
}

#[test]
fn overwrite_existing_file() {
    let temp = tempdir().unwrap();
    let scope = ScopeConfig::new().with_fs_scope(PathScope::new().allow(temp.path()));
    let router = create_router_with_scope(scope);

    let file_path = temp.path().join("overwrite.txt");
    let file_path_str = file_path.to_string_lossy().to_string();

    // Write v1
    let req1 = PluginRequest::new(
        "fs",
        "write_file",
        serde_json::json!({ "path": file_path_str, "contents": "version1" }),
    );
    assert!(router.handle(req1).success);

    // Write v2 (overwrite)
    let req2 = PluginRequest::new(
        "fs",
        "write_file",
        serde_json::json!({ "path": file_path_str, "contents": "version2" }),
    );
    assert!(router.handle(req2).success);

    // Read back - should be v2
    let read_req = PluginRequest::new(
        "fs",
        "read_file",
        serde_json::json!({ "path": file_path_str }),
    );
    let resp = router.handle(read_req);
    assert!(resp.success);
    assert_eq!(resp.data.unwrap(), "version2");
}

#[test]
fn read_dir_multiple_files() {
    let temp = tempdir().unwrap();
    let scope = ScopeConfig::new().with_fs_scope(PathScope::new().allow(temp.path()));
    let router = create_router_with_scope(scope);

    // Create multiple files
    for i in 0..5 {
        std::fs::write(temp.path().join(format!("file{i}.txt")), format!("content{i}")).unwrap();
    }

    let req = PluginRequest::new(
        "fs",
        "read_dir",
        serde_json::json!({ "path": temp.path().to_string_lossy() }),
    );
    let resp = router.handle(req);
    assert!(resp.success);
    let data = resp.data.unwrap();
    assert!(data.is_array());
    assert!(data.as_array().unwrap().len() >= 5);
}

#[test]
fn copy_then_modify_independent() {
    // Verify copy creates an independent file
    let temp = tempdir().unwrap();
    let scope = ScopeConfig::new().with_fs_scope(PathScope::new().allow(temp.path()));
    let router = create_router_with_scope(scope);

    let src = temp.path().join("src.txt");
    let dst = temp.path().join("dst.txt");
    std::fs::write(&src, "original").unwrap();

    let copy_req = PluginRequest::new(
        "fs",
        "copy",
        serde_json::json!({ "from": src.to_string_lossy(), "to": dst.to_string_lossy() }),
    );
    assert!(router.handle(copy_req).success);

    // Modify source
    let write_req = PluginRequest::new(
        "fs",
        "write_file",
        serde_json::json!({ "path": src.to_string_lossy(), "contents": "modified" }),
    );
    assert!(router.handle(write_req).success);

    // Destination should still have original content
    let read_dst = PluginRequest::new(
        "fs",
        "read_file",
        serde_json::json!({ "path": dst.to_string_lossy() }),
    );
    let resp = router.handle(read_dst);
    assert!(resp.success);
    assert_eq!(resp.data.unwrap(), "original");
}


#[test]
fn concurrent_writes_to_different_files() {
    let temp = Arc::new(tempdir().unwrap());

    let handles: Vec<_> = (0..8)
        .map(|i| {
            let tmp = temp.clone();
            thread::spawn(move || {
                let scope =
                    ScopeConfig::new().with_fs_scope(PathScope::new().allow(tmp.path()));
                let router = create_router_with_scope(scope);

                let path = tmp.path().join(format!("concurrent_{i}.txt"));
                let write_req = PluginRequest::new(
                    "fs",
                    "write_file",
                    serde_json::json!({
                        "path": path.to_string_lossy(),
                        "contents": format!("thread_{i}_data")
                    }),
                );
                let resp = router.handle(write_req);
                assert!(resp.success, "Thread {i} write failed");

                // Read back
                let read_req = PluginRequest::new(
                    "fs",
                    "read_file",
                    serde_json::json!({ "path": path.to_string_lossy() }),
                );
                let resp = router.handle(read_req);
                assert!(resp.success);
                resp.data.unwrap().to_string()
            })
        })
        .collect();

    for (i, h) in handles.into_iter().enumerate() {
        let content = h.join().unwrap();
        assert!(content.contains(&format!("thread_{i}_data")));
    }
}

#[test]
fn remove_directory() {
    let temp = tempdir().unwrap();
    let scope = ScopeConfig::new().with_fs_scope(PathScope::new().allow(temp.path()));
    let router = create_router_with_scope(scope);

    let dir_path = temp.path().join("remove_me");
    std::fs::create_dir(&dir_path).unwrap();
    assert!(dir_path.exists());

    let req = PluginRequest::new(
        "fs",
        "remove",
        serde_json::json!({ "path": dir_path.to_string_lossy() }),
    );
    let resp = router.handle(req);
    assert!(resp.success);
    assert!(!dir_path.exists());
}

#[test]
fn write_large_file() {
    let temp = tempdir().unwrap();
    let scope = ScopeConfig::new().with_fs_scope(PathScope::new().allow(temp.path()));
    let router = create_router_with_scope(scope);

    let file_path = temp.path().join("large.txt");
    let file_path_str = file_path.to_string_lossy().to_string();
    let large_content = "A".repeat(64 * 1024); // 64 KB

    let write_req = PluginRequest::new(
        "fs",
        "write_file",
        serde_json::json!({ "path": file_path_str, "contents": large_content }),
    );
    assert!(router.handle(write_req).success);

    let stat_req = PluginRequest::new(
        "fs",
        "stat",
        serde_json::json!({ "path": file_path_str }),
    );
    let stat_resp = router.handle(stat_req);
    assert!(stat_resp.success);
    assert_eq!(stat_resp.data.unwrap()["size"].as_u64().unwrap(), 64 * 1024);
}

// Helper: minimal base64 decode for tests (no external dep)


fn base64_decode(s: &str) -> Vec<u8> {
    let table: [u8; 128] = {
        let mut t = [255u8; 128];
        let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        for (i, &c) in chars.iter().enumerate() {
            t[c as usize] = i as u8;
        }
        t
    };
    let bytes: Vec<u8> = s.bytes().filter(|b| *b != b'=').collect();
    let mut result = Vec::new();
    for chunk in bytes.chunks(4) {
        let v: Vec<u8> = chunk.iter().map(|&b| table[b as usize]).collect();
        result.push((v[0] << 2) | (v[1] >> 4));
        if chunk.len() > 2 {
            result.push((v[1] << 4) | (v[2] >> 2));
        }
        if chunk.len() > 3 {
            result.push((v[2] << 6) | v[3]);
        }
    }
    result
}
