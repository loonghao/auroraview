//! Icon conversion utilities
//!
//! This module provides automatic icon format conversion and multi-resolution ICO generation.
//!
//! ## Features
//!
//! - Automatic format detection (PNG, JPG, ICO)
//! - PNG/JPG to multi-resolution ICO conversion
//! - PNG data extraction for window icons
//! - Standard ICO sizes: 16x16, 24x24, 32x32, 48x48, 64x64, 128x128, 256x256

use crate::error::{PackError, PackResult};
use image::{DynamicImage, ImageFormat};
use std::fs;
use std::io::Cursor;
use std::path::Path;

/// Standard ICO sizes for multi-resolution icons
const ICO_SIZES: &[u32] = &[16, 24, 32, 48, 64, 128, 256];

/// Icon data with both ICO and PNG representations
#[derive(Debug, Clone)]
pub struct IconData {
    /// Multi-resolution ICO data (for Windows executable icon)
    pub ico_data: Vec<u8>,
    /// PNG data (for window title bar icon)
    pub png_data: Vec<u8>,
    /// Original format
    pub original_format: IconFormat,
}

/// Supported icon formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconFormat {
    Png,
    Jpeg,
    Ico,
}

impl IconFormat {
    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "png" => Some(Self::Png),
            "jpg" | "jpeg" => Some(Self::Jpeg),
            "ico" => Some(Self::Ico),
            _ => None,
        }
    }

    /// Detect format from magic bytes
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 4 {
            return None;
        }

        // PNG magic: 89 50 4E 47
        if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
            return Some(Self::Png);
        }

        // JPEG magic: FF D8 FF
        if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return Some(Self::Jpeg);
        }

        // ICO magic: 00 00 01 00
        if data.starts_with(&[0x00, 0x00, 0x01, 0x00]) {
            return Some(Self::Ico);
        }

        None
    }
}

/// Load and convert icon from file path
///
/// Supports PNG, JPG, and ICO formats. Automatically converts to:
/// - Multi-resolution ICO for Windows executable
/// - PNG for window title bar icon
pub fn load_icon(path: &Path) -> PackResult<IconData> {
    let data = fs::read(path).map_err(|e| {
        PackError::Config(format!(
            "Failed to read icon file {}: {}",
            path.display(),
            e
        ))
    })?;

    // Detect format
    let format = path
        .extension()
        .and_then(|e| e.to_str())
        .and_then(IconFormat::from_extension)
        .or_else(|| IconFormat::from_bytes(&data))
        .ok_or_else(|| {
            PackError::Config(format!(
                "Unknown icon format for {}: supported formats are PNG, JPG, ICO",
                path.display()
            ))
        })?;

    convert_icon_data(&data, format)
}

/// Convert icon data to both ICO and PNG formats
pub fn convert_icon_data(data: &[u8], format: IconFormat) -> PackResult<IconData> {
    match format {
        IconFormat::Ico => {
            // Already ICO, extract PNG for window icon
            let png_data = extract_png_from_ico(data)?;
            Ok(IconData {
                ico_data: data.to_vec(),
                png_data,
                original_format: format,
            })
        }
        IconFormat::Png | IconFormat::Jpeg => {
            // Convert to multi-resolution ICO
            let img = load_image(data, format)?;
            let ico_data = create_multi_resolution_ico(&img)?;
            let png_data = if format == IconFormat::Png {
                data.to_vec()
            } else {
                // Convert JPEG to PNG
                image_to_png(&img)?
            };
            Ok(IconData {
                ico_data,
                png_data,
                original_format: format,
            })
        }
    }
}

/// Load image from bytes
fn load_image(data: &[u8], format: IconFormat) -> PackResult<DynamicImage> {
    let img_format = match format {
        IconFormat::Png => ImageFormat::Png,
        IconFormat::Jpeg => ImageFormat::Jpeg,
        IconFormat::Ico => ImageFormat::Ico,
    };

    image::load_from_memory_with_format(data, img_format)
        .map_err(|e| PackError::Config(format!("Failed to load image: {}", e)))
}

/// Create multi-resolution ICO from image
fn create_multi_resolution_ico(img: &DynamicImage) -> PackResult<Vec<u8>> {
    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);

    for &size in ICO_SIZES {
        // Resize image to target size
        let resized = img.resize_exact(size, size, image::imageops::FilterType::Lanczos3);

        // Convert to RGBA
        let rgba = resized.to_rgba8();
        let (width, height) = rgba.dimensions();

        // Create ICO image entry
        let ico_image = ico::IconImage::from_rgba_data(width, height, rgba.into_raw());

        icon_dir.add_entry(ico::IconDirEntry::encode(&ico_image).map_err(|e| {
            PackError::Config(format!(
                "Failed to encode ICO entry for size {}: {}",
                size, e
            ))
        })?);
    }

    // Write ICO to buffer
    let mut buffer = Vec::new();
    icon_dir
        .write(&mut buffer)
        .map_err(|e| PackError::Config(format!("Failed to write ICO: {}", e)))?;

    tracing::info!(
        "Created multi-resolution ICO with sizes: {:?} ({} bytes)",
        ICO_SIZES,
        buffer.len()
    );

    Ok(buffer)
}

/// Extract PNG data from ICO file (uses largest image)
fn extract_png_from_ico(data: &[u8]) -> PackResult<Vec<u8>> {
    let cursor = Cursor::new(data);
    let icon_dir = ico::IconDir::read(cursor)
        .map_err(|e| PackError::Config(format!("Failed to read ICO: {}", e)))?;

    // Find the largest entry
    let entries = icon_dir.entries();
    if entries.is_empty() {
        return Err(PackError::Config("ICO file has no entries".to_string()));
    }

    // Find entry with largest dimensions
    let best_entry = entries
        .iter()
        .max_by_key(|e| e.width() * e.height())
        .unwrap();

    // Decode the entry
    let ico_image = best_entry
        .decode()
        .map_err(|e| PackError::Config(format!("Failed to decode ICO entry: {}", e)))?;

    // Convert to PNG
    let width = ico_image.width();
    let height = ico_image.height();
    let rgba_data = ico_image.rgba_data();

    let img = image::RgbaImage::from_raw(width, height, rgba_data.to_vec())
        .ok_or_else(|| PackError::Config("Failed to create image from ICO data".to_string()))?;

    let mut png_buffer = Vec::new();
    let mut cursor = Cursor::new(&mut png_buffer);
    img.write_to(&mut cursor, ImageFormat::Png)
        .map_err(|e| PackError::Config(format!("Failed to encode PNG: {}", e)))?;

    tracing::info!(
        "Extracted {}x{} PNG from ICO ({} bytes)",
        width,
        height,
        png_buffer.len()
    );

    Ok(png_buffer)
}

/// Convert DynamicImage to PNG bytes
fn image_to_png(img: &DynamicImage) -> PackResult<Vec<u8>> {
    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);
    img.write_to(&mut cursor, ImageFormat::Png)
        .map_err(|e| PackError::Config(format!("Failed to encode PNG: {}", e)))?;
    Ok(buffer)
}

/// Save ICO data to file
pub fn save_ico(data: &[u8], path: &Path) -> PackResult<()> {
    fs::write(path, data)
        .map_err(|e| PackError::Config(format!("Failed to write ICO to {}: {}", path.display(), e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_detection_from_extension() {
        assert_eq!(IconFormat::from_extension("png"), Some(IconFormat::Png));
        assert_eq!(IconFormat::from_extension("PNG"), Some(IconFormat::Png));
        assert_eq!(IconFormat::from_extension("jpg"), Some(IconFormat::Jpeg));
        assert_eq!(IconFormat::from_extension("jpeg"), Some(IconFormat::Jpeg));
        assert_eq!(IconFormat::from_extension("ico"), Some(IconFormat::Ico));
        assert_eq!(IconFormat::from_extension("bmp"), None);
    }

    #[test]
    fn test_format_detection_from_bytes() {
        // PNG magic
        assert_eq!(
            IconFormat::from_bytes(&[0x89, 0x50, 0x4E, 0x47]),
            Some(IconFormat::Png)
        );
        // JPEG magic
        assert_eq!(
            IconFormat::from_bytes(&[0xFF, 0xD8, 0xFF, 0xE0]),
            Some(IconFormat::Jpeg)
        );
        // ICO magic
        assert_eq!(
            IconFormat::from_bytes(&[0x00, 0x00, 0x01, 0x00]),
            Some(IconFormat::Ico)
        );
        // Unknown
        assert_eq!(IconFormat::from_bytes(&[0x00, 0x00, 0x00, 0x00]), None);
    }
}
