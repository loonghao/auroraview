//! Tests for auroraview-pack lib module

use auroraview_pack::{
    PackConfig, PackMode, OVERLAY_MAGIC, OVERLAY_VERSION, VERSION,
};

// =============================================================================
// VERSION tests
// =============================================================================

#[test]
fn version_has_parseable_semver_parts() {
    assert!(!VERSION.is_empty());

    let parts: Vec<&str> = VERSION.split('.').collect();
    assert!(parts.len() >= 2, "Expected at least major.minor in version");

    let major = parts[0].parse::<u64>().expect("major version should be numeric");
    let minor = parts[1]
        .split('-')
        .next()
        .unwrap_or(parts[1])
        .parse::<u64>()
        .expect("minor version should be numeric");

    let _ = (major, minor);
}

#[test]
fn version_not_zero() {
    let parts: Vec<&str> = VERSION.split('.').collect();
    let major: u64 = parts[0].parse().unwrap_or(0);
    let minor: u64 = parts[1].split('-').next().unwrap_or("0").parse().unwrap_or(0);
    // At least major or minor must be non-zero
    assert!(
        major > 0 || minor > 0,
        "Version should not be 0.0.x: {}",
        VERSION
    );
}

#[test]
fn version_is_static_str() {
    // Verify VERSION is a &'static str (compile-time constant)
    let v: &'static str = VERSION;
    assert!(!v.is_empty());
}

// =============================================================================
// OVERLAY_MAGIC / OVERLAY_VERSION tests
// =============================================================================

#[test]
fn overlay_magic_is_avpk() {
    assert_eq!(OVERLAY_MAGIC, b"AVPK");
}

#[test]
fn overlay_magic_length_is_4() {
    assert_eq!(OVERLAY_MAGIC.len(), 4);
}

#[test]
fn overlay_version_is_nonzero() {
    assert!(OVERLAY_VERSION > 0, "OVERLAY_VERSION should be > 0");
}

// =============================================================================
// is_packed tests
// =============================================================================

#[test]
fn is_packed_is_stable_in_test_env() {
    let first = auroraview_pack::is_packed();
    let second = auroraview_pack::is_packed();

    assert!(!first);
    assert_eq!(first, second);
}

#[test]
fn is_packed_returns_false_for_test_binary() {
    // Test runner binaries are never packed apps
    assert!(!auroraview_pack::is_packed());
}

// =============================================================================
// read_overlay tests
// =============================================================================

#[test]
fn read_overlay_returns_none_in_test_env() {
    let overlay = auroraview_pack::read_overlay().expect("read_overlay should succeed in tests");
    assert!(overlay.is_none());
}

#[test]
fn read_overlay_ok_variant_in_test_env() {
    // Should succeed (Ok) even if no overlay present
    assert!(auroraview_pack::read_overlay().is_ok());
}

// =============================================================================
// PackConfig builder API
// =============================================================================

#[test]
fn pack_config_url_sets_mode() {
    let config = PackConfig::url("https://example.com");
    assert!(matches!(config.mode, PackMode::Url { .. }));
    assert_eq!(config.mode.url(), Some("https://example.com"));
}

#[test]
fn pack_config_url_does_not_embed_assets() {
    let config = PackConfig::url("https://example.com");
    assert!(!config.mode.embeds_assets());
}

#[test]
fn pack_config_url_has_no_python() {
    let config = PackConfig::url("https://example.com");
    assert!(!config.mode.has_python());
}

#[test]
fn pack_config_with_title() {
    let config = PackConfig::url("https://example.com").with_title("My App");
    assert_eq!(config.window.title, "My App");
}

#[test]
fn pack_config_with_size() {
    let config = PackConfig::url("https://example.com").with_size(1280, 800);
    assert_eq!(config.window.width, 1280);
    assert_eq!(config.window.height, 800);
}

#[test]
fn pack_config_default_compression_level() {
    let config = PackConfig::url("https://example.com");
    // Default compression level should be 19 (high compression for releases)
    assert_eq!(config.compression_level, 19);
}

#[test]
fn pack_mode_name_url() {
    let mode = PackMode::Url { url: "https://example.com".to_string() };
    assert_eq!(mode.name(), "url");
}

#[test]
fn pack_mode_name_frontend() {
    let mode = PackMode::Frontend { path: std::path::PathBuf::from("./dist") };
    assert_eq!(mode.name(), "frontend");
    assert!(mode.embeds_assets());
}
