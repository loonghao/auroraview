//! Unit and integration tests for plugin router
//!
//! Tests for PluginRouter, PluginRequest, and PluginResponse,
//! including E2E request/response roundtrips, concurrent dispatch,
//! and multi-plugin routing.

use std::sync::{Arc, Mutex};
use std::thread;

use auroraview_plugins::{
    create_router, create_router_with_scope, PathScope, PluginRequest, PluginResponse, ScopeConfig,
};
use rstest::rstest;
use tempfile::tempdir;

// ─── PluginRequest parsing ────────────────────────────────────────────────────

#[test]
fn request_parse_valid() {
    let req = PluginRequest::from_invoke("plugin:fs|read_file", serde_json::json!({}));
    assert!(req.is_some());
    let req = req.unwrap();
    assert_eq!(req.plugin, "fs");
    assert_eq!(req.command, "read_file");
}

#[test]
fn request_parse_invalid_no_prefix() {
    let req = PluginRequest::from_invoke("not_a_plugin", serde_json::json!({}));
    assert!(req.is_none());
}

#[test]
fn request_parse_invalid_missing_command() {
    let req = PluginRequest::from_invoke("plugin:no_command", serde_json::json!({}));
    assert!(req.is_none());
}

#[test]
fn request_new_no_id() {
    let req = PluginRequest::new("fs", "read_file", serde_json::json!({"path": "/test"}));
    assert_eq!(req.plugin, "fs");
    assert_eq!(req.command, "read_file");
    assert!(req.id.is_none());
}

#[test]
fn request_with_id() {
    let req = PluginRequest::new("fs", "read_file", serde_json::json!({})).with_id("req-123");
    assert_eq!(req.id, Some("req-123".to_string()));
}

// ─── PluginResponse construction ─────────────────────────────────────────────

#[test]
fn response_ok_fields() {
    let resp = PluginResponse::ok(serde_json::json!({"result": "success"}));
    assert!(resp.success);
    assert!(resp.data.is_some());
    assert!(resp.error.is_none());
    assert!(resp.code.is_none());
}

#[test]
fn response_err_fields() {
    let resp = PluginResponse::err("File not found", "NOT_FOUND");
    assert!(!resp.success);
    assert!(resp.data.is_none());
    assert_eq!(resp.error, Some("File not found".to_string()));
    assert_eq!(resp.code, Some("NOT_FOUND".to_string()));
}

#[test]
fn response_with_id() {
    let resp = PluginResponse::ok(serde_json::json!({})).with_id(Some("resp-456".to_string()));
    assert_eq!(resp.id, Some("resp-456".to_string()));
}

#[test]
fn response_with_id_none() {
    let resp = PluginResponse::ok(serde_json::json!({})).with_id(None);
    assert!(resp.id.is_none());
}

// ─── Router default plugins ───────────────────────────────────────────────────

#[test]
fn router_has_default_plugins() {
    let router = create_router();
    assert!(router.has_plugin("fs"));
    assert!(router.has_plugin("clipboard"));
    assert!(router.has_plugin("shell"));
    assert!(router.has_plugin("dialog"));
    assert!(router.has_plugin("process"));
}

#[test]
fn router_plugin_names_contains_all() {
    let router = create_router();
    let names = router.plugin_names();
    assert!(names.contains(&"fs"));
    assert!(names.contains(&"clipboard"));
    assert!(names.contains(&"shell"));
    assert!(names.contains(&"dialog"));
    assert!(names.contains(&"process"));
}

#[test]
fn router_plugin_not_found() {
    let mut router = create_router();
    router.scope_mut().enable_plugin("nonexistent");
    let req = PluginRequest::new("nonexistent", "command", serde_json::json!({}));
    let resp = router.handle(req);
    assert!(!resp.success);
    assert_eq!(resp.code, Some("PLUGIN_NOT_FOUND".to_string()));
}

#[test]
fn router_plugin_disabled() {
    let mut router = create_router();
    router.scope_mut().disable_plugin("fs");

    let req = PluginRequest::new("fs", "read_file", serde_json::json!({}));
    let resp = router.handle(req);
    assert!(!resp.success);
    assert_eq!(resp.code, Some("PLUGIN_DISABLED".to_string()));
}

#[test]
fn router_scope_fs_enabled_by_default() {
    let router = create_router();
    let scope = router.scope();
    assert!(scope.is_plugin_enabled("fs"));
}

#[test]
fn router_default_same_as_create() {
    let router = create_router();
    assert!(router.has_plugin("fs"));
}

// ─── ID roundtrip: request ID echoed in response ─────────────────────────────

#[test]
fn request_id_echoed_on_success() {
    let temp = tempdir().unwrap();
    let scope = ScopeConfig::new().with_fs_scope(PathScope::new().allow(temp.path()));
    let router = create_router_with_scope(scope);

    let file_path = temp.path().join("id_test.txt");
    std::fs::write(&file_path, "id echo test").unwrap();

    let req = PluginRequest::new(
        "fs",
        "read_file",
        serde_json::json!({ "path": file_path.to_string_lossy() }),
    )
    .with_id("roundtrip-001");

    let resp = router.handle(req);
    assert!(resp.success);
    assert_eq!(resp.id, Some("roundtrip-001".to_string()));
}

#[test]
fn request_id_echoed_on_error() {
    let temp = tempdir().unwrap();
    let scope = ScopeConfig::new().with_fs_scope(PathScope::new().allow(temp.path()));
    let router = create_router_with_scope(scope);

    // Read non-existent file — expect error with ID echoed
    let req = PluginRequest::new(
        "fs",
        "read_file",
        serde_json::json!({ "path": temp.path().join("missing.txt").to_string_lossy() }),
    )
    .with_id("err-id-42");

    let resp = router.handle(req);
    assert!(!resp.success);
    assert_eq!(resp.id, Some("err-id-42".to_string()));
}

#[test]
fn request_id_echoed_on_disabled_plugin() {
    let mut router = create_router();
    router.scope_mut().disable_plugin("fs");

    let req = PluginRequest::new("fs", "read_file", serde_json::json!({})).with_id("disabled-99");

    let resp = router.handle(req);
    assert!(!resp.success);
    assert_eq!(resp.id, Some("disabled-99".to_string()));
}

#[test]
fn request_id_echoed_on_plugin_not_found() {
    let mut router = create_router();
    router.scope_mut().enable_plugin("ghost");

    let req = PluginRequest::new("ghost", "summon", serde_json::json!({})).with_id("ghost-7");

    let resp = router.handle(req);
    assert!(!resp.success);
    assert_eq!(resp.id, Some("ghost-7".to_string()));
}

#[test]
fn request_without_id_returns_none_id() {
    let temp = tempdir().unwrap();
    let scope = ScopeConfig::new().with_fs_scope(PathScope::new().allow(temp.path()));
    let router = create_router_with_scope(scope);

    let file_path = temp.path().join("no_id.txt");
    std::fs::write(&file_path, "no id").unwrap();

    let req = PluginRequest::new(
        "fs",
        "read_file",
        serde_json::json!({ "path": file_path.to_string_lossy() }),
    );

    let resp = router.handle(req);
    assert!(resp.success);
    assert!(resp.id.is_none());
}

// ─── Multi-plugin dispatch ────────────────────────────────────────────────────

#[test]
fn multi_plugin_dispatch_fs_and_scope() {
    let temp = tempdir().unwrap();
    let scope = ScopeConfig::new().with_fs_scope(PathScope::new().allow(temp.path()));
    let router = create_router_with_scope(scope);

    // fs: write
    let write_path = temp.path().join("multi.txt");
    let write_req = PluginRequest::new(
        "fs",
        "write_file",
        serde_json::json!({
            "path": write_path.to_string_lossy(),
            "contents": "multi-plugin"
        }),
    );
    let wr = router.handle(write_req);
    assert!(wr.success, "fs write failed: {:?}", wr.error);

    // fs: exists
    let exists_req = PluginRequest::new(
        "fs",
        "exists",
        serde_json::json!({ "path": write_path.to_string_lossy() }),
    );
    let er = router.handle(exists_req);
    assert!(er.success);
    assert_eq!(er.data.unwrap()["exists"], true);

    // fs: read
    let read_req = PluginRequest::new(
        "fs",
        "read_file",
        serde_json::json!({ "path": write_path.to_string_lossy() }),
    );
    let rr = router.handle(read_req);
    assert!(rr.success);
    assert_eq!(rr.data.unwrap(), "multi-plugin");
}

#[test]
fn multi_plugin_dispatch_scope_violation_does_not_affect_other_commands() {
    let temp = tempdir().unwrap();
    let scope = ScopeConfig::new().with_fs_scope(PathScope::new().allow(temp.path()));
    let router = create_router_with_scope(scope);

    // Scope violation
    let violation = PluginRequest::new(
        "fs",
        "read_file",
        serde_json::json!({ "path": "/etc/passwd" }),
    );
    let vr = router.handle(violation);
    assert!(!vr.success);

    // Valid operation still succeeds
    let valid_path = temp.path().join("after_violation.txt");
    std::fs::write(&valid_path, "ok").unwrap();
    let ok_req = PluginRequest::new(
        "fs",
        "read_file",
        serde_json::json!({ "path": valid_path.to_string_lossy() }),
    );
    let ok_resp = router.handle(ok_req);
    assert!(ok_resp.success);
}

// ─── Concurrent router dispatch ──────────────────────────────────────────────

#[test]
fn concurrent_router_dispatch() {
    let temp = tempdir().unwrap();
    let temp_path = temp.path().to_path_buf();

    // Pre-create files
    for i in 0..10 {
        let p = temp_path.join(format!("concurrent_{}.txt", i));
        std::fs::write(&p, format!("content_{}", i)).unwrap();
    }

    let scope = ScopeConfig::new().with_fs_scope(PathScope::new().allow(&temp_path));
    let router = Arc::new(create_router_with_scope(scope));

    let results: Arc<Mutex<Vec<bool>>> = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();

    for i in 0..10 {
        let router_clone = Arc::clone(&router);
        let results_clone = Arc::clone(&results);
        let file = temp_path.join(format!("concurrent_{}.txt", i));

        let handle = thread::spawn(move || {
            let req = PluginRequest::new(
                "fs",
                "read_file",
                serde_json::json!({ "path": file.to_string_lossy() }),
            );
            let resp = router_clone.handle(req);
            results_clone.lock().unwrap().push(resp.success);
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    let results = results.lock().unwrap();
    assert_eq!(results.len(), 10);
    assert!(results.iter().all(|&s| s), "Some concurrent reads failed");
}

#[test]
fn concurrent_mixed_plugin_dispatch() {
    let temp = tempdir().unwrap();
    let temp_path = temp.path().to_path_buf();

    let scope = ScopeConfig::new().with_fs_scope(PathScope::new().allow(&temp_path));
    let router = Arc::new(create_router_with_scope(scope));

    let errors: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();

    for i in 0..20 {
        let router_clone = Arc::clone(&router);
        let errors_clone = Arc::clone(&errors);
        let base = temp_path.clone();

        let handle = thread::spawn(move || {
            // Alternate between write and exists
            if i % 2 == 0 {
                let p = base.join(format!("mixed_{}.txt", i));
                let req = PluginRequest::new(
                    "fs",
                    "write_file",
                    serde_json::json!({
                        "path": p.to_string_lossy(),
                        "contents": format!("data_{}", i)
                    }),
                );
                let resp = router_clone.handle(req);
                if !resp.success {
                    errors_clone
                        .lock()
                        .unwrap()
                        .push(resp.error.unwrap_or_default());
                }
            } else {
                let p = base.join(format!("mixed_{}.txt", i - 1));
                let req = PluginRequest::new(
                    "fs",
                    "exists",
                    serde_json::json!({ "path": p.to_string_lossy() }),
                );
                let resp = router_clone.handle(req);
                if !resp.success {
                    errors_clone
                        .lock()
                        .unwrap()
                        .push(resp.error.unwrap_or_default());
                }
            }
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    let errs = errors.lock().unwrap();
    assert!(errs.is_empty(), "Concurrent mixed dispatch errors: {:?}", errs);
}

// ─── rstest parameterized ─────────────────────────────────────────────────────

#[rstest]
#[case("fs")]
#[case("clipboard")]
#[case("shell")]
#[case("dialog")]
#[case("process")]
fn all_default_plugins_present(#[case] name: &str) {
    let router = create_router();
    assert!(router.has_plugin(name));
}

#[rstest]
#[case("req-1")]
#[case("uuid-abc-def")]
#[case("12345")]
#[case("very-long-request-id-with-many-parts-and-dashes-000")]
fn id_roundtrip_various_formats(#[case] id: &str) {
    let temp = tempdir().unwrap();
    let scope = ScopeConfig::new().with_fs_scope(PathScope::new().allow(temp.path()));
    let router = create_router_with_scope(scope);

    let file_path = temp.path().join("id_fmt.txt");
    std::fs::write(&file_path, "id fmt").unwrap();

    let req = PluginRequest::new(
        "fs",
        "read_file",
        serde_json::json!({ "path": file_path.to_string_lossy() }),
    )
    .with_id(id);

    let resp = router.handle(req);
    assert!(resp.success);
    assert_eq!(resp.id, Some(id.to_string()));
}

#[rstest]
#[case("write_file", serde_json::json!({"path": "/tmp/x", "contents": "x"}), "fs")]
#[case("read_file", serde_json::json!({"path": "/tmp/x"}), "fs")]
fn scope_violation_returns_error(
    #[case] command: &str,
    #[case] args: serde_json::Value,
    #[case] plugin: &str,
) {
    // Empty scope = no paths allowed => scope violation
    let scope = ScopeConfig::new().with_fs_scope(PathScope::new());
    let router = create_router_with_scope(scope);

    let req = PluginRequest::new(plugin, command, args);
    let resp = router.handle(req);
    assert!(!resp.success, "Expected error for out-of-scope path");
}

// ─── Plugin register/unregister ───────────────────────────────────────────────

#[test]
fn unregister_removes_plugin() {
    let mut router = create_router();
    assert!(router.has_plugin("fs"));
    router.unregister("fs");
    assert!(!router.has_plugin("fs"));
}

#[test]
fn set_scope_overrides_previous() {
    let mut router = create_router();
    let scope = ScopeConfig::permissive();
    router.set_scope(scope);
    assert!(router.scope().is_plugin_enabled("fs"));
}

// ─── Event callback ──────────────────────────────────────────────────────────

#[test]
fn event_callback_set_and_clear() {
    let router = create_router();
    let called = Arc::new(Mutex::new(false));
    let called_clone = Arc::clone(&called);

    router.set_event_callback(Arc::new(move |_event, _data| {
        *called_clone.lock().unwrap() = true;
    }));

    router.emit_event("test_event", serde_json::json!({"key": "value"}));
    assert!(*called.lock().unwrap(), "Event callback was not called");

    router.clear_event_callback();
    // After clearing, further emits should not crash
    router.emit_event("after_clear", serde_json::json!(null));
}

#[test]
fn emit_event_without_callback_does_not_panic() {
    let router = create_router();
    // No callback set — should not panic
    router.emit_event("no_callback", serde_json::json!({"key": "value"}));
}
