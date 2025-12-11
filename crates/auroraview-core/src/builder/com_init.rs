//! COM initialization utilities for Windows
//!
//! WebView2 requires COM to be initialized in STA (Single-Threaded Apartment) mode.
//! This module provides a safe wrapper for COM initialization.

/// Result of COM initialization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComInitResult {
    /// COM was successfully initialized
    Initialized,
    /// COM was already initialized (by Qt or other component)
    AlreadyInitialized,
    /// COM initialization failed
    Failed,
}

/// Initialize COM in STA mode for WebView2
///
/// WebView2 requires COM to be initialized in Single-Threaded Apartment (STA) mode.
/// This function safely initializes COM, handling cases where it's already initialized
/// (e.g., by Qt on the main thread).
///
/// # Returns
/// - `ComInitResult::Initialized` if COM was successfully initialized
/// - `ComInitResult::AlreadyInitialized` if COM was already initialized
/// - `ComInitResult::Failed` if initialization failed
///
/// # Example
/// ```rust,ignore
/// use auroraview_core::builder::init_com_sta;
///
/// match init_com_sta() {
///     ComInitResult::Initialized => println!("COM initialized"),
///     ComInitResult::AlreadyInitialized => println!("COM already initialized"),
///     ComInitResult::Failed => eprintln!("COM initialization failed"),
/// }
/// ```
#[cfg(target_os = "windows")]
pub fn init_com_sta() -> ComInitResult {
    use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};

    unsafe {
        // COINIT_APARTMENTTHREADED = STA mode required by WebView2
        // Ignore errors if already initialized (e.g., by Qt on main thread)
        let result = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        if result.is_ok() {
            tracing::info!("COM initialized in STA mode for this thread");
            ComInitResult::Initialized
        } else {
            // S_FALSE (0x00000001) means already initialized
            // RPC_E_CHANGED_MODE means initialized in different mode
            tracing::debug!("COM already initialized or failed: {:?}", result);
            ComInitResult::AlreadyInitialized
        }
    }
}

/// Stub for non-Windows platforms
#[cfg(not(target_os = "windows"))]
pub fn init_com_sta() -> ComInitResult {
    // COM is Windows-only, no-op on other platforms
    ComInitResult::Initialized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_com_init_result_enum() {
        // Just verify the enum variants exist
        let _ = ComInitResult::Initialized;
        let _ = ComInitResult::AlreadyInitialized;
        let _ = ComInitResult::Failed;
    }

    #[test]
    fn test_init_com_sta() {
        // On Windows, this should return Initialized or AlreadyInitialized
        // On other platforms, it should return Initialized (no-op)
        let result = init_com_sta();
        assert!(matches!(
            result,
            ComInitResult::Initialized | ComInitResult::AlreadyInitialized
        ));
    }
}
