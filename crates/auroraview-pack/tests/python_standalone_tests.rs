//! Tests for auroraview-pack python_standalone module

use auroraview_pack::{
    get_runtime_cache_dir, PythonRuntimeMeta, PythonStandalone, PythonStandaloneConfig,
    PythonTarget,
};
use rstest::rstest;

#[test]
fn test_target_detection() {
    // Should not panic on supported platforms
    let result = PythonTarget::current();
    #[cfg(any(
        all(target_os = "windows", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64"),
    ))]
    assert!(result.is_ok());
}

#[test]
fn test_download_url() {
    let config = PythonStandaloneConfig {
        version: "3.11.11".to_string(),
        release: Some("20241206".to_string()),
        target: Some("x86_64-pc-windows-msvc".to_string()),
        cache_dir: None,
    };

    let standalone = PythonStandalone::new(config).unwrap();
    let url = standalone.download_url();

    assert!(url.contains("cpython-3.11.11"));
    assert!(url.contains("20241206"));
    assert!(url.contains("x86_64-pc-windows-msvc"));
}

#[test]
fn test_python_paths() {
    assert_eq!(PythonTarget::WindowsX64.python_exe(), "python.exe");
    assert_eq!(PythonTarget::LinuxX64.python_exe(), "python3");
    assert_eq!(PythonTarget::WindowsX64.python_path(), "python/python.exe");
    assert_eq!(PythonTarget::LinuxX64.python_path(), "python/bin/python3");
}

#[test]
fn test_target_triples() {
    assert_eq!(PythonTarget::WindowsX64.triple(), "x86_64-pc-windows-msvc");
    assert_eq!(PythonTarget::LinuxX64.triple(), "x86_64-unknown-linux-gnu");
    assert_eq!(PythonTarget::MacOSX64.triple(), "x86_64-apple-darwin");
    assert_eq!(PythonTarget::MacOSArm64.triple(), "aarch64-apple-darwin");
}

#[test]
fn test_macos_python_paths() {
    assert_eq!(PythonTarget::MacOSX64.python_exe(), "python3");
    assert_eq!(PythonTarget::MacOSArm64.python_exe(), "python3");
    assert_eq!(PythonTarget::MacOSX64.python_path(), "python/bin/python3");
    assert_eq!(PythonTarget::MacOSArm64.python_path(), "python/bin/python3");
}

#[test]
fn test_config_default() {
    let config = PythonStandaloneConfig::default();
    assert_eq!(config.version, "3.11");
    assert!(config.release.is_none());
    assert!(config.target.is_none());
    assert!(config.cache_dir.is_none());
}

#[test]
fn test_standalone_new_with_target() {
    let config = PythonStandaloneConfig {
        version: "3.12".to_string(),
        release: Some("20241206".to_string()),
        target: Some("x86_64-unknown-linux-gnu".to_string()),
        cache_dir: None,
    };

    let standalone = PythonStandalone::new(config).unwrap();
    assert_eq!(standalone.target(), PythonTarget::LinuxX64);
    assert_eq!(standalone.version(), "3.12");
}

#[test]
fn test_standalone_invalid_target() {
    let config = PythonStandaloneConfig {
        version: "3.11".to_string(),
        release: None,
        target: Some("invalid-target".to_string()),
        cache_dir: None,
    };

    let result = PythonStandalone::new(config);
    assert!(result.is_err());
}

#[test]
fn test_cached_path() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = PythonStandaloneConfig {
        version: "3.11".to_string(),
        release: None,
        target: Some("x86_64-pc-windows-msvc".to_string()),
        cache_dir: Some(temp_dir.path().to_path_buf()),
    };

    let standalone = PythonStandalone::new(config).unwrap();
    let cached = standalone.cached_path();

    assert!(cached.to_string_lossy().contains("cpython-3.11"));
    assert!(cached.to_string_lossy().contains("x86_64-pc-windows-msvc"));
    assert!(cached.to_string_lossy().ends_with(".tar.gz"));
}

#[test]
fn test_download_url_all_targets() {
    let targets = [
        ("x86_64-pc-windows-msvc", "x86_64-pc-windows-msvc"),
        ("x86_64-unknown-linux-gnu", "x86_64-unknown-linux-gnu"),
        ("x86_64-apple-darwin", "x86_64-apple-darwin"),
        ("aarch64-apple-darwin", "aarch64-apple-darwin"),
    ];

    for (target_str, expected) in targets {
        let config = PythonStandaloneConfig {
            version: "3.11".to_string(),
            release: Some("20241206".to_string()),
            target: Some(target_str.to_string()),
            cache_dir: None,
        };

        let standalone = PythonStandalone::new(config).unwrap();
        let url = standalone.download_url();

        assert!(
            url.contains(expected),
            "URL should contain {}: {}",
            expected,
            url
        );
        assert!(url.contains("install_only.tar.gz"));
        assert!(url.starts_with("https://github.com/astral-sh/python-build-standalone"));
    }
}

#[test]
fn test_runtime_meta_serialization() {
    let meta = PythonRuntimeMeta {
        version: "3.11.11".to_string(),
        target: "x86_64-pc-windows-msvc".to_string(),
        archive_size: 50_000_000,
    };

    let json = serde_json::to_string(&meta).unwrap();
    assert!(json.contains("3.11.11"));
    assert!(json.contains("x86_64-pc-windows-msvc"));
    assert!(json.contains("50000000"));

    let parsed: PythonRuntimeMeta = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.version, meta.version);
    assert_eq!(parsed.target, meta.target);
    assert_eq!(parsed.archive_size, meta.archive_size);
}

#[test]
fn test_runtime_cache_dir() {
    let cache_dir = get_runtime_cache_dir("test-app");
    assert!(cache_dir.to_string_lossy().contains("AuroraView"));
    assert!(cache_dir.to_string_lossy().contains("runtime"));
    assert!(cache_dir.to_string_lossy().contains("test-app"));
}

#[test]
fn test_cache_dir_custom() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = PythonStandaloneConfig {
        version: "3.11".to_string(),
        release: None,
        target: Some("x86_64-pc-windows-msvc".to_string()),
        cache_dir: Some(temp_dir.path().to_path_buf()),
    };

    let standalone = PythonStandalone::new(config).unwrap();
    assert_eq!(standalone.cache_dir(), temp_dir.path());
}

// ============================================================================
// Additional coverage: version expansion in download URL
// ============================================================================

#[rstest]
#[case("3.10", "3.10.19")]
#[case("3.11", "3.11.14")]
#[case("3.12", "3.12.12")]
#[case("3.13.0", "3.13.0")]
fn download_url_expands_short_version(#[case] short: &str, #[case] expected_full: &str) {
    let config = PythonStandaloneConfig {
        version: short.to_string(),
        release: Some("20251209".to_string()),
        target: Some("x86_64-pc-windows-msvc".to_string()),
        cache_dir: None,
    };
    let standalone = PythonStandalone::new(config).unwrap();
    let url = standalone.download_url();
    assert!(
        url.contains(expected_full),
        "URL '{}' should contain full version '{}'",
        url,
        expected_full
    );
}

#[rstest]
fn url_contains_release_tag() {
    let config = PythonStandaloneConfig {
        version: "3.11".to_string(),
        release: Some("20251209".to_string()),
        target: Some("x86_64-pc-windows-msvc".to_string()),
        cache_dir: None,
    };
    let standalone = PythonStandalone::new(config).unwrap();
    let url = standalone.download_url();
    assert!(url.contains("20251209"));
    assert!(url.contains("/releases/download/20251209/"));
}

#[rstest]
fn url_uses_default_release_when_none() {
    let config = PythonStandaloneConfig {
        version: "3.11".to_string(),
        release: None, // use default
        target: Some("x86_64-pc-windows-msvc".to_string()),
        cache_dir: None,
    };
    let standalone = PythonStandalone::new(config).unwrap();
    let url = standalone.download_url();
    // Should use the built-in default release date
    assert!(url.contains("20251209"));
}

// ============================================================================
// PythonTarget: Clone, Copy, PartialEq, Debug
// ============================================================================

#[rstest]
fn python_target_clone() {
    let t = PythonTarget::WindowsX64;
    let t2 = t;
    assert_eq!(t, t2);
}

#[rstest]
fn python_target_debug() {
    let s = format!("{:?}", PythonTarget::MacOSArm64);
    assert!(s.contains("MacOSArm64"));
}

#[rstest]
fn python_target_inequality() {
    assert_ne!(PythonTarget::WindowsX64, PythonTarget::LinuxX64);
    assert_ne!(PythonTarget::MacOSX64, PythonTarget::MacOSArm64);
}

// ============================================================================
// PythonRuntimeMeta: Clone, Debug, zero archive size edge case
// ============================================================================

#[rstest]
fn runtime_meta_clone() {
    let meta = PythonRuntimeMeta {
        version: "3.11".to_string(),
        target: "linux".to_string(),
        archive_size: 0,
    };
    let meta2 = meta.clone();
    assert_eq!(meta.version, meta2.version);
    assert_eq!(meta.archive_size, meta2.archive_size);
}

#[rstest]
fn runtime_meta_debug() {
    let meta = PythonRuntimeMeta {
        version: "3.12".to_string(),
        target: "macos".to_string(),
        archive_size: 1024,
    };
    let s = format!("{:?}", meta);
    assert!(s.contains("3.12"));
}

#[rstest]
fn runtime_meta_zero_size_roundtrip() {
    let meta = PythonRuntimeMeta {
        version: "3.11".to_string(),
        target: "x86_64-pc-windows-msvc".to_string(),
        archive_size: 0,
    };
    let json = serde_json::to_string(&meta).unwrap();
    let parsed: PythonRuntimeMeta = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.archive_size, 0);
}

// ============================================================================
// get_runtime_cache_dir: different app names
// ============================================================================

#[rstest]
#[case("my-app")]
#[case("app-v2")]
#[case("special_chars")]
fn cache_dir_contains_app_name(#[case] app_name: &str) {
    let dir = get_runtime_cache_dir(app_name);
    assert!(
        dir.to_string_lossy().contains(app_name),
        "Cache dir should contain app name '{}'",
        app_name
    );
}

#[rstest]
fn cache_dirs_differ_by_app_name() {
    let dir1 = get_runtime_cache_dir("app-alpha");
    let dir2 = get_runtime_cache_dir("app-beta");
    assert_ne!(dir1, dir2);
}

// ============================================================================
// PythonStandalone: target() and version() accessors
// ============================================================================

#[rstest]
#[case("x86_64-pc-windows-msvc", PythonTarget::WindowsX64)]
#[case("x86_64-unknown-linux-gnu", PythonTarget::LinuxX64)]
#[case("x86_64-apple-darwin", PythonTarget::MacOSX64)]
#[case("aarch64-apple-darwin", PythonTarget::MacOSArm64)]
fn standalone_target_accessor(#[case] triple: &str, #[case] expected: PythonTarget) {
    let config = PythonStandaloneConfig {
        version: "3.11".to_string(),
        release: None,
        target: Some(triple.to_string()),
        cache_dir: None,
    };
    let standalone = PythonStandalone::new(config).unwrap();
    assert_eq!(standalone.target(), expected);
}

#[rstest]
#[case("3.9")]
#[case("3.10")]
#[case("3.11")]
#[case("3.12")]
#[case("3.11.9")]
fn standalone_version_accessor(#[case] version: &str) {
    let config = PythonStandaloneConfig {
        version: version.to_string(),
        release: None,
        target: Some("x86_64-pc-windows-msvc".to_string()),
        cache_dir: None,
    };
    let standalone = PythonStandalone::new(config).unwrap();
    assert_eq!(standalone.version(), version);
}

// ============================================================================
// cached_path: uses custom cache_dir as base
// ============================================================================

#[rstest]
fn cached_path_uses_custom_base() {
    let temp = tempfile::tempdir().unwrap();
    let config = PythonStandaloneConfig {
        version: "3.12".to_string(),
        release: None,
        target: Some("aarch64-apple-darwin".to_string()),
        cache_dir: Some(temp.path().to_path_buf()),
    };
    let standalone = PythonStandalone::new(config).unwrap();
    let cached = standalone.cached_path();
    assert!(cached.starts_with(temp.path()));
    assert!(cached.to_string_lossy().contains("aarch64-apple-darwin"));
}

