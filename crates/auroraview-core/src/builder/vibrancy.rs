//! Window vibrancy effects (blur, acrylic, mica)
//!
//! This module provides Windows native background blur effects including:
//! - **Blur**: Basic blur effect (Windows 7+)
//! - **Acrylic**: Semi-transparent blur with tint (Windows 10 1809+)
//! - **Mica**: System backdrop material (Windows 11 22000+)
//!
//! # Platform Support
//!
//! | Effect | Windows 10 | Windows 11 |
//! |--------|------------|------------|
//! | Blur   | ✅ 1809+   | ✅         |
//! | Acrylic| ✅ 1809+   | ✅         |
//! | Mica   | ❌         | ✅ 22000+  |
//!
//! # Usage
//!
//! ```rust,ignore
//! use auroraview_core::builder::vibrancy::{apply_blur, apply_acrylic, apply_mica};
//!
//! // Apply blur with custom color
//! apply_blur(hwnd, Some((30, 30, 30, 200)))?;
//!
//! // Apply acrylic effect
//! apply_acrylic(hwnd, Some((30, 30, 30, 150)))?;
//!
//! // Apply mica (Windows 11 only)
//! apply_mica(hwnd, false)?; // false = light, true = dark
//! ```

use serde::{Deserialize, Serialize};

/// Vibrancy effect type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum VibrancyEffect {
    /// No effect
    #[default]
    None,
    /// Basic blur effect (Windows 7+, Windows 10 1809+)
    Blur,
    /// Acrylic blur with tint (Windows 10 1809+)
    Acrylic,
    /// Mica material (Windows 11 22000+)
    Mica,
    /// Mica Alt / Tabbed (Windows 11 22523+)
    MicaAlt,
}

/// RGBA color for blur/acrylic tint
pub type VibrancyColor = (u8, u8, u8, u8);

/// Result of applying vibrancy effect
#[derive(Debug)]
pub struct VibrancyResult {
    /// Whether the effect was applied successfully
    pub success: bool,
    /// The effect that was applied
    pub effect: VibrancyEffect,
    /// Error message if failed
    pub error: Option<String>,
}

impl VibrancyResult {
    #[allow(dead_code)]
    fn success(effect: VibrancyEffect) -> Self {
        Self {
            success: true,
            effect,
            error: None,
        }
    }

    #[allow(dead_code)]
    fn error(effect: VibrancyEffect, msg: impl Into<String>) -> Self {
        Self {
            success: false,
            effect,
            error: Some(msg.into()),
        }
    }
}

// Windows-specific implementation
#[cfg(target_os = "windows")]
mod windows_impl {
    use super::*;
    use std::ffi::c_void;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWINDOWATTRIBUTE};

    /// Windows version detection
    fn get_windows_build() -> u32 {
        #[repr(C)]
        struct OsVersionInfoExW {
            dw_os_version_info_size: u32,
            dw_major_version: u32,
            dw_minor_version: u32,
            dw_build_number: u32,
            dw_platform_id: u32,
            sz_csd_version: [u16; 128],
            w_service_pack_major: u16,
            w_service_pack_minor: u16,
            w_suite_mask: u16,
            w_product_type: u8,
            w_reserved: u8,
        }

        type RtlGetVersionFn = unsafe extern "system" fn(*mut OsVersionInfoExW) -> i32;

        unsafe {
            let ntdll = windows::Win32::System::LibraryLoader::GetModuleHandleW(windows::core::w!(
                "ntdll.dll"
            ));
            if let Ok(ntdll) = ntdll {
                let rtl_get_version = windows::Win32::System::LibraryLoader::GetProcAddress(
                    ntdll,
                    windows::core::s!("RtlGetVersion"),
                );
                if let Some(func) = rtl_get_version {
                    let func: RtlGetVersionFn = std::mem::transmute(func);
                    let mut info = std::mem::zeroed::<OsVersionInfoExW>();
                    info.dw_os_version_info_size = std::mem::size_of::<OsVersionInfoExW>() as u32;
                    if func(&mut info) == 0 {
                        return info.dw_build_number;
                    }
                }
            }
        }
        0
    }

    /// Check if SetWindowCompositionAttribute is available (Win10 1809+)
    pub fn is_swca_supported() -> bool {
        get_windows_build() >= 17763
    }

    /// Check if DWMWA_SYSTEMBACKDROP_TYPE is supported (Win11 22523+)
    pub fn is_backdrop_type_supported() -> bool {
        get_windows_build() >= 22523
    }

    /// Check if Mica is supported (Win11 22000+)
    pub fn is_mica_supported() -> bool {
        get_windows_build() >= 22000
    }

    // AccentPolicy structure for SetWindowCompositionAttribute
    #[repr(C)]
    struct AccentPolicy {
        accent_state: i32,
        accent_flags: i32,
        gradient_color: u32,
        animation_id: i32,
    }

    #[repr(C)]
    struct WindowCompositionAttributeData {
        attribute: i32,
        data: *mut c_void,
        size: usize,
    }

    // Accent states
    const ACCENT_DISABLED: i32 = 0;
    const ACCENT_ENABLE_BLURBEHIND: i32 = 3;
    const ACCENT_ENABLE_ACRYLICBLURBEHIND: i32 = 4;

    // WindowCompositionAttribute
    const WCA_ACCENT_POLICY: i32 = 19;

    // DWMWA constants (some are undocumented)
    const DWMWA_USE_IMMERSIVE_DARK_MODE: u32 = 20;
    const DWMWA_SYSTEMBACKDROP_TYPE: u32 = 38;
    const DWMWA_MICA_EFFECT: u32 = 1029; // Undocumented, for Win11 < 22523

    // DWM_SYSTEMBACKDROP_TYPE values
    const DWMSBT_DISABLE: i32 = 1;
    const DWMSBT_MAINWINDOW: i32 = 2; // Mica
    #[allow(dead_code)]
    const DWMSBT_TRANSIENTWINDOW: i32 = 3; // Acrylic (reserved for future use)
    const DWMSBT_TABBEDWINDOW: i32 = 4; // Mica Alt

    type SetWindowCompositionAttributeFn =
        unsafe extern "system" fn(HWND, *mut WindowCompositionAttributeData) -> i32;

    fn get_swca_function() -> Option<SetWindowCompositionAttributeFn> {
        unsafe {
            let user32 = windows::Win32::System::LibraryLoader::GetModuleHandleW(
                windows::core::w!("user32.dll"),
            )
            .ok()?;
            let proc = windows::Win32::System::LibraryLoader::GetProcAddress(
                user32,
                windows::core::s!("SetWindowCompositionAttribute"),
            )?;
            #[allow(clippy::missing_transmute_annotations)]
            Some(std::mem::transmute(proc))
        }
    }

    fn set_accent_policy(hwnd: isize, accent_state: i32, color: Option<VibrancyColor>) -> bool {
        let swca = match get_swca_function() {
            Some(f) => f,
            None => return false,
        };

        let gradient_color = color
            .map(|(r, g, b, a)| (a as u32) << 24 | (b as u32) << 16 | (g as u32) << 8 | (r as u32))
            .unwrap_or(0);

        let mut policy = AccentPolicy {
            accent_state,
            accent_flags: if color.is_some() { 2 } else { 0 }, // 2 = use gradient color
            gradient_color,
            animation_id: 0,
        };

        let mut data = WindowCompositionAttributeData {
            attribute: WCA_ACCENT_POLICY,
            data: &mut policy as *mut _ as *mut c_void,
            size: std::mem::size_of::<AccentPolicy>(),
        };

        unsafe {
            let hwnd_win = HWND(hwnd as *mut c_void);
            swca(hwnd_win, &mut data) != 0
        }
    }

    /// Apply blur effect to a window
    pub fn apply_blur(hwnd: isize, color: Option<VibrancyColor>) -> VibrancyResult {
        if !is_swca_supported() {
            return VibrancyResult::error(
                VibrancyEffect::Blur,
                "Blur requires Windows 10 1809 or later",
            );
        }

        if set_accent_policy(hwnd, ACCENT_ENABLE_BLURBEHIND, color) {
            tracing::info!("Applied blur effect to HWND 0x{:X}", hwnd);
            VibrancyResult::success(VibrancyEffect::Blur)
        } else {
            VibrancyResult::error(VibrancyEffect::Blur, "Failed to apply blur effect")
        }
    }

    /// Clear blur effect from a window
    pub fn clear_blur(hwnd: isize) -> VibrancyResult {
        if set_accent_policy(hwnd, ACCENT_DISABLED, None) {
            tracing::info!("Cleared blur effect from HWND 0x{:X}", hwnd);
            VibrancyResult::success(VibrancyEffect::None)
        } else {
            VibrancyResult::error(VibrancyEffect::None, "Failed to clear blur effect")
        }
    }

    /// Apply acrylic effect to a window
    pub fn apply_acrylic(hwnd: isize, color: Option<VibrancyColor>) -> VibrancyResult {
        if !is_swca_supported() {
            return VibrancyResult::error(
                VibrancyEffect::Acrylic,
                "Acrylic requires Windows 10 1809 or later",
            );
        }

        // For acrylic, alpha must not be 0
        let color = color.map(|(r, g, b, a)| (r, g, b, if a == 0 { 1 } else { a }));

        if set_accent_policy(hwnd, ACCENT_ENABLE_ACRYLICBLURBEHIND, color) {
            tracing::info!("Applied acrylic effect to HWND 0x{:X}", hwnd);
            VibrancyResult::success(VibrancyEffect::Acrylic)
        } else {
            VibrancyResult::error(VibrancyEffect::Acrylic, "Failed to apply acrylic effect")
        }
    }

    /// Clear acrylic effect from a window
    pub fn clear_acrylic(hwnd: isize) -> VibrancyResult {
        clear_blur(hwnd)
    }

    /// Apply mica effect to a window (Windows 11 only)
    pub fn apply_mica(hwnd: isize, dark: bool) -> VibrancyResult {
        if !is_mica_supported() {
            return VibrancyResult::error(
                VibrancyEffect::Mica,
                "Mica requires Windows 11 (build 22000) or later",
            );
        }

        unsafe {
            let hwnd_win = HWND(hwnd as *mut c_void);

            // Set dark mode preference
            let dark_mode: i32 = if dark { 1 } else { 0 };
            let _ = DwmSetWindowAttribute(
                hwnd_win,
                DWMWINDOWATTRIBUTE(DWMWA_USE_IMMERSIVE_DARK_MODE as i32),
                &dark_mode as *const _ as *const c_void,
                std::mem::size_of::<i32>() as u32,
            );

            // Try modern API first (Win11 22523+)
            if is_backdrop_type_supported() {
                let backdrop_type: i32 = DWMSBT_MAINWINDOW;
                let result = DwmSetWindowAttribute(
                    hwnd_win,
                    DWMWINDOWATTRIBUTE(DWMWA_SYSTEMBACKDROP_TYPE as i32),
                    &backdrop_type as *const _ as *const c_void,
                    std::mem::size_of::<i32>() as u32,
                );

                if result.is_ok() {
                    tracing::info!(
                        "Applied mica effect to HWND 0x{:X} (DWMWA_SYSTEMBACKDROP_TYPE)",
                        hwnd
                    );
                    return VibrancyResult::success(VibrancyEffect::Mica);
                }
            }

            // Fallback to undocumented API (Win11 22000-22522)
            let mica_effect: i32 = 1;
            let result = DwmSetWindowAttribute(
                hwnd_win,
                DWMWINDOWATTRIBUTE(DWMWA_MICA_EFFECT as i32),
                &mica_effect as *const _ as *const c_void,
                std::mem::size_of::<i32>() as u32,
            );

            if result.is_ok() {
                tracing::info!(
                    "Applied mica effect to HWND 0x{:X} (DWMWA_MICA_EFFECT)",
                    hwnd
                );
                VibrancyResult::success(VibrancyEffect::Mica)
            } else {
                VibrancyResult::error(VibrancyEffect::Mica, "Failed to apply mica effect")
            }
        }
    }

    /// Clear mica effect from a window
    pub fn clear_mica(hwnd: isize) -> VibrancyResult {
        unsafe {
            let hwnd_win = HWND(hwnd as *mut c_void);

            if is_backdrop_type_supported() {
                let backdrop_type: i32 = DWMSBT_DISABLE;
                let _ = DwmSetWindowAttribute(
                    hwnd_win,
                    DWMWINDOWATTRIBUTE(DWMWA_SYSTEMBACKDROP_TYPE as i32),
                    &backdrop_type as *const _ as *const c_void,
                    std::mem::size_of::<i32>() as u32,
                );
            } else {
                let mica_effect: i32 = 0;
                let _ = DwmSetWindowAttribute(
                    hwnd_win,
                    DWMWINDOWATTRIBUTE(DWMWA_MICA_EFFECT as i32),
                    &mica_effect as *const _ as *const c_void,
                    std::mem::size_of::<i32>() as u32,
                );
            }

            tracing::info!("Cleared mica effect from HWND 0x{:X}", hwnd);
            VibrancyResult::success(VibrancyEffect::None)
        }
    }

    /// Apply mica alt (tabbed) effect to a window (Windows 11 22523+ only)
    pub fn apply_mica_alt(hwnd: isize, dark: bool) -> VibrancyResult {
        if !is_backdrop_type_supported() {
            return VibrancyResult::error(
                VibrancyEffect::MicaAlt,
                "Mica Alt requires Windows 11 (build 22523) or later",
            );
        }

        unsafe {
            let hwnd_win = HWND(hwnd as *mut c_void);

            // Set dark mode preference
            let dark_mode: i32 = if dark { 1 } else { 0 };
            let _ = DwmSetWindowAttribute(
                hwnd_win,
                DWMWINDOWATTRIBUTE(DWMWA_USE_IMMERSIVE_DARK_MODE as i32),
                &dark_mode as *const _ as *const c_void,
                std::mem::size_of::<i32>() as u32,
            );

            let backdrop_type: i32 = DWMSBT_TABBEDWINDOW;
            let result = DwmSetWindowAttribute(
                hwnd_win,
                DWMWINDOWATTRIBUTE(DWMWA_SYSTEMBACKDROP_TYPE as i32),
                &backdrop_type as *const _ as *const c_void,
                std::mem::size_of::<i32>() as u32,
            );

            if result.is_ok() {
                tracing::info!("Applied mica alt effect to HWND 0x{:X}", hwnd);
                VibrancyResult::success(VibrancyEffect::MicaAlt)
            } else {
                VibrancyResult::error(VibrancyEffect::MicaAlt, "Failed to apply mica alt effect")
            }
        }
    }

    /// Clear mica alt effect from a window
    pub fn clear_mica_alt(hwnd: isize) -> VibrancyResult {
        clear_mica(hwnd)
    }
}

// Re-export Windows implementation
#[cfg(target_os = "windows")]
pub use windows_impl::*;

// Stubs for non-Windows platforms
#[cfg(not(target_os = "windows"))]
pub fn apply_blur(_hwnd: isize, _color: Option<VibrancyColor>) -> VibrancyResult {
    VibrancyResult::error(VibrancyEffect::Blur, "Blur is only supported on Windows")
}

#[cfg(not(target_os = "windows"))]
pub fn clear_blur(_hwnd: isize) -> VibrancyResult {
    VibrancyResult::error(VibrancyEffect::None, "Blur is only supported on Windows")
}

#[cfg(not(target_os = "windows"))]
pub fn apply_acrylic(_hwnd: isize, _color: Option<VibrancyColor>) -> VibrancyResult {
    VibrancyResult::error(
        VibrancyEffect::Acrylic,
        "Acrylic is only supported on Windows",
    )
}

#[cfg(not(target_os = "windows"))]
pub fn clear_acrylic(_hwnd: isize) -> VibrancyResult {
    VibrancyResult::error(VibrancyEffect::None, "Acrylic is only supported on Windows")
}

#[cfg(not(target_os = "windows"))]
pub fn apply_mica(_hwnd: isize, _dark: bool) -> VibrancyResult {
    VibrancyResult::error(VibrancyEffect::Mica, "Mica is only supported on Windows 11")
}

#[cfg(not(target_os = "windows"))]
pub fn clear_mica(_hwnd: isize) -> VibrancyResult {
    VibrancyResult::error(VibrancyEffect::None, "Mica is only supported on Windows 11")
}

#[cfg(not(target_os = "windows"))]
pub fn apply_mica_alt(_hwnd: isize, _dark: bool) -> VibrancyResult {
    VibrancyResult::error(
        VibrancyEffect::MicaAlt,
        "Mica Alt is only supported on Windows 11",
    )
}

#[cfg(not(target_os = "windows"))]
pub fn clear_mica_alt(_hwnd: isize) -> VibrancyResult {
    VibrancyResult::error(
        VibrancyEffect::None,
        "Mica Alt is only supported on Windows 11",
    )
}

#[cfg(not(target_os = "windows"))]
pub fn is_swca_supported() -> bool {
    false
}

#[cfg(not(target_os = "windows"))]
pub fn is_backdrop_type_supported() -> bool {
    false
}

#[cfg(not(target_os = "windows"))]
pub fn is_mica_supported() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vibrancy_effect_default() {
        let effect = VibrancyEffect::default();
        assert_eq!(effect, VibrancyEffect::None);
    }

    #[test]
    fn test_vibrancy_result_success() {
        let result = VibrancyResult::success(VibrancyEffect::Blur);
        assert!(result.success);
        assert_eq!(result.effect, VibrancyEffect::Blur);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_vibrancy_result_error() {
        let result = VibrancyResult::error(VibrancyEffect::Mica, "Test error");
        assert!(!result.success);
        assert_eq!(result.effect, VibrancyEffect::Mica);
        assert_eq!(result.error, Some("Test error".to_string()));
    }
}
