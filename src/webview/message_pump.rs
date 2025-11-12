//! Windows message pump for embedded mode
//!
//! This module provides a way to process Windows messages without running
//! a full event loop. This is necessary for embedded mode where the host
//! application (Maya, Houdini, etc.) already has its own event loop.

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HWND;
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE,
};

/// Process all pending Windows messages for a specific window (non-blocking)
///
/// This function processes all pending messages in the message queue
/// without blocking. It should be called periodically (e.g., from a timer)
/// to keep the window responsive.
///
/// Returns true if a close message was received, false otherwise.
#[cfg(target_os = "windows")]
pub fn process_messages_for_hwnd(hwnd_value: u64) -> bool {
    use std::ffi::c_void;
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        DestroyWindow, PostMessageW, HTCLOSE, SC_CLOSE, WM_CLOSE, WM_DESTROY, WM_NCLBUTTONDOWN,
        WM_NCLBUTTONUP, WM_QUIT, WM_SYSCOMMAND,
    };

    unsafe {
        let hwnd = HWND(hwnd_value as *mut c_void);
        let mut msg = MSG::default();
        let mut should_close = false;
        let mut message_count = 0;

        tracing::trace!(
            "[process_messages_for_hwnd] START - hwnd=0x{:X}",
            hwnd_value
        );
        tracing::trace!("[process_messages_for_hwnd] HWND pointer: {:?}", hwnd);

        // Process all pending messages for this specific window (non-blocking)
        while PeekMessageW(&mut msg, hwnd, 0, 0, PM_REMOVE).as_bool() {
            message_count += 1;

            // Log close/destroy messages as INFO, others as DEBUG
            if msg.message == WM_CLOSE || msg.message == WM_DESTROY || msg.message == WM_QUIT {
                tracing::info!(
                    "[CRITICAL] [process_messages_for_hwnd] Message #{}: 0x{:04X} (HWND: {:?})",
                    message_count,
                    msg.message,
                    msg.hwnd
                );
            } else if message_count <= 10 {
                tracing::debug!(
                    "[OK] [process_messages_for_hwnd] Message #{}: 0x{:04X} (HWND: {:?})",
                    message_count,
                    msg.message,
                    msg.hwnd
                );
            }

            // Check for window close messages
            if (msg.message == WM_SYSCOMMAND && ((msg.wParam.0 & 0xFFF0) as u32) == SC_CLOSE)
                || (msg.message == WM_NCLBUTTONUP && msg.wParam.0 as u32 == HTCLOSE)
                || (msg.message == WM_NCLBUTTONDOWN && msg.wParam.0 as u32 == HTCLOSE)
            {
                tracing::info!("[process_messages_for_hwnd] Close intent detected (SC_CLOSE/HTCLOSE) -> DestroyWindow + post WM_CLOSE");
                // Aggressive path: destroy window immediately to avoid framework swallowing close
                let _ = DestroyWindow(hwnd);
                // Also post WM_CLOSE as a signal for any listeners
                let _ = PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
                should_close = true;
                continue;
            } else if msg.message == WM_CLOSE {
                tracing::info!("{}", "=".repeat(80));
                tracing::info!(
                    "[OK] [process_messages_for_hwnd] WM_CLOSE received (X button clicked), hwnd={:?}",
                    msg.hwnd
                );

                // Set the close flag to notify Python
                should_close = true;
                tracing::info!("[OK] [process_messages_for_hwnd] should_close set to true");

                // FIX: Actually destroy the window
                // We must call DestroyWindow ourselves because we're handling WM_CLOSE
                // (DefWindowProc won't be called if we've already processed the message)
                let destroy_result = DestroyWindow(hwnd);
                if destroy_result.is_ok() {
                    tracing::info!(
                        "[OK] [process_messages_for_hwnd] ✅ Window destroyed successfully"
                    );
                } else {
                    tracing::warn!("[process_messages_for_hwnd] ⚠️ DestroyWindow failed");
                }

                tracing::info!(
                    "[OK] [process_messages_for_hwnd] Will return to Python for cleanup"
                );
                tracing::info!("{}", "=".repeat(80));
                continue;
            } else if msg.message == WM_DESTROY {
                tracing::info!(
                    "[CRITICAL] [process_messages_for_hwnd] WM_DESTROY received, hwnd={:?}",
                    msg.hwnd
                );
                should_close = true;
            } else if msg.message == WM_QUIT {
                tracing::info!("[CRITICAL] [process_messages_for_hwnd] WM_QUIT received");
                should_close = true;
            }

            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        if message_count > 0 {
            tracing::debug!(
                "[process_messages_for_hwnd] processed {} messages",
                message_count
            );
        } else {
            tracing::trace!("[process_messages_for_hwnd] no messages for this HWND");
        }

        should_close
    }
}

/// Process all pending Windows messages for all windows (non-blocking)
///
/// This is useful when you don't have a specific HWND to target.
#[cfg(target_os = "windows")]
pub fn process_all_messages() -> bool {
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        DestroyWindow, PostMessageW, HTCLOSE, SC_CLOSE, WM_CLOSE, WM_DESTROY, WM_NCLBUTTONDOWN,
        WM_NCLBUTTONUP, WM_QUIT, WM_SYSCOMMAND,
    };

    unsafe {
        let mut msg = MSG::default();
        let mut should_close = false;
        let mut message_count = 0;

        // Process all pending messages for all windows (non-blocking)
        // Pass HWND(null) to process messages for all windows in the current thread
        while PeekMessageW(&mut msg, HWND(std::ptr::null_mut()), 0, 0, PM_REMOVE).as_bool() {
            message_count += 1;

            // Log all messages for debugging
            if message_count <= 10
                || msg.message == WM_CLOSE
                || msg.message == WM_DESTROY
                || msg.message == WM_QUIT
            {
                tracing::debug!(
                    "[OK] [message_pump] Message #{}: 0x{:04X} (HWND: {:?})",
                    message_count,
                    msg.message,
                    msg.hwnd
                );
            }

            // Check for window close messages
            if (msg.message == WM_SYSCOMMAND && ((msg.wParam.0 & 0xFFF0) as u32) == SC_CLOSE)
                || (msg.message == WM_NCLBUTTONUP && msg.wParam.0 as u32 == HTCLOSE)
                || (msg.message == WM_NCLBUTTONDOWN && msg.wParam.0 as u32 == HTCLOSE)
            {
                tracing::info!("[message_pump] Close intent detected (SC_CLOSE/HTCLOSE) -> DestroyWindow + post WM_CLOSE");
                let _ = DestroyWindow(msg.hwnd);
                let _ = PostMessageW(msg.hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
                should_close = true;
                continue;
            } else if msg.message == WM_CLOSE {
                tracing::info!("{}", "=".repeat(80));
                tracing::info!("[OK] [message_pump] WM_CLOSE received (X button clicked)");
                tracing::info!("[OK] [message_pump] Message HWND: {:?}", msg.hwnd);
                tracing::info!("[OK] [message_pump] Setting should_close flag...");

                // Set the close flag
                should_close = true;
                tracing::info!("[OK] [message_pump] should_close set to true");

                // FIX: Actually destroy the window
                let destroy_result = DestroyWindow(msg.hwnd);
                if destroy_result.is_ok() {
                    tracing::info!("[OK] [message_pump] ✅ Window destroyed successfully");
                } else {
                    tracing::warn!("[message_pump] ⚠️ DestroyWindow failed");
                }

                tracing::info!("[OK] [message_pump] Will return to Python for cleanup");
                tracing::info!("{}", "=".repeat(80));
                continue;
            } else if msg.message == WM_DESTROY {
                tracing::info!("{}", "=".repeat(80));
                tracing::info!("[OK] [message_pump] WM_DESTROY received");
                tracing::info!("[OK] [message_pump] Message HWND: {:?}", msg.hwnd);
                should_close = true;
                tracing::info!("[OK] [message_pump] should_close set to true");
                tracing::info!("{}", "=".repeat(80));
            } else if msg.message == WM_QUIT {
                tracing::info!("{}", "=".repeat(80));
                tracing::info!("[OK] [message_pump] WM_QUIT received");
                should_close = true;
                tracing::info!("[OK] [message_pump] should_close set to true");
                tracing::info!("{}", "=".repeat(80));
            }

            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        if message_count > 0 {
            tracing::debug!(
                "[OK] [message_pump] Processed {} messages total",
                message_count
            );
        }

        if should_close {
            tracing::info!("[OK] [message_pump] Returning should_close = true");
        }

        should_close
    }
}

/// Process a limited number of messages for all windows (non-blocking)
/// Useful to ensure child/IPC windows (e.g., WebView2) get serviced without
/// starving the host application's own loop.
#[cfg(target_os = "windows")]
#[allow(dead_code)]
pub fn process_all_messages_limited(max_messages: usize) -> bool {
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        DestroyWindow, PostMessageW, HTCLOSE, SC_CLOSE, WM_CLOSE, WM_DESTROY, WM_NCLBUTTONDOWN,
        WM_NCLBUTTONUP, WM_QUIT, WM_SYSCOMMAND,
    };

    unsafe {
        let mut msg = MSG::default();
        let mut should_close = false;
        let mut message_count = 0usize;

        while message_count < max_messages
            && PeekMessageW(&mut msg, HWND(std::ptr::null_mut()), 0, 0, PM_REMOVE).as_bool()
        {
            message_count += 1;

            if (msg.message == WM_SYSCOMMAND && ((msg.wParam.0 & 0xFFF0) as u32) == SC_CLOSE)
                || (msg.message == WM_NCLBUTTONUP && msg.wParam.0 as u32 == HTCLOSE)
                || (msg.message == WM_NCLBUTTONDOWN && msg.wParam.0 as u32 == HTCLOSE)
            {
                tracing::debug!(
                    "[message_pump] Close intent (limited) -> DestroyWindow + post WM_CLOSE, hwnd={:?}",
                    msg.hwnd
                );
                let _ = DestroyWindow(msg.hwnd);
                let _ = PostMessageW(msg.hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
                should_close = true;
                continue;
            } else if msg.message == WM_CLOSE {
                tracing::debug!(
                    "[message_pump] WM_CLOSE received (limited), hwnd={:?}",
                    msg.hwnd
                );
                should_close = true;

                // FIX: Actually destroy the window
                let destroy_result = DestroyWindow(msg.hwnd);
                if destroy_result.is_ok() {
                    tracing::debug!("[message_pump] ✅ Window destroyed (limited mode)");
                } else {
                    tracing::warn!("[message_pump] ⚠️ DestroyWindow failed (limited mode)");
                }
                continue;
            } else if msg.message == WM_DESTROY {
                tracing::debug!(
                    "[message_pump] WM_DESTROY received (limited), hwnd={:?}",
                    msg.hwnd
                );
                should_close = true;
            } else if msg.message == WM_QUIT {
                tracing::debug!("[message_pump] WM_QUIT received (limited)");
                should_close = true;
            }

            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        if message_count > 0 {
            tracing::trace!(
                "[message_pump] processed {} messages (limited)",
                message_count
            );
        }

        should_close
    }
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::*;

    #[test]
    fn test_process_all_messages_limited_zero() {
        // Should return false and not panic when no messages are processed
        let should_close = process_all_messages_limited(0);
        assert!(!should_close);
    }
}

#[test]
fn test_is_window_valid_zero_hwnd_is_false() {
    // HWND(0) is invalid
    assert!(!is_window_valid(0));
}

#[test]
fn test_process_all_messages_no_messages_returns_false() {
    // When no messages are pending, should return false
    let should_close = process_all_messages();
    assert!(!should_close);
}

#[cfg(not(target_os = "windows"))]
pub fn process_all_messages_limited(_max_messages: usize) -> bool {
    false
}

#[cfg(not(target_os = "windows"))]
pub fn process_messages_for_hwnd(_hwnd: u64) -> bool {
    // No-op on non-Windows platforms
    false
}

#[cfg(not(target_os = "windows"))]
pub fn process_all_messages() -> bool {
    // No-op on non-Windows platforms
    false
}

/// Check if a window handle is still valid (Windows only)
///
/// This function checks if the window still exists and is valid.
/// Useful for detecting when a window has been closed externally.
///
/// Returns true if the window is valid, false otherwise.
#[cfg(target_os = "windows")]
pub fn is_window_valid(hwnd_value: u64) -> bool {
    use std::ffi::c_void;
    use windows::Win32::UI::WindowsAndMessaging::IsWindow;

    unsafe {
        let hwnd = HWND(hwnd_value as *mut c_void);
        IsWindow(hwnd).as_bool()
    }
}

#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
pub fn is_window_valid(_hwnd: u64) -> bool {
    // Always return true on non-Windows platforms
    true
}

/// Process messages for a specific window with enhanced detection (Windows only)
///
/// This function uses a multi-strategy approach to detect window close events:
/// 1. Process messages for the specific HWND
/// 2. Check window validity using IsWindow
/// 3. Process thread messages (for messages not tied to a specific window)
///
/// This is more robust than just using PeekMessageW with a specific HWND,
/// especially in embedded scenarios where close messages might be sent differently.
///
/// Returns true if the window should close, false otherwise.
#[cfg(target_os = "windows")]
#[allow(dead_code)]
pub fn process_messages_enhanced(hwnd_value: u64) -> bool {
    use std::ffi::c_void;
    use windows::Win32::UI::WindowsAndMessaging::{
        IsWindow, SC_CLOSE, WM_CLOSE, WM_DESTROY, WM_QUIT, WM_SYSCOMMAND,
    };

    unsafe {
        let hwnd = HWND(hwnd_value as *mut c_void);
        let mut should_close = false;

        // Strategy 1: Check window validity first
        if !IsWindow(hwnd).as_bool() {
            tracing::info!("[process_messages_enhanced] Window is no longer valid");
            return true;
        }

        // Strategy 2: Process messages for this specific window
        let mut msg = MSG::default();
        let mut message_count = 0;

        while PeekMessageW(&mut msg, hwnd, 0, 0, PM_REMOVE).as_bool() {
            message_count += 1;

            if msg.message == WM_CLOSE || msg.message == WM_DESTROY || msg.message == WM_QUIT {
                tracing::info!(
                    "[process_messages_enhanced] Close message detected: 0x{:04X}",
                    msg.message
                );
                should_close = true;
            } else if msg.message == WM_SYSCOMMAND {
                let wparam = msg.wParam.0 as u32;
                if wparam == SC_CLOSE {
                    tracing::info!("[process_messages_enhanced] SC_CLOSE detected");
                    should_close = true;
                }
            }

            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Strategy 3: Process thread messages (messages not tied to a specific window)
        // This catches WM_QUIT and other thread-level messages
        while PeekMessageW(&mut msg, HWND(std::ptr::null_mut()), 0, 0, PM_REMOVE).as_bool() {
            message_count += 1;

            if msg.message == WM_QUIT {
                tracing::info!("[process_messages_enhanced] WM_QUIT from thread queue");
                should_close = true;
            }

            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        if message_count > 0 {
            tracing::trace!(
                "[process_messages_enhanced] Processed {} messages",
                message_count
            );
        }

        should_close
    }
}

#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
pub fn process_messages_enhanced(_hwnd: u64) -> bool {
    false
}
