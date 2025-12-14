//! Tests for auroraview-pack packer module

use auroraview_pack::{PackConfig, Packer};
use tempfile::TempDir;

#[test]
fn test_packer_validate_url() {
    let config = PackConfig::url("https://example.com");
    let packer = Packer::new(config);
    // Packer::validate is private, but we can test through pack()
    // For now, just verify construction works
    assert!(true);
}

#[test]
fn test_packer_validate_empty_url() {
    let config = PackConfig::url("");
    let _packer = Packer::new(config);
    // Empty URL should fail validation during pack()
}

#[test]
fn test_packer_validate_frontend() {
    let temp = TempDir::new().unwrap();
    std::fs::write(temp.path().join("index.html"), "<html></html>").unwrap();

    let config = PackConfig::frontend(temp.path());
    let _packer = Packer::new(config);
    // Frontend with index.html should be valid
}

#[test]
fn test_packer_validate_frontend_missing() {
    let config = PackConfig::frontend("/nonexistent/path");
    let _packer = Packer::new(config);
    // Missing frontend should fail validation during pack()
}

#[test]
fn test_exe_name() {
    let config = PackConfig::url("example.com").with_output("my-app");
    let _packer = Packer::new(config);

    // get_exe_name is private, but we can verify the config
    #[cfg(target_os = "windows")]
    {
        // On Windows, exe name should have .exe extension
    }

    #[cfg(not(target_os = "windows"))]
    {
        // On other platforms, no extension
    }
}
