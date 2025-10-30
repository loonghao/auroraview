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
    use windows::Win32::UI::WindowsAndMessaging::{WM_CLOSE, WM_DESTROY, WM_QUIT};

    unsafe {
        let hwnd = HWND(hwnd_value as *mut c_void);
        let mut msg = MSG::default();
        let mut should_close = false;
        let mut message_count = 0;

        tracing::info!(
            "🟢 [process_messages_for_hwnd] START - Processing messages for HWND: 0x{:X}",
            hwnd_value
        );
        tracing::info!("🟢 [process_messages_for_hwnd] HWND pointer: {:?}", hwnd);

        // Process all pending messages for this specific window (non-blocking)
        while PeekMessageW(&mut msg, hwnd, 0, 0, PM_REMOVE).as_bool() {
            message_count += 1;

            // Log all messages for debugging
            if message_count <= 10
                || msg.message == WM_CLOSE
                || msg.message == WM_DESTROY
                || msg.message == WM_QUIT
            {
                tracing::debug!(
                    "🟢 [process_messages_for_hwnd] Message #{}: 0x{:04X} (HWND: {:?})",
                    message_count,
                    msg.message,
                    msg.hwnd
                );
            }

            // Check for window close messages
            if msg.message == WM_CLOSE {
                tracing::info!("{}", "=".repeat(80));
                tracing::info!(
                    "🟢 [process_messages_for_hwnd] WM_CLOSE received (X button clicked)"
                );
                tracing::info!(
                    "🟢 [process_messages_for_hwnd] Message HWND: {:?}",
                    msg.hwnd
                );
                tracing::info!("🟢 [process_messages_for_hwnd] Setting should_close flag...");

                // Set the close flag to notify Python
                should_close = true;
                tracing::info!("🟢 [process_messages_for_hwnd] should_close set to true");
                tracing::info!("🟢 [process_messages_for_hwnd] Will return to Python for cleanup");
                tracing::info!("{}", "=".repeat(80));

                // IMPORTANT: Still dispatch WM_CLOSE to allow default window processing
                // This ensures the window's DefWindowProc is called
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
                continue;
            } else if msg.message == WM_DESTROY {
                tracing::info!("{}", "=".repeat(80));
                tracing::info!("🟢 [process_messages_for_hwnd] WM_DESTROY received");
                tracing::info!(
                    "🟢 [process_messages_for_hwnd] Message HWND: {:?}",
                    msg.hwnd
                );
                should_close = true;
                tracing::info!("🟢 [process_messages_for_hwnd] should_close set to true");
                tracing::info!("{}", "=".repeat(80));
            } else if msg.message == WM_QUIT {
                tracing::info!("{}", "=".repeat(80));
                tracing::info!("🟢 [process_messages_for_hwnd] WM_QUIT received");
                should_close = true;
                tracing::info!("🟢 [process_messages_for_hwnd] should_close set to true");
                tracing::info!("{}", "=".repeat(80));
            }

            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        if message_count > 0 {
            tracing::info!(
                "🟢 [process_messages_for_hwnd] Processed {} messages total",
                message_count
            );
        } else {
            tracing::info!("🟢 [process_messages_for_hwnd] No messages found for this HWND");
        }

        if should_close {
            tracing::info!("🟢 [process_messages_for_hwnd] END - Returning should_close = true");
        } else {
            tracing::info!("🟢 [process_messages_for_hwnd] END - Returning should_close = false");
        }

        should_close
    }
}

/// Process all pending Windows messages for all windows (non-blocking)
///
/// This is useful when you don't have a specific HWND to target.
#[cfg(target_os = "windows")]
pub fn process_all_messages() -> bool {
    use windows::Win32::UI::WindowsAndMessaging::{WM_CLOSE, WM_DESTROY, WM_QUIT};

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
                    "🟢 [message_pump] Message #{}: 0x{:04X} (HWND: {:?})",
                    message_count,
                    msg.message,
                    msg.hwnd
                );
            }

            // Check for window close messages
            if msg.message == WM_CLOSE {
                tracing::info!("{}", "=".repeat(80));
                tracing::info!("🟢 [message_pump] WM_CLOSE received (X button clicked)");
                tracing::info!("🟢 [message_pump] Message HWND: {:?}", msg.hwnd);
                tracing::info!("🟢 [message_pump] Setting should_close flag...");

                // Set the close flag - let Python handle the actual window destruction
                // This ensures proper cleanup order and prevents race conditions
                should_close = true;
                tracing::info!("🟢 [message_pump] should_close set to true");
                tracing::info!("🟢 [message_pump] Will return to Python for cleanup");
                tracing::info!("{}", "=".repeat(80));

                // Don't dispatch WM_CLOSE, we've already handled it
                continue;
            } else if msg.message == WM_DESTROY {
                tracing::info!("{}", "=".repeat(80));
                tracing::info!("🟢 [message_pump] WM_DESTROY received");
                tracing::info!("🟢 [message_pump] Message HWND: {:?}", msg.hwnd);
                should_close = true;
                tracing::info!("🟢 [message_pump] should_close set to true");
                tracing::info!("{}", "=".repeat(80));
            } else if msg.message == WM_QUIT {
                tracing::info!("{}", "=".repeat(80));
                tracing::info!("🟢 [message_pump] WM_QUIT received");
                should_close = true;
                tracing::info!("🟢 [message_pump] should_close set to true");
                tracing::info!("{}", "=".repeat(80));
            }

            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        if message_count > 0 {
            tracing::debug!(
                "🟢 [message_pump] Processed {} messages total",
                message_count
            );
        }

        if should_close {
            tracing::info!("🟢 [message_pump] Returning should_close = true");
        }

        should_close
    }
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
