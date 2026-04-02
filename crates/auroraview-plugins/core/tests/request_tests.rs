//! Tests for PluginRequest and PluginResponse

use auroraview_plugin_core::{PluginRequest, PluginResponse};
use rstest::rstest;
use serde_json::{json, Value};

// ── PluginRequest::new ────────────────────────────────────────────────────────

#[test]
fn request_new_stores_fields() {
    let req = PluginRequest::new("fs", "read_file", json!({ "path": "/tmp/a.txt" }));
    assert_eq!(req.plugin, "fs");
    assert_eq!(req.command, "read_file");
    assert_eq!(req.args["path"], "/tmp/a.txt");
    assert!(req.id.is_none());
}

#[test]
fn request_with_id() {
    let req = PluginRequest::new("clipboard", "write", json!({})).with_id("req-42");
    assert_eq!(req.id, Some("req-42".to_string()));
}

// ── PluginRequest::from_invoke ────────────────────────────────────────────────

#[rstest]
#[case("plugin:fs|read_file", "fs", "read_file")]
#[case("plugin:clipboard|write", "clipboard", "write")]
#[case("plugin:shell|execute", "shell", "execute")]
#[case("plugin:dialog|open", "dialog", "open")]
fn request_from_invoke_valid(
    #[case] invoke: &str,
    #[case] expected_plugin: &str,
    #[case] expected_cmd: &str,
) {
    let req = PluginRequest::from_invoke(invoke, json!({}));
    assert!(req.is_some());
    let req = req.unwrap();
    assert_eq!(req.plugin, expected_plugin);
    assert_eq!(req.command, expected_cmd);
}

#[rstest]
#[case("fs|read_file")]        // missing "plugin:" prefix
#[case("plugin:fs")]           // missing "|command"
#[case("")]                    // empty
#[case("plugin:")]             // missing plugin and command
fn request_from_invoke_invalid(#[case] invoke: &str) {
    let req = PluginRequest::from_invoke(invoke, json!({}));
    assert!(req.is_none(), "expected None for input: {}", invoke);
}

#[test]
fn request_from_invoke_preserves_args() {
    let args = json!({ "path": "/etc/hosts", "encoding": "utf-8" });
    let req = PluginRequest::from_invoke("plugin:fs|read_file", args.clone()).unwrap();
    assert_eq!(req.args, args);
}

#[test]
fn request_from_invoke_no_id() {
    let req = PluginRequest::from_invoke("plugin:fs|read_file", json!({})).unwrap();
    assert!(req.id.is_none());
}

// ── PluginRequest Clone / Debug / Serialize / Deserialize ────────────────────

#[test]
fn request_clone() {
    let req = PluginRequest::new("fs", "read_file", json!({"key": "value"})).with_id("id-1");
    let clone = req.clone();
    assert_eq!(clone.plugin, req.plugin);
    assert_eq!(clone.command, req.command);
    assert_eq!(clone.id, req.id);
}

#[test]
fn request_debug_non_empty() {
    let req = PluginRequest::new("fs", "read_file", json!({}));
    let s = format!("{:?}", req);
    assert!(!s.is_empty());
}

#[test]
fn request_serde_roundtrip() {
    let req = PluginRequest::new("shell", "execute", json!({"cmd": "ls"})).with_id("r-1");
    let json_str = serde_json::to_string(&req).unwrap();
    let deserialized: PluginRequest = serde_json::from_str(&json_str).unwrap();
    assert_eq!(deserialized.plugin, "shell");
    assert_eq!(deserialized.command, "execute");
    assert_eq!(deserialized.id, Some("r-1".to_string()));
}

// ── PluginResponse::ok ────────────────────────────────────────────────────────

#[test]
fn response_ok_fields() {
    let resp = PluginResponse::ok(json!({"result": 42}));
    assert!(resp.success);
    assert_eq!(resp.data, Some(json!({"result": 42})));
    assert!(resp.error.is_none());
    assert!(resp.code.is_none());
    assert!(resp.id.is_none());
}

#[test]
fn response_ok_null_data() {
    let resp = PluginResponse::ok(Value::Null);
    assert!(resp.success);
    assert_eq!(resp.data, Some(Value::Null));
}

// ── PluginResponse::err ───────────────────────────────────────────────────────

#[test]
fn response_err_fields() {
    let resp = PluginResponse::err("file not found", "FILE_NOT_FOUND");
    assert!(!resp.success);
    assert!(resp.data.is_none());
    assert_eq!(resp.error, Some("file not found".to_string()));
    assert_eq!(resp.code, Some("FILE_NOT_FOUND".to_string()));
    assert!(resp.id.is_none());
}

#[rstest]
#[case("PLUGIN_NOT_FOUND", "Plugin not found")]
#[case("COMMAND_NOT_FOUND", "Command missing")]
#[case("INVALID_ARGS", "Bad args")]
fn response_err_various_codes(#[case] code: &str, #[case] msg: &str) {
    let resp = PluginResponse::err(msg, code);
    assert!(!resp.success);
    assert_eq!(resp.code.as_deref(), Some(code));
    assert_eq!(resp.error.as_deref(), Some(msg));
}

// ── PluginResponse::with_id ───────────────────────────────────────────────────

#[test]
fn response_with_id_set() {
    let resp = PluginResponse::ok(json!(null)).with_id(Some("resp-99".to_string()));
    assert_eq!(resp.id, Some("resp-99".to_string()));
}

#[test]
fn response_with_id_none() {
    let resp = PluginResponse::ok(json!(null)).with_id(None);
    assert!(resp.id.is_none());
}

// ── PluginResponse Clone / Debug / Serialize / Deserialize ───────────────────

#[test]
fn response_clone() {
    let resp = PluginResponse::ok(json!({"x": 1}));
    let clone = resp.clone();
    assert_eq!(clone.success, resp.success);
    assert_eq!(clone.data, resp.data);
}

#[test]
fn response_debug_non_empty() {
    let resp = PluginResponse::err("oops", "ERR");
    let s = format!("{:?}", resp);
    assert!(!s.is_empty());
}

#[test]
fn response_serde_roundtrip_ok() {
    let resp = PluginResponse::ok(json!({"list": [1, 2, 3]})).with_id(Some("r-2".to_string()));
    let json_str = serde_json::to_string(&resp).unwrap();
    let deserialized: PluginResponse = serde_json::from_str(&json_str).unwrap();
    assert!(deserialized.success);
    assert_eq!(deserialized.id, Some("r-2".to_string()));
}

#[test]
fn response_serde_roundtrip_err() {
    let resp = PluginResponse::err("bad input", "INVALID_ARGS");
    let json_str = serde_json::to_string(&resp).unwrap();
    let deserialized: PluginResponse = serde_json::from_str(&json_str).unwrap();
    assert!(!deserialized.success);
    assert_eq!(deserialized.code.as_deref(), Some("INVALID_ARGS"));
}

// ── Edge: from_invoke with pipe in command name ───────────────────────────────

#[test]
fn request_from_invoke_command_with_extra_pipe() {
    // splitn(2, '|') ensures only first pipe splits plugin|command
    let req = PluginRequest::from_invoke("plugin:fs|read|extra", json!({}));
    assert!(req.is_some());
    let req = req.unwrap();
    assert_eq!(req.plugin, "fs");
    assert_eq!(req.command, "read|extra");
}
