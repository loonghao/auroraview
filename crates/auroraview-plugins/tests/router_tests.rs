//! Unit tests for plugin router
//!
//! Tests for PluginRouter, PluginRequest, and PluginResponse.

use auroraview_plugins::{create_router, PluginRequest, PluginResponse};

#[test]
fn test_plugin_request_parse() {
    let req = PluginRequest::from_invoke("plugin:fs|read_file", serde_json::json!({}));
    assert!(req.is_some());
    let req = req.unwrap();
    assert_eq!(req.plugin, "fs");
    assert_eq!(req.command, "read_file");
}

#[test]
fn test_plugin_request_parse_invalid() {
    let req = PluginRequest::from_invoke("not_a_plugin", serde_json::json!({}));
    assert!(req.is_none());

    let req = PluginRequest::from_invoke("plugin:no_command", serde_json::json!({}));
    assert!(req.is_none());
}

#[test]
fn test_plugin_request_new() {
    let req = PluginRequest::new("fs", "read_file", serde_json::json!({"path": "/test"}));
    assert_eq!(req.plugin, "fs");
    assert_eq!(req.command, "read_file");
    assert!(req.id.is_none());
}

#[test]
fn test_plugin_request_with_id() {
    let req = PluginRequest::new("fs", "read_file", serde_json::json!({})).with_id("req-123");
    assert_eq!(req.id, Some("req-123".to_string()));
}

#[test]
fn test_plugin_response_ok() {
    let resp = PluginResponse::ok(serde_json::json!({"result": "success"}));
    assert!(resp.success);
    assert!(resp.data.is_some());
    assert!(resp.error.is_none());
    assert!(resp.code.is_none());
}

#[test]
fn test_plugin_response_err() {
    let resp = PluginResponse::err("File not found", "NOT_FOUND");
    assert!(!resp.success);
    assert!(resp.data.is_none());
    assert_eq!(resp.error, Some("File not found".to_string()));
    assert_eq!(resp.code, Some("NOT_FOUND".to_string()));
}

#[test]
fn test_plugin_response_with_id() {
    let resp = PluginResponse::ok(serde_json::json!({})).with_id(Some("resp-456".to_string()));
    assert_eq!(resp.id, Some("resp-456".to_string()));
}

#[test]
fn test_router_has_default_plugins() {
    let router = create_router();
    assert!(router.has_plugin("fs"));
    assert!(router.has_plugin("clipboard"));
    assert!(router.has_plugin("shell"));
    assert!(router.has_plugin("dialog"));
    assert!(router.has_plugin("process"));
}

#[test]
fn test_router_plugin_names() {
    let router = create_router();
    let names = router.plugin_names();
    assert!(names.contains(&"fs"));
    assert!(names.contains(&"clipboard"));
    assert!(names.contains(&"shell"));
    assert!(names.contains(&"dialog"));
    assert!(names.contains(&"process"));
}

#[test]
fn test_router_plugin_not_found() {
    let mut router = create_router();
    // Enable the plugin first so we can test the "not found" path
    router.scope_mut().enable_plugin("nonexistent");
    let req = PluginRequest::new("nonexistent", "command", serde_json::json!({}));
    let resp = router.handle(req);
    assert!(!resp.success);
    assert_eq!(resp.code, Some("PLUGIN_NOT_FOUND".to_string()));
}

#[test]
fn test_router_plugin_disabled() {
    let mut router = create_router();
    router.scope_mut().disable_plugin("fs");

    let req = PluginRequest::new("fs", "read_file", serde_json::json!({}));
    let resp = router.handle(req);
    assert!(!resp.success);
    assert_eq!(resp.code, Some("PLUGIN_DISABLED".to_string()));
}

#[test]
fn test_router_scope() {
    let router = create_router();
    let scope = router.scope();
    assert!(scope.is_plugin_enabled("fs"));
}

#[test]
fn test_router_default() {
    // Note: PluginRouter::default() creates an empty router without plugins
    // Use create_router() to get a router with all built-in plugins
    let router = create_router();
    assert!(router.has_plugin("fs"));
}
