//! Window icon loading utilities.
//!
//! This module handles loading custom window icons from files,
//! with a fallback to an embedded default icon.

use std::path::PathBuf;

/// Embedded window icon (32x32 PNG) - used as fallback.
const DEFAULT_ICON_PNG_BYTES: &[u8] = include_bytes!("../../../assets/icons/auroraview-32.png");

/// Load window icon from custom path or use embedded default.
///
/// # Arguments
/// * `custom_icon` - Optional path to custom icon file (PNG, ICO, JPEG, BMP, GIF)
///
/// # Icon Requirements
/// - **Format**: PNG (recommended), ICO, JPEG, BMP, GIF
/// - **Recommended sizes**: 32x32 (taskbar), 64x64 (alt-tab), 256x256 (high-DPI)
/// - **Color depth**: 32-bit RGBA recommended for transparency support
pub fn load_window_icon(custom_icon: Option<&PathBuf>) -> Option<tao::window::Icon> {
    use image::GenericImageView;

    // Try custom icon first
    if let Some(icon_path) = custom_icon {
        if icon_path.exists() {
            match ::image::open(icon_path) {
                Ok(img) => {
                    let (width, height) = img.dimensions();
                    let rgba = img.into_rgba8().into_raw();
                    if let Ok(icon) = tao::window::Icon::from_rgba(rgba, width, height) {
                        tracing::info!("[desktop/icon] Loaded custom icon from {:?}", icon_path);
                        return Some(icon);
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "[desktop/icon] Failed to load custom icon {:?}: {}, using default",
                        icon_path,
                        e
                    );
                }
            }
        } else {
            tracing::warn!(
                "[desktop/icon] Custom icon path does not exist: {:?}, using default",
                icon_path
            );
        }
    }

    // Fall back to embedded default icon
    let img = ::image::load_from_memory(DEFAULT_ICON_PNG_BYTES).ok()?;
    let (width, height) = img.dimensions();
    let rgba = img.into_rgba8().into_raw();

    tao::window::Icon::from_rgba(rgba, width, height).ok()
}
