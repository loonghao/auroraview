//! Tests for PackError variants, Display, Clone, and From conversions

use auroraview_pack::{PackError, PackResult};
use rstest::rstest;
use std::path::PathBuf;

// ============================================================================
// Display tests for each variant
// ============================================================================

#[rstest]
fn display_config() {
    let e = PackError::Config("missing field".to_string());
    assert_eq!(e.to_string(), "Configuration error: missing field");
}

#[rstest]
fn display_invalid_url() {
    let e = PackError::InvalidUrl("not-a-url".to_string());
    assert_eq!(e.to_string(), "Invalid URL: not-a-url");
}

#[rstest]
fn display_frontend_not_found() {
    let p = PathBuf::from("/missing/dist");
    let e = PackError::FrontendNotFound(p.clone());
    let s = e.to_string();
    assert!(s.contains("Frontend path not found"));
    assert!(s.contains("missing"));
}

#[rstest]
fn display_invalid_manifest() {
    let e = PackError::InvalidManifest("bad toml".to_string());
    assert_eq!(e.to_string(), "Invalid manifest: bad toml");
}

#[rstest]
fn display_invalid_overlay() {
    let e = PackError::InvalidOverlay("magic mismatch".to_string());
    assert_eq!(e.to_string(), "Invalid overlay format: magic mismatch");
}

#[rstest]
fn display_asset_not_found() {
    let p = PathBuf::from("icon.png");
    let e = PackError::AssetNotFound(p);
    let s = e.to_string();
    assert!(s.contains("Asset not found"));
    assert!(s.contains("icon.png"));
}

#[rstest]
fn display_bundle() {
    let e = PackError::Bundle("extension error".to_string());
    assert_eq!(e.to_string(), "Bundle error: extension error");
}

#[rstest]
fn display_icon() {
    let e = PackError::Icon("unsupported format".to_string());
    assert_eq!(e.to_string(), "Icon error: unsupported format");
}

#[rstest]
fn display_compression() {
    let e = PackError::Compression("zstd failed".to_string());
    assert_eq!(e.to_string(), "Compression error: zstd failed");
}

#[rstest]
fn display_build() {
    let e = PackError::Build("pyoxidizer not found".to_string());
    assert_eq!(e.to_string(), "Build error: pyoxidizer not found");
}

#[rstest]
fn display_download() {
    let e = PackError::Download("timeout".to_string());
    assert_eq!(e.to_string(), "Download error: timeout");
}

#[rstest]
fn display_resource_edit() {
    let e = PackError::ResourceEdit("PE header corrupt".to_string());
    assert_eq!(e.to_string(), "Resource edit error: PE header corrupt");
}

#[rstest]
fn display_vx_ensure_failed() {
    let e = PackError::VxEnsureFailed("node >= 20 required".to_string());
    assert_eq!(e.to_string(), "vx.ensure validation failed: node >= 20 required");
}

// ============================================================================
// Debug trait
// ============================================================================

#[rstest]
fn debug_config() {
    let e = PackError::Config("x".to_string());
    let s = format!("{:?}", e);
    assert!(s.contains("Config"));
}

#[rstest]
fn debug_io_variant() {
    let io_err = std::io::Error::other("disk full");
    let e = PackError::Io(io_err);
    let s = format!("{:?}", e);
    assert!(s.contains("Io"));
}

// ============================================================================
// Clone implementation
// ============================================================================

#[rstest]
fn clone_config() {
    let e = PackError::Config("cfg".to_string());
    let c = e.clone();
    assert_eq!(c.to_string(), e.to_string());
}

#[rstest]
fn clone_invalid_url() {
    let e = PackError::InvalidUrl("bad".to_string());
    let c = e.clone();
    assert_eq!(c.to_string(), e.to_string());
}

#[rstest]
fn clone_frontend_not_found() {
    let e = PackError::FrontendNotFound(PathBuf::from("/a/b"));
    let c = e.clone();
    assert_eq!(c.to_string(), e.to_string());
}

#[rstest]
fn clone_invalid_manifest() {
    let e = PackError::InvalidManifest("m".to_string());
    let c = e.clone();
    assert_eq!(c.to_string(), e.to_string());
}

#[rstest]
fn clone_invalid_overlay() {
    let e = PackError::InvalidOverlay("o".to_string());
    let c = e.clone();
    assert_eq!(c.to_string(), e.to_string());
}

#[rstest]
fn clone_asset_not_found() {
    let e = PackError::AssetNotFound(PathBuf::from("x.png"));
    let c = e.clone();
    assert_eq!(c.to_string(), e.to_string());
}

#[rstest]
fn clone_bundle() {
    let e = PackError::Bundle("b".to_string());
    let c = e.clone();
    assert_eq!(c.to_string(), e.to_string());
}

#[rstest]
fn clone_icon() {
    let e = PackError::Icon("i".to_string());
    let c = e.clone();
    assert_eq!(c.to_string(), e.to_string());
}

#[rstest]
fn clone_compression() {
    let e = PackError::Compression("c".to_string());
    let c = e.clone();
    assert_eq!(c.to_string(), e.to_string());
}

#[rstest]
fn clone_build() {
    let e = PackError::Build("b".to_string());
    let c = e.clone();
    assert_eq!(c.to_string(), e.to_string());
}

#[rstest]
fn clone_download() {
    let e = PackError::Download("d".to_string());
    let c = e.clone();
    assert_eq!(c.to_string(), e.to_string());
}

#[rstest]
fn clone_resource_edit() {
    let e = PackError::ResourceEdit("r".to_string());
    let c = e.clone();
    assert_eq!(c.to_string(), e.to_string());
}

#[rstest]
fn clone_vx_ensure_failed() {
    let e = PackError::VxEnsureFailed("v".to_string());
    let c = e.clone();
    assert_eq!(c.to_string(), e.to_string());
}

/// Io variant clones to Config (documented behavior in Clone impl)
#[rstest]
fn clone_io_becomes_config() {
    let io_err = std::io::Error::other("disk full");
    let e = PackError::Io(io_err);
    let c = e.clone();
    // Cloned Io → Config("IO error")
    assert!(c.to_string().contains("IO error") || c.to_string().contains("Configuration error"));
}

/// TomlParse variant clones to Config
#[rstest]
fn clone_toml_parse_becomes_config() {
    let toml_err: toml::de::Error = toml::from_str::<toml::Value>("[[invalid").unwrap_err();
    let e = PackError::TomlParse(toml_err);
    let c = e.clone();
    assert!(c.to_string().contains("TOML parse error") || c.to_string().contains("Configuration error"));
}

/// Json variant clones to Config
#[rstest]
fn clone_json_becomes_config() {
    let json_err = serde_json::from_str::<serde_json::Value>("{bad}").unwrap_err();
    let e = PackError::Json(json_err);
    let c = e.clone();
    assert!(c.to_string().contains("JSON error") || c.to_string().contains("Configuration error"));
}

// ============================================================================
// From conversions
// ============================================================================

#[rstest]
fn from_io_error() {
    let io_err = std::io::Error::other("disk full");
    let e: PackError = io_err.into();
    let s = e.to_string();
    assert!(s.contains("I/O error"));
}

#[rstest]
fn from_toml_error() {
    let toml_err: toml::de::Error = toml::from_str::<toml::Value>("[[invalid").unwrap_err();
    let e: PackError = toml_err.into();
    let s = e.to_string();
    assert!(s.contains("TOML parse error"));
}

#[rstest]
fn from_json_error() {
    let json_err = serde_json::from_str::<serde_json::Value>("{bad}").unwrap_err();
    let e: PackError = json_err.into();
    let s = e.to_string();
    assert!(s.contains("JSON error"));
}

// ============================================================================
// PackResult type alias
// ============================================================================

#[rstest]
fn pack_result_ok() {
    let r: PackResult<u32> = Ok(42);
    assert_eq!(r.unwrap(), 42);
}

#[rstest]
fn pack_result_err() {
    let r: PackResult<u32> = Err(PackError::Config("oops".to_string()));
    assert!(r.is_err());
    assert!(r.unwrap_err().to_string().contains("oops"));
}

// ============================================================================
// Error source chain (thiserror integration)
// ============================================================================

#[rstest]
fn io_error_has_source() {
    use std::error::Error;
    let io_err = std::io::Error::other("root cause");
    let e: PackError = io_err.into();
    assert!(e.source().is_some());
}

#[rstest]
fn config_error_no_source() {
    use std::error::Error;
    let e = PackError::Config("no source".to_string());
    assert!(e.source().is_none());
}

// ============================================================================
// Send + Sync bounds
// ============================================================================

fn assert_send_sync<T: Send + Sync>() {}

#[rstest]
fn pack_error_is_send_sync() {
    assert_send_sync::<PackError>();
}

// ============================================================================
// Parametrized: string-holding variants round-trip their message
// ============================================================================

#[rstest]
#[case(PackError::Config("c".to_string()), "c")]
#[case(PackError::InvalidUrl("u".to_string()), "u")]
#[case(PackError::InvalidManifest("m".to_string()), "m")]
#[case(PackError::InvalidOverlay("o".to_string()), "o")]
#[case(PackError::Bundle("b".to_string()), "b")]
#[case(PackError::Icon("i".to_string()), "i")]
#[case(PackError::Compression("z".to_string()), "z")]
#[case(PackError::Build("x".to_string()), "x")]
#[case(PackError::Download("d".to_string()), "d")]
#[case(PackError::ResourceEdit("r".to_string()), "r")]
#[case(PackError::VxEnsureFailed("v".to_string()), "v")]
fn string_variant_message_in_display(#[case] e: PackError, #[case] fragment: &str) {
    assert!(e.to_string().contains(fragment));
}
