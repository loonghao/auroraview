//! Integration tests for CLI

use std::path::PathBuf;
use std::process::Command;

/// Get the path to the compiled CLI binary
fn cli_binary() -> PathBuf {
    // Get the path to the test executable, then navigate to the binary
    let mut path = std::env::current_exe().expect("Failed to get current exe path");

    // Navigate up from the test binary to the target directory
    // test binary is at: target/debug/deps/cli_tests-xxx
    // we want: target/debug/auroraview.exe
    path.pop(); // Remove test binary name
    path.pop(); // Remove "deps"

    #[cfg(windows)]
    path.push("auroraview.exe");
    #[cfg(not(windows))]
    path.push("auroraview");

    path
}

#[test]
fn test_cli_help() {
    let output = Command::new(cli_binary())
        .arg("--help")
        .output()
        .expect("Failed to run CLI");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("AuroraView"));
    assert!(stdout.contains("run"));
    assert!(stdout.contains("pack"));
}

#[test]
fn test_cli_version() {
    let output = Command::new(cli_binary())
        .arg("--version")
        .output()
        .expect("Failed to run CLI");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("auroraview"));
}

#[test]
fn test_cli_run_help() {
    let output = Command::new(cli_binary())
        .args(["run", "--help"])
        .output()
        .expect("Failed to run CLI");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--url"));
    assert!(stdout.contains("--html"));
    assert!(stdout.contains("--title"));
}

#[test]
fn test_cli_pack_help() {
    let output = Command::new(cli_binary())
        .args(["pack", "--help"])
        .output()
        .expect("Failed to run CLI");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--url"));
    assert!(stdout.contains("--frontend"));
    assert!(stdout.contains("--output"));
}

#[test]
fn test_cli_run_missing_args() {
    let output = Command::new(cli_binary())
        .args(["run"])
        .output()
        .expect("Failed to run CLI");

    // Should fail when no URL or HTML is provided
    // The CLI requires either --url or --html
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Should show error about missing required arguments
    // Note: Error message starts with capital "Error:"
    assert!(
        stderr.to_lowercase().contains("error")
            || stderr.contains("--url")
            || stderr.contains("--html"),
        "Expected error message about missing args, got: {}",
        stderr
    );
}
