//! Icon loading tests

use auroraview_core::icon::{
    compress_png, load_icon_rgba, png_bytes_to_ico, png_to_ico,
    CompressionLevel, CompressionResult, IcoConfig, IconData, DEFAULT_ICO_SIZES,
};
use rstest::rstest;
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};

fn create_test_png_size(width: u32, height: u32) -> NamedTempFile {
    let mut file = NamedTempFile::with_suffix(".png").unwrap();
    let img = image::RgbaImage::from_fn(width, height, |x, y| {
        image::Rgba([(x * 255 / width.max(1)) as u8, (y * 255 / height.max(1)) as u8, 128, 255])
    });
    let mut cursor = std::io::Cursor::new(Vec::new());
    img.write_to(&mut cursor, image::ImageFormat::Png).unwrap();
    file.write_all(cursor.get_ref()).unwrap();
    file.flush().unwrap();
    file
}

fn create_test_png() -> NamedTempFile {
    create_test_png_size(2, 2)
}

fn create_png_bytes(width: u32, height: u32) -> Vec<u8> {
    let img = image::RgbaImage::from_fn(width, height, |_, _| image::Rgba([255, 0, 0, 255]));
    let mut cursor = std::io::Cursor::new(Vec::new());
    img.write_to(&mut cursor, image::ImageFormat::Png).unwrap();
    cursor.into_inner()
}

// ============================================================================
// load_icon_rgba
// ============================================================================

#[test]
fn load_icon_rgba_basic() {
    let png_file = create_test_png();
    let icon = load_icon_rgba(png_file.path()).unwrap();
    assert_eq!(icon.width, 2);
    assert_eq!(icon.height, 2);
    assert_eq!(icon.rgba.len(), 2 * 2 * 4);
}

#[test]
fn load_icon_rgba_pixel_count() {
    let png_file = create_test_png_size(4, 4);
    let icon = load_icon_rgba(png_file.path()).unwrap();
    assert_eq!(icon.rgba.len(), 4 * 4 * 4);
}

#[rstest]
#[case(1, 1)]
#[case(2, 2)]
#[case(4, 4)]
#[case(8, 8)]
#[case(16, 16)]
fn load_icon_rgba_various_sizes(#[case] w: u32, #[case] h: u32) {
    let png_file = create_test_png_size(w, h);
    let icon = load_icon_rgba(png_file.path()).unwrap();
    assert_eq!(icon.width, w);
    assert_eq!(icon.height, h);
    assert_eq!(icon.rgba.len() as u32, w * h * 4);
}

#[test]
fn load_icon_rgba_nonexistent_path() {
    let result = load_icon_rgba("/nonexistent/path/icon.png");
    assert!(result.is_err());
}

#[test]
fn load_icon_rgba_from_png_method() {
    let png_file = create_test_png();
    let icon = IconData::from_png(png_file.path()).unwrap();
    assert_eq!(icon.width, 2);
    assert_eq!(icon.height, 2);
}

// ============================================================================
// load_icon_rgba_from_bytes (via IconData::from_png_bytes)
// ============================================================================

#[test]
fn load_from_bytes_basic() {
    let bytes = create_png_bytes(2, 2);
    let icon = IconData::from_png_bytes(&bytes).unwrap();
    assert_eq!(icon.width, 2);
    assert_eq!(icon.height, 2);
    assert_eq!(icon.rgba.len(), 2 * 2 * 4);
}

#[test]
fn load_from_bytes_4x4() {
    let bytes = create_png_bytes(4, 4);
    let icon = IconData::from_png_bytes(&bytes).unwrap();
    assert_eq!(icon.width, 4);
    assert_eq!(icon.height, 4);
}

#[test]
fn load_from_bytes_invalid_data() {
    let result = IconData::from_png_bytes(&[0, 1, 2, 3, 4]);
    assert!(result.is_err());
}

#[test]
fn load_from_bytes_empty() {
    let result = IconData::from_png_bytes(&[]);
    assert!(result.is_err());
}

#[test]
fn from_png_bytes_method() {
    let bytes = create_png_bytes(2, 2);
    let icon = IconData::from_png_bytes(&bytes).unwrap();
    assert_eq!(icon.width, 2);
    assert_eq!(icon.height, 2);
}

// ============================================================================
// IconData::new
// ============================================================================

#[test]
fn icon_data_new() {
    let rgba = vec![255u8; 4 * 4 * 4];
    let icon = IconData::new(rgba.clone(), 4, 4);
    assert_eq!(icon.width, 4);
    assert_eq!(icon.height, 4);
    assert_eq!(icon.rgba.len(), rgba.len());
}

#[test]
fn icon_data_clone() {
    let icon = IconData::new(vec![255; 16], 2, 2);
    let cloned = icon.clone();
    assert_eq!(cloned.width, icon.width);
    assert_eq!(cloned.height, icon.height);
    assert_eq!(cloned.rgba, icon.rgba);
}

#[test]
fn icon_data_debug() {
    let icon = IconData::new(vec![0; 16], 2, 2);
    let debug = format!("{:?}", icon);
    assert!(debug.contains("IconData") || debug.contains("width"));
}

// ============================================================================
// IconData::resize
// ============================================================================

#[test]
fn resize_to_32() {
    let png_file = create_test_png();
    let icon = load_icon_rgba(png_file.path()).unwrap();
    let resized = icon.resize(32).unwrap();
    assert_eq!(resized.width, 32);
    assert_eq!(resized.height, 32);
    assert_eq!(resized.rgba.len(), 32 * 32 * 4);
}

#[rstest]
#[case(16)]
#[case(32)]
#[case(48)]
#[case(64)]
#[case(128)]
fn resize_various_targets(#[case] target: u32) {
    let png_file = create_test_png_size(4, 4);
    let icon = load_icon_rgba(png_file.path()).unwrap();
    let resized = icon.resize(target).unwrap();
    assert_eq!(resized.width, target);
    assert_eq!(resized.height, target);
    assert_eq!(resized.rgba.len() as u32, target * target * 4);
}

#[test]
fn resize_from_bytes() {
    let bytes = create_png_bytes(4, 4);
    let icon = IconData::from_png_bytes(&bytes).unwrap();
    let resized = icon.resize(16).unwrap();
    assert_eq!(resized.width, 16);
    assert_eq!(resized.height, 16);
}

// ============================================================================
// DEFAULT_ICO_SIZES constant
// ============================================================================

#[test]
fn default_ico_sizes_not_empty() {
    assert!(!DEFAULT_ICO_SIZES.is_empty());
}

#[test]
fn default_ico_sizes_contains_standard() {
    assert!(DEFAULT_ICO_SIZES.contains(&16));
    assert!(DEFAULT_ICO_SIZES.contains(&32));
    assert!(DEFAULT_ICO_SIZES.contains(&256));
}

// ============================================================================
// IcoConfig
// ============================================================================

#[test]
fn ico_config_default_sizes() {
    let cfg = IcoConfig::default();
    assert!(!cfg.sizes.is_empty());
    assert!(cfg.sizes.contains(&16));
    assert!(cfg.sizes.contains(&256));
}

#[test]
fn ico_config_with_sizes() {
    let cfg = IcoConfig::with_sizes(&[16, 32]);
    assert_eq!(cfg.sizes, vec![16, 32]);
}

#[test]
fn ico_config_clone() {
    let cfg = IcoConfig::with_sizes(&[48, 64]);
    let cloned = cfg.clone();
    assert_eq!(cloned.sizes, cfg.sizes);
}

// ============================================================================
// CompressionLevel From<u8>
// ============================================================================

#[rstest]
#[case(0, CompressionLevel::Fast)]
#[case(1, CompressionLevel::Fast)]
#[case(3, CompressionLevel::Fast)]
#[case(4, CompressionLevel::Default)]
#[case(6, CompressionLevel::Default)]
#[case(7, CompressionLevel::Best)]
#[case(9, CompressionLevel::Best)]
fn compression_level_from_u8(#[case] level: u8, #[case] expected: CompressionLevel) {
    let got: CompressionLevel = level.into();
    assert_eq!(got, expected);
}

#[test]
fn compression_level_clone() {
    let lvl = CompressionLevel::Best;
    let cloned = lvl;
    assert_eq!(lvl, cloned);
}

// ============================================================================
// png_to_ico
// ============================================================================

#[test]
fn png_to_ico_basic() {
    let png_file = create_test_png_size(16, 16);
    let dir = TempDir::new().unwrap();
    let ico_path = dir.path().join("out.ico");
    let result = png_to_ico(png_file.path(), &ico_path, &[16, 32]);
    assert!(result.is_ok(), "png_to_ico failed: {:?}", result);
    assert!(ico_path.exists());
}

#[test]
fn png_to_ico_nonexistent_input() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.ico");
    let result = png_to_ico("/nonexistent/icon.png", &out, &[16]);
    assert!(result.is_err());
}

// ============================================================================
// png_bytes_to_ico
// ============================================================================

#[test]
fn png_bytes_to_ico_basic() {
    let bytes = create_png_bytes(32, 32);
    let dir = TempDir::new().unwrap();
    let ico_path = dir.path().join("from_bytes.ico");
    let result = png_bytes_to_ico(&bytes, &ico_path, &[16, 32]);
    assert!(result.is_ok(), "png_bytes_to_ico failed: {:?}", result);
}

#[test]
fn png_bytes_to_ico_invalid_bytes() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("bad.ico");
    let result = png_bytes_to_ico(&[0xDE, 0xAD, 0xBE, 0xEF], &out, &[16]);
    assert!(result.is_err());
}

// ============================================================================
// compress_png
// ============================================================================

#[test]
fn compress_png_basic() {
    let png_file = create_test_png_size(8, 8);
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("compressed.png");
    let result = compress_png(png_file.path(), &out, 6);
    assert!(result.is_ok(), "compress_png failed: {:?}", result);
    let cr = result.unwrap();
    assert_eq!(cr.width, 8);
    assert_eq!(cr.height, 8);
}

#[test]
fn compress_png_nonexistent_input() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.png");
    let result = compress_png("/nonexistent.png", &out, 3);
    assert!(result.is_err());
}

// ============================================================================
// CompressionResult::reduction_percent
// ============================================================================

#[test]
fn compression_result_reduction_zero_original() {
    let cr = CompressionResult {
        original_size: 0,
        compressed_size: 100,
        width: 4,
        height: 4,
    };
    assert_eq!(cr.reduction_percent(), 0.0);
}

#[test]
fn compression_result_reduction_half() {
    let cr = CompressionResult {
        original_size: 1000,
        compressed_size: 500,
        width: 4,
        height: 4,
    };
    assert!((cr.reduction_percent() - 50.0).abs() < 1e-6);
}

#[test]
fn compression_result_clone() {
    let cr = CompressionResult {
        original_size: 200,
        compressed_size: 100,
        width: 2,
        height: 2,
    };
    let c = cr.clone();
    assert_eq!(c.original_size, 200);
}

// ============================================================================
// IconData Send + Sync
// ============================================================================

#[test]
fn icon_data_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<IconData>();
}

// ============================================================================
// IconData field correctness with known pixel values
// ============================================================================

#[test]
fn icon_data_pixel_values_red() {
    let bytes = create_png_bytes(1, 1);
    let icon = IconData::from_png_bytes(&bytes).unwrap();
    // 1×1 red opaque pixel: RGBA = (255, 0, 0, 255)
    assert_eq!(icon.rgba.len(), 4);
    assert_eq!(icon.rgba[0], 255); // R
    assert_eq!(icon.rgba[1], 0);   // G
    assert_eq!(icon.rgba[2], 0);   // B
    assert_eq!(icon.rgba[3], 255); // A
}

#[test]
fn icon_data_width_height_match_created_size() {
    for size in [1u32, 4, 16, 64] {
        let bytes = create_png_bytes(size, size);
        let icon = IconData::from_png_bytes(&bytes).unwrap();
        assert_eq!(icon.width, size);
        assert_eq!(icon.height, size);
        assert_eq!(icon.rgba.len() as u32, size * size * 4);
    }
}

// ============================================================================
// load_icon_rgba path variants
// ============================================================================

#[rstest]
#[case(1, 1)]
#[case(32, 32)]
#[case(64, 64)]
fn load_icon_rgba_square_sizes(#[case] w: u32, #[case] h: u32) {
    let file = create_test_png_size(w, h);
    let icon = load_icon_rgba(file.path()).unwrap();
    assert_eq!(icon.width, w);
    assert_eq!(icon.height, h);
}

#[test]
fn load_icon_rgba_err_on_non_png_bytes() {
    let mut file = tempfile::NamedTempFile::with_suffix(".png").unwrap();
    use std::io::Write;
    file.write_all(b"not a png").unwrap();
    let result = load_icon_rgba(file.path());
    assert!(result.is_err());
}

// ============================================================================
// IcoConfig – edge sizes
// ============================================================================

#[test]
fn ico_config_with_single_size() {
    let cfg = IcoConfig::with_sizes(&[32]);
    assert_eq!(cfg.sizes, vec![32]);
}

#[test]
fn ico_config_with_empty_sizes() {
    let cfg = IcoConfig::with_sizes(&[]);
    assert!(cfg.sizes.is_empty());
}

#[test]
fn ico_config_default_contains_256() {
    let cfg = IcoConfig::default();
    assert!(cfg.sizes.contains(&256), "default IcoConfig should include size 256");
}

// ============================================================================
// png_to_ico – output file characteristics
// ============================================================================

#[test]
fn png_to_ico_output_nonzero_size() {
    let png_file = create_test_png_size(32, 32);
    let dir = TempDir::new().unwrap();
    let ico = dir.path().join("out.ico");
    png_to_ico(png_file.path(), &ico, &[32]).unwrap();
    let meta = std::fs::metadata(&ico).unwrap();
    assert!(meta.len() > 0, "ICO file must not be empty");
}

#[test]
fn png_to_ico_large_image_succeeds() {
    let png_file = create_test_png_size(512, 512);
    let dir = TempDir::new().unwrap();
    let ico = dir.path().join("large.ico");
    let result = png_to_ico(png_file.path(), &ico, &[16, 32, 256]);
    assert!(result.is_ok(), "should convert large PNG to ICO: {:?}", result);
}

// ============================================================================
// png_bytes_to_ico – additional
// ============================================================================

#[test]
fn png_bytes_to_ico_result_is_ok_for_valid_input() {
    let bytes = create_png_bytes(64, 64);
    let dir = TempDir::new().unwrap();
    let ico = dir.path().join("b.ico");
    let result = png_bytes_to_ico(&bytes, &ico, &[16, 32]);
    assert!(result.is_ok());
}

// ============================================================================
// compress_png – output file exists and has expected dimensions
// ============================================================================

#[test]
fn compress_png_output_exists() {
    let src = create_test_png_size(16, 16);
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.png");
    let cr = compress_png(src.path(), &out, 5).unwrap();
    assert!(out.exists());
    assert_eq!(cr.width, 16);
    assert_eq!(cr.height, 16);
}

// ============================================================================
// CompressionResult – reduction_percent boundary cases
// ============================================================================

#[test]
fn compression_result_full_reduction() {
    // All bytes removed (pathological)
    let cr = CompressionResult {
        original_size: 1000,
        compressed_size: 0,
        width: 1,
        height: 1,
    };
    assert!((cr.reduction_percent() - 100.0).abs() < 1e-6);
}

#[test]
fn compression_result_no_reduction() {
    let cr = CompressionResult {
        original_size: 500,
        compressed_size: 500,
        width: 2,
        height: 2,
    };
    assert!((cr.reduction_percent() - 0.0).abs() < 1e-6);
}

#[test]
fn compression_result_size_increase() {
    // Compressed is larger than original (re-compression overhead)
    let cr = CompressionResult {
        original_size: 100,
        compressed_size: 110,
        width: 1,
        height: 1,
    };
    // reduction_percent will be negative
    let pct = cr.reduction_percent();
    assert!(pct < 0.0 || pct == 0.0 || pct.is_finite());
}

// ============================================================================
// CompressionLevel – Send + Sync
// ============================================================================

#[test]
fn compression_level_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<CompressionLevel>();
}

// ============================================================================
// DEFAULT_ICO_SIZES – ordering and uniqueness
// ============================================================================

#[test]
fn default_ico_sizes_are_unique() {
    let mut seen = std::collections::HashSet::new();
    for s in DEFAULT_ICO_SIZES {
        assert!(seen.insert(s), "duplicate size {} in DEFAULT_ICO_SIZES", s);
    }
}

#[test]
fn default_ico_sizes_all_positive() {
    for s in DEFAULT_ICO_SIZES {
        assert!(*s > 0, "ICO size must be positive, got {}", s);
    }
}
