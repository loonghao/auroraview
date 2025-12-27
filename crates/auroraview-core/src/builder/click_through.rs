//! Click-through window support with interactive regions
//!
//! This module provides functionality for creating transparent windows that
//! allow mouse events to pass through to underlying windows, while maintaining
//! specific interactive regions where the window can receive input.
//!
//! # Overview
//!
//! The click-through feature is implemented using Windows' `WM_NCHITTEST` message.
//! When a window receives this message, it can return `HTTRANSPARENT` to indicate
//! that the mouse event should be passed to the window below.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │  Transparent Window (click-through)     │
//! │  ┌─────────────┐                        │
//! │  │ Interactive │  ← Mouse events work   │
//! │  │   Region    │                        │
//! │  └─────────────┘                        │
//! │                    ← Click passes through│
//! └─────────────────────────────────────────┘
//! ```
//!
//! # Usage
//!
//! 1. Enable click-through on a window using `enable_click_through()`
//! 2. Register interactive regions using `update_interactive_regions()`
//! 3. The window will pass through clicks outside interactive regions
//!
//! # Example
//!
//! ```rust,ignore
//! use auroraview_core::builder::click_through::{
//!     enable_click_through, update_interactive_regions, InteractiveRegion,
//! };
//!
//! // Enable click-through on window
//! enable_click_through(hwnd)?;
//!
//! // Define interactive regions (from frontend data-interactive elements)
//! let regions = vec![
//!     InteractiveRegion { x: 10, y: 10, width: 100, height: 50 },
//!     InteractiveRegion { x: 200, y: 100, width: 150, height: 80 },
//! ];
//! update_interactive_regions(hwnd, regions)?;
//! ```
//!
//! # Platform Support
//!
//! - **Windows 10/11**: Full support via WM_NCHITTEST
//! - **macOS**: Not yet implemented
//! - **Linux**: Not yet implemented

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

#[cfg(target_os = "windows")]
use std::ffi::c_void;
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, DefWindowProcW, GetWindowLongPtrW, SetWindowLongPtrW, GWLP_WNDPROC, WNDPROC,
};

/// An interactive region within a click-through window
///
/// Coordinates are relative to the window's client area.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct InteractiveRegion {
    /// X coordinate (left edge) in pixels
    pub x: i32,
    /// Y coordinate (top edge) in pixels
    pub y: i32,
    /// Width in pixels
    pub width: i32,
    /// Height in pixels
    pub height: i32,
}

impl InteractiveRegion {
    /// Create a new interactive region
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Check if a point is within this region
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}

/// Configuration for click-through behavior
#[derive(Debug, Clone, Default)]
pub struct ClickThroughConfig {
    /// Whether click-through is enabled
    pub enabled: bool,
    /// Interactive regions where clicks are NOT passed through
    pub regions: Vec<InteractiveRegion>,
}

impl ClickThroughConfig {
    /// Create a new click-through configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable click-through
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set interactive regions
    pub fn with_regions(mut self, regions: Vec<InteractiveRegion>) -> Self {
        self.regions = regions;
        self
    }

    /// Check if a point should be interactive (not passed through)
    pub fn is_interactive(&self, x: i32, y: i32) -> bool {
        if !self.enabled {
            return true; // Not in click-through mode, everything is interactive
        }
        self.regions.iter().any(|r| r.contains(x, y))
    }
}

/// Global storage for click-through configurations per window
static CLICK_THROUGH_DATA: std::sync::LazyLock<RwLock<HashMap<isize, ClickThroughState>>> =
    std::sync::LazyLock::new(|| RwLock::new(HashMap::new()));

/// State for a click-through enabled window
#[derive(Debug)]
struct ClickThroughState {
    /// Original window procedure
    #[cfg(target_os = "windows")]
    original_wndproc: isize,
    /// Click-through configuration (reserved for future region-based click-through)
    #[allow(dead_code)]
    config: ClickThroughConfig,
}

/// Result of enabling click-through on a window
#[derive(Debug)]
pub struct ClickThroughResult {
    /// Whether click-through was successfully enabled
    pub success: bool,
    /// Original window procedure (for restoration)
    pub original_wndproc: isize,
}

/// WM_NCHITTEST message constant
#[cfg(target_os = "windows")]
const WM_NCHITTEST: u32 = 0x0084;

/// HTTRANSPARENT - indicates click should pass through
#[cfg(target_os = "windows")]
const HTTRANSPARENT: isize = -1;

/// Enable click-through on a window
///
/// This subclasses the window to intercept WM_NCHITTEST messages and
/// return HTTRANSPARENT for areas outside interactive regions.
///
/// # Arguments
/// * `hwnd` - Window handle
///
/// # Returns
/// Result containing the original window procedure, or error
///
/// # Safety
/// Uses unsafe Windows API calls for window subclassing.
#[cfg(target_os = "windows")]
pub fn enable_click_through(hwnd: isize) -> Result<ClickThroughResult, String> {
    unsafe {
        let hwnd_win = HWND(hwnd as *mut c_void);

        // Check if already enabled
        if CLICK_THROUGH_DATA.read().unwrap().contains_key(&hwnd) {
            return Err("Click-through already enabled for this window".to_string());
        }

        // Get the original window procedure
        let original_wndproc = GetWindowLongPtrW(hwnd_win, GWLP_WNDPROC);
        if original_wndproc == 0 {
            return Err("Failed to get original window procedure".to_string());
        }

        // Store state
        let state = ClickThroughState {
            original_wndproc,
            config: ClickThroughConfig {
                enabled: true,
                regions: Vec::new(),
            },
        };
        CLICK_THROUGH_DATA.write().unwrap().insert(hwnd, state);

        // Subclass the window
        #[allow(clippy::fn_to_numeric_cast)]
        let result = SetWindowLongPtrW(hwnd_win, GWLP_WNDPROC, click_through_wndproc as isize);
        if result == 0 {
            // Rollback
            CLICK_THROUGH_DATA.write().unwrap().remove(&hwnd);
            return Err("Failed to subclass window".to_string());
        }

        tracing::info!(
            "Enabled click-through for HWND 0x{:X} (original wndproc: 0x{:X})",
            hwnd,
            original_wndproc
        );

        Ok(ClickThroughResult {
            success: true,
            original_wndproc,
        })
    }
}

/// Disable click-through on a window
///
/// Restores the original window procedure.
#[cfg(target_os = "windows")]
pub fn disable_click_through(hwnd: isize) -> Result<(), String> {
    unsafe {
        let hwnd_win = HWND(hwnd as *mut c_void);

        // Get and remove state
        let state = CLICK_THROUGH_DATA
            .write()
            .unwrap()
            .remove(&hwnd)
            .ok_or("Click-through not enabled for this window")?;

        // Restore original window procedure
        SetWindowLongPtrW(hwnd_win, GWLP_WNDPROC, state.original_wndproc);

        tracing::info!(
            "Disabled click-through for HWND 0x{:X} (restored wndproc: 0x{:X})",
            hwnd,
            state.original_wndproc
        );

        Ok(())
    }
}

/// Update interactive regions for a click-through window
///
/// # Arguments
/// * `hwnd` - Window handle
/// * `regions` - New list of interactive regions
#[cfg(target_os = "windows")]
pub fn update_interactive_regions(
    hwnd: isize,
    regions: Vec<InteractiveRegion>,
) -> Result<(), String> {
    let mut data = CLICK_THROUGH_DATA.write().unwrap();
    let state = data
        .get_mut(&hwnd)
        .ok_or("Click-through not enabled for this window")?;

    tracing::debug!(
        "Updated interactive regions for HWND 0x{:X}: {} regions",
        hwnd,
        regions.len()
    );

    state.config.regions = regions;
    Ok(())
}

/// Get current interactive regions for a window
#[cfg(target_os = "windows")]
pub fn get_interactive_regions(hwnd: isize) -> Option<Vec<InteractiveRegion>> {
    CLICK_THROUGH_DATA
        .read()
        .unwrap()
        .get(&hwnd)
        .map(|state| state.config.regions.clone())
}

/// Check if click-through is enabled for a window
pub fn is_click_through_enabled(hwnd: isize) -> bool {
    CLICK_THROUGH_DATA.read().unwrap().contains_key(&hwnd)
}

/// Window procedure for click-through windows
#[cfg(target_os = "windows")]
unsafe extern "system" fn click_through_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let hwnd_isize = hwnd.0 as isize;

    if msg == WM_NCHITTEST {
        // Get mouse position from lparam
        let x = (lparam.0 & 0xFFFF) as i16 as i32;
        let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

        // Convert screen coordinates to client coordinates
        let mut point = windows::Win32::Foundation::POINT { x, y };
        let _ = windows::Win32::Graphics::Gdi::ScreenToClient(hwnd, &mut point);

        // Check if point is in an interactive region
        let data = CLICK_THROUGH_DATA.read().unwrap();
        if let Some(state) = data.get(&hwnd_isize) {
            if state.config.enabled && !state.config.is_interactive(point.x, point.y) {
                // Pass through - let click go to window below
                return LRESULT(HTTRANSPARENT);
            }
        }
    }

    // Call original window procedure
    let data = CLICK_THROUGH_DATA.read().unwrap();
    if let Some(state) = data.get(&hwnd_isize) {
        let original_wndproc: WNDPROC =
            std::mem::transmute::<isize, WNDPROC>(state.original_wndproc);
        if let Some(proc) = original_wndproc {
            return CallWindowProcW(Some(proc), hwnd, msg, wparam, lparam);
        }
    }

    DefWindowProcW(hwnd, msg, wparam, lparam)
}

// Stubs for non-Windows platforms
#[cfg(not(target_os = "windows"))]
pub fn enable_click_through(_hwnd: isize) -> Result<ClickThroughResult, String> {
    Err("Click-through is only supported on Windows".to_string())
}

#[cfg(not(target_os = "windows"))]
pub fn disable_click_through(_hwnd: isize) -> Result<(), String> {
    Err("Click-through is only supported on Windows".to_string())
}

#[cfg(not(target_os = "windows"))]
pub fn update_interactive_regions(
    _hwnd: isize,
    _regions: Vec<InteractiveRegion>,
) -> Result<(), String> {
    Err("Click-through is only supported on Windows".to_string())
}

#[cfg(not(target_os = "windows"))]
pub fn get_interactive_regions(_hwnd: isize) -> Option<Vec<InteractiveRegion>> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interactive_region_contains() {
        let region = InteractiveRegion::new(10, 20, 100, 50);

        // Inside
        assert!(region.contains(10, 20)); // Top-left corner
        assert!(region.contains(50, 40)); // Center
        assert!(region.contains(109, 69)); // Bottom-right (exclusive)

        // Outside
        assert!(!region.contains(9, 20)); // Left of region
        assert!(!region.contains(10, 19)); // Above region
        assert!(!region.contains(110, 20)); // Right of region
        assert!(!region.contains(10, 70)); // Below region
    }

    #[test]
    fn test_click_through_config() {
        let config = ClickThroughConfig::new()
            .with_enabled(true)
            .with_regions(vec![
                InteractiveRegion::new(0, 0, 100, 100),
                InteractiveRegion::new(200, 200, 50, 50),
            ]);

        assert!(config.enabled);
        assert_eq!(config.regions.len(), 2);

        // Test is_interactive
        assert!(config.is_interactive(50, 50)); // In first region
        assert!(config.is_interactive(225, 225)); // In second region
        assert!(!config.is_interactive(150, 150)); // Outside all regions
    }

    #[test]
    fn test_click_through_config_disabled() {
        let config = ClickThroughConfig::new().with_enabled(false);

        // When disabled, everything should be interactive
        assert!(config.is_interactive(0, 0));
        assert!(config.is_interactive(1000, 1000));
    }
}
