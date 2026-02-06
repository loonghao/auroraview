//! Tests for auroraview-pack python_standalone module

use auroraview_pack::{
    get_runtime_cache_dir, PythonRuntimeMeta, PythonStandalone, PythonStandaloneConfig,
    PythonTarget,
};

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
