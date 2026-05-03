//! General utilities
//!
//! Common utility functions used across AuroraView components.

use std::path::PathBuf;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Initialize logging for the library
///
/// Sets up tracing with environment-based filtering via RUST_LOG
pub fn init_logging() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("warn,auroraview=info"));

        tracing_subscriber::registry()
            .with(fmt::layer().with_target(true).with_thread_ids(true))
            .with(filter)
            .try_init()
            .ok();
    });
}

/// Escape a string for use in JavaScript
pub fn escape_js_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Escape JSON string for embedding in JavaScript code
///
/// This is different from `escape_js_string` - it handles JSON values
/// that need to be embedded in JavaScript string literals.
pub fn escape_json_for_js(json: &str) -> String {
    json.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

/// Parse a size string like "800x600" into (width, height)
pub fn parse_size(s: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = s.split('x').collect();
    if parts.len() == 2 {
        let width = parts[0].trim().parse().ok()?;
        let height = parts[1].trim().parse().ok()?;
        Some((width, height))
    } else {
        None
    }
}

/// Get the WebView data directory for user data storage
///
/// On Windows: `%LOCALAPPDATA%\AuroraView\webview_data`
/// On macOS: `~/Library/Application Support/AuroraView/webview_data`
/// On Linux: `~/.local/share/auroraview/webview_data`
pub fn get_webview_data_dir() -> PathBuf {
    let base = if cfg!(windows) {
        std::env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| dirs::data_local_dir().unwrap_or_else(|| PathBuf::from(".")))
    } else {
        dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."))
    };

    base.join("AuroraView").join("webview_data")
}

/// Get the extensions directory for browser extensions
///
/// On Windows: `%LOCALAPPDATA%\AuroraView\extensions`
/// On macOS: `~/Library/Application Support/AuroraView/extensions`
/// On Linux: `~/.local/share/auroraview/extensions`
pub fn get_extensions_dir() -> PathBuf {
    let base = if cfg!(windows) {
        std::env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| dirs::data_local_dir().unwrap_or_else(|| PathBuf::from(".")))
    } else {
        dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."))
    };

    base.join("AuroraView").join("extensions")
}

/// Get the cache directory for temporary files
///
/// On Windows: `%LOCALAPPDATA%\AuroraView\cache`
/// On macOS: `~/Library/Caches/AuroraView`
/// On Linux: `~/.cache/auroraview`
pub fn get_cache_dir() -> PathBuf {
    if cfg!(windows) {
        std::env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| dirs::cache_dir().unwrap_or_else(|| PathBuf::from(".")))
            .join("AuroraView")
            .join("cache")
    } else {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("auroraview")
    }
}

/// Ensure a directory exists, creating it if necessary
pub fn ensure_dir_exists(path: &PathBuf) -> std::io::Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Check if a process with the given PID is still running.
///
/// On Windows, uses `OpenProcess` with `PROCESS_QUERY_LIMITED_INFORMATION`.
/// On macOS/Linux, uses `kill -0` to probe without sending a signal.
///
/// Returns `true` if the process exists.
#[cfg(target_os = "windows")]
pub fn is_process_alive(pid: u32) -> bool {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

    // SAFETY: OpenProcess and CloseHandle are safe Win32 calls.
    // OpenProcess returns an error if the process doesn't exist, and
    // CloseHandle releases the kernel handle. No UB possible here.
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid);
        if let Ok(h) = handle {
            let _ = CloseHandle(h);
            true
        } else {
            false
        }
    }
}

/// Check if a process with the given PID is still running.
#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn is_process_alive(pid: u32) -> bool {
    use std::process::Command;

    // `kill -0` checks if the process exists without sending a signal
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Check if a process with the given PID is still running (unsupported platform stub).
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub fn is_process_alive(_pid: u32) -> bool {
    // Cannot determine on this platform; assume alive
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_js_string_basic() {
        assert_eq!(escape_js_string("hello"), "hello");
    }

    #[test]
    fn test_escape_js_string_backslash() {
        assert_eq!(escape_js_string("hello\\world"), "hello\\\\world");
    }

    #[test]
    fn test_escape_js_string_double_quote() {
        assert_eq!(escape_js_string("hello \"world\""), "hello \\\"world\\\"");
    }

    #[test]
    fn test_escape_js_string_single_quote() {
        assert_eq!(escape_js_string("it's"), "it\\'s");
    }

    #[test]
    fn test_escape_js_string_newline() {
        assert_eq!(escape_js_string("line1\nline2"), "line1\\nline2");
    }

    #[test]
    fn test_escape_js_string_carriage_return() {
        assert_eq!(escape_js_string("line1\rline2"), "line1\\rline2");
    }

    #[test]
    fn test_escape_js_string_tab() {
        assert_eq!(escape_js_string("col1\tcol2"), "col1\\tcol2");
    }

    #[test]
    fn test_escape_json_for_js_basic() {
        assert_eq!(
            escape_json_for_js("{\"key\": \"value\"}"),
            "{\\\"key\\\": \\\"value\\\"}"
        );
    }

    #[test]
    fn test_escape_json_for_js_backslash() {
        assert_eq!(escape_json_for_js("C:\\path"), "C:\\\\path");
    }

    #[test]
    fn test_escape_json_for_js_newline() {
        assert_eq!(escape_json_for_js("line1\nline2"), "line1\\nline2");
    }

    #[test]
    fn test_parse_size_valid() {
        assert_eq!(parse_size("800x600"), Some((800, 600)));
        assert_eq!(parse_size("1024x768"), Some((1024, 768)));
    }

    #[test]
    fn test_parse_size_invalid() {
        assert_eq!(parse_size("800"), None);
        assert_eq!(parse_size(""), None);
        assert_eq!(parse_size("abcxdef"), None);
    }
}
