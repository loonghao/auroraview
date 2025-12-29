//! Integration tests for CLI

use std::path::PathBuf;
use std::process::Command;

/// Get the path to the compiled CLI binary
fn cli_binary() -> PathBuf {
    // Cargo sets this for integration tests (when the binary target exists).
    // Keep a fallback path resolver for robustness.
    if let Some(path) = std::env::var_os("CARGO_BIN_EXE_auroraview-cli") {
        return PathBuf::from(path);
    }

    // Get the path to the test executable, then navigate to the binary
    let mut path = std::env::current_exe().expect("Failed to get current exe path");

    // Navigate up from the test binary to the target directory
    // test binary is at: target/debug/deps/cli_tests-xxx
    // we want: target/debug/auroraview-cli.exe
    path.pop(); // Remove test binary name
    path.pop(); // Remove "deps"

    #[cfg(windows)]
    path.push("auroraview-cli.exe");
    #[cfg(not(windows))]
    path.push("auroraview-cli");

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

#[test]
fn test_cli_info() {
    let output = Command::new(cli_binary())
        .args(["info"])
        .output()
        .expect("Failed to run CLI");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("AuroraView CLI Information"));
    assert!(stdout.contains("Version:"));
    assert!(stdout.contains("Dependencies:"));
    assert!(stdout.contains("Cargo:"));
}

#[test]
fn test_cli_pack_new_options() {
    let output = Command::new(cli_binary())
        .args(["pack", "--help"])
        .output()
        .expect("Failed to run CLI");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Verify new options are available
    assert!(stdout.contains("--frameless"));
    assert!(stdout.contains("--always-on-top"));
    assert!(stdout.contains("--no-resize"));
    assert!(stdout.contains("--user-agent"));
}
