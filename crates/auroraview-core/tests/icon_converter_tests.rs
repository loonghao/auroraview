//! Tests for icon converter and compress modules

use auroraview_core::icon::{
    compress_and_resize, compress_png, png_bytes_to_ico, png_to_ico, CompressionLevel,
};
use rstest::rstest;
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};

fn create_test_png(size: u32) -> NamedTempFile {
    let mut file = NamedTempFile::with_suffix(".png").unwrap();
    let img = image::RgbaImage::from_fn(size, size, |x, y| {
        image::Rgba([(x % 256) as u8, (y % 256) as u8, 128, 255])
    });
    let mut cursor = std::io::Cursor::new(Vec::new());
    img.write_to(&mut cursor, image::ImageFormat::Png).unwrap();
    file.write_all(cursor.get_ref()).unwrap();
    file.flush().unwrap();
    file
}

fn create_simple_test_png() -> NamedTempFile {
    let mut file = NamedTempFile::with_suffix(".png").unwrap();
    let img = image::RgbaImage::from_fn(64, 64, |_, _| image::Rgba([255, 0, 0, 255]));
    let mut cursor = std::io::Cursor::new(Vec::new());
    img.write_to(&mut cursor, image::ImageFormat::Png).unwrap();
    file.write_all(cursor.get_ref()).unwrap();
    file.flush().unwrap();
    file
}

// ============================================================================
// PNG to ICO converter tests
// ============================================================================

#[rstest]
fn test_png_to_ico() {
    let png_file = create_simple_test_png();
    let temp_dir = TempDir::new().unwrap();
    let ico_path = temp_dir.path().join("test.ico");

    png_to_ico(png_file.path(), &ico_path, &[16, 32]).unwrap();

    assert!(ico_path.exists());
    let metadata = std::fs::metadata(&ico_path).unwrap();
    assert!(metadata.len() > 0);
}

#[rstest]
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

// ============================================================================
// PNG compression tests
// ============================================================================

#[rstest]
fn test_compress_png() {
    let png_file = create_test_png(256);
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("compressed.png");

    let result = compress_png(png_file.path(), &output_path, 9).unwrap();

    assert!(output_path.exists());
    assert_eq!(result.width, 256);
    assert_eq!(result.height, 256);
}

#[rstest]
fn test_compress_and_resize() {
    let png_file = create_test_png(512);
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("resized.png");

    let result = compress_and_resize(png_file.path(), &output_path, 128, 9).unwrap();

    assert!(output_path.exists());
    assert!(result.width <= 128);
    assert!(result.height <= 128);
}

#[rstest]
fn test_compression_level_conversion() {
    assert_eq!(CompressionLevel::from(1), CompressionLevel::Fast);
    assert_eq!(CompressionLevel::from(5), CompressionLevel::Default);
    assert_eq!(CompressionLevel::from(9), CompressionLevel::Best);
}
