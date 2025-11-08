//! Windows-specific window management

use super::PlatformWindowManager;
use crate::webview::lifecycle::{CloseReason, LifecycleManager};
use parking_lot::Mutex;
use scopeguard::defer;
use std::ffi::c_void;
use std::sync::Arc;
use tracing::{debug, info, trace, warn};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, IsWindow, PeekMessageW, TranslateMessage, HTCLOSE, MSG, PM_REMOVE, SC_CLOSE,
    WM_CLOSE, WM_DESTROY, WM_NCLBUTTONDOWN, WM_NCLBUTTONUP, WM_QUIT, WM_SYSCOMMAND,
};

/// Windows-specific window manager
///
/// Note: We store HWND as u64 instead of HWND directly to ensure Send + Sync
pub struct WindowsWindowManager {
    hwnd_value: u64,
    lifecycle: Arc<Mutex<Option<Arc<LifecycleManager>>>>,
}

impl WindowsWindowManager {
    /// Create a new Windows window manager
    pub fn new(hwnd_value: u64) -> Self {
        info!(
            "[WindowsWindowManager] Created for HWND: 0x{:X}",
            hwnd_value
        );

        Self {
            hwnd_value,
            lifecycle: Arc::new(Mutex::new(None)),
        }
    }

    /// Get HWND from stored value
    fn hwnd(&self) -> HWND {
        HWND(self.hwnd_value as *mut c_void)
    }

    /// Process Windows messages with improved close detection
    fn process_windows_messages(&self) -> bool {
        defer! {
            trace!("[WindowsWindowManager] Message processing completed");
        }

        let hwnd = self.hwnd();

        unsafe {
            let mut msg = MSG::default();
            let mut should_close = false;
            let mut message_count = 0;

            // Process all pending messages (non-blocking)
            while PeekMessageW(&mut msg, hwnd, 0, 0, PM_REMOVE).as_bool() {
                message_count += 1;

                // Log important messages
                if message_count <= 10
                    || msg.message == WM_CLOSE
                    || msg.message == WM_DESTROY
                    || msg.message == WM_QUIT
                {
                    debug!(
                        "[WindowsWindowManager] Message #{}: 0x{:04X}",
                        message_count, msg.message
                    );
                }

                // Detect close intent from various sources
                if self.is_close_message(&msg) {
                    info!(
                        "[WindowsWindowManager] Close message detected: 0x{:04X}",
                        msg.message
                    );

                    let hwnd = self.hwnd();

                    // Notify lifecycle manager FIRST (before destruction)
                    if let Some(lifecycle) = self.lifecycle.lock().as_ref() {
                        let reason = self.determine_close_reason(&msg);
                        let _ = lifecycle.request_close(reason);
                    }

                    should_close = true;

                    // FIX: Actually destroy the window for WM_CLOSE and WM_SYSCOMMAND/SC_CLOSE
                    // This is required because we're handling the message ourselves
                    // (DefWindowProc won't be called if we mark it as handled)
                    match msg.message {
                        WM_CLOSE => {
                            use windows::Win32::UI::WindowsAndMessaging::DestroyWindow;
                            if DestroyWindow(hwnd).is_ok() {
                                info!("[WindowsWindowManager] ✅ Window destroyed successfully (WM_CLOSE)");
                            } else {
                                warn!(
                                    "[WindowsWindowManager] ⚠️ DestroyWindow failed for WM_CLOSE"
                                );
                            }
                        }
                        WM_SYSCOMMAND if ((msg.wParam.0 & 0xFFF0) as u32) == SC_CLOSE => {
                            use windows::Win32::UI::WindowsAndMessaging::DestroyWindow;
                            if DestroyWindow(hwnd).is_ok() {
                                info!("[WindowsWindowManager] ✅ Window destroyed successfully (SC_CLOSE)");
                            } else {
                                warn!(
                                    "[WindowsWindowManager] ⚠️ DestroyWindow failed for SC_CLOSE"
                                );
                            }
                        }
                        _ => {
                            // For WM_DESTROY, WM_QUIT, WM_NCLBUTTONUP/DOWN, just dispatch
                            // These don't require explicit DestroyWindow call
                            let _ = TranslateMessage(&msg);
                            DispatchMessageW(&msg);
                        }
                    }

                    continue;
                }

                // Dispatch other messages normally
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            if message_count > 0 {
                trace!(
                    "[WindowsWindowManager] Processed {} messages",
                    message_count
                );
            }

            should_close
        }
    }

    /// Check if a message indicates window close
    fn is_close_message(&self, msg: &MSG) -> bool {
        msg.message == WM_CLOSE
            || msg.message == WM_DESTROY
            || msg.message == WM_QUIT
            || (msg.message == WM_SYSCOMMAND && ((msg.wParam.0 & 0xFFF0) as u32) == SC_CLOSE)
            || (msg.message == WM_NCLBUTTONUP && msg.wParam.0 as u32 == HTCLOSE)
            || (msg.message == WM_NCLBUTTONDOWN && msg.wParam.0 as u32 == HTCLOSE)
    }

    /// Determine the reason for close based on the message
    fn determine_close_reason(&self, msg: &MSG) -> CloseReason {
        match msg.message {
            WM_CLOSE => CloseReason::UserRequest,
            WM_SYSCOMMAND if ((msg.wParam.0 & 0xFFF0) as u32) == SC_CLOSE => {
                CloseReason::UserRequest
            }
            WM_DESTROY => CloseReason::ParentClosed,
            WM_QUIT => CloseReason::SystemShutdown,
            _ => CloseReason::UserRequest,
        }
    }
}

impl PlatformWindowManager for WindowsWindowManager {
    fn process_events(&self) -> bool {
        // First check lifecycle state
        if let Some(lifecycle) = self.lifecycle.lock().as_ref() {
            if let Some(reason) = lifecycle.check_close_requested() {
                info!(
                    "[WindowsWindowManager] Close already requested: {:?}",
                    reason
                );
                return true;
            }
        }

        // Then process Windows messages
        self.process_windows_messages()
    }

    fn setup_close_handlers(&self, lifecycle: Arc<LifecycleManager>) {
        *self.lifecycle.lock() = Some(lifecycle.clone());

        info!("[WindowsWindowManager] Close handlers configured");
    }

    fn cleanup(&self) {
        defer! {
            info!("[WindowsWindowManager] Cleanup completed");
        }

        info!(
            "[WindowsWindowManager] Starting cleanup for HWND: 0x{:X}",
            self.hwnd_value
        );

        // Clear lifecycle reference
        *self.lifecycle.lock() = None;
    }

    fn is_window_valid(&self) -> bool {
        let hwnd = self.hwnd();
        let is_valid = unsafe { IsWindow(hwnd).as_bool() };

        if !is_valid {
            warn!(
                "[WindowsWindowManager] Window is no longer valid: 0x{:X}",
                self.hwnd_value
            );
        }

        is_valid
    }
}

// Safety: HWND values (as u64) can be safely sent between threads
unsafe impl Send for WindowsWindowManager {}
unsafe impl Sync for WindowsWindowManager {}
