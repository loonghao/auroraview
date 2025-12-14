//! Integration tests for Python IPC communication
//!
//! These tests verify the JSON-RPC communication between Rust and Python.

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
fn test_python_sends_ready_signal() {
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
fn test_python_handles_multiple_requests() {
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
fn test_python_exits_on_stdin_close() {
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
fn test_python_error_response() {
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

/// Test the actual packed.py module if available
#[test]
fn test_real_packed_module() {
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
