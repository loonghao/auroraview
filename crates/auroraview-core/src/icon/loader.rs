//! Icon loading utilities
//!
//! Load PNG images and convert to RGBA data for use with window icons.

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use image::codecs::png::PngDecoder;
use image::io::Reader as ImageReader;
use image::{DynamicImage, ImageDecoder};

use crate::backend::WebViewError;

/// Icon data containing RGBA pixels and dimensions
#[derive(Debug, Clone)]
pub struct IconData {
    /// RGBA pixel data (4 bytes per pixel)
    pub rgba: Vec<u8>,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
}

impl IconData {
    /// Create new IconData from RGBA bytes and dimensions
    pub fn new(rgba: Vec<u8>, width: u32, height: u32) -> Self {
        Self {
            rgba,
            width,
            height,
        }
    }

    /// Load from PNG file
    pub fn from_png<P: AsRef<Path>>(path: P) -> Result<Self, WebViewError> {
        load_icon_rgba(path)
    }

    /// Load from PNG bytes
    pub fn from_png_bytes(bytes: &[u8]) -> Result<Self, WebViewError> {
        load_icon_rgba_from_bytes(bytes)
    }

    /// Resize to target size (maintains aspect ratio, centers in square)
    pub fn resize(&self, target_size: u32) -> Result<Self, WebViewError> {
        let img = DynamicImage::ImageRgba8(
            image::RgbaImage::from_raw(self.width, self.height, self.rgba.clone())
                .ok_or_else(|| WebViewError::Icon("Invalid RGBA data".into()))?,
        );

        let resized = img.resize(
            target_size,
            target_size,
            image::imageops::FilterType::Lanczos3,
        );

        let rgba8 = resized.to_rgba8();
        Ok(Self {
            rgba: rgba8.into_raw(),
            width: target_size,
            height: target_size,
        })
    }
}

/// Load PNG image and return RGBA data for window icon
///
/// # Arguments
/// * `path` - Path to PNG file
///
/// # Returns
/// * `IconData` containing RGBA bytes and dimensions
///
/// # Example
/// ```no_run
/// use auroraview_core::icon::load_icon_rgba;
///
/// let icon = load_icon_rgba("icon.png").unwrap();
/// println!("Icon size: {}x{}", icon.width, icon.height);
/// ```
pub fn load_icon_rgba<P: AsRef<Path>>(path: P) -> Result<IconData, WebViewError> {
    let path = path.as_ref();

    let file = File::open(path).map_err(|e| {
        WebViewError::Icon(format!(
            "Failed to open icon file '{}': {}",
            path.display(),
            e
        ))
    })?;

    let reader = BufReader::new(file);
    let decoder = PngDecoder::new(reader).map_err(|e| {
        WebViewError::Icon(format!("Failed to decode PNG '{}': {}", path.display(), e))
    })?;

    let (width, height) = decoder.dimensions();

    let img = ImageReader::open(path)
        .map_err(|e| WebViewError::Icon(format!("Failed to read image: {}", e)))?
        .decode()
        .map_err(|e| WebViewError::Icon(format!("Failed to decode image: {}", e)))?;

    let rgba = img.to_rgba8().into_raw();

    Ok(IconData {
        rgba,
        width,
        height,
    })
}

/// Load PNG from bytes and return RGBA data
///
/// # Arguments
/// * `bytes` - PNG file bytes
///
/// # Returns
/// * `IconData` containing RGBA bytes and dimensions
pub fn load_icon_rgba_from_bytes(bytes: &[u8]) -> Result<IconData, WebViewError> {
    let img = image::load_from_memory(bytes)
        .map_err(|e| WebViewError::Icon(format!("Failed to decode PNG from bytes: {}", e)))?;

    let rgba8 = img.to_rgba8();
    let (width, height) = rgba8.dimensions();

    Ok(IconData {
        rgba: rgba8.into_raw(),
        width,
        height,
    })
}
