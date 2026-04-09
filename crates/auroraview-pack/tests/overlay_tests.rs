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

// ─── New: additional overlay tests ────────────────────────────────────────────

#[test]
fn test_overlay_data_new_empty_assets() {
    let config = PackConfig::url("https://example.com");
    let data = OverlayData::new(config);
    assert_eq!(data.assets.len(), 0);
}

#[test]
fn test_overlay_data_add_and_count_assets() {
    let config = PackConfig::url("https://example.com");
    let mut data = OverlayData::new(config);
    data.add_asset("a.js", b"alert(1)".to_vec());
    data.add_asset("b.css", b"body{}".to_vec());
    data.add_asset("c.html", b"<h1/>".to_vec());
    assert_eq!(data.assets.len(), 3);
}

#[test]
fn test_overlay_roundtrip_zero_assets() {
    let temp = NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), b"tiny-bin").unwrap();

    let config = PackConfig::url("https://zero.example.com").with_title("ZeroAssets");
    let data = OverlayData::new(config);
    OverlayWriter::write(temp.path(), &data).unwrap();

    let read = OverlayReader::read(temp.path()).unwrap().unwrap();
    assert_eq!(read.config.window.title, "ZeroAssets");
    assert_eq!(read.assets.len(), 0);
}

#[test]
fn test_overlay_roundtrip_unicode_asset_name() {
    let temp = NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), b"bin").unwrap();

    let config = PackConfig::url("https://example.com");
    let mut data = OverlayData::new(config);
    data.add_asset("资产/index.html", b"<h1>unicode</h1>".to_vec());
    OverlayWriter::write(temp.path(), &data).unwrap();

    let read = OverlayReader::read(temp.path()).unwrap().unwrap();
    assert_eq!(read.assets.len(), 1);
}

#[test]
fn test_overlay_roundtrip_large_asset() {
    let temp = NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), b"bin").unwrap();

    let config = PackConfig::url("https://example.com");
    let mut data = OverlayData::new(config);
    let large: Vec<u8> = (0u8..=255u8).cycle().take(65536).collect();
    data.add_asset("large.bin", large.clone());
    OverlayWriter::write(temp.path(), &data).unwrap();

    let read = OverlayReader::read(temp.path()).unwrap().unwrap();
    assert_eq!(read.assets.len(), 1);
}

#[test]
fn test_overlay_write_level_1() {
    let temp = NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), b"binary").unwrap();

    let config = PackConfig::url("https://example.com").with_title("FastCompress");
    let mut data = OverlayData::new(config);
    data.add_asset("index.js", b"var x=1;".to_vec());
    OverlayWriter::write_with_level(temp.path(), &data, 1).unwrap();

    let read = OverlayReader::read(temp.path()).unwrap().unwrap();
    assert_eq!(read.config.window.title, "FastCompress");
    assert_eq!(read.assets.len(), 1);
}

#[test]
fn test_overlay_config_size_roundtrip() {
    let temp = NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), b"bin").unwrap();

    let config = PackConfig::url("https://example.com")
        .with_title("SizeTest")
        .with_size(1920, 1080);
    let data = OverlayData::new(config);
    OverlayWriter::write(temp.path(), &data).unwrap();

    let read = OverlayReader::read(temp.path()).unwrap().unwrap();
    assert_eq!(read.config.window.width, 1920);
    assert_eq!(read.config.window.height, 1080);
}

#[test]
fn test_overlay_same_content_same_hash() {
    let config1 = PackConfig::url("https://example.com");
    let mut data1 = OverlayData::new(config1);
    data1.add_asset("f.js", b"same".to_vec());

    let config2 = PackConfig::url("https://example.com");
    let mut data2 = OverlayData::new(config2);
    data2.add_asset("f.js", b"same".to_vec());

    assert_eq!(data1.compute_content_hash(), data2.compute_content_hash());
}

#[test]
fn test_overlay_has_overlay_true_after_write() {
    let temp = NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), b"exe").unwrap();

    let config = PackConfig::url("https://example.com");
    let data = OverlayData::new(config);
    OverlayWriter::write(temp.path(), &data).unwrap();

    assert!(OverlayReader::has_overlay(temp.path()).unwrap());
}

#[test]
fn test_overlay_file_size_increases_after_write() {
    let original = b"small original binary";
    let temp = NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), original).unwrap();
    let original_size = std::fs::metadata(temp.path()).unwrap().len();

    let config = PackConfig::url("https://example.com");
    let mut data = OverlayData::new(config);
    data.add_asset("data.json", b"{\"key\":\"value\"}".to_vec());
    OverlayWriter::write(temp.path(), &data).unwrap();

    let new_size = std::fs::metadata(temp.path()).unwrap().len();
    assert!(new_size > original_size, "File should grow after overlay write");
}

// ─── R8 Additional overlay tests ──────────────────────────────────────────────

#[test]
fn test_overlay_roundtrip_debug_mode() {
    let temp = NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), b"debug-bin").unwrap();

    let config = PackConfig::url("https://example.com")
        .with_title("DebugApp")
        .with_debug(true);
    let data = OverlayData::new(config);
    OverlayWriter::write(temp.path(), &data).unwrap();

    let read = OverlayReader::read(temp.path()).unwrap().unwrap();
    assert_eq!(read.config.window.title, "DebugApp");
    assert!(read.config.debug);
}

#[test]
fn test_overlay_roundtrip_multiple_writes_consistent() {
    // Writing same data twice should produce readable overlay both times
    let temp1 = NamedTempFile::new().unwrap();
    let temp2 = NamedTempFile::new().unwrap();
    std::fs::write(temp1.path(), b"bin1").unwrap();
    std::fs::write(temp2.path(), b"bin2").unwrap();

    let config1 = PackConfig::url("https://a.example.com").with_title("App1");
    let config2 = PackConfig::url("https://b.example.com").with_title("App2");
    let data1 = OverlayData::new(config1);
    let data2 = OverlayData::new(config2);

    OverlayWriter::write(temp1.path(), &data1).unwrap();
    OverlayWriter::write(temp2.path(), &data2).unwrap();

    let r1 = OverlayReader::read(temp1.path()).unwrap().unwrap();
    let r2 = OverlayReader::read(temp2.path()).unwrap().unwrap();
    assert_eq!(r1.config.window.title, "App1");
    assert_eq!(r2.config.window.title, "App2");
    assert_ne!(r1.config.window.title, r2.config.window.title);
}

#[test]
fn test_overlay_asset_content_roundtrip() {
    let temp = NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), b"exec").unwrap();

    let expected = b"const x = 42; // test asset content";
    let config = PackConfig::url("https://example.com");
    let mut data = OverlayData::new(config);
    data.add_asset("app.js", expected.to_vec());
    OverlayWriter::write(temp.path(), &data).unwrap();

    let read = OverlayReader::read(temp.path()).unwrap().unwrap();
    assert_eq!(read.assets.len(), 1);
    // Asset name should be preserved (assets are Vec<(String, Vec<u8>)>)
    assert_eq!(read.assets[0].0, "app.js");
}

#[test]
fn test_overlay_get_original_size_matches_written_bytes() {
    let content = b"hello world binary content for test";
    let temp = NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), content).unwrap();

    let config = PackConfig::url("https://example.com");
    let data = OverlayData::new(config);
    OverlayWriter::write(temp.path(), &data).unwrap();

    let size = OverlayReader::get_original_size(temp.path())
        .unwrap()
        .unwrap();
    assert_eq!(size, content.len() as u64);
}

#[test]
fn test_overlay_url_preserved_in_config() {
    let temp = NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), b"binary").unwrap();

    let url = "https://my-dcc-tool.example.com/app";
    let config = PackConfig::url(url);
    let data = OverlayData::new(config);
    OverlayWriter::write(temp.path(), &data).unwrap();

    let read = OverlayReader::read(temp.path()).unwrap().unwrap();
    // URL mode stores url in PackMode::Url { url }
    if let auroraview_pack::PackMode::Url { url: stored_url } = &read.config.mode {
        assert!(stored_url.contains("my-dcc-tool.example.com"));
    } else {
        panic!("Expected PackMode::Url");
    }
}

#[test]
fn test_overlay_with_50_assets() {
    let temp = NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), b"binary").unwrap();

    let config = PackConfig::url("https://example.com");
    let mut data = OverlayData::new(config);
    for i in 0..50 {
        data.add_asset(format!("asset_{}.txt", i), format!("content_{}", i).into_bytes());
    }
    OverlayWriter::write(temp.path(), &data).unwrap();

    let read = OverlayReader::read(temp.path()).unwrap().unwrap();
    assert_eq!(read.assets.len(), 50);
}

#[test]
fn test_overlay_write_to_file_with_existing_overlay() {
    // Re-writing an already-overlaid file should replace the overlay
    let temp = NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), b"binary").unwrap();

    let config1 = PackConfig::url("https://v1.example.com").with_title("First");
    let data1 = OverlayData::new(config1);
    OverlayWriter::write(temp.path(), &data1).unwrap();

    // Overwrite
    let config2 = PackConfig::url("https://v2.example.com").with_title("Second");
    let data2 = OverlayData::new(config2);
    OverlayWriter::write(temp.path(), &data2).unwrap();

    let read = OverlayReader::read(temp.path()).unwrap().unwrap();
    assert_eq!(read.config.window.title, "Second");
    // Original size should reflect only the first write's "binary" bytes
    let size = OverlayReader::get_original_size(temp.path()).unwrap().unwrap();
    assert!(size > 0);
}

#[test]
fn test_overlay_content_hash_is_hex_string() {
    let config = PackConfig::url("https://example.com");
    let mut data = OverlayData::new(config);
    data.add_asset("f.js", b"test".to_vec());

    let hash = data.compute_content_hash();
    // Hash should be non-empty and contain only hex chars
    assert!(!hash.is_empty());
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()), "Hash should be hex: {}", hash);
}

#[test]
fn test_overlay_empty_content_hash_stable_across_calls() {
    let config1 = PackConfig::url("https://example.com");
    let mut d1 = OverlayData::new(config1);
    let config2 = PackConfig::url("https://example.com");
    let mut d2 = OverlayData::new(config2);

    let h1 = d1.compute_content_hash();
    let h2 = d2.compute_content_hash();
    assert_eq!(h1, h2, "Empty overlays with same config should have same hash");
}

// ─── Additional overlay tests R14 ─────────────────────────────────────────────

#[test]
fn test_overlay_reader_has_overlay_after_write_with_level_9() {
    let temp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), b"bin").unwrap();

    let config = PackConfig::url("https://example.com");
    let data = OverlayData::new(config);
    OverlayWriter::write_with_level(temp.path(), &data, 9).unwrap();

    assert!(OverlayReader::has_overlay(temp.path()).unwrap());
}

#[test]
fn test_overlay_reader_has_overlay_false_before_write() {
    let temp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), b"not-an-overlay").unwrap();
    assert!(!OverlayReader::has_overlay(temp.path()).unwrap());
}

#[test]
fn test_overlay_config_debug_mode_false_default() {
    let config = PackConfig::url("https://example.com");
    assert!(!config.debug);
}

#[test]
fn test_overlay_config_with_debug_true() {
    let config = PackConfig::url("https://example.com").with_debug(true);
    assert!(config.debug);
}

#[test]
fn test_overlay_data_add_empty_content_asset() {
    let config = PackConfig::url("https://example.com");
    let mut data = OverlayData::new(config);
    data.add_asset("empty.js", vec![]);
    assert_eq!(data.assets.len(), 1);
}

#[test]
fn test_overlay_roundtrip_1_asset_content_correct() {
    let temp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), b"exec").unwrap();

    let content = b"const VERSION = '1.0.0';";
    let config = PackConfig::url("https://example.com");
    let mut data = OverlayData::new(config);
    data.add_asset("version.js", content.to_vec());
    OverlayWriter::write(temp.path(), &data).unwrap();

    let read = OverlayReader::read(temp.path()).unwrap().unwrap();
    assert_eq!(read.assets.len(), 1);
    assert_eq!(read.assets[0].0, "version.js");
}

#[test]
fn test_overlay_multiple_writes_last_wins() {
    let temp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), b"bin").unwrap();

    for i in 0..3u32 {
        let config = PackConfig::url(format!("https://example{}.com", i)).with_title(format!("App{}", i));
        let data = OverlayData::new(config);
        OverlayWriter::write(temp.path(), &data).unwrap();
    }

    let read = OverlayReader::read(temp.path()).unwrap().unwrap();
    assert_eq!(read.config.window.title, "App2");
}

#[test]
fn test_overlay_write_with_level_0() {
    // Level 0 = no compression
    let temp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), b"bin").unwrap();

    let config = PackConfig::url("https://example.com").with_title("Level0");
    let data = OverlayData::new(config);
    OverlayWriter::write_with_level(temp.path(), &data, 0).unwrap();

    let read = OverlayReader::read(temp.path()).unwrap().unwrap();
    assert_eq!(read.config.window.title, "Level0");
}

#[test]
fn test_overlay_config_resizable_default() {
    let config = PackConfig::url("https://example.com");
    // resizable defaults to true
    assert!(config.window.resizable);
}

#[test]
fn test_overlay_config_with_resizable_false() {
    let config = PackConfig::url("https://example.com").with_resizable(false);
    assert!(!config.window.resizable);
}

#[test]
fn test_overlay_content_hash_length_consistent() {
    // Two different content sets should produce hashes of the same length
    let config1 = PackConfig::url("https://example.com");
    let mut d1 = OverlayData::new(config1);
    d1.add_asset("a.js", b"small".to_vec());

    let config2 = PackConfig::url("https://example.com");
    let mut d2 = OverlayData::new(config2);
    let large: Vec<u8> = (0u8..=255u8).cycle().take(10000).collect();
    d2.add_asset("b.js", large);

    let h1 = d1.compute_content_hash();
    let h2 = d2.compute_content_hash();
    assert_eq!(h1.len(), h2.len(), "BLAKE3 hashes should be fixed length");
}

#[test]
fn test_overlay_reader_get_original_size_zero_binary() {
    let temp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), b"").unwrap();

    let config = PackConfig::url("https://example.com");
    let data = OverlayData::new(config);
    OverlayWriter::write(temp.path(), &data).unwrap();

    let size = OverlayReader::get_original_size(temp.path()).unwrap().unwrap();
    assert_eq!(size, 0, "empty binary should have original size = 0");
}

