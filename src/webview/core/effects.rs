//! AuroraView Core - Window Effects APIs
//!
//! This module provides window visual effects:
//! - Click-through with interactive regions
//! - Background blur (Acrylic, Mica)

use pyo3::prelude::*;

use super::AuroraView;
use auroraview_core::builder::{
    apply_acrylic, apply_blur, apply_mica, apply_mica_alt, clear_acrylic, clear_blur, clear_mica,
    clear_mica_alt, disable_click_through, enable_click_through, get_interactive_regions,
    is_click_through_enabled, update_interactive_regions, InteractiveRegion,
};

impl AuroraView {
    /// Get HWND from cached value (doesn't require borrowing inner)
    #[cfg(target_os = "windows")]
    fn get_cached_hwnd(&self) -> PyResult<isize> {
        self.cached_hwnd
            .try_borrow()
            .map_err(|_| pyo3::exceptions::PyRuntimeError::new_err("HWND cache is busy"))?
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("HWND not available"))
            .map(|h| h as isize)
    }

    #[cfg(not(target_os = "windows"))]
    fn get_cached_hwnd(&self) -> PyResult<isize> {
        Err(pyo3::exceptions::PyRuntimeError::new_err(
            "Window effects are only supported on Windows",
        ))
    }
}

#[pymethods]
impl AuroraView {
    // ========================================
    // Click-Through APIs
    // ========================================

    /// Enable click-through mode for the window.
    ///
    /// When enabled, mouse events will pass through the window to underlying windows,
    /// except for areas marked as interactive regions.
    ///
    /// # Example
    /// ```python
    /// webview.enable_click_through()
    /// webview.update_interactive_regions([
    ///     {"x": 10, "y": 10, "width": 100, "height": 50}
    /// ])
    /// ```
    ///
    /// # Platform Support
    /// - Windows 10/11: Full support
    /// - macOS/Linux: Not supported
    fn enable_click_through(&self) -> PyResult<bool> {
        let hwnd = self.get_cached_hwnd()?;
        match enable_click_through(hwnd) {
            Ok(result) => Ok(result.success),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e)),
        }
    }

    /// Disable click-through mode for the window.
    ///
    /// Restores normal mouse event handling.
    fn disable_click_through(&self) -> PyResult<()> {
        let hwnd = self.get_cached_hwnd()?;
        disable_click_through(hwnd).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }

    /// Check if click-through is enabled for the window.
    fn is_click_through_enabled(&self) -> PyResult<bool> {
        let hwnd = self.get_cached_hwnd()?;
        Ok(is_click_through_enabled(hwnd))
    }

    /// Update interactive regions for click-through mode.
    ///
    /// Interactive regions are areas where mouse events are NOT passed through.
    /// Typically these correspond to UI elements that the user should be able to click.
    ///
    /// # Arguments
    /// * `regions` - List of region dictionaries with keys: x, y, width, height
    ///
    /// # Example
    /// ```python
    /// webview.update_interactive_regions([
    ///     {"x": 10, "y": 10, "width": 100, "height": 50},
    ///     {"x": 200, "y": 100, "width": 150, "height": 80},
    /// ])
    /// ```
    fn update_interactive_regions(&self, regions: Vec<PyRegion>) -> PyResult<()> {
        let hwnd = self.get_cached_hwnd()?;
        let regions: Vec<InteractiveRegion> = regions
            .into_iter()
            .map(|r| InteractiveRegion::new(r.x, r.y, r.width, r.height))
            .collect();
        update_interactive_regions(hwnd, regions).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }

    /// Get current interactive regions.
    ///
    /// Returns a list of region dictionaries with keys: x, y, width, height
    fn get_interactive_regions(&self) -> PyResult<Vec<PyRegion>> {
        let hwnd = self.get_cached_hwnd()?;
        let regions = get_interactive_regions(hwnd).unwrap_or_default();
        Ok(regions
            .into_iter()
            .map(|r| PyRegion {
                x: r.x,
                y: r.y,
                width: r.width,
                height: r.height,
            })
            .collect())
    }

    // ========================================
    // Vibrancy (Background Blur) APIs
    // ========================================

    /// Apply blur effect to the window background.
    ///
    /// # Arguments
    /// * `color` - Optional RGBA tuple (r, g, b, a) for blur tint. Values 0-255.
    ///
    /// # Example
    /// ```python
    /// # Default blur
    /// webview.apply_blur()
    ///
    /// # Blur with dark tint
    /// webview.apply_blur((30, 30, 30, 200))
    /// ```
    ///
    /// # Platform Support
    /// - Windows 10 1809+: Full support
    /// - Windows 11: Full support
    /// - macOS/Linux: Not supported
    #[pyo3(signature = (color=None))]
    fn apply_blur(&self, color: Option<(u8, u8, u8, u8)>) -> PyResult<bool> {
        let hwnd = self.get_cached_hwnd()?;
        let result = apply_blur(hwnd, color);
        if result.success {
            Ok(true)
        } else {
            Err(pyo3::exceptions::PyRuntimeError::new_err(
                result.error.unwrap_or_else(|| "Unknown error".to_string()),
            ))
        }
    }

    /// Clear blur effect from the window.
    fn clear_blur(&self) -> PyResult<()> {
        let hwnd = self.get_cached_hwnd()?;
        let result = clear_blur(hwnd);
        if result.success {
            Ok(())
        } else {
            Err(pyo3::exceptions::PyRuntimeError::new_err(
                result.error.unwrap_or_else(|| "Unknown error".to_string()),
            ))
        }
    }

    /// Apply acrylic effect to the window background.
    ///
    /// Acrylic is a semi-transparent blur effect with noise texture.
    ///
    /// # Arguments
    /// * `color` - Optional RGBA tuple (r, g, b, a) for acrylic tint. Values 0-255.
    ///             Note: alpha must not be 0 for acrylic to work.
    ///
    /// # Example
    /// ```python
    /// # Acrylic with dark tint
    /// webview.apply_acrylic((30, 30, 30, 150))
    /// ```
    ///
    /// # Platform Support
    /// - Windows 10 1809+: Full support
    /// - Windows 11: Full support (may have performance issues in some versions)
    /// - macOS/Linux: Not supported
    #[pyo3(signature = (color=None))]
    fn apply_acrylic(&self, color: Option<(u8, u8, u8, u8)>) -> PyResult<bool> {
        let hwnd = self.get_cached_hwnd()?;
        let result = apply_acrylic(hwnd, color);
        if result.success {
            Ok(true)
        } else {
            Err(pyo3::exceptions::PyRuntimeError::new_err(
                result.error.unwrap_or_else(|| "Unknown error".to_string()),
            ))
        }
    }

    /// Clear acrylic effect from the window.
    fn clear_acrylic(&self) -> PyResult<()> {
        let hwnd = self.get_cached_hwnd()?;
        let result = clear_acrylic(hwnd);
        if result.success {
            Ok(())
        } else {
            Err(pyo3::exceptions::PyRuntimeError::new_err(
                result.error.unwrap_or_else(|| "Unknown error".to_string()),
            ))
        }
    }

    /// Apply Mica effect to the window background.
    ///
    /// Mica is a Windows 11 material that samples the desktop wallpaper
    /// to create a subtle, personalized backdrop.
    ///
    /// # Arguments
    /// * `dark` - Whether to use dark mode variant
    ///
    /// # Example
    /// ```python
    /// # Light mode mica
    /// webview.apply_mica(dark=False)
    ///
    /// # Dark mode mica
    /// webview.apply_mica(dark=True)
    /// ```
    ///
    /// # Platform Support
    /// - Windows 11 (build 22000+): Full support
    /// - Windows 10: Not supported
    /// - macOS/Linux: Not supported
    #[pyo3(signature = (dark=false))]
    fn apply_mica(&self, dark: bool) -> PyResult<bool> {
        let hwnd = self.get_cached_hwnd()?;
        let result = apply_mica(hwnd, dark);
        if result.success {
            Ok(true)
        } else {
            Err(pyo3::exceptions::PyRuntimeError::new_err(
                result.error.unwrap_or_else(|| "Unknown error".to_string()),
            ))
        }
    }

    /// Clear Mica effect from the window.
    fn clear_mica(&self) -> PyResult<()> {
        let hwnd = self.get_cached_hwnd()?;
        let result = clear_mica(hwnd);
        if result.success {
            Ok(())
        } else {
            Err(pyo3::exceptions::PyRuntimeError::new_err(
                result.error.unwrap_or_else(|| "Unknown error".to_string()),
            ))
        }
    }

    /// Apply Mica Alt (Tabbed) effect to the window background.
    ///
    /// Mica Alt is a variant of Mica with a more prominent backdrop,
    /// typically used for tabbed windows.
    ///
    /// # Arguments
    /// * `dark` - Whether to use dark mode variant
    ///
    /// # Platform Support
    /// - Windows 11 (build 22523+): Full support
    /// - Earlier versions: Not supported
    #[pyo3(signature = (dark=false))]
    fn apply_mica_alt(&self, dark: bool) -> PyResult<bool> {
        let hwnd = self.get_cached_hwnd()?;
        let result = apply_mica_alt(hwnd, dark);
        if result.success {
            Ok(true)
        } else {
            Err(pyo3::exceptions::PyRuntimeError::new_err(
                result.error.unwrap_or_else(|| "Unknown error".to_string()),
            ))
        }
    }

    /// Clear Mica Alt effect from the window.
    fn clear_mica_alt(&self) -> PyResult<()> {
        let hwnd = self.get_cached_hwnd()?;
        let result = clear_mica_alt(hwnd);
        if result.success {
            Ok(())
        } else {
            Err(pyo3::exceptions::PyRuntimeError::new_err(
                result.error.unwrap_or_else(|| "Unknown error".to_string()),
            ))
        }
    }

    /// Get the current vibrancy effect applied to the window.
    ///
    /// Returns one of: "none", "blur", "acrylic", "mica", "mica_alt"
    fn vibrancy_effect(&self) -> PyResult<String> {
        // Note: We don't track the current effect state, so this returns "none"
        // A proper implementation would track the applied effect
        Ok("none".to_string())
    }
}

/// Python-friendly region struct
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyRegion {
    #[pyo3(get, set)]
    pub x: i32,
    #[pyo3(get, set)]
    pub y: i32,
    #[pyo3(get, set)]
    pub width: i32,
    #[pyo3(get, set)]
    pub height: i32,
}

#[pymethods]
impl PyRegion {
    #[new]
    fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Region(x={}, y={}, width={}, height={})",
            self.x, self.y, self.width, self.height
        )
    }
}
