//! Integration tests for Packer (PackGenerator)
//!
//! These tests verify the overlay-based packaging functionality.
//!
//! ## Cleanup Mechanism
//!
//! All tests use `tempfile::tempdir()` for creating temporary directories.
//! The `TempDir` type automatically removes the directory when it goes out of scope.

use std::fs;

use auroraview_pack::{BundleStrategy, PackConfig, PackMode, Packer, PythonBundleConfig};
use tempfile::tempdir;

// ============================================================================
// Configuration Tests
// ============================================================================

#[test]
fn test_pack_config_url_mode() {
    let config = PackConfig::url("https://example.com")
        .with_output("test-app")
        .with_title("Test App")
        .with_size(1024, 768);

    assert_eq!(config.output_name, "test-app");
    assert_eq!(config.window.title, "Test App");
    assert_eq!(config.window.width, 1024);
    assert_eq!(config.window.height, 768);
    assert!(matches!(config.mode, PackMode::Url { .. }));
}

#[test]
fn test_pack_config_frontend_mode() {
    let config = PackConfig::frontend("./dist")
        .with_output("frontend-app")
        .with_title("Frontend App");

    assert_eq!(config.output_name, "frontend-app");
    assert!(matches!(config.mode, PackMode::Frontend { .. }));
}

#[test]
fn test_pack_config_fullstack_mode() {
    let config = PackConfig::fullstack("./dist", "main:run")
        .with_output("fullstack-app")
        .with_title("FullStack App");

    assert_eq!(config.output_name, "fullstack-app");
    assert!(matches!(config.mode, PackMode::FullStack { .. }));

    if let PackMode::FullStack { python, .. } = &config.mode {
        assert_eq!(python.entry_point, "main:run");
    }
}

#[test]
fn test_pack_config_builder_pattern() {
    let config = PackConfig::url("example.com")
        .with_output("my-app")
        .with_title("My App")
        .with_size(1920, 1080)
        .with_debug(true)
        .with_frameless(true)
        .with_always_on_top(true)
        .with_resizable(false);

    assert_eq!(config.output_name, "my-app");
    assert_eq!(config.window.title, "My App");
    assert_eq!(config.window.width, 1920);
    assert_eq!(config.window.height, 1080);
    assert!(config.debug);
    assert!(config.window.frameless);
    assert!(config.window.always_on_top);
    assert!(!config.window.resizable);
}

// ============================================================================
// Pack Mode Tests
// ============================================================================

#[test]
fn test_pack_mode_name() {
    let url_mode = PackMode::Url {
        url: "https://example.com".to_string(),
    };
    assert_eq!(url_mode.name(), "url");

    let frontend_mode = PackMode::Frontend {
        path: "./dist".into(),
    };
    assert_eq!(frontend_mode.name(), "frontend");
}

#[test]
fn test_pack_mode_embeds_assets() {
    let url_mode = PackMode::Url {
        url: "https://example.com".to_string(),
    };
    assert!(!url_mode.embeds_assets());

    let frontend_mode = PackMode::Frontend {
        path: "./dist".into(),
    };
    assert!(frontend_mode.embeds_assets());
}

#[test]
fn test_pack_mode_has_python() {
    let url_mode = PackMode::Url {
        url: "https://example.com".to_string(),
    };
    assert!(!url_mode.has_python());

    let frontend_mode = PackMode::Frontend {
        path: "./dist".into(),
    };
    assert!(!frontend_mode.has_python());
}

// ============================================================================
// Packer Validation Tests (via pack() which calls validate internally)
// ============================================================================

#[test]
fn test_packer_url_mode_empty_url() {
    let temp_dir = tempdir().expect("Failed to create temp directory");

    let config = PackConfig::url("")
        .with_output("test-app")
        .with_output_dir(temp_dir.path());

    let packer = Packer::new(config);
    let result = packer.pack();

    assert!(result.is_err(), "Empty URL should fail validation");
}

#[test]
fn test_packer_frontend_mode_not_found() {
    let temp_dir = tempdir().expect("Failed to create temp directory");

    let config = PackConfig::frontend("/nonexistent/path")
        .with_output("test-app")
        .with_output_dir(temp_dir.path());

    let packer = Packer::new(config);
    let result = packer.pack();

    assert!(result.is_err(), "Nonexistent frontend path should fail");
}

#[test]
fn test_packer_frontend_mode_no_index_html() {
    let input_temp = tempdir().expect("Failed to create input temp directory");
    let output_temp = tempdir().expect("Failed to create output temp directory");

    // Create empty directory (no index.html)
    let config = PackConfig::frontend(input_temp.path())
        .with_output("test-app")
        .with_output_dir(output_temp.path());

    let packer = Packer::new(config);
    let result = packer.pack();

    assert!(result.is_err(), "Frontend without index.html should fail");
}

// ============================================================================
// Bundle Builder Tests
// ============================================================================

#[test]
fn test_bundle_builder_directory() {
    use auroraview_pack::BundleBuilder;

    let temp_dir = tempdir().expect("Failed to create temp directory");

    // Create test files
    fs::write(temp_dir.path().join("index.html"), "<html></html>").unwrap();
    fs::write(temp_dir.path().join("style.css"), "body { }").unwrap();
    fs::create_dir(temp_dir.path().join("js")).unwrap();
    fs::write(temp_dir.path().join("js/app.js"), "console.log('hi')").unwrap();

    let bundle = BundleBuilder::new(temp_dir.path()).build().unwrap();

    assert_eq!(bundle.len(), 3);
    assert!(bundle.total_size() > 0);
}

#[test]
fn test_bundle_builder_single_file() {
    use auroraview_pack::BundleBuilder;

    let temp_dir = tempdir().expect("Failed to create temp directory");
    let html_path = temp_dir.path().join("page.html");
    fs::write(&html_path, "<html>test</html>").unwrap();

    let bundle = BundleBuilder::new(&html_path).build().unwrap();

    assert_eq!(bundle.len(), 1);
    assert_eq!(bundle.assets()[0].0, "index.html");
}

#[test]
fn test_bundle_builder_excludes() {
    use auroraview_pack::BundleBuilder;

    let temp_dir = tempdir().expect("Failed to create temp directory");

    fs::write(temp_dir.path().join("index.html"), "<html></html>").unwrap();
    fs::write(temp_dir.path().join("app.js.map"), "sourcemap").unwrap();
    fs::write(temp_dir.path().join(".DS_Store"), "").unwrap();

    let bundle = BundleBuilder::new(temp_dir.path()).build().unwrap();

    // Should only include index.html (excludes .map and .DS_Store)
    assert_eq!(bundle.len(), 1);
    assert_eq!(bundle.assets()[0].0, "index.html");
}

// ============================================================================
// Overlay Tests
// ============================================================================

#[test]
fn test_overlay_roundtrip() {
    use auroraview_pack::{OverlayData, OverlayReader, OverlayWriter};
    use tempfile::NamedTempFile;

    // Create a temp file with some content
    let temp = NamedTempFile::new().unwrap();
    fs::write(temp.path(), b"fake executable content").unwrap();

    // Create overlay data
    let config = PackConfig::url("https://example.com").with_title("Test App");
    let mut data = OverlayData::new(config);
    data.add_asset("index.html", b"<html></html>".to_vec());
    data.add_asset("style.css", b"body { }".to_vec());

    // Write overlay
    OverlayWriter::write(temp.path(), &data).unwrap();

    // Verify overlay exists
    assert!(OverlayReader::has_overlay(temp.path()).unwrap());

    // Read overlay
    let read_data = OverlayReader::read(temp.path()).unwrap().unwrap();
    assert_eq!(read_data.config.window.title, "Test App");
    assert_eq!(read_data.assets.len(), 2);

    // Verify original size
    let original_size = OverlayReader::get_original_size(temp.path())
        .unwrap()
        .unwrap();
    assert_eq!(original_size, b"fake executable content".len() as u64);
}

#[test]
fn test_no_overlay() {
    use auroraview_pack::OverlayReader;
    use tempfile::NamedTempFile;

    let temp = NamedTempFile::new().unwrap();
    fs::write(temp.path(), b"just a regular file").unwrap();

    assert!(!OverlayReader::has_overlay(temp.path()).unwrap());
    assert!(OverlayReader::read(temp.path()).unwrap().is_none());
}

// ============================================================================
// Manifest Tests
// ============================================================================

#[test]
fn test_manifest_parse_minimal() {
    use auroraview_pack::Manifest;

    let toml = r#"
[package]
name = "test-app"

[app]
title = "Test App"
url = "https://example.com"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert_eq!(manifest.package.name, "test-app");
    assert_eq!(manifest.app.title, "Test App");
    assert_eq!(manifest.app.url, Some("https://example.com".to_string()));
}

#[test]
fn test_manifest_parse_fullstack() {
    use auroraview_pack::Manifest;

    let toml = r#"
[package]
name = "my-app"
version = "1.0.0"

[app]
title = "My Application"
frontend_path = "./dist"

[window]
width = 1280
height = 720

[python]
enabled = true
version = "3.11"
entry_point = "main:run"
packages = ["pyyaml", "requests"]

[debug]
enabled = true
devtools = true
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert_eq!(manifest.package.name, "my-app");
    assert!(manifest.is_fullstack());
    assert!(manifest.python.is_some());

    let python = manifest.python.unwrap();
    assert!(python.enabled);
    assert_eq!(python.version, "3.11");
    assert_eq!(python.entry_point, Some("main:run".to_string()));
    assert_eq!(python.packages, vec!["pyyaml", "requests"]);
}

#[test]
fn test_manifest_validate() {
    use auroraview_pack::Manifest;

    // Missing both url and frontend_path
    let toml = r#"
[package]
name = "test"

[app]
title = "Test"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert!(manifest.validate().is_err());

    // Both url and frontend_path specified
    let toml = r#"
[package]
name = "test"

[app]
title = "Test"
url = "https://example.com"
frontend_path = "./dist"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert!(manifest.validate().is_err());
}

// ============================================================================
// FullStack Mode Tests
// ============================================================================

#[test]
fn test_fullstack_mode_has_python() {
    let fullstack_mode = PackMode::FullStack {
        frontend_path: "./dist".into(),
        python: Box::new(PythonBundleConfig::default()),
    };
    assert!(fullstack_mode.has_python());
    assert!(fullstack_mode.embeds_assets());
    assert_eq!(fullstack_mode.name(), "fullstack");
}

#[test]
fn test_fullstack_config_with_strategy() {
    let python_config = PythonBundleConfig {
        entry_point: "main:run".to_string(),
        strategy: BundleStrategy::Standalone,
        version: "3.11".to_string(),
        ..Default::default()
    };

    let config = PackConfig::fullstack_with_config("./dist", python_config);

    if let PackMode::FullStack { python, .. } = &config.mode {
        assert_eq!(python.strategy, BundleStrategy::Standalone);
        assert_eq!(python.version, "3.11");
    } else {
        panic!("Expected FullStack mode");
    }
}

#[test]
fn test_fullstack_config_embedded_strategy() {
    let python_config = PythonBundleConfig {
        entry_point: "app:main".to_string(),
        strategy: BundleStrategy::Embedded,
        ..Default::default()
    };

    let config = PackConfig::fullstack_with_config("./dist", python_config);

    if let PackMode::FullStack { python, .. } = &config.mode {
        assert_eq!(python.strategy, BundleStrategy::Embedded);
    } else {
        panic!("Expected FullStack mode");
    }
}

#[test]
fn test_fullstack_config_with_packages() {
    let python_config = PythonBundleConfig {
        entry_point: "main:run".to_string(),
        packages: vec![
            "pyyaml".to_string(),
            "requests".to_string(),
            "flask".to_string(),
        ],
        ..Default::default()
    };

    let config = PackConfig::fullstack_with_config("./dist", python_config);

    if let PackMode::FullStack { python, .. } = &config.mode {
        assert_eq!(python.packages.len(), 3);
        assert!(python.packages.contains(&"pyyaml".to_string()));
        assert!(python.packages.contains(&"requests".to_string()));
        assert!(python.packages.contains(&"flask".to_string()));
    } else {
        panic!("Expected FullStack mode");
    }
}

#[test]
fn test_packer_fullstack_validation_missing_frontend() {
    let temp_dir = tempdir().expect("Failed to create temp directory");

    let python_config = PythonBundleConfig {
        entry_point: "main:run".to_string(),
        ..Default::default()
    };

    let config = PackConfig::fullstack_with_config("/nonexistent/path", python_config)
        .with_output("test-app")
        .with_output_dir(temp_dir.path());

    let packer = Packer::new(config);
    let result = packer.pack();

    assert!(result.is_err(), "Missing frontend should fail validation");
}

#[test]
fn test_packer_fullstack_validation_empty_entry_point() {
    let frontend_temp = tempdir().expect("Failed to create frontend temp directory");
    let output_temp = tempdir().expect("Failed to create output temp directory");

    // Create index.html
    fs::write(frontend_temp.path().join("index.html"), "<html></html>").unwrap();

    let python_config = PythonBundleConfig {
        entry_point: String::new(), // Empty entry point
        ..Default::default()
    };

    let config = PackConfig::fullstack_with_config(frontend_temp.path(), python_config)
        .with_output("test-app")
        .with_output_dir(output_temp.path());

    let packer = Packer::new(config);
    let result = packer.pack();

    assert!(result.is_err(), "Empty entry point should fail validation");
}

#[test]
fn test_overlay_fullstack_roundtrip() {
    use auroraview_pack::{OverlayData, OverlayReader, OverlayWriter};
    use tempfile::NamedTempFile;

    // Create a temp file with some content
    let temp = NamedTempFile::new().unwrap();
    fs::write(temp.path(), b"fake executable content").unwrap();

    // Create fullstack overlay data
    let python_config = PythonBundleConfig {
        entry_point: "main:run".to_string(),
        strategy: BundleStrategy::Standalone,
        version: "3.11".to_string(),
        packages: vec!["pyyaml".to_string()],
        ..Default::default()
    };

    let config =
        PackConfig::fullstack_with_config("./dist", python_config).with_title("FullStack App");

    let mut data = OverlayData::new(config);
    data.add_asset("frontend/index.html", b"<html></html>".to_vec());
    data.add_asset("frontend/style.css", b"body { }".to_vec());
    data.add_asset("python/main.py", b"def run(): pass".to_vec());

    // Write overlay
    OverlayWriter::write(temp.path(), &data).unwrap();

    // Verify overlay exists
    assert!(OverlayReader::has_overlay(temp.path()).unwrap());

    // Read overlay
    let read_data = OverlayReader::read(temp.path()).unwrap().unwrap();
    assert_eq!(read_data.config.window.title, "FullStack App");
    assert_eq!(read_data.assets.len(), 3);

    // Verify assets
    let asset_paths: Vec<&str> = read_data.assets.iter().map(|(p, _)| p.as_str()).collect();
    assert!(asset_paths.contains(&"frontend/index.html"));
    assert!(asset_paths.contains(&"frontend/style.css"));
    assert!(asset_paths.contains(&"python/main.py"));
}

#[test]
fn test_manifest_parse_fullstack_standalone() {
    use auroraview_pack::Manifest;

    let toml = r#"
[package]
name = "my-fullstack-app"
version = "1.0.0"

[app]
title = "FullStack Application"
frontend_path = "./dist"

[window]
width = 1200
height = 800

[python]
enabled = true
version = "3.11"
entry_point = "main:run_gallery"
packages = ["pyyaml"]
strategy = "standalone"

[debug]
enabled = true
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert_eq!(manifest.package.name, "my-fullstack-app");
    assert!(manifest.is_fullstack());
    assert!(manifest.python.is_some());

    let python = manifest.python.unwrap();
    assert!(python.enabled);
    assert_eq!(python.version, "3.11");
    assert_eq!(python.entry_point, Some("main:run_gallery".to_string()));
    assert_eq!(python.strategy, "standalone");
}

#[test]
fn test_manifest_parse_fullstack_embedded() {
    use auroraview_pack::Manifest;

    let toml = r#"
[package]
name = "embedded-app"

[app]
title = "Embedded App"
frontend_path = "./dist"

[python]
enabled = true
entry_point = "app:main"
strategy = "embedded"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert!(manifest.is_fullstack());

    let python = manifest.python.unwrap();
    assert_eq!(python.strategy, "embedded");
}

#[test]
fn test_bundle_strategy_equality() {
    assert_eq!(BundleStrategy::Standalone, BundleStrategy::Standalone);
    assert_eq!(BundleStrategy::Embedded, BundleStrategy::Embedded);
    assert_eq!(BundleStrategy::Portable, BundleStrategy::Portable);
    assert_eq!(BundleStrategy::System, BundleStrategy::System);
    assert_eq!(BundleStrategy::PyOxidizer, BundleStrategy::PyOxidizer);

    assert_ne!(BundleStrategy::Standalone, BundleStrategy::Embedded);
    assert_ne!(BundleStrategy::Portable, BundleStrategy::System);
}

#[test]
fn test_python_bundle_config_with_include_paths() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let python_dir = temp_dir.path().join("python");
    fs::create_dir(&python_dir).unwrap();
    fs::write(python_dir.join("main.py"), "def run(): pass").unwrap();

    let python_config = PythonBundleConfig {
        entry_point: "main:run".to_string(),
        include_paths: vec![python_dir.clone()],
        ..Default::default()
    };

    assert_eq!(python_config.include_paths.len(), 1);
    assert_eq!(python_config.include_paths[0], python_dir);
}

#[test]
fn test_overlay_with_python_runtime_meta() {
    use auroraview_pack::{OverlayData, OverlayReader, OverlayWriter, PythonRuntimeMeta};
    use tempfile::NamedTempFile;

    let temp = NamedTempFile::new().unwrap();
    fs::write(temp.path(), b"fake executable").unwrap();

    let config = PackConfig::fullstack("./dist", "main:run").with_title("Runtime Test");
    let mut data = OverlayData::new(config);

    // Add Python runtime metadata
    let meta = PythonRuntimeMeta {
        version: "3.11.11".to_string(),
        target: "x86_64-pc-windows-msvc".to_string(),
        archive_size: 50_000_000,
    };
    let meta_json = serde_json::to_vec(&meta).unwrap();
    data.add_asset("python_runtime.json", meta_json);

    // Add frontend and python assets
    data.add_asset("frontend/index.html", b"<html></html>".to_vec());
    data.add_asset("python/main.py", b"def run(): pass".to_vec());

    OverlayWriter::write(temp.path(), &data).unwrap();

    let read_data = OverlayReader::read(temp.path()).unwrap().unwrap();
    assert_eq!(read_data.assets.len(), 3);

    // Find and verify runtime metadata
    let meta_asset = read_data
        .assets
        .iter()
        .find(|(p, _)| p == "python_runtime.json");
    assert!(meta_asset.is_some());

    let (_, content) = meta_asset.unwrap();
    let parsed_meta: PythonRuntimeMeta = serde_json::from_slice(content).unwrap();
    assert_eq!(parsed_meta.version, "3.11.11");
    assert_eq!(parsed_meta.target, "x86_64-pc-windows-msvc");
}

// ============================================================================
// Python Standalone Tests
// ============================================================================

#[test]
fn test_python_target_all_platforms() {
    use auroraview_pack::PythonTarget;

    let targets = [
        (
            PythonTarget::WindowsX64,
            "x86_64-pc-windows-msvc",
            "python.exe",
        ),
        (
            PythonTarget::LinuxX64,
            "x86_64-unknown-linux-gnu",
            "python3",
        ),
        (PythonTarget::MacOSX64, "x86_64-apple-darwin", "python3"),
        (PythonTarget::MacOSArm64, "aarch64-apple-darwin", "python3"),
    ];

    for (target, expected_triple, expected_exe) in targets {
        assert_eq!(target.triple(), expected_triple);
        assert_eq!(target.python_exe(), expected_exe);
    }
}

#[test]
fn test_python_standalone_config() {
    use auroraview_pack::{PythonStandalone, PythonStandaloneConfig};

    let config = PythonStandaloneConfig {
        version: "3.12".to_string(),
        release: Some("20241206".to_string()),
        target: Some("x86_64-pc-windows-msvc".to_string()),
        cache_dir: None,
    };

    let standalone = PythonStandalone::new(config).unwrap();
    assert_eq!(standalone.version(), "3.12");

    let url = standalone.download_url();
    assert!(url.contains("cpython-3.12"));
    assert!(url.contains("20241206"));
    assert!(url.contains("x86_64-pc-windows-msvc"));
    assert!(url.contains("install_only.tar.gz"));
}

#[test]
fn test_python_standalone_cache_dir() {
    use auroraview_pack::{PythonStandalone, PythonStandaloneConfig};

    let temp_dir = tempdir().expect("Failed to create temp directory");

    let config = PythonStandaloneConfig {
        version: "3.11".to_string(),
        release: None,
        target: Some("x86_64-unknown-linux-gnu".to_string()),
        cache_dir: Some(temp_dir.path().to_path_buf()),
    };

    let standalone = PythonStandalone::new(config).unwrap();
    assert_eq!(standalone.cache_dir(), temp_dir.path());

    let cached_path = standalone.cached_path();
    assert!(cached_path.to_string_lossy().contains("cpython-3.11"));
    assert!(cached_path
        .to_string_lossy()
        .contains("x86_64-unknown-linux-gnu"));
}
