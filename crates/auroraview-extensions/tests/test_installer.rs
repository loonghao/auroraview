//! Integration tests for installer utilities (src/installer.rs)

use std::fs;

use tempfile::tempdir;

use auroraview_extensions::installer::{
    archive_suffix_from_url, extract_crx_data, extract_store_extension_id, find_extension_root,
    is_valid_extension_id, resolve_extension_source_url, ExtensionSourceKind,
};

// ─── is_valid_extension_id ────────────────────────────────────────────────────

#[test]
fn test_valid_extension_id_32_lowercase() {
    assert!(is_valid_extension_id("abcdefghijklmnopqrstuvwxyzabcdef"));
}

#[test]
fn test_invalid_extension_id_short() {
    assert!(!is_valid_extension_id("short"));
}

#[test]
fn test_invalid_extension_id_uppercase() {
    assert!(!is_valid_extension_id("ABCDEFGHIJKLMNOPQRSTUVWXYZABCDEF"));
}

#[test]
fn test_invalid_extension_id_with_digit() {
    // contains digit — not valid per spec (a-p only), but our check uses is_ascii_lowercase
    // digits are not ascii_lowercase → should return false
    assert!(!is_valid_extension_id("abcdefghijklmnopqrstuvwxyzabcde1"));
}

// ─── resolve_extension_source_url ─────────────────────────────────────────────

#[test]
fn test_resolve_empty_url_error() {
    let result = resolve_extension_source_url("");
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("empty"));
}

#[test]
fn test_resolve_whitespace_url_error() {
    let result = resolve_extension_source_url("   ");
    assert!(result.is_err());
}

#[test]
fn test_resolve_crx_url() {
    let result =
        resolve_extension_source_url("https://example.com/extension.crx").unwrap();
    assert_eq!(result.kind, ExtensionSourceKind::Crx);
    assert_eq!(result.download_url, "https://example.com/extension.crx");
    assert!(result.store_extension_id.is_none());
}

#[test]
fn test_resolve_generic_archive_url() {
    let result = resolve_extension_source_url("https://example.com/ext.zip").unwrap();
    assert_eq!(result.kind, ExtensionSourceKind::Archive);
    assert_eq!(result.original_url, "https://example.com/ext.zip");
}

#[test]
fn test_resolve_chrome_web_store_url() {
    let ext_id = "abcdefghijklmnopqrstuvwxyzabcdef";
    let url = format!(
        "https://chromewebstore.google.com/detail/my-ext/{}",
        ext_id
    );
    let result = resolve_extension_source_url(&url).unwrap();
    assert_eq!(result.kind, ExtensionSourceKind::Crx);
    assert_eq!(result.store_extension_id, Some(ext_id.to_string()));
    assert!(result
        .download_url
        .contains("clients2.google.com/service/update2/crx"));
}

#[test]
fn test_resolve_edge_store_url() {
    let ext_id = "abcdefghijklmnopqrstuvwxyzabcdef";
    let url = format!(
        "https://microsoftedge.microsoft.com/addons/detail/my-ext/{}",
        ext_id
    );
    let result = resolve_extension_source_url(&url).unwrap();
    assert_eq!(result.kind, ExtensionSourceKind::Crx);
    assert_eq!(result.store_extension_id, Some(ext_id.to_string()));
    assert!(result
        .download_url
        .contains("edge.microsoft.com/extensionwebstorebase"));
}

// ─── archive_suffix_from_url ──────────────────────────────────────────────────

#[test]
fn test_suffix_crx_kind_always_crx() {
    assert_eq!(
        archive_suffix_from_url("anything.zip", ExtensionSourceKind::Crx),
        "crx"
    );
}

#[test]
fn test_suffix_tar_gz() {
    assert_eq!(
        archive_suffix_from_url(
            "https://example.com/ext.tar.gz",
            ExtensionSourceKind::Archive
        ),
        "tar.gz"
    );
}

#[test]
fn test_suffix_tgz() {
    assert_eq!(
        archive_suffix_from_url("https://example.com/ext.tgz", ExtensionSourceKind::Archive),
        "tgz"
    );
}

#[test]
fn test_suffix_tar() {
    assert_eq!(
        archive_suffix_from_url("https://example.com/ext.tar", ExtensionSourceKind::Archive),
        "tar"
    );
}

#[test]
fn test_suffix_default_zip() {
    assert_eq!(
        archive_suffix_from_url(
            "https://example.com/ext?token=abc",
            ExtensionSourceKind::Archive
        ),
        "zip"
    );
}

#[test]
fn test_suffix_query_params_stripped() {
    assert_eq!(
        archive_suffix_from_url(
            "https://example.com/ext.tar.gz?v=1",
            ExtensionSourceKind::Archive
        ),
        "tar.gz"
    );
}

// ─── extract_store_extension_id ───────────────────────────────────────────────

#[test]
fn test_extract_chrome_webstore_id() {
    let ext_id = "abcdefghijklmnopqrstuvwxyzabcdef";
    let url = format!(
        "https://chromewebstore.google.com/detail/name/{}",
        ext_id
    );
    assert_eq!(
        extract_store_extension_id(&url),
        Some(ext_id.to_string())
    );
}

#[test]
fn test_extract_old_chrome_store_id() {
    let ext_id = "abcdefghijklmnopqrstuvwxyzabcdef";
    let url = format!(
        "https://chrome.google.com/webstore/detail/name/{}",
        ext_id
    );
    assert_eq!(
        extract_store_extension_id(&url),
        Some(ext_id.to_string())
    );
}

#[test]
fn test_extract_non_store_url_none() {
    assert!(extract_store_extension_id("https://example.com/ext").is_none());
}

#[test]
fn test_extract_invalid_url_none() {
    assert!(extract_store_extension_id("not-a-url").is_none());
}

// ─── extract_crx_data ─────────────────────────────────────────────────────────

#[test]
fn test_extract_crx_data_too_small() {
    let dir = tempdir().unwrap();
    let result = extract_crx_data(&[0u8; 8], dir.path());
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("too small") || err.contains("small"));
}

#[test]
fn test_extract_crx_data_bad_magic() {
    let dir = tempdir().unwrap();
    let mut data = vec![0u8; 20];
    // Magic should be "Cr24" — set to something else
    data[0..4].copy_from_slice(b"XXXX");
    let result = extract_crx_data(&data, dir.path());
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("magic") || err.contains("Invalid CRX"));
}

#[test]
fn test_extract_crx_data_unsupported_version() {
    let dir = tempdir().unwrap();
    let mut data = vec![0u8; 20];
    data[0..4].copy_from_slice(b"Cr24");
    // version = 99 (unsupported)
    let version: u32 = 99;
    data[4..8].copy_from_slice(&version.to_le_bytes());
    let result = extract_crx_data(&data, dir.path());
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Unsupported") || err.contains("version") || err.contains("99"));
}

#[test]
fn test_extract_crx_data_crx2_header_too_short() {
    let dir = tempdir().unwrap();
    // Version 2 requires at least 16 bytes
    let mut data = vec![0u8; 12];
    data[0..4].copy_from_slice(b"Cr24");
    let version: u32 = 2;
    data[4..8].copy_from_slice(&version.to_le_bytes());
    let result = extract_crx_data(&data, dir.path());
    assert!(result.is_err());
}

#[test]
fn test_extract_crx_data_crx3_header_exceeds_file() {
    let dir = tempdir().unwrap();
    let mut data = vec![0u8; 20];
    data[0..4].copy_from_slice(b"Cr24");
    let version: u32 = 3;
    data[4..8].copy_from_slice(&version.to_le_bytes());
    // header_len = u32::MAX → offset would overflow
    let header_len: u32 = u32::MAX;
    data[8..12].copy_from_slice(&header_len.to_le_bytes());
    let result = extract_crx_data(&data, dir.path());
    assert!(result.is_err());
}

// ─── find_extension_root ──────────────────────────────────────────────────────

#[test]
fn test_find_extension_root_at_top_level() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("manifest.json"), "{}").unwrap();

    let found = find_extension_root(dir.path());
    assert_eq!(found, Some(dir.path().to_path_buf()));
}

#[test]
fn test_find_extension_root_nested() {
    let dir = tempdir().unwrap();
    let sub = dir.path().join("inner");
    fs::create_dir_all(&sub).unwrap();
    fs::write(sub.join("manifest.json"), "{}").unwrap();

    let found = find_extension_root(dir.path()).unwrap();
    assert_eq!(found, sub);
}

#[test]
fn test_find_extension_root_prefers_shallowest() {
    let dir = tempdir().unwrap();
    let level1 = dir.path().join("a");
    let level2 = level1.join("b");
    fs::create_dir_all(&level2).unwrap();
    // Both levels have manifest.json — should return the shallowest
    fs::write(level1.join("manifest.json"), "{}").unwrap();
    fs::write(level2.join("manifest.json"), "{}").unwrap();

    let found = find_extension_root(dir.path()).unwrap();
    assert_eq!(found, level1);
}

#[test]
fn test_find_extension_root_none_when_no_manifest() {
    let dir = tempdir().unwrap();
    let found = find_extension_root(dir.path());
    assert!(found.is_none());
}
