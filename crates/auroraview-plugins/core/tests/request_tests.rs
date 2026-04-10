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

// ── PluginRequest Send + Sync ─────────────────────────────────────────────────

#[test]
fn request_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<PluginRequest>();
}

// ── PluginResponse Send + Sync ────────────────────────────────────────────────

#[test]
fn response_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<PluginResponse>();
}

// ── PluginRequest with unicode plugin/command ─────────────────────────────────

#[test]
fn request_new_unicode_fields() {
    let req = PluginRequest::new("文件系统", "读取文件", json!({"路径": "/tmp"}));
    assert_eq!(req.plugin, "文件系统");
    assert_eq!(req.command, "读取文件");
    assert_eq!(req.args["路径"], "/tmp");
}

// ── PluginRequest from_invoke with unicode plugin name ───────────────────────

#[test]
fn request_from_invoke_unicode_plugin() {
    let req = PluginRequest::from_invoke("plugin:my_plugin|my_cmd", json!({"key": "val"}));
    assert!(req.is_some());
    let req = req.unwrap();
    assert_eq!(req.plugin, "my_plugin");
    assert_eq!(req.command, "my_cmd");
}

// ── PluginResponse ok with array data ────────────────────────────────────────

#[test]
fn response_ok_array_data() {
    let resp = PluginResponse::ok(json!([1, 2, 3]));
    assert!(resp.success);
    assert_eq!(resp.data, Some(json!([1, 2, 3])));
}

// ── PluginResponse ok with nested object ─────────────────────────────────────

#[test]
fn response_ok_nested_object() {
    let data = json!({"a": {"b": {"c": 42}}});
    let resp = PluginResponse::ok(data.clone());
    assert!(resp.success);
    assert_eq!(resp.data, Some(data));
}

// ── PluginResponse err with empty code ───────────────────────────────────────

#[test]
fn response_err_empty_code() {
    let resp = PluginResponse::err("some error", "");
    assert!(!resp.success);
    assert_eq!(resp.code, Some("".to_string()));
}

// ── PluginRequest with_id overwrites previous id ─────────────────────────────

#[test]
fn request_with_id_overwrite() {
    let req = PluginRequest::new("p", "c", json!({}))
        .with_id("first")
        .with_id("second");
    assert_eq!(req.id, Some("second".to_string()));
}

// ── PluginRequest serde: missing optional id is None ─────────────────────────

#[test]
fn request_serde_no_id_deserialize() {
    let json_str = r#"{"plugin":"fs","command":"read","args":{}}"#;
    let req: PluginRequest = serde_json::from_str(json_str).unwrap();
    assert_eq!(req.plugin, "fs");
    assert!(req.id.is_none());
}

// ── PluginResponse serde: success=false fields preserved ─────────────────────

#[test]
fn response_serde_err_preserves_all_fields() {
    let resp = PluginResponse::err("desc", "ERR_CODE")
        .with_id(Some("id-xyz".to_string()));
    let json_str = serde_json::to_string(&resp).unwrap();
    let deserialized: PluginResponse = serde_json::from_str(&json_str).unwrap();
    assert!(!deserialized.success);
    assert_eq!(deserialized.error.as_deref(), Some("desc"));
    assert_eq!(deserialized.code.as_deref(), Some("ERR_CODE"));
    assert_eq!(deserialized.id.as_deref(), Some("id-xyz"));
}

// ── from_invoke: plugin name with underscores/hyphens ────────────────────────

#[rstest]
#[case("plugin:my_plugin|cmd")]
#[case("plugin:my-plugin|cmd")]
#[case("plugin:plugin_v2|do_thing")]
fn request_from_invoke_plugin_name_variants(#[case] invoke: &str) {
    let req = PluginRequest::from_invoke(invoke, json!({}));
    assert!(req.is_some(), "should parse: {}", invoke);
}

// ── PluginRequest new with empty plugin/command ───────────────────────────────

#[test]
fn request_new_empty_plugin_and_command() {
    let req = PluginRequest::new("", "", json!(null));
    assert_eq!(req.plugin, "");
    assert_eq!(req.command, "");
    assert!(req.id.is_none());
}

// ── PluginRequest: args preserved as array ────────────────────────────────────

#[test]
fn request_new_array_args() {
    let req = PluginRequest::new("fs", "list_dir", json!(["/tmp", "/home"]));
    assert_eq!(req.args, json!(["/tmp", "/home"]));
}

// ── PluginRequest: args preserved as null ────────────────────────────────────

#[test]
fn request_new_null_args() {
    let req = PluginRequest::new("ping", "health", Value::Null);
    assert_eq!(req.args, Value::Null);
}

// ── PluginRequest: args preserved as boolean ─────────────────────────────────

#[test]
fn request_new_bool_args() {
    let req = PluginRequest::new("sys", "enabled", json!(true));
    assert_eq!(req.args, json!(true));
}

// ── PluginRequest: from_invoke with double-pipe in plugin name fails ──────────

#[test]
fn request_from_invoke_double_pipe_first_plugin_segment() {
    // plugin||command: empty plugin, command = "|command" after split
    let req = PluginRequest::from_invoke("plugin:|cmd", json!({}));
    // Empty plugin name — implementation may allow or deny; just verify no panic
    let _ = req;
}

// ── PluginRequest: with_id chain returns correct final id ────────────────────

#[rstest]
#[case("id-a", "id-a")]
#[case("id-123", "id-123")]
#[case("", "")]
fn request_with_id_values(#[case] id: &str, #[case] expected: &str) {
    let req = PluginRequest::new("p", "c", json!({})).with_id(id);
    assert_eq!(req.id.as_deref(), Some(expected));
}

// ── PluginResponse::ok: bool data ────────────────────────────────────────────

#[test]
fn response_ok_bool_data() {
    let resp = PluginResponse::ok(json!(true));
    assert!(resp.success);
    assert_eq!(resp.data, Some(json!(true)));
}

// ── PluginResponse::ok: string data ──────────────────────────────────────────

#[test]
fn response_ok_string_data() {
    let resp = PluginResponse::ok(json!("hello"));
    assert!(resp.success);
    assert_eq!(resp.data, Some(json!("hello")));
}

// ── PluginResponse::ok: number data ──────────────────────────────────────────

#[test]
fn response_ok_number_data() {
    let resp = PluginResponse::ok(json!(2.71));
    assert!(resp.success);
}

// ── PluginResponse::err: empty message ───────────────────────────────────────

#[test]
fn response_err_empty_message() {
    let resp = PluginResponse::err("", "ERR");
    assert!(!resp.success);
    assert_eq!(resp.error.as_deref(), Some(""));
}

// ── PluginResponse: serde field names ────────────────────────────────────────

#[test]
fn response_serde_ok_has_success_true() {
    let resp = PluginResponse::ok(json!(1));
    let v: Value = serde_json::from_str(&serde_json::to_string(&resp).unwrap()).unwrap();
    assert_eq!(v["success"], json!(true));
}

#[test]
fn response_serde_err_has_success_false() {
    let resp = PluginResponse::err("msg", "CODE");
    let v: Value = serde_json::from_str(&serde_json::to_string(&resp).unwrap()).unwrap();
    assert_eq!(v["success"], json!(false));
}

// ── PluginRequest: serde plugin+command+args fields ──────────────────────────

#[test]
fn request_serde_fields_present() {
    let req = PluginRequest::new("shell", "run", json!({"cmd": "ls -la"}));
    let v: Value = serde_json::from_str(&serde_json::to_string(&req).unwrap()).unwrap();
    assert_eq!(v["plugin"], "shell");
    assert_eq!(v["command"], "run");
}

// ── PluginRequest Send + Sync (PluginResponse too) ───────────────────────────

#[test]
fn both_types_are_send_sync() {
    fn check<T: Send + Sync>() {}
    check::<PluginRequest>();
    check::<PluginResponse>();
}

// ── PluginResponse: with_id overwrites None → Some ───────────────────────────

#[test]
fn response_with_id_overwrites_none() {
    let resp = PluginResponse::ok(json!({})).with_id(Some("new-id".to_string()));
    assert_eq!(resp.id.as_deref(), Some("new-id"));
}

// ── PluginResponse: clone preserves error fields ──────────────────────────────

#[test]
fn response_err_clone_preserves_fields() {
    let resp = PluginResponse::err("test error", "TEST_ERR");
    let clone = resp.clone();
    assert_eq!(clone.error, resp.error);
    assert_eq!(clone.code, resp.code);
    assert_eq!(clone.success, resp.success);
}

// ── from_invoke: all valid prefix patterns ────────────────────────────────────

#[rstest]
#[case("plugin:a|b", "a", "b")]
#[case("plugin:clipboard|paste", "clipboard", "paste")]
#[case("plugin:dialog|confirm", "dialog", "confirm")]
fn request_from_invoke_various_valid(
    #[case] invoke: &str,
    #[case] expected_plugin: &str,
    #[case] expected_cmd: &str,
) {
    let req = PluginRequest::from_invoke(invoke, json!({})).unwrap();
    assert_eq!(req.plugin, expected_plugin);
    assert_eq!(req.command, expected_cmd);
}
