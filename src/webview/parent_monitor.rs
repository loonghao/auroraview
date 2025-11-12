//! Parent window lifecycle monitoring
//!
//! This module provides functionality to monitor the parent window's lifecycle
//! and automatically close the WebView when the parent window is destroyed.
//! This is essential for DCC integration (Maya, Houdini, etc.) to prevent
//! orphaned WebView windows when the DCC application closes.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
#[cfg(any(test, target_os = "windows"))]
use std::time::Duration;

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HWND;
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::IsWindow;

/// Parent window monitor
///
/// Monitors the parent window's lifecycle and notifies when it's destroyed.
/// Uses a background thread to periodically check if the parent window is still valid.
pub struct ParentWindowMonitor {
    /// Whether the monitor is running
    running: Arc<AtomicBool>,
    /// Monitor thread handle
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl ParentWindowMonitor {
    /// Create a new parent window monitor
    ///
    /// # Arguments
    /// * `parent_hwnd` - Parent window handle (HWND on Windows)
    /// * `on_parent_destroyed` - Callback to invoke when parent is destroyed
    /// * `check_interval_ms` - How often to check parent window (milliseconds)
    ///
    /// # Returns
    /// A new ParentWindowMonitor instance
    #[cfg(target_os = "windows")]
    #[allow(dead_code)]
    pub fn new<F>(parent_hwnd: u64, on_parent_destroyed: F, check_interval_ms: u64) -> Self
    where
        F: Fn() + Send + 'static,
    {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        tracing::info!(
            "[ParentMonitor] Starting monitor for parent HWND: 0x{:x}",
            parent_hwnd
        );

        let thread_handle = thread::spawn(move || {
            let hwnd = HWND(parent_hwnd as isize as *mut _);
            let check_interval = Duration::from_millis(check_interval_ms);

            tracing::info!(
                "[ParentMonitor] Monitor thread started, checking every {}ms",
                check_interval_ms
            );

            while running_clone.load(Ordering::Relaxed) {
                // Check if parent window is still valid
                let is_valid = unsafe { IsWindow(hwnd).as_bool() };

                if !is_valid {
                    tracing::warn!(
                        "[ParentMonitor] Parent window 0x{:x} is no longer valid!",
                        parent_hwnd
                    );
                    tracing::info!("[ParentMonitor] Invoking parent destroyed callback...");

                    // Invoke the callback
                    on_parent_destroyed();

                    tracing::info!("[ParentMonitor] Callback invoked, stopping monitor");
                    break;
                }

                // Sleep before next check
                thread::sleep(check_interval);
            }

            tracing::info!("[ParentMonitor] Monitor thread exiting");
        });

        Self {
            running,
            thread_handle: Some(thread_handle),
        }
    }

    /// Create a new parent window monitor (non-Windows platforms)
    #[cfg(not(target_os = "windows"))]
    #[allow(dead_code)]
    pub fn new<F>(_parent_hwnd: u64, _on_parent_destroyed: F, _check_interval_ms: u64) -> Self
    where
        F: Fn() + Send + 'static,
    {
        tracing::warn!("[ParentMonitor] Parent window monitoring not supported on this platform");
        Self {
            running: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
        }
    }

    /// Stop monitoring the parent window
    pub fn stop(&mut self) {
        tracing::info!("[ParentMonitor] Stopping monitor...");
        self.running.store(false, Ordering::Relaxed);

        if let Some(handle) = self.thread_handle.take() {
            tracing::info!("[ParentMonitor] Waiting for monitor thread to finish...");
            if let Err(e) = handle.join() {
                tracing::error!("[ParentMonitor] Error joining monitor thread: {:?}", e);
            } else {
                tracing::info!("[ParentMonitor] Monitor thread stopped successfully");
            }
        }
    }

    /// Check if the monitor is still running
    #[allow(dead_code)]
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}

impl Drop for ParentWindowMonitor {
    fn drop(&mut self) {
        tracing::info!("[ParentMonitor] Dropping monitor, ensuring cleanup...");
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(target_os = "windows")]
    use std::sync::atomic::AtomicUsize;

    #[test]
    #[cfg(target_os = "windows")]
    fn test_parent_monitor_invalid_hwnd() {
        // Use an invalid HWND (0)
        let callback_count = Arc::new(AtomicUsize::new(0));
        let callback_count_clone = callback_count.clone();

        let mut monitor = ParentWindowMonitor::new(
            0, // Invalid HWND
            move || {
                callback_count_clone.fetch_add(1, Ordering::Relaxed);
            },
            100, // Check every 100ms
        );

        // Wait for callback to be invoked
        thread::sleep(Duration::from_millis(500));

        // Callback should have been invoked
        assert!(callback_count.load(Ordering::Relaxed) > 0);

        monitor.stop();
    }

    #[test]
    fn test_parent_monitor_stop() {
        let mut monitor = ParentWindowMonitor::new(0, || {}, 100);

        assert!(monitor.is_running());

        monitor.stop();

        // Give it a moment to stop
        thread::sleep(Duration::from_millis(200));

        assert!(!monitor.is_running());
    }
}
