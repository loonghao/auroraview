//! Parent Process Monitor
//!
//! This module monitors the parent process and automatically exits
//! the Sidecar when the parent process dies.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Default interval for checking parent process (in milliseconds).
const DEFAULT_CHECK_INTERVAL_MS: u64 = 1000;

/// Parent process monitor.
///
/// Periodically checks if the parent process is still alive.
/// When the parent dies, signals the Sidecar to exit.
pub struct ParentMonitor {
    parent_pid: u32,
    running: Arc<AtomicBool>,
    check_interval: Duration,
}

impl ParentMonitor {
    /// Create a new parent monitor.
    pub fn new(parent_pid: u32) -> Self {
        Self {
            parent_pid,
            running: Arc::new(AtomicBool::new(false)),
            check_interval: Duration::from_millis(DEFAULT_CHECK_INTERVAL_MS),
        }
    }

    /// Set the check interval.
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.check_interval = interval;
        self
    }

    /// Start monitoring in a background thread.
    ///
    /// Returns a handle that can be used to check if parent is alive
    /// or to stop monitoring.
    pub fn start(self) -> ParentMonitorHandle {
        let running = self.running.clone();
        running.store(true, Ordering::SeqCst);

        let parent_alive = Arc::new(AtomicBool::new(true));
        let parent_alive_clone = parent_alive.clone();
        let parent_pid = self.parent_pid;
        let check_interval = self.check_interval;

        let handle = thread::spawn(move || {
            tracing::info!(
                "[ParentMonitor] Started monitoring parent PID: {}",
                parent_pid
            );

            while running.load(Ordering::SeqCst) {
                if !is_process_alive(parent_pid) {
                    tracing::warn!(
                        "[ParentMonitor] Parent process {} is no longer alive",
                        parent_pid
                    );
                    parent_alive_clone.store(false, Ordering::SeqCst);
                    break;
                }

                thread::sleep(check_interval);
            }

            tracing::info!("[ParentMonitor] Stopped");
        });

        ParentMonitorHandle {
            running: self.running,
            parent_alive,
            _handle: handle,
        }
    }
}

/// Handle for the parent monitor.
pub struct ParentMonitorHandle {
    running: Arc<AtomicBool>,
    parent_alive: Arc<AtomicBool>,
    _handle: thread::JoinHandle<()>,
}

impl ParentMonitorHandle {
    /// Check if parent process is still alive.
    pub fn is_parent_alive(&self) -> bool {
        self.parent_alive.load(Ordering::SeqCst)
    }

    /// Stop monitoring.
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

/// Check if a process with the given PID is alive.
///
/// On Windows, we need to check both:
/// 1. Whether we can open the process handle
/// 2. Whether the process has actually exited (using GetExitCodeProcess)
#[cfg(windows)]
fn is_process_alive(pid: u32) -> bool {
    use windows::Win32::Foundation::{CloseHandle, STILL_ACTIVE};
    use windows::Win32::System::Threading::{
        GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    };

    unsafe {
        let handle = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(h) => h,
            Err(_) => {
                // Cannot open process - it doesn't exist
                return false;
            }
        };

        // Check if process has exited
        let mut exit_code: u32 = 0;
        let result = GetExitCodeProcess(handle, &mut exit_code);
        let _ = CloseHandle(handle);

        match result {
            Ok(_) => {
                // STILL_ACTIVE (259) means process is running
                // Any other value means it has exited
                exit_code == STILL_ACTIVE.0 as u32
            }
            Err(_) => {
                // If we can't get exit code, assume process is dead
                false
            }
        }
    }
}

#[cfg(unix)]
fn is_process_alive(pid: u32) -> bool {
    use nix::sys::signal::{kill, Signal};
    use nix::unistd::Pid;

    // Sending signal 0 checks if process exists without affecting it
    match kill(Pid::from_raw(pid as i32), None) {
        Ok(_) => true,
        Err(nix::errno::Errno::ESRCH) => false, // No such process
        Err(_) => true, // Other errors mean process exists but we can't signal it
    }
}

#[cfg(not(any(windows, unix)))]
fn is_process_alive(_pid: u32) -> bool {
    // Fallback: always assume alive
    tracing::warn!("[ParentMonitor] Platform not supported, assuming parent alive");
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_process_is_alive() {
        let pid = std::process::id();
        assert!(is_process_alive(pid));
    }

    #[test]
    fn test_invalid_pid_not_alive() {
        // Use a very large PID that's unlikely to exist
        let pid = u32::MAX - 1;
        assert!(!is_process_alive(pid));
    }
}
