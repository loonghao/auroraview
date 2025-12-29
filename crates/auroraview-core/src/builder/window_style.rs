//! Window style utilities for WebView embedding
//!
//! This module provides platform-specific window style manipulation
//! for embedding WebView as a child window or setting owner relationships.
//!
//! # Window Relationships on Windows
//!
//! ## Child Window (WS_CHILD)
//! - Window is contained within parent's client area
//! - Cannot be moved independently
//! - Coordinates relative to parent
//! - Use for: Embedding WebView in Qt widgets
//!
//! ## Owner Window (GWLP_HWNDPARENT)
//! - Window stays above owner in Z-order
//! - Hidden when owner is minimized
//! - Destroyed when owner is destroyed
//! - Can be positioned freely on screen
//! - Use for: Floating tool windows, dialogs
//!
//! # Official Documentation
//! - [Window Features](https://learn.microsoft.com/en-us/windows/win32/winmsg/window-features)
//! - [SetWindowLongPtrW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowlongptrw)

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HWND;
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Dwm::{
    DwmExtendFrameIntoClientArea, DwmSetWindowAttribute, DWMWA_NCRENDERING_POLICY,
};
#[cfg(target_os = "windows")]
use windows::Win32::UI::Controls::MARGINS;
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongW, SetParent, SetWindowLongPtrW, SetWindowLongW, SetWindowPos, GWLP_HWNDPARENT,
    GWL_EXSTYLE, GWL_STYLE, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
    WS_BORDER, WS_CAPTION, WS_CHILD, WS_CLIPCHILDREN, WS_DLGFRAME, WS_EX_CLIENTEDGE,
    WS_EX_DLGMODALFRAME, WS_EX_LAYERED, WS_EX_STATICEDGE, WS_EX_TOOLWINDOW, WS_EX_WINDOWEDGE,
    WS_POPUP, WS_THICKFRAME,
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

/// Result of applying owner window style
#[derive(Debug)]
pub struct OwnerWindowStyleResult {
    /// Original extended style
    pub old_ex_style: i32,
    /// New extended style
    pub new_ex_style: i32,
    /// Whether tool window style was applied
    pub tool_window: bool,
}

/// Apply owner relationship to a window.
///
/// This function sets up an owner-owned relationship between windows:
/// - The owned window stays above the owner in Z-order
/// - The owned window is hidden when the owner is minimized
/// - The owned window is destroyed when the owner is destroyed
/// - The owned window can be positioned freely on screen
///
/// # Arguments
/// * `hwnd` - Handle to the window to modify
/// * `owner_hwnd` - Handle to the owner window
/// * `tool_window` - If true, applies WS_EX_TOOLWINDOW style (hides from taskbar/Alt+Tab)
///
/// # Official Documentation
/// - [Owned Windows](https://learn.microsoft.com/en-us/windows/win32/winmsg/window-features#owned-windows)
/// - [SetWindowLongPtrW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowlongptrw)
/// - [WS_EX_TOOLWINDOW](https://learn.microsoft.com/en-us/windows/win32/winmsg/extended-window-styles)
///
/// # Safety
/// This function uses unsafe Windows API calls.
#[cfg(target_os = "windows")]
pub fn apply_owner_window_style(
    hwnd: isize,
    owner_hwnd: u64,
    tool_window: bool,
) -> OwnerWindowStyleResult {
    unsafe {
        let hwnd_win = HWND(hwnd as *mut _);

        // Get current extended style
        let ex_style = GetWindowLongW(hwnd_win, GWL_EXSTYLE);

        // Apply WS_EX_TOOLWINDOW if requested.
        // This hides the window from taskbar and Alt+Tab.
        // Also clear WS_EX_APPWINDOW to avoid forcing taskbar presence.
        const WS_EX_APPWINDOW_BITS: i32 = 0x00040000;
        let new_ex_style = if tool_window {
            (ex_style | (WS_EX_TOOLWINDOW.0 as i32)) & !WS_EX_APPWINDOW_BITS
        } else {
            ex_style
        };

        if new_ex_style != ex_style {
            SetWindowLongW(hwnd_win, GWL_EXSTYLE, new_ex_style);
        }

        // Set owner relationship using GWLP_HWNDPARENT
        // This is different from SetParent - it sets owner, not parent
        // For popup windows, this establishes owner relationship
        SetWindowLongPtrW(hwnd_win, GWLP_HWNDPARENT, owner_hwnd as isize);

        // Apply style changes
        let _ = SetWindowPos(
            hwnd_win,
            None,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        );

        tracing::info!(
            "Applied owner relationship: HWND 0x{:X} -> Owner 0x{:X} (tool_window: {}, ex_style 0x{:08X} -> 0x{:08X})",
            hwnd,
            owner_hwnd,
            tool_window,
            ex_style,
            new_ex_style
        );

        OwnerWindowStyleResult {
            old_ex_style: ex_style,
            new_ex_style,
            tool_window,
        }
    }
}

/// Stub for non-Windows platforms
#[cfg(not(target_os = "windows"))]
pub fn apply_owner_window_style(
    _hwnd: isize,
    _owner_hwnd: u64,
    _tool_window: bool,
) -> OwnerWindowStyleResult {
    OwnerWindowStyleResult {
        old_ex_style: 0,
        new_ex_style: 0,
        tool_window: false,
    }
}

/// Apply WS_EX_TOOLWINDOW style to a window.
///
/// This hides the window from the taskbar and Alt+Tab window switcher.
///
/// Note: WS_EX_APPWINDOW can force a top-level window to appear in the taskbar.
/// For floating tool windows we clear WS_EX_APPWINDOW to ensure it stays hidden.
///
/// # Arguments
/// * `hwnd` - Handle to the window to modify
///
/// # Official Documentation
/// - [WS_EX_TOOLWINDOW](https://learn.microsoft.com/en-us/windows/win32/winmsg/extended-window-styles)
#[cfg(target_os = "windows")]
pub fn apply_tool_window_style(hwnd: isize) {
    unsafe {
        let hwnd_win = HWND(hwnd as *mut _);

        // Get current extended style
        let ex_style = GetWindowLongW(hwnd_win, GWL_EXSTYLE);

        // WinUser.h constant (stable): WS_EX_APPWINDOW
        const WS_EX_APPWINDOW_BITS: i32 = 0x00040000;

        // Add WS_EX_TOOLWINDOW and clear WS_EX_APPWINDOW
        let new_ex_style = (ex_style | (WS_EX_TOOLWINDOW.0 as i32)) & !WS_EX_APPWINDOW_BITS;

        SetWindowLongW(hwnd_win, GWL_EXSTYLE, new_ex_style);

        // Apply style changes
        let _ = SetWindowPos(
            hwnd_win,
            None,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        );

        tracing::info!(
            "Applied tool window style: HWND 0x{:X} (ex_style 0x{:08X} -> 0x{:08X})",
            hwnd,
            ex_style,
            new_ex_style
        );
    }
}

/// Stub for non-Windows platforms
#[cfg(not(target_os = "windows"))]
pub fn apply_tool_window_style(_hwnd: isize) {
    // No-op on non-Windows platforms
}

/// Result of applying frameless (no-titlebar) window style.
#[derive(Debug)]
pub struct FramelessWindowStyleResult {
    /// Original window style
    pub old_style: i32,
    /// New window style
    pub new_style: i32,
    /// Original extended style
    pub old_ex_style: i32,
    /// New extended style
    pub new_ex_style: i32,
}

/// Compute the new style/ex_style values for a frameless window.
///
/// This is a pure helper that does not call Win32 APIs.
///
/// On Windows 11 with certain `tao`/`wry` combinations, `with_decorations(false)` may not
/// fully remove `WS_CAPTION`/`WS_THICKFRAME`. This helper defines the canonical bit-masks
/// we want to remove when making a window truly frameless.
pub fn compute_frameless_window_styles(style: i32, ex_style: i32) -> (i32, i32) {
    // WinUser.h constants (stable): keep them local to avoid OS-gated imports.
    const WS_CAPTION_BITS: i32 = 0x00C00000;
    const WS_THICKFRAME_BITS: i32 = 0x00040000;
    const WS_BORDER_BITS: i32 = 0x00800000;
    const WS_DLGFRAME_BITS: i32 = 0x00400000;

    // Also remove system menu / min-max boxes.
    // Keeping these bits on Windows 11 can result in a "ghost" caption area even when
    // WS_CAPTION is cleared (depending on DWM / window type).
    const WS_SYSMENU_BITS: i32 = 0x00080000;
    const WS_MINIMIZEBOX_BITS: i32 = 0x00020000;
    const WS_MAXIMIZEBOX_BITS: i32 = 0x00010000;

    const WS_EX_DLGMODALFRAME_BITS: i32 = 0x00000001;
    const WS_EX_WINDOWEDGE_BITS: i32 = 0x00000100;
    const WS_EX_CLIENTEDGE_BITS: i32 = 0x00000200;
    const WS_EX_STATICEDGE_BITS: i32 = 0x00020000;

    let new_style = style
        & !WS_CAPTION_BITS
        & !WS_THICKFRAME_BITS
        & !WS_BORDER_BITS
        & !WS_DLGFRAME_BITS
        & !WS_SYSMENU_BITS
        & !WS_MINIMIZEBOX_BITS
        & !WS_MAXIMIZEBOX_BITS;

    let new_ex_style = ex_style
        & !WS_EX_DLGMODALFRAME_BITS
        & !WS_EX_WINDOWEDGE_BITS
        & !WS_EX_CLIENTEDGE_BITS
        & !WS_EX_STATICEDGE_BITS;

    (new_style, new_ex_style)
}

/// Force-remove title bar and borders from an existing top-level window.
///
/// This is a Win32 fallback for cases where `tao::WindowBuilder::with_decorations(false)`
/// does not fully take effect on Windows 11.
///
/// Call this after the window is created (and preferably after WebView2 init if you are
/// also applying tool-window/owner styles that might affect WebView2 creation).
#[cfg(target_os = "windows")]
pub fn apply_frameless_window_style(hwnd: isize) -> Result<FramelessWindowStyleResult, String> {
    unsafe {
        let hwnd_win = HWND(hwnd as *mut _);

        let style = GetWindowLongW(hwnd_win, GWL_STYLE);
        let ex_style = GetWindowLongW(hwnd_win, GWL_EXSTYLE);

        let (new_style, new_ex_style) = compute_frameless_window_styles(style, ex_style);

        if new_style != style {
            SetWindowLongW(hwnd_win, GWL_STYLE, new_style);
        }
        if new_ex_style != ex_style {
            SetWindowLongW(hwnd_win, GWL_EXSTYLE, new_ex_style);
        }

        // Apply style changes
        let _ = SetWindowPos(
            hwnd_win,
            None,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        );

        tracing::info!(
            "Applied frameless window style: HWND 0x{:X} (style 0x{:08X} -> 0x{:08X}, ex_style 0x{:08X} -> 0x{:08X})",
            hwnd,
            style,
            new_style,
            ex_style,
            new_ex_style
        );

        Ok(FramelessWindowStyleResult {
            old_style: style,
            new_style,
            old_ex_style: ex_style,
            new_ex_style,
        })
    }
}

/// Stub for non-Windows platforms
#[cfg(not(target_os = "windows"))]
pub fn apply_frameless_window_style(_hwnd: isize) -> Result<FramelessWindowStyleResult, String> {
    Err("apply_frameless_window_style is only supported on Windows".to_string())
}

/// Compute the new style/ex_style values for a borderless popup window.
///
/// This builds on `compute_frameless_window_styles` and additionally forces `WS_POPUP`.
/// This is the most reliable way to get rid of the Win11 title bar / caption buttons.
pub fn compute_frameless_popup_window_styles(style: i32, ex_style: i32) -> (i32, i32) {
    // WinUser.h constants
    const WS_POPUP_BITS: i32 = 0x80000000u32 as i32;
    const WS_CHILD_BITS: i32 = 0x40000000;

    let (base_style, base_ex_style) = compute_frameless_window_styles(style, ex_style);

    // Ensure we are a top-level popup window (not a child window)
    let new_style = (base_style & !WS_CHILD_BITS) | WS_POPUP_BITS;

    (new_style, base_ex_style)
}

/// Force-remove title bar and borders from an existing top-level window by switching to `WS_POPUP`.
///
/// This is a stronger Win32 fallback than `apply_frameless_window_style` and is intended for
/// transparent/frameless tool windows on Windows 11 where DWM may still draw a caption area.
#[cfg(target_os = "windows")]
pub fn apply_frameless_popup_window_style(
    hwnd: isize,
) -> Result<FramelessWindowStyleResult, String> {
    unsafe {
        let hwnd_win = HWND(hwnd as *mut _);

        let style = GetWindowLongW(hwnd_win, GWL_STYLE);
        let ex_style = GetWindowLongW(hwnd_win, GWL_EXSTYLE);

        let (new_style, new_ex_style) = compute_frameless_popup_window_styles(style, ex_style);

        if new_style != style {
            SetWindowLongW(hwnd_win, GWL_STYLE, new_style);
        }
        if new_ex_style != ex_style {
            SetWindowLongW(hwnd_win, GWL_EXSTYLE, new_ex_style);
        }

        let _ = SetWindowPos(
            hwnd_win,
            None,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        );

        tracing::info!(
            "Applied frameless popup window style: HWND 0x{:X} (style 0x{:08X} -> 0x{:08X}, ex_style 0x{:08X} -> 0x{:08X})",
            hwnd,
            style,
            new_style,
            ex_style,
            new_ex_style
        );

        Ok(FramelessWindowStyleResult {
            old_style: style,
            new_style,
            old_ex_style: ex_style,
            new_ex_style,
        })
    }
}

/// Stub for non-Windows platforms
#[cfg(not(target_os = "windows"))]
pub fn apply_frameless_popup_window_style(
    _hwnd: isize,
) -> Result<FramelessWindowStyleResult, String> {
    Err("apply_frameless_popup_window_style is only supported on Windows".to_string())
}

/// Disable window shadow and Win11 frame effects for undecorated (frameless) windows.
///
/// This uses DWM (Desktop Window Manager) attributes to suppress non-client rendering.
/// For transparent frameless tool windows on Windows 11, it's common to also see a subtle
/// border/glow/corner rounding. We explicitly clear those where available.
///
/// # Arguments
/// * `hwnd` - Handle to the window to modify
///
/// # Official Documentation
/// - [DwmSetWindowAttribute](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/nf-dwmapi-dwmsetwindowattribute)
/// - [DWMWA_NCRENDERING_POLICY](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/ne-dwmapi-dwmwindowattribute)
#[cfg(target_os = "windows")]
pub fn disable_window_shadow(hwnd: isize) {
    unsafe {
        let hwnd_win = HWND(hwnd as *mut _);

        // DWMNCRP_DISABLED = 1 - Disable non-client area rendering (removes shadow)
        let policy: u32 = 1; // DWMNCRP_DISABLED

        let result = DwmSetWindowAttribute(
            hwnd_win,
            DWMWA_NCRENDERING_POLICY,
            &policy as *const _ as *const _,
            std::mem::size_of::<u32>() as u32,
        );

        if result.is_ok() {
            tracing::info!(
                "Disabled window shadow: HWND 0x{:X} (DWMWA_NCRENDERING_POLICY = DWMNCRP_DISABLED)",
                hwnd
            );
        } else {
            tracing::warn!(
                "Failed to disable window shadow: HWND 0x{:X}, HRESULT: {:?}",
                hwnd,
                result
            );
        }

        // Extra Win11 frame effects suppression.
        // We intentionally construct DWMWINDOWATTRIBUTE by numeric value to avoid SDK/feature gating.
        // Values (stable since Win11):
        // - 33: DWMWA_WINDOW_CORNER_PREFERENCE
        // - 34: DWMWA_BORDER_COLOR
        // - 35: DWMWA_CAPTION_COLOR
        // - 36: DWMWA_TEXT_COLOR
        // - 37: DWMWA_VISIBLE_FRAME_BORDER_THICKNESS
        use windows::Win32::Graphics::Dwm::DWMWINDOWATTRIBUTE;

        // DWMWCP_DONOTROUND = 1
        let corner_pref: u32 = 1;
        let _ = DwmSetWindowAttribute(
            hwnd_win,
            DWMWINDOWATTRIBUTE(33),
            &corner_pref as *const _ as *const _,
            std::mem::size_of::<u32>() as u32,
        );

        // DWMWA_COLOR_NONE = 0xFFFFFFFE
        let color_none: u32 = 0xFFFFFFFE;
        let _ = DwmSetWindowAttribute(
            hwnd_win,
            DWMWINDOWATTRIBUTE(34),
            &color_none as *const _ as *const _,
            std::mem::size_of::<u32>() as u32,
        );
        let _ = DwmSetWindowAttribute(
            hwnd_win,
            DWMWINDOWATTRIBUTE(35),
            &color_none as *const _ as *const _,
            std::mem::size_of::<u32>() as u32,
        );
        let _ = DwmSetWindowAttribute(
            hwnd_win,
            DWMWINDOWATTRIBUTE(36),
            &color_none as *const _ as *const _,
            std::mem::size_of::<u32>() as u32,
        );

        let border_thickness: u32 = 0;
        let _ = DwmSetWindowAttribute(
            hwnd_win,
            DWMWINDOWATTRIBUTE(37),
            &border_thickness as *const _ as *const _,
            std::mem::size_of::<u32>() as u32,
        );
    }
}

/// Stub for non-Windows platforms
#[cfg(not(target_os = "windows"))]
pub fn disable_window_shadow(_hwnd: isize) {
    // No-op on non-Windows platforms
}

/// Extend DWM frame into client area for transparent windows.
///
/// This function uses `DwmExtendFrameIntoClientArea` to extend the window frame
/// into the entire client area, which is required for proper transparent window
/// rendering with WebView2.
///
/// **CRITICAL**: This fixes the rendering artifacts (black stripes) that appear
/// when dragging transparent WebView2 windows. Without this, the window may show
/// visual glitches during movement.
///
/// # Arguments
/// * `hwnd` - Handle to the window to modify
///
/// # Official Documentation
/// - [DwmExtendFrameIntoClientArea](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/nf-dwmapi-dwmextendframeintoclientarea)
#[cfg(target_os = "windows")]
pub fn extend_frame_into_client_area(hwnd: isize) {
    tracing::info!(
        "[extend_frame_into_client_area] Called with HWND 0x{:X}",
        hwnd
    );
    unsafe {
        let hwnd_win = HWND(hwnd as *mut _);

        // Extend frame into entire client area (-1 means extend to entire window)
        // This is required for proper transparent window rendering
        let margins = MARGINS {
            cxLeftWidth: -1,
            cxRightWidth: -1,
            cyTopHeight: -1,
            cyBottomHeight: -1,
        };

        let result = DwmExtendFrameIntoClientArea(hwnd_win, &margins);

        if result.is_ok() {
            tracing::info!(
                "[OK] Extended DWM frame into client area: HWND 0x{:X} (margins: -1 all sides)",
                hwnd
            );
        } else {
            tracing::warn!(
                "[WARN] Failed to extend DWM frame: HWND 0x{:X}, HRESULT: {:?}",
                hwnd,
                result
            );
        }
    }
}

/// Stub for non-Windows platforms
#[cfg(not(target_os = "windows"))]
pub fn extend_frame_into_client_area(_hwnd: isize) {
    // No-op on non-Windows platforms
}

/// Apply WS_EX_LAYERED style for transparent windows.
///
/// **Note**: This function is provided for advanced use cases but is typically
/// NOT needed for WebView2 transparent windows. WebView2 handles transparency
/// internally through its own compositor.
///
/// The WS_EX_LAYERED style enables per-pixel alpha blending for traditional
/// GDI-based windows, but may interfere with WebView2's rendering.
///
/// # Arguments
/// * `hwnd` - Handle to the window to modify
///
/// # Official Documentation
/// - [WS_EX_LAYERED](https://learn.microsoft.com/en-us/windows/win32/winmsg/extended-window-styles)
/// - [Layered Windows](https://learn.microsoft.com/en-us/windows/win32/winmsg/window-features#layered-windows)
#[cfg(target_os = "windows")]
#[allow(dead_code)]
pub fn apply_layered_window_style(hwnd: isize) {
    unsafe {
        let hwnd_win = HWND(hwnd as *mut _);

        // Get current extended style
        let ex_style = GetWindowLongW(hwnd_win, GWL_EXSTYLE);

        // Add WS_EX_LAYERED for per-pixel alpha transparency
        // Note: Do NOT add WS_EX_TRANSPARENT as it makes the window click-through
        let new_ex_style = ex_style | (WS_EX_LAYERED.0 as i32);

        if new_ex_style != ex_style {
            SetWindowLongW(hwnd_win, GWL_EXSTYLE, new_ex_style);

            // Apply style changes
            let _ = SetWindowPos(
                hwnd_win,
                None,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
            );

            tracing::info!(
                "Applied WS_EX_LAYERED: HWND 0x{:X} (ex_style 0x{:08X} -> 0x{:08X})",
                hwnd,
                ex_style,
                new_ex_style
            );
        }
    }
}

/// Stub for non-Windows platforms
#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
pub fn apply_layered_window_style(_hwnd: isize) {
    // No-op on non-Windows platforms
}

/// Optimize transparent window for better resize performance.
///
/// This function applies several optimizations to reduce flickering and
/// improve rendering performance during window resize operations:
///
/// 1. Disables WM_ERASEBKGND handling to prevent background flashing
/// 2. Sets CS_HREDRAW and CS_VREDRAW to force full redraws
/// 3. Enables double buffering via WS_EX_COMPOSITED
///
/// **Note**: Call this AFTER the window and WebView are created.
///
/// # Arguments
/// * `hwnd` - Handle to the window to optimize
///
/// # Official Documentation
/// - [Window Class Styles](https://learn.microsoft.com/en-us/windows/win32/winmsg/window-class-styles)
/// - [Extended Window Styles](https://learn.microsoft.com/en-us/windows/win32/winmsg/extended-window-styles)
#[cfg(target_os = "windows")]
pub fn optimize_transparent_window_resize(hwnd: isize) {
    use windows::Win32::UI::WindowsAndMessaging::{
        GetClassLongPtrW, SetClassLongPtrW, CS_HREDRAW, CS_VREDRAW, GCL_STYLE,
    };

    tracing::info!(
        "[optimize_transparent_window_resize] Called with HWND 0x{:X}",
        hwnd
    );

    unsafe {
        let hwnd_win = HWND(hwnd as *mut _);

        // Get current class style
        let class_style = GetClassLongPtrW(hwnd_win, GCL_STYLE);

        // Add CS_HREDRAW and CS_VREDRAW for better resize handling
        // These cause the entire window to be redrawn when resized
        let new_class_style =
            class_style as isize | (CS_HREDRAW.0 as isize) | (CS_VREDRAW.0 as isize);

        if new_class_style != class_style as isize {
            SetClassLongPtrW(hwnd_win, GCL_STYLE, new_class_style);
            tracing::debug!(
                "Applied CS_HREDRAW|CS_VREDRAW: HWND 0x{:X} (class_style 0x{:X} -> 0x{:X})",
                hwnd,
                class_style,
                new_class_style
            );
        }

        // Get current extended style
        let ex_style = GetWindowLongW(hwnd_win, GWL_EXSTYLE);

        // Add WS_EX_COMPOSITED for double-buffered rendering
        // This reduces flicker during resize by buffering paints
        const WS_EX_COMPOSITED: i32 = 0x02000000;
        let new_ex_style = ex_style | WS_EX_COMPOSITED;

        if new_ex_style != ex_style {
            SetWindowLongW(hwnd_win, GWL_EXSTYLE, new_ex_style);

            // Apply style changes
            let _ = SetWindowPos(
                hwnd_win,
                None,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
            );

            tracing::info!(
                "Applied WS_EX_COMPOSITED for transparent window: HWND 0x{:X} (ex_style 0x{:08X} -> 0x{:08X})",
                hwnd,
                ex_style,
                new_ex_style
            );
        }
    }
}

/// Stub for non-Windows platforms
#[cfg(not(target_os = "windows"))]
pub fn optimize_transparent_window_resize(_hwnd: isize) {
    // No-op on non-Windows platforms
}

/// Remove WS_CLIPCHILDREN style from a window for proper transparency.
///
/// **CRITICAL for transparent windows on Windows 11!**
///
/// By default, tao/winit adds the `WS_CLIPCHILDREN` style to windows, which prevents
/// child windows (like WebView2) from rendering transparent content correctly.
/// The parent window clips the child window area, causing the transparency to
/// show through to whatever is behind the window instead of showing the WebView content.
///
/// This function removes the `WS_CLIPCHILDREN` style to fix transparency issues.
///
/// # When to use
/// Call this AFTER the window is created but BEFORE showing it, for any window with:
/// - `transparent=True`
/// - `frame=False` (frameless/undecorated)
///
/// # Arguments
/// * `hwnd` - Handle to the window to modify
///
/// # Official Documentation
/// - [WS_CLIPCHILDREN](https://learn.microsoft.com/en-us/windows/win32/winmsg/window-styles)
/// - [wry issue #1212](https://github.com/tauri-apps/wry/issues/1212)
#[cfg(target_os = "windows")]
pub fn remove_clip_children_style(hwnd: isize) {
    unsafe {
        let hwnd_win = HWND(hwnd as *mut _);

        // Get current window style
        let style = GetWindowLongW(hwnd_win, GWL_STYLE);

        // Check if WS_CLIPCHILDREN is set
        if (style & WS_CLIPCHILDREN.0 as i32) != 0 {
            // Remove WS_CLIPCHILDREN
            let new_style = style & !(WS_CLIPCHILDREN.0 as i32);

            SetWindowLongW(hwnd_win, GWL_STYLE, new_style);

            // Apply style changes
            let _ = SetWindowPos(
                hwnd_win,
                None,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
            );

            tracing::info!(
                "[OK] Removed WS_CLIPCHILDREN for transparent window: HWND 0x{:X} (style 0x{:08X} -> 0x{:08X})",
                hwnd,
                style,
                new_style
            );
        } else {
            tracing::debug!(
                "WS_CLIPCHILDREN not set on HWND 0x{:X}, no change needed",
                hwnd
            );
        }
    }
}

/// Stub for non-Windows platforms
#[cfg(not(target_os = "windows"))]
pub fn remove_clip_children_style(_hwnd: isize) {
    // No-op on non-Windows platforms
}
