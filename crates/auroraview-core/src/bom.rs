//! Browser Object Model (BOM) API - Core Implementation
//!
//! This module provides core BOM APIs aligned with Tauri's WebView/Window API design.
//! These APIs are shared across all modes: standalone, CLI, pack, and embedded (DCC).
//!
//! ## Navigation APIs
//! - `go_back()` - Navigate back in history
//! - `go_forward()` - Navigate forward in history  
//! - `reload()` - Reload current page
//! - `get_current_url()` - Get current page URL
//!
//! ## Zoom APIs
//! - `set_zoom(scale)` - Set zoom level (1.0 = 100%)
//!
//! ## Window Control APIs
//! - `minimize()` - Minimize window
//! - `maximize()` - Maximize window
//! - `unmaximize()` - Restore window from maximized state
//! - `is_maximized()` - Check if window is maximized
//! - `is_minimized()` - Check if window is minimized
//! - `set_fullscreen(fullscreen)` - Set fullscreen mode
//! - `is_fullscreen()` - Check if window is in fullscreen
//! - `center()` - Center window on screen
//! - `set_size(width, height)` - Set window size
//! - `get_size()` - Get window size
//! - `get_position()` - Get window position
//!
//! ## Clear Data APIs
//! - `clear_all_browsing_data()` - Clear all browsing data

/// Result type for BOM operations
pub type BomResult<T> = Result<T, BomError>;

/// Error type for BOM operations
#[derive(Debug, Clone)]
pub enum BomError {
    /// WebView not available or locked
    WebViewUnavailable,
    /// Window not available
    WindowUnavailable,
    /// JavaScript execution failed
    JsExecutionFailed(String),
    /// Platform not supported for this operation
    PlatformNotSupported,
    /// Operation failed
    OperationFailed(String),
}

impl std::fmt::Display for BomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BomError::WebViewUnavailable => write!(f, "WebView not available or locked"),
            BomError::WindowUnavailable => write!(f, "Window not available"),
            BomError::JsExecutionFailed(msg) => write!(f, "JavaScript execution failed: {}", msg),
            BomError::PlatformNotSupported => {
                write!(f, "Platform not supported for this operation")
            }
            BomError::OperationFailed(msg) => write!(f, "Operation failed: {}", msg),
        }
    }
}

impl std::error::Error for BomError {}

/// Window size in physical pixels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PhysicalSize {
    pub width: u32,
    pub height: u32,
}

impl PhysicalSize {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

/// Window position in physical pixels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PhysicalPosition {
    pub x: i32,
    pub y: i32,
}

impl PhysicalPosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// JavaScript templates for BOM operations
///
/// NOTE: Most JavaScript code has been migrated to individual .js files
/// in `src/assets/js/bom/` for better maintainability. These constants
/// are kept for simple one-liners that don't warrant separate files.
///
/// Full JavaScript implementations are in:
/// - `src/assets/js/bom/navigation_tracker.js` - Navigation and loading state tracking
/// - `src/assets/js/bom/dom_events.js` - DOM and window events
/// - `src/assets/js/bom/browsing_data.js` - Storage and cookie management
/// - `src/assets/js/bom/navigation_api.js` - Navigation utility functions
/// - `src/assets/js/bom/zoom_api.js` - Zoom control functions
pub mod js {
    /// Navigate back in history
    pub const GO_BACK: &str =
        "window.__auroraview_goBack ? window.__auroraview_goBack() : history.back()";

    /// Navigate forward in history
    pub const GO_FORWARD: &str =
        "window.__auroraview_goForward ? window.__auroraview_goForward() : history.forward()";

    /// Stop loading current page
    pub const STOP: &str = "window.__auroraview_stop ? window.__auroraview_stop() : window.stop()";

    /// Check if can go back in history
    pub const CAN_GO_BACK: &str =
        "window.__auroraview_canGoBack ? window.__auroraview_canGoBack() : history.length > 1";

    /// Check if can go forward in history
    pub const CAN_GO_FORWARD: &str =
        "window.__auroraview_canGoForward ? window.__auroraview_canGoForward() : false";

    /// Check if page is currently loading
    pub const IS_LOADING: &str = "window.__auroraview_isLoading ? window.__auroraview_isLoading() : (document.readyState !== 'complete')";

    /// Get current load progress (0-100)
    pub const GET_LOAD_PROGRESS: &str = "window.__auroraview_getLoadProgress ? window.__auroraview_getLoadProgress() : (document.readyState === 'complete' ? 100 : 0)";

    /// Reload current page
    pub const RELOAD: &str =
        "window.__auroraview_reload ? window.__auroraview_reload() : location.reload()";

    /// Get current URL
    pub const GET_CURRENT_URL: &str =
        "window.__auroraview_getCurrentUrl ? window.__auroraview_getCurrentUrl() : location.href";

    /// Clear all browsing data
    pub const CLEAR_ALL_BROWSING_DATA: &str =
        "window.__auroraview_clearAllBrowsingData && window.__auroraview_clearAllBrowsingData()";

    /// Clear localStorage only
    pub const CLEAR_LOCAL_STORAGE: &str =
        "window.__auroraview_clearLocalStorage && window.__auroraview_clearLocalStorage()";

    /// Clear sessionStorage only
    pub const CLEAR_SESSION_STORAGE: &str =
        "window.__auroraview_clearSessionStorage && window.__auroraview_clearSessionStorage()";

    /// Clear IndexedDB
    pub const CLEAR_INDEXED_DB: &str =
        "window.__auroraview_clearIndexedDB && window.__auroraview_clearIndexedDB()";

    /// Clear cookies
    pub const CLEAR_COOKIES: &str =
        "window.__auroraview_clearCookies && window.__auroraview_clearCookies()";

    /// Build zoom script
    pub fn set_zoom(scale_factor: f64) -> String {
        format!(
            "window.__auroraview_setZoom ? window.__auroraview_setZoom({}) : (document.body.style.zoom = '{}')",
            scale_factor, scale_factor
        )
    }

    /// Get current zoom level
    pub const GET_ZOOM: &str = "window.__auroraview_getZoom ? window.__auroraview_getZoom() : 1.0";

    /// Zoom in
    pub const ZOOM_IN: &str = "window.__auroraview_zoomIn && window.__auroraview_zoomIn()";

    /// Zoom out
    pub const ZOOM_OUT: &str = "window.__auroraview_zoomOut && window.__auroraview_zoomOut()";

    /// Reset zoom
    pub const RESET_ZOOM: &str = "window.__auroraview_resetZoom && window.__auroraview_resetZoom()";
}

/// Trait for BOM navigation operations
///
/// This trait defines the navigation API that can be implemented by different
/// WebView backends (standalone, CLI, embedded, etc.)
pub trait NavigationApi {
    /// Navigate back in history
    fn go_back(&self) -> BomResult<()>;

    /// Navigate forward in history
    fn go_forward(&self) -> BomResult<()>;

    /// Stop loading current page
    fn stop(&self) -> BomResult<()>;

    /// Check if can navigate back in history
    fn can_go_back(&self) -> BomResult<bool>;

    /// Check if can navigate forward in history
    fn can_go_forward(&self) -> BomResult<bool>;

    /// Reload current page
    fn reload(&self) -> BomResult<()>;

    /// Get current URL (async via callback)
    fn get_current_url(&self, callback: Box<dyn FnOnce(String) + Send>) -> BomResult<()>;
}

/// Trait for BOM zoom operations
pub trait ZoomApi {
    /// Set zoom level (1.0 = 100%, 1.5 = 150%, etc.)
    fn set_zoom(&self, scale_factor: f64) -> BomResult<()>;

    /// Get current zoom level
    fn zoom(&self) -> BomResult<f64>;
}

/// Trait for BOM window control operations
pub trait WindowControlApi {
    /// Minimize window
    fn minimize(&self) -> BomResult<()>;

    /// Maximize window
    fn maximize(&self) -> BomResult<()>;

    /// Unmaximize (restore) window
    fn unmaximize(&self) -> BomResult<()>;

    /// Toggle maximize state
    fn toggle_maximize(&self) -> BomResult<()>;

    /// Check if window is maximized
    fn is_maximized(&self) -> BomResult<bool>;

    /// Check if window is minimized
    fn is_minimized(&self) -> BomResult<bool>;

    /// Set fullscreen mode
    fn set_fullscreen(&self, fullscreen: bool) -> BomResult<()>;

    /// Check if window is in fullscreen mode
    fn is_fullscreen(&self) -> BomResult<bool>;

    /// Set window visibility
    fn set_visible(&self, visible: bool) -> BomResult<()>;

    /// Check if window is visible
    fn is_visible(&self) -> BomResult<bool>;

    /// Check if window has focus
    fn is_focused(&self) -> BomResult<bool>;

    /// Request focus for the window
    fn set_focus(&self) -> BomResult<()>;

    /// Set window title
    fn set_title(&self, title: &str) -> BomResult<()>;

    /// Get window title
    fn title(&self) -> BomResult<String>;

    /// Set window size
    fn set_size(&self, width: u32, height: u32) -> BomResult<()>;

    /// Get window inner size
    fn inner_size(&self) -> BomResult<PhysicalSize>;

    /// Get window outer size (including decorations)
    fn outer_size(&self) -> BomResult<PhysicalSize>;

    /// Get window position
    fn position(&self) -> BomResult<PhysicalPosition>;

    /// Set window position
    fn set_position(&self, x: i32, y: i32) -> BomResult<()>;

    /// Center window on screen
    fn center(&self) -> BomResult<()>;

    /// Set window decorations (title bar, borders)
    fn set_decorations(&self, decorations: bool) -> BomResult<()>;

    /// Set window resizable
    fn set_resizable(&self, resizable: bool) -> BomResult<()>;

    /// Set minimum window size
    fn set_min_size(&self, width: u32, height: u32) -> BomResult<()>;

    /// Set maximum window size
    fn set_max_size(&self, width: u32, height: u32) -> BomResult<()>;

    /// Set always on top
    fn set_always_on_top(&self, always_on_top: bool) -> BomResult<()>;

    /// Check if always on top
    fn is_always_on_top(&self) -> BomResult<bool>;
}

/// Trait for clearing browsing data
pub trait ClearDataApi {
    /// Clear all browsing data (localStorage, sessionStorage, IndexedDB, cookies)
    fn clear_all_browsing_data(&self) -> BomResult<()>;
}

/// Combined BOM API trait
///
/// Implement this for types that support all BOM operations.
pub trait BomApi: NavigationApi + ZoomApi + WindowControlApi + ClearDataApi {}

// Blanket implementation for types that implement all sub-traits
impl<T> BomApi for T where T: NavigationApi + ZoomApi + WindowControlApi + ClearDataApi {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physical_size() {
        let size = PhysicalSize::new(800, 600);
        assert_eq!(size.width, 800);
        assert_eq!(size.height, 600);
    }

    #[test]
    fn test_physical_position() {
        let pos = PhysicalPosition::new(100, 200);
        assert_eq!(pos.x, 100);
        assert_eq!(pos.y, 200);
    }

    #[test]
    fn test_bom_error_display() {
        let err = BomError::WebViewUnavailable;
        assert_eq!(err.to_string(), "WebView not available or locked");

        let err = BomError::JsExecutionFailed("syntax error".to_string());
        assert!(err.to_string().contains("syntax error"));
    }

    #[test]
    fn test_js_templates() {
        // These use fallback pattern for custom protocol support
        assert!(js::GO_BACK.contains("history.back()"));
        assert!(js::GO_FORWARD.contains("history.forward()"));
        assert!(js::STOP.contains("window.stop()"));
        assert!(js::RELOAD.contains("location.reload()"));

        let zoom_script = js::set_zoom(1.5);
        assert!(zoom_script.contains("1.5"));
    }
}
