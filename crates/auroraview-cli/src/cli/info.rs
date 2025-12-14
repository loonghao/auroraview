//! Info command - Display environment and version information

use anyhow::Result;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Run a command with timeout and get output
fn run_command_with_timeout(cmd: &str, args: &[&str], timeout: Duration) -> Option<String> {
    let mut child = Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    let start = Instant::now();

    // Poll for completion with timeout
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if status.success() {
                    let output = child.wait_with_output().ok()?;
                    return String::from_utf8(output.stdout).ok();
                }
                return None;
            }
            Ok(None) => {
                if start.elapsed() > timeout {
                    // Timeout - kill the process
                    let _ = child.kill();
                    let _ = child.wait();
                    return None;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(_) => return None,
        }
    }
}

/// Check if a command exists with timeout
fn command_exists_with_timeout(cmd: &str, timeout: Duration) -> bool {
    let mut child = match Command::new(cmd)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return false,
    };

    let start = Instant::now();

    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status.success(),
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return false;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(_) => return false,
        }
    }
}

/// Get rustc version (simplified)
fn rustc_version() -> String {
    run_command_with_timeout("rustc", &["--version"], Duration::from_secs(5))
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

    // Check for required tools (with 3 second timeout each)
    println!("Dependencies:");
    let timeout = Duration::from_secs(3);

    // Check cargo
    let cargo_ok = command_exists_with_timeout("cargo", timeout);
    println!(
        "  Cargo: {}",
        if cargo_ok { "Available" } else { "Not found" }
    );

    // Check PyOxidizer
    let pyoxidizer_ok = command_exists_with_timeout("pyoxidizer", timeout);
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
