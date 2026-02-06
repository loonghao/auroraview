//! Tests for auroraview-pack overlay module

use auroraview_pack::{OverlayData, OverlayReader, OverlayWriter, PackConfig};
use tempfile::NamedTempFile;

#[test]
fn test_overlay_roundtrip() {
    // Create a temp file with some content
    let temp = NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), b"fake executable content").unwrap();

    // Create overlay data
    let config = PackConfig::url("https://example.com").with_title("Test App");
    let mut data = OverlayData::new(config);
    data.add_asset("index.html", b"<html></html>".to_vec());
    data.add_asset("style.css", b"body { }".to_vec());

    // Write overlay
    OverlayWriter::write(temp.path(), &data).unwrap();

    // Verify overlay exists
    assert!(OverlayReader::has_overlay(temp.path()).unwrap());

    // Read overlay
    let read_data = OverlayReader::read(temp.path()).unwrap().unwrap();
    assert_eq!(read_data.config.window.title, "Test App");
    assert_eq!(read_data.assets.len(), 2);

    // Verify original size
    let original_size = OverlayReader::get_original_size(temp.path())
        .unwrap()
        .unwrap();
    assert_eq!(original_size, b"fake executable content".len() as u64);
}

#[test]
fn test_no_overlay() {
    let temp = NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), b"just a regular file").unwrap();

    assert!(!OverlayReader::has_overlay(temp.path()).unwrap());
    assert!(OverlayReader::read(temp.path()).unwrap().is_none());
}
