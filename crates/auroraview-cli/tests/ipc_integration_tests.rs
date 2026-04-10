//! Integration tests for Python IPC communication
//!
//! These tests verify the JSON-RPC communication between Rust and Python.
//! They require Python to be available and are skipped in CI coverage runs.

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

/// Get the Python command - try python3 first (Linux), then python (Windows)
fn get_python_command() -> &'static str {
    // Check if python3 exists (common on Linux/macOS)
    if Command::new("python3")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return "python3";
    }
    // Fall back to python (Windows)
    "python"
}

/// Test that Python packed module sends ready signal on startup
#[test]
#[ignore = "requires Python runtime, run with --ignored"]
fn python_sends_ready_signal() {
    // Create a minimal Python script that mimics packed mode behavior
    let python_code = r#"
import json
import sys

# Simulate packed mode API server startup
handlers = ["get_samples", "get_categories", "run_sample"]
ready_signal = json.dumps({"type": "ready", "handlers": handlers})
print(ready_signal, flush=True)

# Read one request and respond
line = sys.stdin.readline()
if line:
    request = json.loads(line)
    response = {
        "id": request.get("id", ""),
        "ok": True,
        "result": {"echo": request.get("method", "")}
    }
    print(json.dumps(response), flush=True)
"#;

    let mut child = Command::new(get_python_command())
        .args(["-c", python_code])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed to start Python");

    let mut stdin = child.stdin.take().expect("Failed to get stdin");
    let stdout = child.stdout.take().expect("Failed to get stdout");
    let mut reader = BufReader::new(stdout);

    // Read ready signal
    let mut ready_line = String::new();
    reader
        .read_line(&mut ready_line)
        .expect("Failed to read ready signal");

    let ready_msg: serde_json::Value =
        serde_json::from_str(&ready_line).expect("Failed to parse ready signal");

    assert_eq!(ready_msg["type"], "ready");
    assert!(ready_msg["handlers"].is_array());

    // Send a test request
    let request = serde_json::json!({
        "id": "test_1",
        "method": "echo",
        "params": null
    });
    writeln!(stdin, "{}", request).expect("Failed to write request");
    stdin.flush().expect("Failed to flush");

    // Read response
    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .expect("Failed to read response");

    let response: serde_json::Value =
        serde_json::from_str(&response_line).expect("Failed to parse response");

    assert_eq!(response["id"], "test_1");
    assert_eq!(response["ok"], true);

    // Clean up
    drop(stdin);
    let _ = child.wait();
}

/// Test that Python handles multiple requests correctly
#[test]
#[ignore = "requires Python runtime, run with --ignored"]
fn python_handles_multiple_requests() {
    let python_code = r#"
import json
import sys

# Send ready signal
print(json.dumps({"type": "ready", "handlers": ["echo"]}), flush=True)

# Handle multiple requests
for _ in range(3):
    line = sys.stdin.readline()
    if not line:
        break
    request = json.loads(line)
    response = {
        "id": request.get("id", ""),
        "ok": True,
        "result": {"count": int(request.get("id", "0").split("_")[-1])}
    }
    print(json.dumps(response), flush=True)
"#;

    let mut child = Command::new(get_python_command())
        .args(["-c", python_code])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed to start Python");

    let mut stdin = child.stdin.take().expect("Failed to get stdin");
    let stdout = child.stdout.take().expect("Failed to get stdout");
    let mut reader = BufReader::new(stdout);

    // Read ready signal
    let mut ready_line = String::new();
    reader.read_line(&mut ready_line).unwrap();
    assert!(ready_line.contains("ready"));

    // Send multiple requests
    for i in 1..=3 {
        let request = serde_json::json!({
            "id": format!("req_{}", i),
            "method": "echo",
            "params": null
        });
        writeln!(stdin, "{}", request).unwrap();
        stdin.flush().unwrap();

        let mut response_line = String::new();
        reader.read_line(&mut response_line).unwrap();

        let response: serde_json::Value = serde_json::from_str(&response_line).unwrap();
        assert_eq!(response["id"], format!("req_{}", i));
        assert_eq!(response["ok"], true);
        assert_eq!(response["result"]["count"], i);
    }

    drop(stdin);
    let _ = child.wait();
}

/// Test that closing stdin causes Python to exit gracefully
#[test]
#[ignore = "requires Python runtime, run with --ignored"]
fn python_exits_on_stdin_close() {
    let python_code = r#"
import json
import sys

# Send ready signal
print(json.dumps({"type": "ready", "handlers": []}), flush=True)

# Wait for stdin to close
while True:
    line = sys.stdin.readline()
    if not line:
        # EOF - exit gracefully
        sys.exit(0)
"#;

    let mut child = Command::new(get_python_command())
        .args(["-c", python_code])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed to start Python");

    let stdin = child.stdin.take().expect("Failed to get stdin");
    let stdout = child.stdout.take().expect("Failed to get stdout");
    let mut reader = BufReader::new(stdout);

    // Read ready signal
    let mut ready_line = String::new();
    reader.read_line(&mut ready_line).unwrap();
    assert!(ready_line.contains("ready"));

    // Close stdin
    drop(stdin);

    // Wait for process to exit (with timeout via try_wait loop)
    let mut attempts = 0;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                attempts += 1;
                if attempts > 50 {
                    // 5 seconds timeout
                    child.kill().ok();
                    panic!("Process did not exit within timeout");
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(e) => panic!("Failed to wait: {}", e),
        }
    };

    assert!(status.success(), "Python should exit with code 0");
}

/// Test error handling when Python returns an error
#[test]
#[ignore = "requires Python runtime, run with --ignored"]
fn python_error_response() {
    let python_code = r#"
import json
import sys

# Send ready signal
print(json.dumps({"type": "ready", "handlers": ["fail"]}), flush=True)

# Handle request with error
line = sys.stdin.readline()
if line:
    request = json.loads(line)
    response = {
        "id": request.get("id", ""),
        "ok": False,
        "error": {
            "name": "TestError",
            "message": "This is a test error"
        }
    }
    print(json.dumps(response), flush=True)
"#;

    let mut child = Command::new(get_python_command())
        .args(["-c", python_code])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed to start Python");

    let mut stdin = child.stdin.take().expect("Failed to get stdin");
    let stdout = child.stdout.take().expect("Failed to get stdout");
    let mut reader = BufReader::new(stdout);

    // Read ready signal
    let mut ready_line = String::new();
    reader.read_line(&mut ready_line).unwrap();

    // Send request
    let request = serde_json::json!({
        "id": "error_test",
        "method": "fail",
        "params": null
    });
    writeln!(stdin, "{}", request).unwrap();
    stdin.flush().unwrap();

    // Read error response
    let mut response_line = String::new();
    reader.read_line(&mut response_line).unwrap();

    let response: serde_json::Value = serde_json::from_str(&response_line).unwrap();
    assert_eq!(response["id"], "error_test");
    assert_eq!(response["ok"], false);
    assert_eq!(response["error"]["name"], "TestError");
    assert!(response["error"]["message"]
        .as_str()
        .unwrap()
        .contains("test error"));

    drop(stdin);
    let _ = child.wait();
}

// =============================================================================
// JSON-RPC message format tests (no Python runtime required)
// =============================================================================

mod jsonrpc_format_tests {
    /// Build a JSON-RPC call message
    fn make_call(id: &str, method: &str, params: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "type": "call",
            "id": id,
            "method": method,
            "params": params
        })
    }

    /// Build a successful JSON-RPC result message
    fn make_result_ok(id: &str, result: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "ok": true,
            "result": result
        })
    }

    /// Build a failed JSON-RPC result message
    fn make_result_err(id: &str, name: &str, message: &str) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "ok": false,
            "error": { "name": name, "message": message }
        })
    }

    #[test]
    fn call_message_fields() {
        let msg = make_call("req_1", "api.echo", serde_json::json!({"text": "hello"}));
        assert_eq!(msg["type"], "call");
        assert_eq!(msg["id"], "req_1");
        assert_eq!(msg["method"], "api.echo");
        assert_eq!(msg["params"]["text"], "hello");
    }

    #[test]
    fn result_ok_fields() {
        let r = make_result_ok("req_1", serde_json::json!(42));
        assert_eq!(r["id"], "req_1");
        assert_eq!(r["ok"], true);
        assert_eq!(r["result"], 42);
    }

    #[test]
    fn result_err_fields() {
        let r = make_result_err("req_1", "NotFound", "method not found");
        assert_eq!(r["id"], "req_1");
        assert_eq!(r["ok"], false);
        assert_eq!(r["error"]["name"], "NotFound");
        assert!(r["error"]["message"].as_str().unwrap().contains("not found"));
    }

    #[test]
    fn call_id_roundtrip_in_response() {
        let call_id = "av_call_99887766_42";
        let call = make_call(call_id, "get_samples", serde_json::json!(null));
        let response = make_result_ok(call_id, serde_json::json!([]));

        let req_id = call["id"].as_str().unwrap();
        let res_id = response["id"].as_str().unwrap();
        assert_eq!(req_id, res_id);
    }

    #[test]
    fn params_null_is_valid() {
        let msg = make_call("id1", "list_items", serde_json::json!(null));
        assert!(msg["params"].is_null());
    }

    #[test]
    fn params_array_is_valid() {
        let msg = make_call("id2", "add", serde_json::json!([1, 2]));
        assert!(msg["params"].is_array());
        assert_eq!(msg["params"][0], 1);
        assert_eq!(msg["params"][1], 2);
    }

    #[test]
    fn params_object_is_valid() {
        let msg = make_call("id3", "export", serde_json::json!({"path": "/tmp/out.fbx"}));
        assert!(msg["params"].is_object());
        assert_eq!(msg["params"]["path"], "/tmp/out.fbx");
    }

    #[test]
    fn ready_signal_required_fields() {
        let ready = serde_json::json!({
            "type": "ready",
            "handlers": ["get_samples", "run_sample"]
        });
        assert_eq!(ready["type"], "ready");
        assert!(ready["handlers"].is_array());
        assert_eq!(ready["handlers"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn ready_signal_empty_handlers() {
        let ready = serde_json::json!({"type": "ready", "handlers": []});
        assert_eq!(ready["type"], "ready");
        assert_eq!(ready["handlers"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn ready_signal_multiple_handlers() {
        let handlers = vec!["get_samples", "get_categories", "run_sample", "get_config"];
        let ready = serde_json::json!({ "type": "ready", "handlers": handlers });
        assert_eq!(ready["handlers"].as_array().unwrap().len(), 4);
    }

    #[test]
    fn error_response_with_code() {
        let r = serde_json::json!({
            "id": "x",
            "ok": false,
            "error": {
                "name": "TypeError",
                "message": "Expected string",
                "code": 400
            }
        });
        assert_eq!(r["error"]["code"], 400);
    }

    #[test]
    fn call_method_namespace_dot() {
        let namespaces = ["api.echo", "tool.apply", "scene.export", "dcc.maya.get_selection"];
        for ns in &namespaces {
            let msg = make_call("id", ns, serde_json::json!(null));
            assert_eq!(msg["method"].as_str().unwrap(), *ns);
        }
    }

    #[test]
    fn result_can_be_object() {
        let r = make_result_ok("x", serde_json::json!({"items": [1, 2, 3], "total": 3}));
        assert_eq!(r["result"]["total"], 3);
        assert_eq!(r["result"]["items"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn result_can_be_string() {
        let r = make_result_ok("x", serde_json::json!("hello world"));
        assert_eq!(r["result"], "hello world");
    }

    #[test]
    fn result_can_be_bool() {
        let r = make_result_ok("x", serde_json::json!(false));
        assert_eq!(r["result"], false);
    }

    #[test]
    fn ipc_serialized_parse() {
        let raw = r#"{"type":"call","id":"1","method":"api.echo","params":{"message":"test"}}"#;
        let msg: serde_json::Value = serde_json::from_str(raw).unwrap();
        assert_eq!(msg["type"], "call");
        assert_eq!(msg["id"], "1");
        assert_eq!(msg["method"], "api.echo");
        assert_eq!(msg["params"]["message"], "test");
    }

    #[test]
    fn backend_exit_error_format() {
        let r = serde_json::json!({
            "id": "abc",
            "ok": false,
            "error": {
                "name": "PythonBackendError",
                "message": "Python backend process has exited"
            }
        });
        assert_eq!(r["ok"], false);
        assert_eq!(r["error"]["name"], "PythonBackendError");
        assert!(r["error"]["message"].as_str().unwrap().contains("exited"));
    }

    #[test]
    fn method_not_found_error() {
        let r = make_result_err("y", "MethodNotFound", "method 'unknown' not found");
        assert_eq!(r["error"]["name"], "MethodNotFound");
        assert!(r["error"]["message"].as_str().unwrap().contains("unknown"));
    }

    #[test]
    fn unique_call_ids_sequential() {
        let ids: Vec<String> = (0..10).map(|i| format!("av_call_{}_1", i)).collect();
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(unique.len(), 10);
    }

    #[test]
    fn call_serialize_compact() {
        let msg = make_call("1", "echo", serde_json::json!(null));
        let s = serde_json::to_string(&msg).unwrap();
        // Should not contain pretty-print newlines at the top level
        assert!(!s.starts_with("{\n"));
    }

    // ─── Extended coverage ────────────────────────────────────────────────

    #[test]
    fn call_method_empty_params() {
        let msg = make_call("a", "list_all", serde_json::json!({}));
        assert_eq!(msg["method"], "list_all");
        assert!(msg["params"].is_object());
    }

    #[test]
    fn result_ok_null_result() {
        let r = make_result_ok("n", serde_json::json!(null));
        assert!(r["result"].is_null());
    }

    #[test]
    fn result_ok_nested_array() {
        let r = make_result_ok("arr", serde_json::json!([[1, 2], [3, 4]]));
        assert_eq!(r["result"][0][1], 2);
        assert_eq!(r["result"][1][0], 3);
    }

    #[test]
    fn error_message_is_string() {
        let r = make_result_err("e", "SomeError", "Something went wrong");
        assert!(r["error"]["message"].is_string());
    }

    #[test]
    fn error_name_is_string() {
        let r = make_result_err("e", "NetworkError", "timeout");
        assert!(r["error"]["name"].is_string());
    }

    #[test]
    fn call_id_is_string() {
        let msg = make_call("my_id", "fn", serde_json::json!(null));
        assert!(msg["id"].is_string());
    }

    #[test]
    fn type_field_is_call() {
        let msg = make_call("1", "fn", serde_json::json!(null));
        assert_eq!(msg["type"].as_str().unwrap(), "call");
    }

    #[test]
    fn ok_field_is_bool_true() {
        let r = make_result_ok("x", serde_json::json!(0));
        assert!(r["ok"].is_boolean());
        assert!(r["ok"].as_bool().unwrap());
    }

    #[test]
    fn ok_field_is_bool_false() {
        let r = make_result_err("x", "Err", "msg");
        assert!(r["ok"].is_boolean());
        assert!(!r["ok"].as_bool().unwrap());
    }

    #[test]
    fn call_with_numeric_param() {
        let msg = make_call("n1", "set_volume", serde_json::json!({"level": 0.8}));
        let level = msg["params"]["level"].as_f64().unwrap();
        assert!((level - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn call_with_bool_param() {
        let msg = make_call("b1", "set_enabled", serde_json::json!({"enabled": true}));
        assert_eq!(msg["params"]["enabled"], true);
    }

    #[test]
    fn ids_are_unique_across_calls() {
        let ids: Vec<serde_json::Value> = (0..5)
            .map(|i| make_call(&format!("id_{}", i), "fn", serde_json::json!(null))["id"].clone())
            .collect();
        let as_strings: std::collections::HashSet<String> = ids
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        assert_eq!(as_strings.len(), 5, "IDs should all be unique");
    }

    #[test]
    fn error_without_code_field() {
        let r = make_result_err("z", "BaseError", "no code");
        // "code" should not be present in a basic error
        assert!(r["error"]["code"].is_null());
    }

    #[test]
    fn result_ok_with_large_number() {
        let r = make_result_ok("big", serde_json::json!(u64::MAX));
        // u64::MAX may exceed serde_json's i64, but should be representable
        assert!(!r["result"].is_null());
    }

    #[test]
    fn params_deeply_nested() {
        let msg = make_call(
            "deep",
            "process",
            serde_json::json!({"a": {"b": {"c": {"d": 99}}}}),
        );
        assert_eq!(msg["params"]["a"]["b"]["c"]["d"], 99);
    }

    #[test]
    fn ready_signal_handlers_are_strings() {
        let ready = serde_json::json!({"type": "ready", "handlers": ["fn1", "fn2"]});
        for h in ready["handlers"].as_array().unwrap() {
            assert!(h.is_string());
        }
    }

    #[test]
    fn call_method_with_underscore() {
        let msg = make_call("u", "export_scene_as_fbx", serde_json::json!(null));
        assert_eq!(msg["method"], "export_scene_as_fbx");
    }
}

/// Test the actual packed.py module if available
#[test]
#[ignore = "requires Python runtime, run with --ignored"]
fn real_packed_module() {
    // This test uses the actual packed.py module
    let project_root = std::env::current_dir()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    let python_path = project_root.join("python");

    // Create a test script that uses the real packed module
    let python_code = format!(
        r#"
import sys
sys.path.insert(0, r"{}")

import json
from auroraview.core.packed import _handle_request

# Test _handle_request with a mock handler
def echo_handler(message):
    return {{"echo": message}}

bound_functions = {{"echo": echo_handler}}

# Test successful call
request = {{"id": "test_1", "method": "echo", "params": {{"message": "hello"}}}}
response = _handle_request(request, bound_functions)
print(json.dumps(response), flush=True)

# Test method not found
request2 = {{"id": "test_2", "method": "unknown", "params": None}}
response2 = _handle_request(request2, bound_functions)
print(json.dumps(response2), flush=True)
"#,
        python_path.display()
    );

    let output = Command::new(get_python_command())
        .args(["-c", &python_code])
        .output()
        .expect("Failed to run Python");

    if !output.status.success() {
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Python script failed");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.trim().lines().collect();

    assert_eq!(lines.len(), 2, "Expected 2 response lines");

    // Check first response (successful)
    let response1: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(response1["id"], "test_1");
    assert_eq!(response1["ok"], true);
    assert_eq!(response1["result"]["echo"], "hello");

    // Check second response (method not found)
    let response2: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
    assert_eq!(response2["id"], "test_2");
    assert_eq!(response2["ok"], false);
    assert_eq!(response2["error"]["name"], "MethodNotFound");
}
