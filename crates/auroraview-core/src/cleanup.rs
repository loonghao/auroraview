//! WebView User Data Directory Cleanup
//!
//! This module handles cleanup of stale WebView user data directories
//! left behind by crashed or improperly terminated processes.
//!
//! # Problem
//! Each AuroraView process creates a unique WebView user data folder:
//! - Windows: `%LOCALAPPDATA%\AuroraView\WebView2\process_{PID}`
//! - macOS: `~/Library/Application Support/AuroraView/WebView/process_{PID}`
//! - Linux: `~/.local/share/auroraview/webview/process_{PID}`
//!
//! When processes crash or are terminated without proper cleanup:
//! - The directories and LOCK files remain
//! - WebView may fail to initialize if it tries to use locked resources
//! - New processes may hang trying to acquire locks
//!
//! # Solution
//! On startup, scan the WebView directory and remove directories whose
//! owning process (by PID) is no longer running.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{debug, info, warn};

/// Flag to ensure cleanup only runs once per process
static CLEANUP_DONE: AtomicBool = AtomicBool::new(false);

/// Get the base WebView data directory path for the current platform
pub fn get_webview_base_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let local_app_data = std::env::var("LOCALAPPDATA").ok()?;
        Some(
            PathBuf::from(local_app_data)
                .join("AuroraView")
                .join("WebView2"),
        )
    }

    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir()?;
        Some(
            home.join("Library")
                .join("Application Support")
                .join("AuroraView")
                .join("WebView"),
        )
    }

    #[cfg(target_os = "linux")]
    {
        let data_dir = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .unwrap_or_default()
                    .join(".local")
                    .join("share")
            });
        Some(data_dir.join("auroraview").join("webview"))
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        None // Unsupported platform
    }
}

/// Check if a process with the given PID is still running
#[cfg(target_os = "windows")]
fn is_process_alive(pid: u32) -> bool {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

    // Skip checking current process
    if pid == std::process::id() {
        return true;
    }

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

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn is_process_alive(pid: u32) -> bool {
    use std::process::Command;

    // Skip checking current process
    if pid == std::process::id() {
        return true;
    }

    // Use kill -0 to check if process exists (doesn't actually send a signal)
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn is_process_alive(_pid: u32) -> bool {
    true // Always return true on unsupported platforms to prevent accidental cleanup
}

/// Extract PID from a directory name like "process_12345"
fn extract_pid_from_dir_name(name: &str) -> Option<u32> {
    if let Some(pid_str) = name.strip_prefix("process_") {
        pid_str.parse().ok()
    } else {
        None
    }
}

/// Clean up stale WebView user data directories
///
/// This function scans the WebView data directory for directories
/// named `process_XXXXX` and removes those whose owning process is dead.
///
/// # Returns
/// - `Ok(count)` - Number of directories cleaned up
/// - `Err(msg)` - Error message if cleanup failed
///
/// # Safety
/// This function:
/// - Only runs once per process (uses atomic flag)
/// - Skips the current process's directory
/// - Only removes directories that match the `process_XXXXX` pattern
/// - Checks process status before removal
pub fn cleanup_stale_webview_dirs() -> Result<usize, String> {
    // Only run once per process
    if CLEANUP_DONE.swap(true, Ordering::SeqCst) {
        debug!("[cleanup] Already performed cleanup in this process");
        return Ok(0);
    }

    let base_dir = match get_webview_base_dir() {
        Some(dir) => dir,
        None => {
            debug!("[cleanup] WebView cleanup not supported on this platform");
            return Ok(0);
        }
    };

    if !base_dir.exists() {
        debug!("[cleanup] WebView base dir does not exist: {:?}", base_dir);
        return Ok(0);
    }

    let current_pid = std::process::id();
    let mut cleaned = 0;
    let mut errors = Vec::new();

    info!(
        "[cleanup] Scanning for stale WebView directories in {:?}",
        base_dir
    );

    let entries = match fs::read_dir(&base_dir) {
        Ok(e) => e,
        Err(e) => {
            return Err(format!("Failed to read WebView directory: {}", e));
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();

        // Skip if not a directory
        if !path.is_dir() {
            continue;
        }

        // Get directory name
        let dir_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        // Skip if not a process directory
        let pid = match extract_pid_from_dir_name(&dir_name) {
            Some(pid) => pid,
            None => {
                // Skip non-process directories (like "EBWebView")
                continue;
            }
        };

        // Skip current process
        if pid == current_pid {
            debug!("[cleanup] Skipping current process directory: {}", dir_name);
            continue;
        }

        // Check if process is still alive
        if is_process_alive(pid) {
            debug!("[cleanup] Process {} is still alive, skipping", pid);
            continue;
        }

        // Process is dead, remove the directory
        info!(
            "[cleanup] Removing stale directory for dead process {}: {:?}",
            pid, path
        );

        match fs::remove_dir_all(&path) {
            Ok(()) => {
                cleaned += 1;
                info!("[cleanup] Successfully removed: {:?}", path);
            }
            Err(e) => {
                // Log but don't fail - some files might be locked
                let err_msg = format!("Failed to remove {:?}: {}", path, e);
                warn!("[cleanup] {}", err_msg);
                errors.push(err_msg);
            }
        }
    }

    if cleaned > 0 {
        info!("[cleanup] Cleaned up {} stale WebView directories", cleaned);
    }

    if !errors.is_empty() {
        warn!(
            "[cleanup] {} directories could not be removed (may be locked)",
            errors.len()
        );
    }

    Ok(cleaned)
}

/// Clean up the current process's WebView user data directory
///
/// Call this when the application is shutting down normally.
/// This helps prevent directory accumulation over time.
///
/// Note: This may fail if WebView is still running - that's okay,
/// the directory will be cleaned up on next startup.
pub fn cleanup_current_process_dir() -> Result<(), String> {
    let base_dir = match get_webview_base_dir() {
        Some(dir) => dir,
        None => return Ok(()),
    };

    let current_pid = std::process::id();
    let process_dir = base_dir.join(format!("process_{}", current_pid));

    if !process_dir.exists() {
        debug!(
            "[cleanup] Current process dir does not exist: {:?}",
            process_dir
        );
        return Ok(());
    }

    info!(
        "[cleanup] Cleaning up current process directory: {:?}",
        process_dir
    );

    // Note: This may fail if WebView is still running
    // That's okay - we'll clean it up on next startup
    match fs::remove_dir_all(&process_dir) {
        Ok(()) => {
            info!("[cleanup] Successfully removed current process directory");
            Ok(())
        }
        Err(e) => {
            // Don't fail - just log the warning
            warn!(
                "[cleanup] Could not remove current process dir (may still be in use): {}",
                e
            );
            Ok(())
        }
    }
}

/// Get the process-specific WebView data directory path
///
/// Returns the path that should be used for the current process's
/// WebView user data directory.
pub fn get_process_data_dir() -> Option<PathBuf> {
    let base_dir = get_webview_base_dir()?;
    let pid = std::process::id();
    Some(base_dir.join(format!("process_{}", pid)))
}

/// Cleanup statistics
#[derive(Debug, Clone, Default)]
pub struct CleanupStats {
    /// Total number of process directories found
    pub total_dirs: usize,
    /// Number of directories with alive processes
    pub alive_dirs: usize,
    /// Number of directories with dead processes (stale)
    pub stale_dirs: usize,
    /// Total size of stale directories in bytes
    pub stale_size_bytes: u64,
}

/// Get statistics about WebView directories without performing cleanup
pub fn get_cleanup_stats() -> CleanupStats {
    let mut stats = CleanupStats::default();

    let base_dir = match get_webview_base_dir() {
        Some(dir) => dir,
        None => return stats,
    };

    if !base_dir.exists() {
        return stats;
    }

    let current_pid = std::process::id();

    let entries = match fs::read_dir(&base_dir) {
        Ok(e) => e,
        Err(_) => return stats,
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let dir_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        let pid = match extract_pid_from_dir_name(&dir_name) {
            Some(pid) => pid,
            None => continue,
        };

        stats.total_dirs += 1;

        if pid == current_pid || is_process_alive(pid) {
            stats.alive_dirs += 1;
        } else {
            stats.stale_dirs += 1;
            // Calculate size of stale directory
            if let Ok(size) = calculate_dir_size(&path) {
                stats.stale_size_bytes += size;
            }
        }
    }

    stats
}

/// Calculate the total size of a directory recursively
fn calculate_dir_size(path: &PathBuf) -> std::io::Result<u64> {
    let mut size = 0;

    if path.is_file() {
        return Ok(fs::metadata(path)?.len());
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            size += fs::metadata(&path)?.len();
        } else if path.is_dir() {
            size += calculate_dir_size(&path)?;
        }
    }

    Ok(size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_pid_from_dir_name() {
        assert_eq!(extract_pid_from_dir_name("process_12345"), Some(12345));
        assert_eq!(extract_pid_from_dir_name("process_1"), Some(1));
        assert_eq!(extract_pid_from_dir_name("process_0"), Some(0));
        assert_eq!(extract_pid_from_dir_name("process_"), None);
        assert_eq!(extract_pid_from_dir_name("process_abc"), None);
        assert_eq!(extract_pid_from_dir_name("EBWebView"), None);
        assert_eq!(extract_pid_from_dir_name(""), None);
    }

    #[test]
    fn test_get_webview_base_dir() {
        // Should return Some on supported platforms
        #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
        {
            let dir = get_webview_base_dir();
            assert!(dir.is_some());
            let dir = dir.unwrap();
            assert!(
                dir.to_string_lossy().contains("AuroraView")
                    || dir.to_string_lossy().contains("auroraview")
            );
        }
    }

    #[test]
    fn test_get_process_data_dir() {
        #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
        {
            let dir = get_process_data_dir();
            assert!(dir.is_some());
            let dir = dir.unwrap();
            let pid = std::process::id();
            assert!(dir.to_string_lossy().contains(&format!("process_{}", pid)));
        }
    }

    #[test]
    fn test_get_cleanup_stats() {
        // This test just ensures the function doesn't panic
        let stats = get_cleanup_stats();
        // Stats should have valid values (may be zero if no directories exist)
        assert!(stats.alive_dirs <= stats.total_dirs);
        assert!(stats.stale_dirs <= stats.total_dirs);
    }
}
