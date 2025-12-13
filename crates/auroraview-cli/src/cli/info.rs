//! Info command - Display environment and version information

use anyhow::Result;

/// Get rustc version (simplified)
fn rustc_version() -> String {
    std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
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
    let cargo_ok = std::process::Command::new("cargo")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    println!(
        "  Cargo: {}",
        if cargo_ok {
            "Available"
        } else {
            "Not found"
        }
    );

    // Check PyOxidizer
    let pyoxidizer_ok = std::process::Command::new("pyoxidizer")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
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
