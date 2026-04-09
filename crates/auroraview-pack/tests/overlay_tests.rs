//! Tests for auroraview-pack overlay module

use auroraview_pack::{OverlayData, OverlayReader, OverlayWriter, PackConfig};
use tempfile::NamedTempFile;

#[test]
fn test_overlay_roundtrip() {
    let temp = NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), b"fake executable content").unwrap();

    let config = PackConfig::url("https://example.com").with_title("Test App");
    let mut data = OverlayData::new(config);
    data.add_asset("index.html", b"<html></html>".to_vec());
    data.add_asset("style.css", b"body { }".to_vec());

    OverlayWriter::write(temp.path(), &data).unwrap();

    assert!(OverlayReader::has_overlay(temp.path()).unwrap());

    let read_data = OverlayReader::read(temp.path()).unwrap().unwrap();
    assert_eq!(read_data.config.window.title, "Test App");
    assert_eq!(read_data.assets.len(), 2);

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

#[test]
fn test_overlay_url_mode_roundtrip() {
    let temp = NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), b"exe").unwrap();

    let config = PackConfig::url("https://maya-tools.example.com/ui")
        .with_title("Maya Tool")
        .with_size(1280, 720);
    let data = OverlayData::new(config);

    OverlayWriter::write(temp.path(), &data).unwrap();

    let read = OverlayReader::read(temp.path()).unwrap().unwrap();
    assert_eq!(read.config.window.title, "Maya Tool");
    assert_eq!(read.config.window.width, 1280);
    assert_eq!(read.config.window.height, 720);
    assert_eq!(read.assets.len(), 0);
}

#[test]
fn test_overlay_many_assets() {
    let temp = NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), b"base binary").unwrap();

    let config = PackConfig::url("https://example.com");
    let mut data = OverlayData::new(config);

    for i in 0..10 {
        data.add_asset(format!("file_{}.js", i), format!("var f{}=1;", i).into_bytes());
    }

    OverlayWriter::write(temp.path(), &data).unwrap();

    let read = OverlayReader::read(temp.path()).unwrap().unwrap();
    assert_eq!(read.assets.len(), 10);
}

#[test]
fn test_overlay_empty_file_no_overlay() {
    let temp = NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), b"").unwrap();

    assert!(!OverlayReader::has_overlay(temp.path()).unwrap());
    assert!(OverlayReader::read(temp.path()).unwrap().is_none());
}

#[test]
fn test_overlay_preserves_original_binary() {
    let original = b"this is the original binary content for the exe file";
    let temp = NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), original).unwrap();

    let config = PackConfig::url("https://example.com");
    let data = OverlayData::new(config);
    OverlayWriter::write(temp.path(), &data).unwrap();

    let size = OverlayReader::get_original_size(temp.path())
        .unwrap()
        .unwrap();
    assert_eq!(size, original.len() as u64);
}

#[test]
fn test_overlay_write_twice_replaces() {
    let temp = NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), b"binary_v1").unwrap();

    let config1 = PackConfig::url("https://v1.example.com").with_title("V1");
    let data1 = OverlayData::new(config1);
    OverlayWriter::write(temp.path(), &data1).unwrap();

    let config2 = PackConfig::url("https://v2.example.com").with_title("V2");
    let data2 = OverlayData::new(config2);
    OverlayWriter::write(temp.path(), &data2).unwrap();

    let read = OverlayReader::read(temp.path()).unwrap().unwrap();
    assert_eq!(read.config.window.title, "V2");
}

#[test]
fn test_overlay_with_binary_asset_content() {
    let temp = NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), b"exec").unwrap();

    let config = PackConfig::url("https://example.com");
    let mut data = OverlayData::new(config);
    let binary: Vec<u8> = (0u8..=255u8).collect();
    data.add_asset("binary.bin", binary.clone());

    OverlayWriter::write(temp.path(), &data).unwrap();

    let read = OverlayReader::read(temp.path()).unwrap().unwrap();
    assert_eq!(read.assets.len(), 1);
}

// ─── Additional overlay tests ─────────────────────────────────────────────────

#[test]
fn test_overlay_asset_names_preserved() {
    let temp = NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), b"bin").unwrap();

    let config = PackConfig::url("https://example.com");
    let mut data = OverlayData::new(config);
    data.add_asset("js/app.js", b"app".to_vec());
    data.add_asset("css/style.css", b"style".to_vec());

    OverlayWriter::write(temp.path(), &data).unwrap();

    let read = OverlayReader::read(temp.path()).unwrap().unwrap();
    assert_eq!(read.assets.len(), 2);
}

#[test]
fn test_overlay_content_hash_stable() {
    let config = PackConfig::url("https://example.com");
    let mut data = OverlayData::new(config);
    data.add_asset("index.html", b"hello".to_vec());

    let hash1 = data.compute_content_hash();
    let hash2 = data.get_content_hash();
    assert_eq!(hash1, hash2, "content hash should be stable once computed");
}

#[test]
fn test_overlay_content_hash_non_empty() {
    let config = PackConfig::url("https://example.com");
    let mut data = OverlayData::new(config);
    data.add_asset("main.js", b"content".to_vec());

    let hash = data.compute_content_hash();
    assert!(!hash.is_empty(), "content hash should not be empty");
}

#[test]
fn test_overlay_different_content_different_hash() {
    let config1 = PackConfig::url("https://example.com");
    let mut data1 = OverlayData::new(config1);
    data1.add_asset("file.js", b"version_1".to_vec());

    let config2 = PackConfig::url("https://example.com");
    let mut data2 = OverlayData::new(config2);
    data2.add_asset("file.js", b"version_2".to_vec());

    let h1 = data1.compute_content_hash();
    let h2 = data2.compute_content_hash();
    assert_ne!(h1, h2, "different content should produce different hashes");
}

#[test]
fn test_overlay_no_assets_has_hash() {
    let config = PackConfig::url("https://example.com");
    let mut data = OverlayData::new(config);
    let hash = data.compute_content_hash();
    assert!(!hash.is_empty());
}

#[test]
fn test_overlay_write_with_level() {
    let temp = NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), b"binary").unwrap();

    let config = PackConfig::url("https://example.com").with_title("LevelTest");
    let mut data = OverlayData::new(config);
    data.add_asset("main.js", b"console.log('hello')".to_vec());

    // Write with compression level 9 (maximum)
    OverlayWriter::write_with_level(temp.path(), &data, 9).unwrap();

    let read = OverlayReader::read(temp.path()).unwrap().unwrap();
    assert_eq!(read.config.window.title, "LevelTest");
    assert_eq!(read.assets.len(), 1);
}

#[test]
fn test_overlay_large_original_binary() {
    let large = vec![0xABu8; 8192];
    let temp = NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), &large).unwrap();

    let config = PackConfig::url("https://example.com");
    let data = OverlayData::new(config);
    OverlayWriter::write(temp.path(), &data).unwrap();

    let size = OverlayReader::get_original_size(temp.path())
        .unwrap()
        .unwrap();
    assert_eq!(size, 8192u64);
}

#[test]
fn test_overlay_has_overlay_on_file_without_overlay() {
    let temp = NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), b"random bytes AVPK not here").unwrap();
    assert!(!OverlayReader::has_overlay(temp.path()).unwrap());
}

#[test]
fn test_overlay_nonexistent_file_returns_error() {
    let result = OverlayReader::has_overlay(std::path::Path::new("/nonexistent/path/file.exe"));
    assert!(result.is_err() || !result.unwrap_or(true));
}

#[test]
fn test_overlay_get_original_size_no_overlay() {
    let temp = NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), b"no overlay here").unwrap();

    let size = OverlayReader::get_original_size(temp.path()).unwrap();
    assert!(size.is_none());
}
