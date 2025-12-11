//! Window style utilities for WebView embedding
//!
//! This module provides platform-specific window style manipulation
//! for embedding WebView as a child window.

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HWND;
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongW, SetParent, SetWindowLongW, SetWindowPos, GWL_EXSTYLE, GWL_STYLE,
    SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, WS_BORDER, WS_CAPTION,
    WS_CHILD, WS_DLGFRAME, WS_EX_CLIENTEDGE, WS_EX_DLGMODALFRAME, WS_EX_STATICEDGE,
    WS_EX_WINDOWEDGE, WS_POPUP, WS_THICKFRAME,
};

/// Options for applying child window style
#[derive(Debug, Clone, Copy, Default)]
pub struct ChildWindowStyleOptions {
    /// Whether to force window position to (0, 0) within parent
    /// Set to true for DCC/Qt embedding, false for standalone mode
    pub force_position: bool,
}

impl ChildWindowStyleOptions {
    /// Create options for DCC/Qt embedding (forces position to 0,0)
    pub fn for_dcc_embedding() -> Self {
        Self {
            force_position: true,
        }
    }

    /// Create options for standalone mode (preserves position)
    pub fn for_standalone() -> Self {
        Self {
            force_position: false,
        }
    }
}

/// Result of applying child window style
#[derive(Debug)]
pub struct ChildWindowStyleResult {
    /// Original window style
    pub old_style: i32,
    /// New window style
    pub new_style: i32,
    /// Original extended style
    pub old_ex_style: i32,
    /// New extended style
    pub new_ex_style: i32,
}

/// Apply WS_CHILD style to a window and set its parent
///
/// This function:
/// 1. Removes popup/caption/thickframe/border styles
/// 2. Adds WS_CHILD style
/// 3. Removes extended styles that cause white borders
/// 4. Sets the parent window
/// 5. Applies style changes
///
/// # Arguments
/// * `hwnd` - Handle to the window to modify
/// * `parent_hwnd` - Handle to the parent window
/// * `options` - Options for style application
///
/// # Returns
/// Result containing old and new styles, or error message
///
/// # Safety
/// This function uses unsafe Windows API calls.
#[cfg(target_os = "windows")]
pub fn apply_child_window_style(
    hwnd: isize,
    parent_hwnd: isize,
    options: ChildWindowStyleOptions,
) -> Result<ChildWindowStyleResult, String> {
    unsafe {
        let hwnd_win = HWND(hwnd as *mut _);
        let parent_hwnd_win = HWND(parent_hwnd as *mut _);

        // Get current window styles
        let style = GetWindowLongW(hwnd_win, GWL_STYLE);
        let ex_style = GetWindowLongW(hwnd_win, GWL_EXSTYLE);

        // Remove popup/caption/thickframe/border styles and add WS_CHILD
        // WS_CHILD windows cannot be moved independently of their parent
        let new_style = (style
            & !(WS_POPUP.0 as i32)
            & !(WS_CAPTION.0 as i32)
            & !(WS_THICKFRAME.0 as i32)
            & !(WS_BORDER.0 as i32)
            & !(WS_DLGFRAME.0 as i32))
            | (WS_CHILD.0 as i32);

        // Remove extended styles that can cause white borders
        // WS_EX_STATICEDGE, WS_EX_CLIENTEDGE, WS_EX_WINDOWEDGE are particularly problematic
        let new_ex_style = ex_style
            & !(WS_EX_STATICEDGE.0 as i32)
            & !(WS_EX_CLIENTEDGE.0 as i32)
            & !(WS_EX_WINDOWEDGE.0 as i32)
            & !(WS_EX_DLGMODALFRAME.0 as i32);

        SetWindowLongW(hwnd_win, GWL_STYLE, new_style);
        SetWindowLongW(hwnd_win, GWL_EXSTYLE, new_ex_style);

        // Ensure parent is set correctly (in case tao didn't do it)
        let _ = SetParent(hwnd_win, Some(parent_hwnd_win));

        // Apply style changes
        let flags = if options.force_position {
            // For DCC/Qt embedding: force position to (0, 0) within parent
            // CRITICAL: Remove SWP_NOMOVE to force position to (0, 0)
            // This prevents the WebView from being dragged/offset within the Qt container
            SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED
        } else {
            // For standalone: preserve current position
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED
        };

        let _ = SetWindowPos(hwnd_win, None, 0, 0, 0, 0, flags);

        tracing::info!(
            "Applied WS_CHILD style: HWND 0x{:X} -> Parent 0x{:X} (style 0x{:08X} -> 0x{:08X}, ex_style 0x{:08X} -> 0x{:08X})",
            hwnd,
            parent_hwnd,
            style,
            new_style,
            ex_style,
            new_ex_style
        );

        Ok(ChildWindowStyleResult {
            old_style: style,
            new_style,
            old_ex_style: ex_style,
            new_ex_style,
        })
    }
}

/// Stub for non-Windows platforms
#[cfg(not(target_os = "windows"))]
pub fn apply_child_window_style(
    _hwnd: isize,
    _parent_hwnd: isize,
    _options: ChildWindowStyleOptions,
) -> Result<ChildWindowStyleResult, String> {
    Err("apply_child_window_style is only supported on Windows".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_options_default() {
        let opts = ChildWindowStyleOptions::default();
        assert!(!opts.force_position);
    }

    #[test]
    fn test_options_for_dcc() {
        let opts = ChildWindowStyleOptions::for_dcc_embedding();
        assert!(opts.force_position);
    }

    #[test]
    fn test_options_for_standalone() {
        let opts = ChildWindowStyleOptions::for_standalone();
        assert!(!opts.force_position);
    }
}
