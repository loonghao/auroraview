//! PNG compression utilities
//!
//! Compress PNG images with configurable quality levels.

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::io::Reader as ImageReader;
use image::{ColorType, ImageEncoder};

use crate::backend::WebViewError;

/// PNG compression level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionLevel {
    /// Fast compression (level 1-3)
    Fast,
    /// Default compression (level 4-6)
    Default,
    /// Best compression (level 7-9)
    Best,
}

impl From<u8> for CompressionLevel {
    fn from(level: u8) -> Self {
        match level {
            0..=3 => CompressionLevel::Fast,
            4..=6 => CompressionLevel::Default,
            _ => CompressionLevel::Best,
        }
    }
}

impl From<CompressionLevel> for CompressionType {
    fn from(level: CompressionLevel) -> Self {
        match level {
            CompressionLevel::Fast => CompressionType::Fast,
            CompressionLevel::Default => CompressionType::Default,
            CompressionLevel::Best => CompressionType::Best,
        }
    }
}

/// Compress PNG image
///
/// # Arguments
/// * `input` - Path to input PNG file
/// * `output` - Path to output PNG file (can be same as input to overwrite)
/// * `level` - Compression level (1-9, higher = smaller file but slower)
///
/// # Example
/// ```no_run
/// use auroraview_core::icon::compress_png;
///
/// compress_png("large.png", "small.png", 9).unwrap();
/// ```
pub fn compress_png<P: AsRef<Path>, Q: AsRef<Path>>(
    input: P,
    output: Q,
    level: u8,
) -> Result<CompressionResult, WebViewError> {
    let input = input.as_ref();
    let output = output.as_ref();

    // Get original file size
    let original_size = std::fs::metadata(input).map(|m| m.len()).unwrap_or(0);

    // Load image
    let img = ImageReader::open(input)
        .map_err(|e| WebViewError::Icon(format!("Failed to open '{}': {}", input.display(), e)))?
        .decode()
        .map_err(|e| {
            WebViewError::Icon(format!("Failed to decode '{}': {}", input.display(), e))
        })?;

    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    // Create output file
    let file = File::create(output).map_err(|e| {
        WebViewError::Icon(format!("Failed to create '{}': {}", output.display(), e))
    })?;
    let writer = BufWriter::new(file);

    // Create encoder with compression settings
    let compression_level: CompressionLevel = level.into();
    let encoder =
        PngEncoder::new_with_quality(writer, compression_level.into(), FilterType::Adaptive);

    // Encode image
    encoder
        .write_image(&rgba, width, height, ColorType::Rgba8)
        .map_err(|e| WebViewError::Icon(format!("Failed to encode PNG: {}", e)))?;

    // Get compressed file size
    let compressed_size = std::fs::metadata(output).map(|m| m.len()).unwrap_or(0);

    let result = CompressionResult {
        original_size,
        compressed_size,
        width,
        height,
    };

    tracing::info!(
        "Compressed '{}' -> '{}': {} -> {} ({:.1}% reduction)",
        input.display(),
        output.display(),
        format_size(original_size),
        format_size(compressed_size),
        result.reduction_percent()
    );

    Ok(result)
}

/// Compress PNG and resize
///
/// # Arguments
/// * `input` - Path to input PNG file
/// * `output` - Path to output PNG file
/// * `max_size` - Maximum width/height (maintains aspect ratio)
/// * `level` - Compression level (1-9)
pub fn compress_and_resize<P: AsRef<Path>, Q: AsRef<Path>>(
    input: P,
    output: Q,
    max_size: u32,
    level: u8,
) -> Result<CompressionResult, WebViewError> {
    let input = input.as_ref();
    let output = output.as_ref();

    let original_size = std::fs::metadata(input).map(|m| m.len()).unwrap_or(0);

    // Load and resize image
    let img = ImageReader::open(input)
        .map_err(|e| WebViewError::Icon(format!("Failed to open '{}': {}", input.display(), e)))?
        .decode()
        .map_err(|e| {
            WebViewError::Icon(format!("Failed to decode '{}': {}", input.display(), e))
        })?;

    let resized = img.resize(max_size, max_size, image::imageops::FilterType::Lanczos3);

    let rgba = resized.to_rgba8();
    let (width, height) = rgba.dimensions();

    // Create output file
    let file = File::create(output).map_err(|e| {
        WebViewError::Icon(format!("Failed to create '{}': {}", output.display(), e))
    })?;
    let writer = BufWriter::new(file);

    let compression_level: CompressionLevel = level.into();
    let encoder =
        PngEncoder::new_with_quality(writer, compression_level.into(), FilterType::Adaptive);

    encoder
        .write_image(&rgba, width, height, ColorType::Rgba8)
        .map_err(|e| WebViewError::Icon(format!("Failed to encode PNG: {}", e)))?;

    let compressed_size = std::fs::metadata(output).map(|m| m.len()).unwrap_or(0);

    let result = CompressionResult {
        original_size,
        compressed_size,
        width,
        height,
    };

    tracing::info!(
        "Compressed and resized '{}' -> '{}': {}x{}, {} -> {} ({:.1}% reduction)",
        input.display(),
        output.display(),
        width,
        height,
        format_size(original_size),
        format_size(compressed_size),
        result.reduction_percent()
    );

    Ok(result)
}

/// Result of PNG compression
#[derive(Debug, Clone)]
pub struct CompressionResult {
    /// Original file size in bytes
    pub original_size: u64,
    /// Compressed file size in bytes
    pub compressed_size: u64,
    /// Image width
    pub width: u32,
    /// Image height
    pub height: u32,
}

impl CompressionResult {
    /// Calculate size reduction percentage
    pub fn reduction_percent(&self) -> f64 {
        if self.original_size == 0 {
            return 0.0;
        }
        (1.0 - (self.compressed_size as f64 / self.original_size as f64)) * 100.0
    }
}

/// Format file size for display
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;

    if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
