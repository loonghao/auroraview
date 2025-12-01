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
pub mod js {
    /// Navigate back in history
    pub const GO_BACK: &str = "history.back()";

    /// Navigate forward in history
    pub const GO_FORWARD: &str = "history.forward()";

    /// Reload current page
    pub const RELOAD: &str = "location.reload()";

    /// Get current URL (stores in window variable for IPC retrieval)
    pub const GET_CURRENT_URL: &str =
        "window.__auroraview_current_url = location.href; location.href";

    /// Clear all browsing data (localStorage, sessionStorage, IndexedDB, cookies)
    pub const CLEAR_ALL_BROWSING_DATA: &str = r#"
(function() {
    // Clear localStorage
    try { localStorage.clear(); } catch(e) {}
    // Clear sessionStorage
    try { sessionStorage.clear(); } catch(e) {}
    // Clear IndexedDB databases
    if (indexedDB && indexedDB.databases) {
        indexedDB.databases().then(dbs => {
            dbs.forEach(db => {
                try { indexedDB.deleteDatabase(db.name); } catch(e) {}
            });
        }).catch(() => {});
    }
    // Clear accessible cookies
    document.cookie.split(";").forEach(c => {
        document.cookie = c.replace(/^ +/, "")
            .replace(/=.*/, "=;expires=" + new Date().toUTCString() + ";path=/");
    });
    console.log('[AuroraView BOM] Browsing data cleared');
})();
"#;

    /// Build zoom script
    pub fn set_zoom(scale_factor: f64) -> String {
        format!(
            "document.body.style.zoom = '{}'; console.log('[AuroraView BOM] Zoom set to {}');",
            scale_factor, scale_factor
        )
    }
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
        assert_eq!(js::GO_BACK, "history.back()");
        assert_eq!(js::GO_FORWARD, "history.forward()");
        assert_eq!(js::RELOAD, "location.reload()");

        let zoom_script = js::set_zoom(1.5);
        assert!(zoom_script.contains("1.5"));
    }
}
