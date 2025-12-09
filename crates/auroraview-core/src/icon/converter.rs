//! PNG to ICO converter
//!
//! Convert PNG images to ICO format for Windows executable icons.

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use image::codecs::ico::{IcoEncoder, IcoFrame};
use image::imageops::FilterType;
use image::io::Reader as ImageReader;
use image::{ColorType, DynamicImage};

use super::DEFAULT_ICO_SIZES;
use crate::backend::WebViewError;

/// ICO conversion configuration
#[derive(Debug, Clone)]
pub struct IcoConfig {
    /// Icon sizes to include in ICO file
    pub sizes: Vec<u32>,
    /// Resize filter type
    pub filter: FilterType,
}

impl Default for IcoConfig {
    fn default() -> Self {
        Self {
            sizes: DEFAULT_ICO_SIZES.to_vec(),
            filter: FilterType::Lanczos3,
        }
    }
}

impl IcoConfig {
    /// Create config with custom sizes
    pub fn with_sizes(sizes: &[u32]) -> Self {
        Self {
            sizes: sizes.to_vec(),
            ..Default::default()
        }
    }
}

/// Convert PNG to ICO format
///
/// Creates an ICO file containing multiple icon sizes for Windows compatibility.
///
/// # Arguments
/// * `input` - Path to input PNG file
/// * `output` - Path to output ICO file
/// * `sizes` - Icon sizes to include (e.g., [16, 32, 48, 256])
///
/// # Example
/// ```no_run
/// use auroraview_core::icon::png_to_ico;
///
/// png_to_ico("logo.png", "app.ico", &[16, 32, 48, 256]).unwrap();
/// ```
pub fn png_to_ico<P: AsRef<Path>, Q: AsRef<Path>>(
    input: P,
    output: Q,
    sizes: &[u32],
) -> Result<(), WebViewError> {
    let input = input.as_ref();
    let output = output.as_ref();

    // Load source image
    let img = ImageReader::open(input)
        .map_err(|e| WebViewError::Icon(format!("Failed to open '{}': {}", input.display(), e)))?
        .decode()
        .map_err(|e| {
            WebViewError::Icon(format!("Failed to decode '{}': {}", input.display(), e))
        })?;

    png_to_ico_with_config(img, output, &IcoConfig::with_sizes(sizes))
}

/// Convert PNG bytes to ICO file
///
/// # Arguments
/// * `png_bytes` - PNG file bytes
/// * `output` - Path to output ICO file
/// * `sizes` - Icon sizes to include
pub fn png_bytes_to_ico<P: AsRef<Path>>(
    png_bytes: &[u8],
    output: P,
    sizes: &[u32],
) -> Result<(), WebViewError> {
    let img = image::load_from_memory(png_bytes)
        .map_err(|e| WebViewError::Icon(format!("Failed to decode PNG bytes: {}", e)))?;

    png_to_ico_with_config(img, output.as_ref(), &IcoConfig::with_sizes(sizes))
}

/// Convert DynamicImage to ICO with configuration
fn png_to_ico_with_config(
    img: DynamicImage,
    output: &Path,
    config: &IcoConfig,
) -> Result<(), WebViewError> {
    // Create output file
    let file = File::create(output).map_err(|e| {
        WebViewError::Icon(format!("Failed to create '{}': {}", output.display(), e))
    })?;
    let writer = BufWriter::new(file);

    // Create ICO encoder
    let encoder = IcoEncoder::new(writer);

    // Generate frames for each size
    let mut frames = Vec::new();
    for &size in &config.sizes {
        let resized = img.resize_exact(size, size, config.filter);
        let rgba = resized.to_rgba8();

        let frame = IcoFrame::as_png(rgba.as_raw(), size, size, ColorType::Rgba8)
            .map_err(|e| WebViewError::Icon(format!("Failed to create ICO frame: {}", e)))?;

        frames.push(frame);
    }

    // Encode all frames
    encoder
        .encode_images(&frames)
        .map_err(|e| WebViewError::Icon(format!("Failed to encode ICO: {}", e)))?;

    tracing::info!(
        "Created ICO file '{}' with sizes: {:?}",
        output.display(),
        config.sizes
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

    fn create_test_png() -> NamedTempFile {
        let mut file = NamedTempFile::with_suffix(".png").unwrap();
        let img = image::RgbaImage::from_fn(64, 64, |_, _| image::Rgba([255, 0, 0, 255]));
        let mut cursor = std::io::Cursor::new(Vec::new());
        img.write_to(&mut cursor, image::ImageFormat::Png).unwrap();
        file.write_all(cursor.get_ref()).unwrap();
        file.flush().unwrap();
        file
    }

    #[test]
    fn test_png_to_ico() {
        let png_file = create_test_png();
        let temp_dir = TempDir::new().unwrap();
        let ico_path = temp_dir.path().join("test.ico");

        png_to_ico(png_file.path(), &ico_path, &[16, 32]).unwrap();

        assert!(ico_path.exists());
        let metadata = std::fs::metadata(&ico_path).unwrap();
        assert!(metadata.len() > 0);
    }

    #[test]
    fn test_png_bytes_to_ico() {
        let img = image::RgbaImage::from_fn(64, 64, |_, _| image::Rgba([0, 255, 0, 255]));
        let mut cursor = std::io::Cursor::new(Vec::new());
        img.write_to(&mut cursor, image::ImageFormat::Png).unwrap();
        let png_bytes = cursor.into_inner();

        let temp_dir = TempDir::new().unwrap();
        let ico_path = temp_dir.path().join("test.ico");

        png_bytes_to_ico(&png_bytes, &ico_path, &[16, 32, 48]).unwrap();

        assert!(ico_path.exists());
    }
}
