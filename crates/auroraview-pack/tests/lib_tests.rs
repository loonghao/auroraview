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
    const _: () = assert!(OVERLAY_VERSION > 0);
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
    auroraview_pack::read_overlay().expect("read_overlay should not fail in test env");
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

// ============================================================================
// R10 Extensions
// ============================================================================

#[test]
fn overlay_magic_is_ascii() {
    for b in OVERLAY_MAGIC.iter() {
        assert!(b.is_ascii(), "OVERLAY_MAGIC bytes should be ASCII");
    }
}

#[test]
fn overlay_magic_as_str() {
    let s = std::str::from_utf8(OVERLAY_MAGIC).expect("OVERLAY_MAGIC should be valid UTF-8");
    assert_eq!(s, "AVPK");
}

#[test]
fn overlay_version_fits_in_u32() {
    // OVERLAY_VERSION should be representable as u32 (for 4-byte LE encoding)
    let _v: u32 = OVERLAY_VERSION;
}

#[test]
fn version_contains_dot() {
    assert!(VERSION.contains('.'), "VERSION should contain at least one dot: {}", VERSION);
}

#[test]
fn version_not_empty() {
    assert!(!VERSION.trim().is_empty());
}

#[test]
fn pack_config_url_mode_name() {
    let config = PackConfig::url("https://example.com");
    assert_eq!(config.mode.name(), "url");
}

#[test]
fn pack_config_url_returns_correct_url() {
    let url = "https://my-dcc-tool.example.com/ui";
    let config = PackConfig::url(url);
    assert_eq!(config.mode.url(), Some(url));
}

#[test]
fn pack_config_url_no_python() {
    let config = PackConfig::url("https://example.com");
    assert!(!config.mode.has_python());
}

#[test]
fn pack_config_url_does_not_embed_assets_r10() {
    let config = PackConfig::url("https://example.com");
    assert!(!config.mode.embeds_assets());
}

#[test]
fn pack_config_frontend_embeds_assets() {
    let config_mode = PackMode::Frontend {
        path: std::path::PathBuf::from("./dist"),
    };
    assert!(config_mode.embeds_assets());
}

#[test]
fn pack_config_with_title_preserves_url() {
    let config = PackConfig::url("https://example.com/app")
        .with_title("DCC Tool");
    assert_eq!(config.window.title, "DCC Tool");
    assert_eq!(config.mode.url(), Some("https://example.com/app"));
}

#[test]
fn pack_config_with_size_preserves_title() {
    let config = PackConfig::url("https://example.com")
        .with_title("Test")
        .with_size(1920, 1080);
    assert_eq!(config.window.title, "Test");
    assert_eq!(config.window.width, 1920);
    assert_eq!(config.window.height, 1080);
}

#[test]
fn pack_config_default_size() {
    let config = PackConfig::url("https://example.com");
    // Default sizes should be reasonable
    assert!(config.window.width > 0);
    assert!(config.window.height > 0);
}

#[test]
fn pack_config_compression_level_range() {
    let config = PackConfig::url("https://example.com");
    // Compression level should be between 1 and 22 for zstd
    assert!(config.compression_level >= 1);
    assert!(config.compression_level <= 22);
}

#[test]
fn pack_mode_frontend_no_url() {
    let mode = PackMode::Frontend { path: std::path::PathBuf::from("./dist") };
    assert!(mode.url().is_none());
}

#[test]
fn pack_mode_frontend_has_no_python() {
    let mode = PackMode::Frontend { path: std::path::PathBuf::from("./dist") };
    assert!(!mode.has_python());
}

#[test]
fn is_packed_never_true_in_tests() {
    // Called multiple times — always false in test environment
    for _ in 0..5 {
        assert!(!auroraview_pack::is_packed());
    }
}

#[test]
fn read_overlay_none_in_tests() {
    let overlay = auroraview_pack::read_overlay().unwrap();
    assert!(overlay.is_none());
}

#[test]
fn pack_config_with_size_zero_values() {
    // Zero values should be stored as-is (validation is caller's concern)
    let config = PackConfig::url("https://example.com").with_size(0, 0);
    assert_eq!(config.window.width, 0);
    assert_eq!(config.window.height, 0);
}

#[test]
fn pack_config_with_title_empty() {
    let config = PackConfig::url("https://example.com").with_title("");
    assert_eq!(config.window.title, "");
}

#[test]
fn pack_config_with_title_unicode() {
    let config = PackConfig::url("https://example.com").with_title("ツール");
    assert_eq!(config.window.title, "ツール");
}

#[test]
fn version_major_numeric() {
    let parts: Vec<&str> = VERSION.split('.').collect();
    assert!(!parts.is_empty());
    parts[0]
        .parse::<u64>()
        .unwrap_or_else(|_| panic!("Major version '{}' should be numeric", parts[0]));
}

#[test]
fn overlay_magic_bytes_eq_avpk() {
    assert_eq!(OVERLAY_MAGIC[0], b'A');
    assert_eq!(OVERLAY_MAGIC[1], b'V');
    assert_eq!(OVERLAY_MAGIC[2], b'P');
    assert_eq!(OVERLAY_MAGIC[3], b'K');
}

#[test]
fn pack_config_url_different_schemes() {
    let urls = [
        "https://secure.example.com",
        "http://local.example.com:8080",
        "file:///path/to/index.html",
    ];
    for url in urls {
        let config = PackConfig::url(url);
        assert_eq!(config.mode.url(), Some(url));
    }
}
