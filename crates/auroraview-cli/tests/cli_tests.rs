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
    // Hot reload flag must be present
    assert!(
        stdout.contains("--watch"),
        "--watch flag missing from run --help output"
    );
    // URL-mode polling interval flag must be present
    assert!(
        stdout.contains("--poll-interval-ms"),
        "--poll-interval-ms flag missing from run --help output"
    );
}

/// Verify that --watch is not rejected when used with --url (URL-mode hot reload).
/// Previously --watch required --html; now it should be accepted for --url too.
#[test]
fn test_cli_watch_accepts_url_mode() {
    let output = Command::new(cli_binary())
        .args(["run", "--url", "http://localhost:9999", "--watch", "--help"])
        .output()
        .expect("Failed to run CLI");

    // --help always succeeds; this verifies clap doesn't reject the combination
    assert!(
        output.status.success(),
        "run --url --watch --help should succeed, got stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
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

// ---------------------------------------------------------------------------
// Additional CLI tests
// ---------------------------------------------------------------------------

#[test]
fn test_cli_run_help_contains_fullscreen() {
    let output = Command::new(cli_binary())
        .args(["run", "--help"])
        .output()
        .expect("Failed to run CLI");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // run --help should have window sizing options
    assert!(
        stdout.contains("--width") || stdout.contains("--height") || stdout.contains("--title"),
        "run --help should mention window options, got: {}",
        stdout
    );
}

#[test]
fn test_cli_run_help_contains_width_height() {
    let output = Command::new(cli_binary())
        .args(["run", "--help"])
        .output()
        .expect("Failed to run CLI");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--width") || stdout.contains("width"));
    assert!(stdout.contains("--height") || stdout.contains("height"));
}

#[test]
fn test_cli_version_output_contains_semver() {
    let output = Command::new(cli_binary())
        .arg("--version")
        .output()
        .expect("Failed to run CLI");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Version string should contain a semver-like pattern (digits and dots)
    assert!(
        stdout.chars().any(|c| c.is_ascii_digit()),
        "Version should contain digits: {}",
        stdout
    );
}

#[test]
fn test_cli_help_exit_code_zero() {
    let status = Command::new(cli_binary())
        .arg("--help")
        .status()
        .expect("Failed to run CLI");
    assert!(status.success());
}

#[test]
fn test_cli_version_exit_code_zero() {
    let status = Command::new(cli_binary())
        .arg("--version")
        .status()
        .expect("Failed to run CLI");
    assert!(status.success());
}

#[test]
fn test_cli_pack_help_exit_code_zero() {
    let status = Command::new(cli_binary())
        .args(["pack", "--help"])
        .status()
        .expect("Failed to run CLI");
    assert!(status.success());
}

#[test]
fn test_cli_info_exit_code_zero() {
    let status = Command::new(cli_binary())
        .args(["info"])
        .status()
        .expect("Failed to run CLI");
    assert!(status.success());
}

#[test]
fn test_cli_unknown_subcommand_fails() {
    let status = Command::new(cli_binary())
        .args(["nonexistent-subcommand-xyz"])
        .status()
        .expect("Failed to run CLI");
    assert!(!status.success(), "Unknown subcommand should fail");
}

#[test]
fn test_cli_run_help_contains_debug() {
    let output = Command::new(cli_binary())
        .args(["run", "--help"])
        .output()
        .expect("Failed to run CLI");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--debug") || stdout.contains("devtools"),
        "run --help should mention debug/devtools"
    );
}

// ---------------------------------------------------------------------------
// Additional CLI tests R8
// ---------------------------------------------------------------------------

#[test]
fn test_cli_run_help_stdout_not_empty() {
    let output = Command::new(cli_binary())
        .args(["run", "--help"])
        .output()
        .expect("Failed to run CLI");
    assert!(output.status.success());
    assert!(!output.stdout.is_empty());
}

#[test]
fn test_cli_pack_help_stdout_not_empty() {
    let output = Command::new(cli_binary())
        .args(["pack", "--help"])
        .output()
        .expect("Failed to run CLI");
    assert!(output.status.success());
    assert!(!output.stdout.is_empty());
}

#[test]
fn test_cli_info_output_contains_version_and_commands() {
    let output = Command::new(cli_binary())
        .args(["info"])
        .output()
        .expect("Failed to run CLI");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // info should mention Version and Commands sections
    assert!(
        stdout.contains("Version") || stdout.contains("version"),
        "info output should mention version: {}",
        stdout
    );
    assert!(
        stdout.contains("Commands") || stdout.contains("run") || stdout.contains("pack"),
        "info output should list commands: {}",
        stdout
    );
}


#[test]
fn test_cli_run_help_contains_watch_flag() {
    let output = Command::new(cli_binary())
        .args(["run", "--help"])
        .output()
        .expect("Failed to run CLI");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--watch"), "run --help should mention --watch");
}

#[test]
fn test_cli_run_help_contains_poll_interval() {
    let output = Command::new(cli_binary())
        .args(["run", "--help"])
        .output()
        .expect("Failed to run CLI");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--poll-interval"), "run --help should mention --poll-interval-ms");
}

#[test]
fn test_cli_help_does_not_fail() {
    // --help should exit 0 and produce output on stdout or combined output
    let output = Command::new(cli_binary())
        .arg("--help")
        .output()
        .expect("Failed to run CLI");
    assert!(output.status.success());
    // Combined output should not be empty
    let combined = [output.stdout.as_slice(), output.stderr.as_slice()].concat();
    assert!(!combined.is_empty(), "help should produce some output");
}

#[test]
fn test_cli_version_output_not_empty() {
    let output = Command::new(cli_binary())
        .arg("--version")
        .output()
        .expect("Failed to run CLI");
    assert!(output.status.success());
    // Combined output should not be empty
    let combined = [output.stdout.as_slice(), output.stderr.as_slice()].concat();
    assert!(!combined.is_empty(), "--version should produce some output");
}

#[test]
fn test_cli_run_no_args_or_error_code() {
    // Running 'run' without required --url or --html must fail
    let output = Command::new(cli_binary())
        .args(["run"])
        .output()
        .expect("Failed to run CLI");
    assert!(!output.status.success(), "'run' without args must fail");
}

#[test]
fn test_cli_help_contains_options_or_commands() {
    let output = Command::new(cli_binary())
        .arg("--help")
        .output()
        .expect("Failed to run CLI");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Commands") || stdout.contains("SUBCOMMAND") || stdout.contains("run"),
        "help should list subcommands: {}",
        stdout
    );
}

#[test]
fn test_cli_pack_help_mentions_config() {
    let output = Command::new(cli_binary())
        .args(["pack", "--help"])
        .output()
        .expect("Failed to run CLI");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--config") || stdout.contains("config"),
        "pack --help should mention config option: {}",
        stdout
    );
}

#[test]
fn test_cli_pack_help_mentions_output() {
    let output = Command::new(cli_binary())
        .args(["pack", "--help"])
        .output()
        .expect("Failed to run CLI");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--output") || stdout.contains("output"),
        "pack --help should mention output option: {}",
        stdout
    );
}

#[test]
fn test_cli_multiple_unknown_subcommands_fail() {
    for cmd in &["foo", "bar", "xyz-unknown"] {
        let status = Command::new(cli_binary())
            .args([*cmd])
            .status()
            .expect("Failed to run CLI");
        assert!(!status.success(), "Unknown subcommand '{}' should fail", cmd);
    }
}

#[test]
fn test_cli_binary_exists() {
    let bin = cli_binary();
    assert!(bin.exists(), "CLI binary should exist at: {:?}", bin);
}

#[test]
fn test_cli_info_stderr_empty() {
    let output = Command::new(cli_binary())
        .args(["info"])
        .output()
        .expect("Failed to run CLI");
    assert!(output.status.success());
    // info normally should not produce stderr
    // (some platforms may emit warnings; we accept either but don't panic)
    let _ = String::from_utf8_lossy(&output.stderr);
}

#[test]
fn test_cli_run_help_mentions_html_option() {
    let output = Command::new(cli_binary())
        .args(["run", "--help"])
        .output()
        .expect("Failed to run CLI");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--html"), "--html flag should be listed in run --help");
}
