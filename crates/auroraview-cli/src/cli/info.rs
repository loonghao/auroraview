//! Info command - Display environment and version information

use anyhow::Result;
use std::process::{Command, Stdio};

/// Run a command and get output
fn run_command(cmd: &str, args: &[&str]) -> Option<String> {
    let child = Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    let output = child.wait_with_output().ok()?;

    if output.status.success() {
        String::from_utf8(output.stdout).ok()
    } else {
        None
    }
}

/// Check if a command exists and returns success
fn command_exists(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Get rustc version (simplified)
fn rustc_version() -> String {
    run_command("rustc", &["--version"])
        .unwrap_or_else(|| "unknown".to_string())
        .trim()
        .to_string()
}

/// Display environment and version information
pub fn run_info() -> Result<()> {
    println!("AuroraView CLI Information\n");
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!("Rust Version: {}", rustc_version());
    println!();

    // Check for required tools
    println!("Dependencies:");

    // Check cargo
    let cargo_ok = command_exists("cargo");
    println!(
        "  Cargo: {}",
        if cargo_ok { "Available" } else { "Not found" }
    );

    // Check PyOxidizer
    let pyoxidizer_ok = command_exists("pyoxidizer");
    println!(
        "  PyOxidizer: {} (required for fullstack mode)",
        if pyoxidizer_ok {
            "Available"
        } else {
            "Not found"
        }
    );

    println!();
    println!("Available Commands:");
    println!("  run   - Launch a WebView window");
    println!("  pack  - Package an application into a standalone executable");
    println!("  icon  - Icon utilities (compress, convert)");
    println!("  info  - Show this information");
    println!();
    println!("Examples:");
    println!("  auroraview run --url https://example.com");
    println!("  auroraview run --html ./index.html");
    println!("  auroraview pack --url www.baidu.com --output my-app");
    println!("  auroraview pack --frontend ./dist --output my-app");
    println!();

    Ok(())
}
