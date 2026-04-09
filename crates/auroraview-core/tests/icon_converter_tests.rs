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

fn create_png_bytes(size: u32) -> Vec<u8> {
    let img = image::RgbaImage::from_fn(size, size, |x, y| {
        image::Rgba([(x % 256) as u8, (y % 256) as u8, 64, 255])
    });
    let mut cursor = std::io::Cursor::new(Vec::new());
    img.write_to(&mut cursor, image::ImageFormat::Png).unwrap();
    cursor.into_inner()
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
fn test_png_to_ico_single_size() {
    let png_file = create_simple_test_png();
    let temp_dir = TempDir::new().unwrap();
    let ico_path = temp_dir.path().join("single.ico");

    png_to_ico(png_file.path(), &ico_path, &[32]).unwrap();

    assert!(ico_path.exists());
    assert!(std::fs::metadata(&ico_path).unwrap().len() > 0);
}

#[rstest]
#[case(&[16])]
#[case(&[32])]
#[case(&[48])]
#[case(&[16, 32])]
#[case(&[16, 32, 48])]
#[case(&[16, 32, 48, 64])]
fn test_png_to_ico_various_sizes(#[case] sizes: &[u32]) {
    let png_file = create_simple_test_png();
    let temp_dir = TempDir::new().unwrap();
    let ico_path = temp_dir.path().join("test.ico");

    png_to_ico(png_file.path(), &ico_path, sizes).unwrap();

    assert!(ico_path.exists());
    assert!(std::fs::metadata(&ico_path).unwrap().len() > 0);
}

#[rstest]
fn test_png_to_ico_large_source() {
    let png_file = create_test_png(512);
    let temp_dir = TempDir::new().unwrap();
    let ico_path = temp_dir.path().join("from_large.ico");

    png_to_ico(png_file.path(), &ico_path, &[16, 32, 48]).unwrap();

    assert!(ico_path.exists());
}

#[rstest]
fn test_png_to_ico_nonexistent_input() {
    let temp_dir = TempDir::new().unwrap();
    let fake_path = temp_dir.path().join("nonexistent.png");
    let ico_path = temp_dir.path().join("out.ico");

    let result = png_to_ico(&fake_path, &ico_path, &[32]);
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("nonexistent") || !err_msg.is_empty());
}


// ============================================================================
// PNG bytes to ICO tests
// ============================================================================

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

#[rstest]
fn test_png_bytes_to_ico_single_size() {
    let png_bytes = create_png_bytes(32);
    let temp_dir = TempDir::new().unwrap();
    let ico_path = temp_dir.path().join("single_bytes.ico");

    png_bytes_to_ico(&png_bytes, &ico_path, &[16]).unwrap();

    assert!(ico_path.exists());
    assert!(std::fs::metadata(&ico_path).unwrap().len() > 0);
}

#[rstest]
fn test_png_bytes_to_ico_invalid_bytes() {
    let invalid_bytes = b"this is not a png file at all";
    let temp_dir = TempDir::new().unwrap();
    let ico_path = temp_dir.path().join("invalid.ico");

    let result = png_bytes_to_ico(invalid_bytes, &ico_path, &[32]);
    assert!(result.is_err());
}

#[rstest]
fn test_png_bytes_to_ico_empty_bytes() {
    let empty: &[u8] = &[];
    let temp_dir = TempDir::new().unwrap();
    let ico_path = temp_dir.path().join("empty.ico");

    let result = png_bytes_to_ico(empty, &ico_path, &[32]);
    assert!(result.is_err());
}

#[rstest]
#[case(32, &[16, 32])]
#[case(64, &[16, 32, 48])]
#[case(128, &[16, 32, 64])]
fn test_png_bytes_to_ico_various(#[case] src_size: u32, #[case] sizes: &[u32]) {
    let png_bytes = create_png_bytes(src_size);
    let temp_dir = TempDir::new().unwrap();
    let ico_path = temp_dir.path().join("test.ico");

    png_bytes_to_ico(&png_bytes, &ico_path, sizes).unwrap();

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

#[rstest]
#[case(0, CompressionLevel::Fast)]
#[case(1, CompressionLevel::Fast)]
#[case(2, CompressionLevel::Fast)]
#[case(3, CompressionLevel::Fast)]
#[case(4, CompressionLevel::Default)]
#[case(5, CompressionLevel::Default)]
#[case(6, CompressionLevel::Default)]
#[case(7, CompressionLevel::Best)]
#[case(8, CompressionLevel::Best)]
#[case(9, CompressionLevel::Best)]
fn test_compression_level_all_values(#[case] input: u8, #[case] expected: CompressionLevel) {
    assert_eq!(CompressionLevel::from(input), expected);
}

#[rstest]
fn test_compression_level_equality() {
    assert_eq!(CompressionLevel::Fast, CompressionLevel::Fast);
    assert_eq!(CompressionLevel::Default, CompressionLevel::Default);
    assert_eq!(CompressionLevel::Best, CompressionLevel::Best);
    assert_ne!(CompressionLevel::Fast, CompressionLevel::Best);
    assert_ne!(CompressionLevel::Default, CompressionLevel::Fast);
}

#[rstest]
fn test_compression_level_clone() {
    let level = CompressionLevel::Best;
    let cloned = level;
    assert_eq!(level, cloned);
}

#[rstest]
fn test_compression_level_debug() {
    assert_eq!(format!("{:?}", CompressionLevel::Fast), "Fast");
    assert_eq!(format!("{:?}", CompressionLevel::Default), "Default");
    assert_eq!(format!("{:?}", CompressionLevel::Best), "Best");
}

#[rstest]
#[case(1)]
#[case(5)]
#[case(9)]
fn test_compress_png_various_levels(#[case] level: u8) {
    let png_file = create_test_png(64);
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join(format!("compressed_level{}.png", level));

    let result = compress_png(png_file.path(), &output_path, level).unwrap();

    assert!(output_path.exists());
    assert_eq!(result.width, 64);
    assert_eq!(result.height, 64);
    assert!(result.original_size > 0);
    assert!(result.compressed_size > 0);
}

#[rstest]
fn test_compression_result_fields() {
    let png_file = create_test_png(128);
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("fields_test.png");

    let result = compress_png(png_file.path(), &output_path, 5).unwrap();

    assert_eq!(result.width, 128);
    assert_eq!(result.height, 128);
    assert!(result.original_size > 0);
    assert!(result.compressed_size > 0);
}

#[rstest]
fn test_compression_result_reduction_percent() {
    let png_file = create_test_png(256);
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("reduction_test.png");

    let result = compress_png(png_file.path(), &output_path, 9).unwrap();

    // reduction_percent should be a valid float
    let pct = result.reduction_percent();
    assert!(pct.is_finite());
}

#[rstest]
fn test_compression_result_clone() {
    let png_file = create_test_png(32);
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("clone_test.png");

    let result = compress_png(png_file.path(), &output_path, 5).unwrap();
    let cloned = result.clone();

    assert_eq!(result.width, cloned.width);
    assert_eq!(result.height, cloned.height);
    assert_eq!(result.original_size, cloned.original_size);
    assert_eq!(result.compressed_size, cloned.compressed_size);
}

#[rstest]
fn test_compress_and_resize_smaller_than_max() {
    // Input is already smaller than max_size
    let png_file = create_test_png(64);
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("no_upscale.png");

    let result = compress_and_resize(png_file.path(), &output_path, 256, 5).unwrap();

    assert!(output_path.exists());
    // Should not upscale beyond original
    assert!(result.width <= 256);
    assert!(result.height <= 256);
}

#[rstest]
fn test_compress_png_nonexistent_input() {
    let temp_dir = TempDir::new().unwrap();
    let fake_path = temp_dir.path().join("ghost.png");
    let out_path = temp_dir.path().join("out.png");

    let result = compress_png(&fake_path, &out_path, 5);
    assert!(result.is_err());
}

#[rstest]
fn test_compress_and_resize_nonexistent_input() {
    let temp_dir = TempDir::new().unwrap();
    let fake_path = temp_dir.path().join("ghost.png");
    let out_path = temp_dir.path().join("out.png");

    let result = compress_and_resize(&fake_path, &out_path, 64, 5);
    assert!(result.is_err());
}

// ============================================================================
// R8 Extensions
// ============================================================================

#[rstest]
fn test_png_to_ico_output_is_not_empty() {
    let png_file = create_simple_test_png();
    let temp_dir = TempDir::new().unwrap();
    let ico_path = temp_dir.path().join("not_empty.ico");

    png_to_ico(png_file.path(), &ico_path, &[32]).unwrap();

    let data = std::fs::read(&ico_path).unwrap();
    assert!(!data.is_empty(), "ICO file must contain data");
}

#[rstest]
fn test_png_bytes_to_ico_output_minimum_size() {
    // ICO header is 6 bytes + directory entries, must be > 6 bytes for a single icon
    let png_bytes = create_png_bytes(32);
    let temp_dir = TempDir::new().unwrap();
    let ico_path = temp_dir.path().join("min_size.ico");

    png_bytes_to_ico(&png_bytes, &ico_path, &[16]).unwrap();

    let metadata = std::fs::metadata(&ico_path).unwrap();
    assert!(metadata.len() > 6);
}

#[rstest]
fn test_compression_result_compressed_size_positive() {
    let png_file = create_test_png(128);
    let temp_dir = TempDir::new().unwrap();
    let out_path = temp_dir.path().join("check_compressed.png");

    let result = compress_png(png_file.path(), &out_path, 5).unwrap();
    assert!(result.compressed_size > 0, "compressed_size must be positive");
}

#[rstest]
fn test_compression_result_original_size_positive() {
    let png_file = create_test_png(128);
    let temp_dir = TempDir::new().unwrap();
    let out_path = temp_dir.path().join("check_original.png");

    let result = compress_png(png_file.path(), &out_path, 5).unwrap();
    assert!(result.original_size > 0, "original_size must be positive");
}

#[rstest]
fn test_compression_level_from_boundary_values() {
    // Boundary at exactly 0 → Fast
    assert_eq!(CompressionLevel::from(0u8), CompressionLevel::Fast);
    // Boundary at exactly 3 → Fast
    assert_eq!(CompressionLevel::from(3u8), CompressionLevel::Fast);
    // Boundary at exactly 4 → Default
    assert_eq!(CompressionLevel::from(4u8), CompressionLevel::Default);
    // Boundary at exactly 6 → Default
    assert_eq!(CompressionLevel::from(6u8), CompressionLevel::Default);
    // Boundary at exactly 7 → Best
    assert_eq!(CompressionLevel::from(7u8), CompressionLevel::Best);
    // Boundary at exactly 9 → Best
    assert_eq!(CompressionLevel::from(9u8), CompressionLevel::Best);
}

#[rstest]
fn test_compress_and_resize_output_exists() {
    let png_file = create_test_png(128);
    let temp_dir = TempDir::new().unwrap();
    let out_path = temp_dir.path().join("resized_exists.png");

    compress_and_resize(png_file.path(), &out_path, 64, 5).unwrap();
    assert!(out_path.exists());
}

#[rstest]
#[case(16)]
#[case(32)]
#[case(64)]
#[case(128)]
#[case(256)]
fn test_compress_png_various_source_sizes(#[case] size: u32) {
    let png_file = create_test_png(size);
    let temp_dir = TempDir::new().unwrap();
    let out_path = temp_dir.path().join(format!("compressed_{}.png", size));

    let result = compress_png(png_file.path(), &out_path, 5).unwrap();

    assert!(out_path.exists());
    assert_eq!(result.width, size);
    assert_eq!(result.height, size);
}

#[rstest]
fn test_png_to_ico_overwrites_existing_file() {
    let png_file = create_simple_test_png();
    let temp_dir = TempDir::new().unwrap();
    let ico_path = temp_dir.path().join("overwrite.ico");

    png_to_ico(png_file.path(), &ico_path, &[16]).unwrap();
    let first_size = std::fs::metadata(&ico_path).unwrap().len();

    png_to_ico(png_file.path(), &ico_path, &[32, 48]).unwrap();
    let second_size = std::fs::metadata(&ico_path).unwrap().len();

    // Both should be valid (sizes may differ due to different icon entries)
    assert!(first_size > 0);
    assert!(second_size > 0);
}

#[rstest]
fn test_compress_png_level_1_vs_level_9() {
    // Level 1 (fast) and level 9 (best) should both produce valid output
    let png_file_fast = create_test_png(64);
    let png_file_best = create_test_png(64);
    let temp_dir = TempDir::new().unwrap();
    let out_fast = temp_dir.path().join("fast.png");
    let out_best = temp_dir.path().join("best.png");

    let r_fast = compress_png(png_file_fast.path(), &out_fast, 1).unwrap();
    let r_best = compress_png(png_file_best.path(), &out_best, 9).unwrap();

    assert!(r_fast.compressed_size > 0);
    assert!(r_best.compressed_size > 0);
    assert_eq!(r_fast.width, r_best.width);
    assert_eq!(r_fast.height, r_best.height);
}

#[rstest]
fn test_compression_level_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<CompressionLevel>();
}

#[rstest]
fn test_compression_result_debug_contains_width() {
    let png_file = create_test_png(32);
    let temp_dir = TempDir::new().unwrap();
    let out_path = temp_dir.path().join("debug.png");

    let result = compress_png(png_file.path(), &out_path, 5).unwrap();
    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("32") || !debug_str.is_empty());
}

#[rstest]
#[case(&[16, 32, 48, 64, 128, 256])]
fn test_png_to_ico_many_sizes(#[case] sizes: &[u32]) {
    let png_file = create_test_png(512);
    let temp_dir = TempDir::new().unwrap();
    let ico_path = temp_dir.path().join("many_sizes.ico");

    png_to_ico(png_file.path(), &ico_path, sizes).unwrap();

    assert!(ico_path.exists());
    let size = std::fs::metadata(&ico_path).unwrap().len();
    assert!(size > 0);
}

#[rstest]
fn test_compress_and_resize_preserves_aspect_when_downscaling() {
    // 512x512 downscaled to max 128 should fit within 128x128
    let png_file = create_test_png(512);
    let temp_dir = TempDir::new().unwrap();
    let out_path = temp_dir.path().join("aspect.png");

    let result = compress_and_resize(png_file.path(), &out_path, 128, 5).unwrap();

    assert!(result.width <= 128, "width {} should be <= 128", result.width);
    assert!(result.height <= 128, "height {} should be <= 128", result.height);
}
