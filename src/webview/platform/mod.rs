//! Platform-specific window management
//!
//! This module provides platform-specific implementations for window
//! lifecycle management and event handling.

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

use crate::webview::lifecycle::LifecycleManager;
use std::sync::Arc;

/// Platform-specific window manager trait
#[allow(dead_code)]
pub trait PlatformWindowManager: Send + Sync {
    /// Process platform-specific events (non-blocking)
    /// Returns true if window should close
    fn process_events(&self) -> bool;

    /// Setup platform-specific close handlers
    fn setup_close_handlers(&self, lifecycle: Arc<LifecycleManager>);

    /// Cleanup platform-specific resources
    fn cleanup(&self);

    /// Check if window is still valid
    fn is_window_valid(&self) -> bool;
}

/// Create platform-specific window manager
#[cfg(target_os = "windows")]
#[allow(dead_code)]
pub fn create_platform_manager(hwnd: u64) -> Box<dyn PlatformWindowManager> {
    Box::new(windows::WindowsWindowManager::new(hwnd))
}

#[cfg(target_os = "macos")]
#[allow(dead_code)]
pub fn create_platform_manager(ns_window: *mut std::ffi::c_void) -> Box<dyn PlatformWindowManager> {
    Box::new(macos::MacOSWindowManager::new(ns_window))
}

#[cfg(target_os = "linux")]
#[allow(dead_code)]
pub fn create_platform_manager(x11_window: u64) -> Box<dyn PlatformWindowManager> {
    Box::new(linux::LinuxWindowManager::new(x11_window))
}
