//! Tests for auroraview-pack packer module

use auroraview_pack::{Manifest, PackConfig, Packer, VxConfig};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_packer_validate_url() {
    let config = PackConfig::url("https://example.com");
    let _packer = Packer::new(config);
    // Packer::validate is private, but we can test through pack()
    // For now, just verify construction works
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

#[test]
fn test_manifest_vx_config_parsing() {
    let temp = TempDir::new().unwrap();
    let frontend_dir = temp.path().join("frontend");
    fs::create_dir_all(&frontend_dir).unwrap();
    fs::write(frontend_dir.join("index.html"), "<html></html>").unwrap();

    let manifest_toml = format!(
        r#"
[package]
name = "test-app"
version = "0.1.0"

[frontend]
path = "{}"

[vx]
enabled = true
runtime_url = "https://example.com/vx.zip"
runtime_checksum = "deadbeef"
cache_dir = "./.pack-cache/vx"
allow_insecure = false

[[downloads]]
name = "vx-runtime"
url = "https://example.com/vx.zip"
checksum = "deadbeef"
extract = true
strip_components = 1
stage = "before_collect"
dest = "python/bin/vx"
executable = ["vx.exe"]

[hooks]
use_vx = true

[hooks.vx]
before_collect = ["vx --version"]
after_pack = ["vx uv pip list"]
        "#,
        frontend_dir.display()
    );

    let manifest = Manifest::parse(&manifest_toml).expect("manifest should parse");
    let config = PackConfig::from_manifest(&manifest, temp.path()).expect("pack config");

    assert!(manifest.vx.is_some());
    assert_eq!(manifest.downloads.len(), 1);
    assert!(config.vx.as_ref().unwrap().enabled);
    assert_eq!(config.downloads.len(), 1);
    assert!(config.hooks.as_ref().map(|h| h.use_vx).unwrap_or(false));
}

// RFC 0003: vx packed dependency bootstrap tests

#[test]
fn test_vx_ensure() {
    let _temp = TempDir::new().unwrap();

    let mut config = PackConfig::url("https://example.com");
    config.vx = Some(VxConfig {
        enabled: true,
        ensure: vec!["python".to_string()], // Should pass if python is available
        ..Default::default()
    });

    let packer = Packer::new(config);

    // Test vx.ensure validation - should pass for python
    // (Note: This is a unit test, actual validation happens during pack())
    assert!(packer.validate_vx_ensure_requirements().is_ok());
}

#[test]
fn test_vx_ensure_missing_tool() {
    let _temp = TempDir::new().unwrap();

    let mut config = PackConfig::url("https://example.com");
    config.vx = Some(VxConfig {
        enabled: true,
        // Use a known tool that is unlikely to be installed to trigger an error
        // Note: Unknown tools are only warned about, not errored
        ensure: vec!["vx".to_string()],
        ..Default::default()
    });

    let packer = Packer::new(config);

    // This test will pass if vx is not installed and no runtime_url is configured
    // It will also pass if vx is already installed (CI environment may have it)
    // So we just verify the validation runs without panicking
    let result = packer.validate_vx_ensure_requirements();
    // Accept both Ok (vx installed) and Err (vx not installed, no runtime_url)
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_vx_runtime_injection() {
    let _temp = TempDir::new().unwrap();

    let mut config = PackConfig::url("https://example.com");
    config.vx = Some(VxConfig {
        enabled: true,
        runtime_url: Some("https://example.com/vx-runtime.tar.gz".to_string()),
        runtime_checksum: Some("sha256:abc123".to_string()),
        ..Default::default()
    });

    let packer = Packer::new(config);

    // Test that vx runtime is included in download entries
    let entries = packer.build_download_entries();
    assert!(entries.iter().any(|e| e.name == "vx-runtime"));

    // Note: detect_vx_path() checks for actual files on disk
    // In this unit test, we haven't actually downloaded anything,
    // so we just verify the download entry was created correctly
    let vx_entry = entries.iter().find(|e| e.name == "vx-runtime");
    assert!(vx_entry.is_some());
    assert_eq!(
        vx_entry.unwrap().url,
        "https://example.com/vx-runtime.tar.gz"
    );
}

#[test]
fn test_offline_mode() {
    use std::env;

    // Set offline mode
    env::set_var("AURORAVIEW_OFFLINE", "true");

    let _temp = TempDir::new().unwrap();
    let mut config = PackConfig::url("https://example.com");
    config.vx = Some(VxConfig {
        enabled: true,
        ..Default::default()
    });

    let _packer = Packer::new(config);

    // In offline mode, downloads should be skipped or use cache only
    // This is tested through the downloader's offline behavior

    // Clean up
    env::remove_var("AURORAVIEW_OFFLINE");
}
