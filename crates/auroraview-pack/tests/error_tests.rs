//! Tests for PackError variants, Display messages, From conversions and Clone

use auroraview_pack::{PackError, PackResult};
use rstest::rstest;
use std::io;
use std::path::PathBuf;

// ============================================================================
// PackError Display messages
// ============================================================================

#[rstest]
fn test_pack_error_config_display() {
    let err = PackError::Config("missing field 'name'".to_string());
    let msg = err.to_string();
    assert!(msg.contains("Configuration error"));
    assert!(msg.contains("missing field 'name'"));
}

#[rstest]
fn test_pack_error_invalid_url_display() {
    let err = PackError::InvalidUrl("not-a-url".to_string());
    let msg = err.to_string();
    assert!(msg.contains("Invalid URL"));
    assert!(msg.contains("not-a-url"));
}

#[rstest]
fn test_pack_error_frontend_not_found_display() {
    let err = PackError::FrontendNotFound(PathBuf::from("/missing/dist"));
    let msg = err.to_string();
    assert!(msg.contains("Frontend path not found"));
    assert!(msg.contains("dist"));
}

#[rstest]
fn test_pack_error_invalid_manifest_display() {
    let err = PackError::InvalidManifest("bad schema".to_string());
    let msg = err.to_string();
    assert!(msg.contains("Invalid manifest"));
    assert!(msg.contains("bad schema"));
}

#[rstest]
fn test_pack_error_invalid_overlay_display() {
    let err = PackError::InvalidOverlay("magic mismatch".to_string());
    let msg = err.to_string();
    assert!(msg.contains("Invalid overlay format"));
    assert!(msg.contains("magic mismatch"));
}

#[rstest]
fn test_pack_error_asset_not_found_display() {
    let err = PackError::AssetNotFound(PathBuf::from("icons/app.ico"));
    let msg = err.to_string();
    assert!(msg.contains("Asset not found"));
    assert!(msg.contains("app.ico"));
}

#[rstest]
fn test_pack_error_bundle_display() {
    let err = PackError::Bundle("zip failed".to_string());
    let msg = err.to_string();
    assert!(msg.contains("Bundle error"));
    assert!(msg.contains("zip failed"));
}

#[rstest]
fn test_pack_error_icon_display() {
    let err = PackError::Icon("unsupported format".to_string());
    let msg = err.to_string();
    assert!(msg.contains("Icon error"));
    assert!(msg.contains("unsupported format"));
}

#[rstest]
fn test_pack_error_compression_display() {
    let err = PackError::Compression("zstd error".to_string());
    let msg = err.to_string();
    assert!(msg.contains("Compression error"));
    assert!(msg.contains("zstd error"));
}

#[rstest]
fn test_pack_error_build_display() {
    let err = PackError::Build("PyOxidizer failed".to_string());
    let msg = err.to_string();
    assert!(msg.contains("Build error"));
    assert!(msg.contains("PyOxidizer failed"));
}

#[rstest]
fn test_pack_error_download_display() {
    let err = PackError::Download("connection refused".to_string());
    let msg = err.to_string();
    assert!(msg.contains("Download error"));
    assert!(msg.contains("connection refused"));
}

#[rstest]
fn test_pack_error_resource_edit_display() {
    let err = PackError::ResourceEdit("PE header corrupt".to_string());
    let msg = err.to_string();
    assert!(msg.contains("Resource edit error"));
    assert!(msg.contains("PE header corrupt"));
}

#[rstest]
fn test_pack_error_vx_ensure_failed_display() {
    let err = PackError::VxEnsureFailed("tool not found".to_string());
    let msg = err.to_string();
    assert!(msg.contains("vx.ensure validation failed"));
    assert!(msg.contains("tool not found"));
}

// ============================================================================
// From conversions
// ============================================================================

#[rstest]
fn test_pack_error_from_io_error() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let err: PackError = io_err.into();
    assert!(matches!(err, PackError::Io(_)));
    let msg = err.to_string();
    assert!(msg.contains("I/O error"));
}

#[rstest]
fn test_pack_error_from_io_error_permission_denied() {
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
    let err: PackError = io_err.into();
    assert!(matches!(err, PackError::Io(_)));
}

#[rstest]
fn test_pack_error_from_serde_json() {
    let json_err: Result<serde_json::Value, _> = serde_json::from_str("{bad json}");
    let err: PackError = json_err.unwrap_err().into();
    assert!(matches!(err, PackError::Json(_)));
    assert!(err.to_string().contains("JSON error"));
}

#[rstest]
fn test_pack_error_from_toml_parse() {
    let toml_err: Result<toml::Value, _> = toml::from_str("[bad\ntoml");
    let err: PackError = toml_err.unwrap_err().into();
    assert!(matches!(err, PackError::TomlParse(_)));
    assert!(err.to_string().contains("TOML parse error"));
}

// ============================================================================
// Debug representation
// ============================================================================

#[rstest]
fn test_pack_error_debug_config() {
    let err = PackError::Config("oops".to_string());
    let debug = format!("{:?}", err);
    assert!(debug.contains("Config"));
    assert!(debug.contains("oops"));
}

#[rstest]
fn test_pack_error_debug_frontend_not_found() {
    let err = PackError::FrontendNotFound(PathBuf::from("/my/path"));
    let debug = format!("{:?}", err);
    assert!(debug.contains("FrontendNotFound"));
}

#[rstest]
fn test_pack_error_debug_io() {
    let err = PackError::Io(io::Error::new(io::ErrorKind::Other, "other"));
    let debug = format!("{:?}", err);
    assert!(debug.contains("Io"));
}

// ============================================================================
// Clone behaviour
// ============================================================================

#[rstest]
fn test_pack_error_clone_config() {
    let original = PackError::Config("cfg".to_string());
    let cloned = original.clone();
    assert!(matches!(cloned, PackError::Config(_)));
    assert!(cloned.to_string().contains("cfg"));
}

#[rstest]
fn test_pack_error_clone_invalid_url() {
    let original = PackError::InvalidUrl("bad".to_string());
    let cloned = original.clone();
    assert!(matches!(cloned, PackError::InvalidUrl(_)));
}

#[rstest]
fn test_pack_error_clone_frontend_not_found() {
    let original = PackError::FrontendNotFound(PathBuf::from("/a/b"));
    let cloned = original.clone();
    match cloned {
        PackError::FrontendNotFound(p) => assert_eq!(p, PathBuf::from("/a/b")),
        _ => panic!("expected FrontendNotFound"),
    }
}

#[rstest]
fn test_pack_error_clone_invalid_manifest() {
    let original = PackError::InvalidManifest("bad".to_string());
    let cloned = original.clone();
    assert!(matches!(cloned, PackError::InvalidManifest(_)));
}

#[rstest]
fn test_pack_error_clone_invalid_overlay() {
    let original = PackError::InvalidOverlay("bad".to_string());
    let cloned = original.clone();
    assert!(matches!(cloned, PackError::InvalidOverlay(_)));
}

#[rstest]
fn test_pack_error_clone_asset_not_found() {
    let original = PackError::AssetNotFound(PathBuf::from("icon.ico"));
    let cloned = original.clone();
    match cloned {
        PackError::AssetNotFound(p) => assert_eq!(p, PathBuf::from("icon.ico")),
        _ => panic!("expected AssetNotFound"),
    }
}

#[rstest]
fn test_pack_error_clone_bundle() {
    let original = PackError::Bundle("b".to_string());
    let cloned = original.clone();
    assert!(matches!(cloned, PackError::Bundle(_)));
}

#[rstest]
fn test_pack_error_clone_icon() {
    let original = PackError::Icon("ico".to_string());
    let cloned = original.clone();
    assert!(matches!(cloned, PackError::Icon(_)));
}

#[rstest]
fn test_pack_error_clone_compression() {
    let original = PackError::Compression("zstd".to_string());
    let cloned = original.clone();
    assert!(matches!(cloned, PackError::Compression(_)));
}

#[rstest]
fn test_pack_error_clone_build() {
    let original = PackError::Build("b".to_string());
    let cloned = original.clone();
    assert!(matches!(cloned, PackError::Build(_)));
}

#[rstest]
fn test_pack_error_clone_download() {
    let original = PackError::Download("d".to_string());
    let cloned = original.clone();
    assert!(matches!(cloned, PackError::Download(_)));
}

#[rstest]
fn test_pack_error_clone_resource_edit() {
    let original = PackError::ResourceEdit("r".to_string());
    let cloned = original.clone();
    assert!(matches!(cloned, PackError::ResourceEdit(_)));
}

#[rstest]
fn test_pack_error_clone_vx_ensure_failed() {
    let original = PackError::VxEnsureFailed("v".to_string());
    let cloned = original.clone();
    assert!(matches!(cloned, PackError::VxEnsureFailed(_)));
}

/// Io clone downgrades to Config variant (documented behaviour in Clone impl)
#[rstest]
fn test_pack_error_clone_io_downgrades_to_config() {
    let original = PackError::Io(io::Error::new(io::ErrorKind::Other, "io"));
    let cloned = original.clone();
    assert!(matches!(cloned, PackError::Config(_)));
}

/// TomlParse clone downgrades to Config variant
#[rstest]
fn test_pack_error_clone_toml_parse_downgrades_to_config() {
    let toml_err: Result<toml::Value, _> = toml::from_str("[bad\n");
    let original = PackError::TomlParse(toml_err.unwrap_err());
    let cloned = original.clone();
    assert!(matches!(cloned, PackError::Config(_)));
}

/// Json clone downgrades to Config variant
#[rstest]
fn test_pack_error_clone_json_downgrades_to_config() {
    let json_err: Result<serde_json::Value, _> = serde_json::from_str("{{");
    let original = PackError::Json(json_err.unwrap_err());
    let cloned = original.clone();
    assert!(matches!(cloned, PackError::Config(_)));
}

// ============================================================================
// PackResult type alias
// ============================================================================

#[rstest]
fn test_pack_result_ok() {
    let result: PackResult<u32> = Ok(42);
    assert_eq!(result.unwrap(), 42);
}

#[rstest]
fn test_pack_result_err() {
    let result: PackResult<u32> = Err(PackError::Config("bad".to_string()));
    assert!(result.is_err());
}

#[rstest]
fn test_pack_result_err_message_propagated() {
    let result: PackResult<String> = Err(PackError::Download("timeout".to_string()));
    let err = result.unwrap_err();
    assert!(err.to_string().contains("timeout"));
}

// ============================================================================
// Parameterized Display check for string-payload variants
// ============================================================================

#[rstest]
#[case(PackError::Config("x".into()), "Configuration error")]
#[case(PackError::InvalidUrl("x".into()), "Invalid URL")]
#[case(PackError::InvalidManifest("x".into()), "Invalid manifest")]
#[case(PackError::InvalidOverlay("x".into()), "Invalid overlay format")]
#[case(PackError::Bundle("x".into()), "Bundle error")]
#[case(PackError::Icon("x".into()), "Icon error")]
#[case(PackError::Compression("x".into()), "Compression error")]
#[case(PackError::Build("x".into()), "Build error")]
#[case(PackError::Download("x".into()), "Download error")]
#[case(PackError::ResourceEdit("x".into()), "Resource edit error")]
#[case(PackError::VxEnsureFailed("x".into()), "vx.ensure validation failed")]
fn test_pack_error_display_prefix(#[case] err: PackError, #[case] expected_prefix: &str) {
    let msg = err.to_string();
    assert!(
        msg.contains(expected_prefix),
        "expected '{}' in '{}'",
        expected_prefix,
        msg
    );
}
