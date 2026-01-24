//! Tray icon loading

use crate::error::{DesktopError, Result};
use std::path::PathBuf;
use tray_icon::Icon;

/// Default tray icon
const DEFAULT_ICON: &[u8] = include_bytes!("../../assets/icons/auroraview-32.png");

/// Load tray icon from config or fallback
pub fn load_tray_icon(
    tray_icon_path: Option<&PathBuf>,
    window_icon_path: Option<&PathBuf>,
) -> Result<Icon> {
    use image::GenericImageView;

    // Try tray-specific icon first
    if let Some(path) = tray_icon_path {
        if path.exists() {
            if let Ok(img) = image::open(path) {
                let (width, height) = img.dimensions();
                let rgba = img.into_rgba8().into_raw();
                if let Ok(icon) = Icon::from_rgba(rgba, width, height) {
                    return Ok(icon);
                }
            }
        }
    }

    // Try window icon
    if let Some(path) = window_icon_path {
        if path.exists() {
            if let Ok(img) = image::open(path) {
                let (width, height) = img.dimensions();
                let rgba = img.into_rgba8().into_raw();
                if let Ok(icon) = Icon::from_rgba(rgba, width, height) {
                    return Ok(icon);
                }
            }
        }
    }

    // Fall back to default
    let img =
        image::load_from_memory(DEFAULT_ICON).map_err(|e| DesktopError::Tray(e.to_string()))?;
    let (width, height) = img.dimensions();
    let rgba = img.into_rgba8().into_raw();
    Icon::from_rgba(rgba, width, height).map_err(|e| DesktopError::Tray(e.to_string()))
}
