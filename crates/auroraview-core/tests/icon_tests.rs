//! Icon loading tests

use auroraview_core::icon::load_icon_rgba;
use std::io::Write;
use tempfile::NamedTempFile;

fn create_test_png() -> NamedTempFile {
    // Create a minimal 2x2 PNG
    let mut file = NamedTempFile::with_suffix(".png").unwrap();

    // Create a 2x2 red image
    let img = image::RgbaImage::from_fn(2, 2, |_, _| image::Rgba([255, 0, 0, 255]));

    let mut cursor = std::io::Cursor::new(Vec::new());
    img.write_to(&mut cursor, image::ImageFormat::Png).unwrap();

    file.write_all(cursor.get_ref()).unwrap();
    file.flush().unwrap();
    file
}

#[test]
fn test_load_icon_rgba() {
    let png_file = create_test_png();
    let icon = load_icon_rgba(png_file.path()).unwrap();

    assert_eq!(icon.width, 2);
    assert_eq!(icon.height, 2);
    assert_eq!(icon.rgba.len(), 2 * 2 * 4); // 4 bytes per pixel
}

#[test]
fn test_icon_data_resize() {
    let png_file = create_test_png();
    let icon = load_icon_rgba(png_file.path()).unwrap();
    let resized = icon.resize(32).unwrap();

    assert_eq!(resized.width, 32);
    assert_eq!(resized.height, 32);
}
