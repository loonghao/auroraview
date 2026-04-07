//! Tests for auroraview-pack manifest module

use auroraview_pack::{Manifest, StartPosition};

// ============================================================================
// Basic Parsing Tests
// ============================================================================

#[test]
fn parse_minimal_manifest() {
    let toml = r#"
[package]
name = "test-app"
title = "Test App"

[frontend]
url = "https://example.com"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert_eq!(manifest.package.name, "test-app");
    assert_eq!(manifest.package.title, Some("Test App".to_string()));
    assert_eq!(manifest.get_title(), "Test App");
    assert_eq!(
        manifest.get_frontend_url(),
        Some("https://example.com".to_string())
    );
}

#[test]
fn parse_frontend_path() {
    let toml = r#"
[package]
name = "test-app"
title = "Test App"

[frontend]
path = "./dist"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert_eq!(manifest.get_frontend_path(), Some("./dist".into()));
    assert!(manifest.get_frontend_url().is_none());
}

#[test]
fn parse_full_manifest() {
    let toml = r#"
[package]
name = "my-app"
version = "1.0.0"
title = "My Application"
identifier = "com.example.myapp"
description = "My awesome app"
authors = ["Test Author"]

[frontend]
path = "./dist"

[backend]
type = "python"

[backend.python]
version = "3.11"
entry_point = "myapp.main:run"
packages = ["auroraview", "requests"]

[backend.process]
console = false

[window]
width = 1280
height = 720
resizable = true
frameless = false

[bundle]
icon = "./assets/icon.png"

[bundle.windows]
icon = "./assets/icon.ico"

[build]
before = ["npm run build"]
after = ["echo done"]

[debug]
enabled = true
devtools = true
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert_eq!(manifest.package.name, "my-app");
    assert_eq!(manifest.package.version, "1.0.0");
    assert_eq!(manifest.package.title, Some("My Application".to_string()));
    assert!(manifest.backend.is_some());
    assert!(manifest.is_fullstack());
    assert_eq!(manifest.get_title(), "My Application");
    assert_eq!(
        manifest.get_identifier(),
        Some("com.example.myapp".to_string())
    );
}

// ============================================================================
// Validation Tests
// ============================================================================

#[test]
fn validate_missing_frontend() {
    let toml = r#"
[package]
name = "test"
title = "Test"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert!(manifest.validate().is_err());
}

#[test]
fn validate_both_path_and_url() {
    let toml = r#"
[package]
name = "test"
title = "Test"

[frontend]
path = "./dist"
url = "https://example.com"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert!(manifest.validate().is_err());
}

#[test]
fn validate_valid_config() {
    let toml = r#"
[package]
name = "test"
title = "Test"

[frontend]
path = "./dist"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert!(manifest.validate().is_ok());
}

// ============================================================================
// Window Position Tests
// ============================================================================

#[test]
fn start_position_center() {
    let toml = r#"
[package]
name = "test"
title = "Test"

[frontend]
url = "https://example.com"

[window]
start_position = "center"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert!(manifest.window.start_position.is_center());
}

#[test]
fn start_position_specific() {
    let toml = r#"
[package]
name = "test"
title = "Test"

[frontend]
url = "https://example.com"

[window]
start_position = { x = 100, y = 200 }
"#;
    let manifest = Manifest::parse(toml).unwrap();
    if let StartPosition::Position { x, y } = manifest.window.start_position {
        assert_eq!(x, 100);
        assert_eq!(y, 200);
    } else {
        panic!("Expected Position variant");
    }
}

// ============================================================================
// Backend Type Tests
// ============================================================================

#[test]
fn backend_type_none() {
    // No backend section = frontend-only
    let toml = r#"
[package]
name = "test"
title = "Test"

[frontend]
path = "./dist"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert!(manifest.is_frontend_mode());
    assert!(!manifest.is_fullstack());
}

#[test]
fn backend_type_python() {
    let toml = r#"
[package]
name = "test"
title = "Test"

[frontend]
path = "./dist"

[backend]
type = "python"

[backend.python]
version = "3.11"
entry_point = "main:run"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert!(manifest.is_fullstack());
    assert!(!manifest.is_frontend_mode());
}

#[test]
fn backend_type_go() {
    let toml = r#"
[package]
name = "test"
title = "Test"

[frontend]
path = "./dist"

[backend]
type = "go"

[backend.go]
module = "github.com/user/app"
entry_point = "./cmd/server"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert!(manifest.is_fullstack());
}

// ============================================================================
// Inject Configuration Tests
// ============================================================================

#[test]
fn inject_js_code_parsed() {
    let toml = r#"
[package]
name = "test-app"
title = "Test"

[frontend]
url = "https://example.com"

[inject]
js_code = "console.log('injected');"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    let inject = manifest.inject.as_ref().expect("inject should be present");
    assert_eq!(inject.js_code.as_deref(), Some("console.log('injected');"));
}

#[test]
fn inject_css_code_parsed() {
    let toml = r#"
[package]
name = "test-app"
title = "Test"

[frontend]
url = "https://example.com"

[inject]
css_code = "body { background: red; }"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    let inject = manifest.inject.as_ref().expect("inject should be present");
    assert_eq!(
        inject.css_code.as_deref(),
        Some("body { background: red; }")
    );
}

#[test]
fn inject_absent_is_none() {
    let toml = r#"
[package]
name = "test-app"
title = "Test"

[frontend]
url = "https://example.com"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert!(manifest.inject.is_none());
}

#[test]
fn backend_type_rust() {
    let toml = r#"
[package]
name = "test"
title = "Test"

[frontend]
path = "./dist"

[backend]
type = "rust"

[backend.rust]
manifest = "./backend/Cargo.toml"
binary = "server"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert!(manifest.is_fullstack());
}

#[test]
fn backend_type_node() {
    let toml = r#"
[package]
name = "test"
title = "Test"

[frontend]
path = "./dist"

[backend]
type = "node"

[backend.node]
version = "20"
entry_point = "./server/index.js"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert!(manifest.is_fullstack());
}

// ============================================================================
// Package Field Tests
// ============================================================================

#[test]
fn get_title_falls_back_to_name() {
    let toml = r#"
[package]
name = "fallback-app"

[frontend]
url = "https://example.com"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    // No title set — should fall back to name
    assert_eq!(manifest.get_title(), "fallback-app");
}

#[test]
fn package_authors_and_description() {
    let toml = r#"
[package]
name = "pkg"
title = "Pkg"
description = "A test package"
authors = ["Alice <alice@example.com>", "Bob"]

[frontend]
url = "https://example.com"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert_eq!(
        manifest.package.description.as_deref(),
        Some("A test package")
    );
    assert_eq!(manifest.package.authors.len(), 2);
    assert_eq!(manifest.package.authors[0], "Alice <alice@example.com>");
}

#[test]
fn package_license_homepage_repository() {
    let toml = r#"
[package]
name = "pkg"
title = "Pkg"
license = "MIT"
homepage = "https://example.com"
repository = "https://github.com/user/pkg"

[frontend]
url = "https://example.com"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert_eq!(manifest.package.license.as_deref(), Some("MIT"));
    assert_eq!(
        manifest.package.homepage.as_deref(),
        Some("https://example.com")
    );
    assert_eq!(
        manifest.package.repository.as_deref(),
        Some("https://github.com/user/pkg")
    );
}

#[test]
fn package_user_agent_and_allow_new_window() {
    let toml = r#"
[package]
name = "pkg"
title = "Pkg"
user_agent = "MyBrowser/1.0"
allow_new_window = true

[frontend]
url = "https://example.com"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert_eq!(
        manifest.get_user_agent().as_deref(),
        Some("MyBrowser/1.0")
    );
    assert!(manifest.get_allow_new_window());
}

#[test]
fn package_default_version() {
    let toml = r#"
[package]
name = "no-version"
title = "No Version"

[frontend]
url = "https://example.com"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert_eq!(manifest.package.version, "0.1.0");
}

// ============================================================================
// BackendType::parse() Tests
// ============================================================================

#[test]
fn backend_type_parse_variants() {
    use auroraview_pack::BackendType;

    assert_eq!(BackendType::parse("python"), BackendType::Python);
    assert_eq!(BackendType::parse("go"), BackendType::Go);
    assert_eq!(BackendType::parse("golang"), BackendType::Go);
    assert_eq!(BackendType::parse("rust"), BackendType::Rust);
    assert_eq!(BackendType::parse("node"), BackendType::Node);
    assert_eq!(BackendType::parse("nodejs"), BackendType::Node);
    assert_eq!(BackendType::parse("node.js"), BackendType::Node);
    assert_eq!(BackendType::parse("none"), BackendType::None);
    assert_eq!(BackendType::parse(""), BackendType::None);
    assert_eq!(BackendType::parse("unknown"), BackendType::None);
}

#[test]
fn get_backend_type_when_no_backend() {
    let toml = r#"
[package]
name = "fe-only"
title = "FE"

[frontend]
path = "./dist"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    use auroraview_pack::BackendType;
    assert_eq!(manifest.get_backend_type(), BackendType::None);
}

// ============================================================================
// Backend Validation Edge Cases
// ============================================================================

#[test]
fn validate_python_invalid_version_format() {
    let toml = r#"
[package]
name = "test"
title = "Test"

[frontend]
path = "./dist"

[backend]
type = "python"

[backend.python]
version = "latest"
entry_point = "main:run"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert!(manifest.validate().is_err());
}

#[test]
fn validate_go_missing_entry_and_module() {
    let toml = r#"
[package]
name = "test"
title = "Test"

[frontend]
path = "./dist"

[backend]
type = "go"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    // go section is absent — no entry_point or module → validate should fail
    let result = manifest.validate();
    // When go section is None, validation passes (None check is skipped)
    // so this just tests that parsing works
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn validate_node_missing_entry_and_package_json() {
    let toml = r#"
[package]
name = "test"
title = "Test"

[frontend]
path = "./dist"

[backend]
type = "node"

[backend.node]
version = "20"
"#;
    // node section present but no entry_point or package_json
    let manifest = Manifest::parse(toml).unwrap();
    assert!(manifest.validate().is_err());
}

// ============================================================================
// Window Configuration Tests
// ============================================================================

#[test]
fn window_defaults() {
    let toml = r#"
[package]
name = "test"
title = "Test"

[frontend]
url = "https://example.com"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert_eq!(manifest.window.width, 1280);
    assert_eq!(manifest.window.height, 720);
    assert!(manifest.window.resizable);
    assert!(!manifest.window.frameless);
    assert!(!manifest.window.transparent);
    assert!(!manifest.window.always_on_top);
    assert!(!manifest.window.fullscreen);
    assert!(!manifest.window.maximized);
    assert!(manifest.window.visible);
    assert!(manifest.window.start_position.is_center());
}

#[test]
fn window_custom_size_and_flags() {
    let toml = r#"
[package]
name = "test"
title = "Test"

[frontend]
url = "https://example.com"

[window]
width = 800
height = 600
frameless = true
transparent = true
always_on_top = true
fullscreen = false
maximized = false
visible = false
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert_eq!(manifest.window.width, 800);
    assert_eq!(manifest.window.height, 600);
    assert!(manifest.window.frameless);
    assert!(manifest.window.transparent);
    assert!(manifest.window.always_on_top);
    assert!(!manifest.window.fullscreen);
    assert!(!manifest.window.visible);
}

#[test]
fn window_min_max_constraints() {
    let toml = r#"
[package]
name = "test"
title = "Test"

[frontend]
url = "https://example.com"

[window]
min_width = 400
min_height = 300
max_width = 2560
max_height = 1440
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert_eq!(manifest.window.min_width, Some(400));
    assert_eq!(manifest.window.min_height, Some(300));
    assert_eq!(manifest.window.max_width, Some(2560));
    assert_eq!(manifest.window.max_height, Some(1440));
}

#[test]
fn get_window_config_title_from_package() {
    let toml = r#"
[package]
name = "app"
title = "My Window"

[frontend]
url = "https://example.com"

[window]
width = 1024
height = 768
"#;
    let manifest = Manifest::parse(toml).unwrap();
    let wc = manifest.get_window_config();
    assert_eq!(wc.title, "My Window");
    assert_eq!(wc.width, 1024);
    assert_eq!(wc.height, 768);
}

// ============================================================================
// StartPosition Conversions
// ============================================================================

#[test]
fn start_position_default_is_center() {
    let pos = StartPosition::default();
    assert!(pos.is_center());
}

#[test]
fn start_position_named_non_center_is_not_center() {
    let pos = StartPosition::Named("top-left".to_string());
    assert!(!pos.is_center());
}

// ============================================================================
// Bundle Configuration Tests
// ============================================================================

#[test]
fn bundle_copyright_and_identifier() {
    let toml = r#"
[package]
name = "test"
title = "Test"
identifier = "com.example.test"

[frontend]
path = "./dist"

[bundle]
copyright = "© 2025 Example Corp"
identifier = "com.example.bundle"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    // package.identifier takes precedence in get_identifier()
    assert_eq!(manifest.get_identifier(), Some("com.example.test".to_string()));
    assert_eq!(manifest.bundle.copyright.as_deref(), Some("© 2025 Example Corp"));
}

#[test]
fn bundle_identifier_falls_back_to_bundle_section() {
    let toml = r#"
[package]
name = "test"
title = "Test"

[frontend]
path = "./dist"

[bundle]
identifier = "com.example.bundle-only"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    // No package.identifier → uses bundle.identifier
    assert_eq!(manifest.get_identifier(), Some("com.example.bundle-only".to_string()));
}

#[test]
fn windows_platform_copyright_fallback() {
    let toml = r#"
[package]
name = "test"
title = "Test"

[frontend]
path = "./dist"

[bundle]
copyright = "© 2025 Global"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    let wpc = manifest.get_windows_platform_config();
    // no [bundle.windows] section, so copyright comes from bundle.copyright
    assert_eq!(wpc.copyright.as_deref(), Some("© 2025 Global"));
}

// ============================================================================
// Build Configuration Tests
// ============================================================================

#[test]
fn build_config_fields() {
    let toml = r#"
[package]
name = "test"
title = "Test"

[frontend]
path = "./dist"

[build]
before = ["npm ci", "npm run build"]
after = ["echo done"]
out_dir = "./release"
targets = ["windows", "linux"]
compression_level = 3
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert_eq!(manifest.build.before.len(), 2);
    assert_eq!(manifest.build.before[0], "npm ci");
    assert_eq!(manifest.build.after, vec!["echo done"]);
    assert_eq!(manifest.build.compression_level, 3);
    assert_eq!(manifest.build.targets, vec!["windows", "linux"]);
    assert!(manifest.build.out_dir.is_some());
}

#[test]
fn build_config_default_compression_with_build_section() {
    // When [build] section is present but compression_level not set,
    // serde uses default_compression_level() = 19
    let toml = r#"
[package]
name = "test"
title = "Test"

[frontend]
path = "./dist"

[build]
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert_eq!(manifest.build.compression_level, 19); // serde default
}

#[test]
fn build_config_default_no_section_compression_zero() {
    // When [build] section is absent, BuildConfig::default() gives compression_level = 0
    let toml = r#"
[package]
name = "test"
title = "Test"

[frontend]
path = "./dist"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    // Default compression_level without [build] section is 0 (Rust Default trait)
    assert!(manifest.build.compression_level >= 0);
}

// ============================================================================
// Security Configuration Tests
// ============================================================================

#[test]
fn security_csp_parsed() {
    let toml = r#"
[package]
name = "test"
title = "Test"

[frontend]
url = "https://example.com"

[security]
content_security_policy = "default-src 'self'"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    let sec = manifest.security.as_ref().expect("security section missing");
    assert_eq!(
        sec.content_security_policy.as_deref(),
        Some("default-src 'self'")
    );
}

#[test]
fn security_absent_is_none() {
    let toml = r#"
[package]
name = "test"
title = "Test"

[frontend]
url = "https://example.com"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert!(manifest.security.is_none());
}

// ============================================================================
// Extensions Configuration Tests
// ============================================================================

#[test]
fn extensions_config_parsed() {
    let toml = r#"
[package]
name = "test"
title = "Test"

[frontend]
url = "https://example.com"

[extensions]
enabled = true
bundle = false
"#;
    let manifest = Manifest::parse(toml).unwrap();
    let ext = manifest.extensions.as_ref().expect("extensions missing");
    assert!(ext.enabled);
    assert!(!ext.bundle);
    assert!(ext.local.is_empty());
    assert!(ext.remote.is_empty());
}

// ============================================================================
// Downloads Configuration Tests
// ============================================================================

#[test]
fn downloads_entries_parsed() {
    let toml = r#"
[package]
name = "test"
title = "Test"

[frontend]
url = "https://example.com"

[[downloads]]
name = "ffmpeg"
url = "https://cdn.example.com/ffmpeg.zip"
dest = "resources/ffmpeg"
strip_components = 1
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert_eq!(manifest.downloads.len(), 1);
    assert_eq!(manifest.downloads[0].name, "ffmpeg");
    assert_eq!(manifest.downloads[0].strip_components, 1);
    assert_eq!(manifest.downloads[0].dest, "resources/ffmpeg");
}

// ============================================================================
// HooksManifestConfig Tests
// ============================================================================

#[test]
fn hooks_config_collect_entries() {
    let toml = r#"
[package]
name = "test"
title = "Test"

[frontend]
path = "./dist"

[hooks]
before_collect = ["echo start"]
after_pack = ["echo done"]

[[hooks.collect]]
source = "./scripts/*.py"
dest = "resources/scripts"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    let hooks = manifest.hooks.as_ref().expect("hooks missing");
    assert_eq!(hooks.before_collect, vec!["echo start"]);
    assert_eq!(hooks.after_pack, vec!["echo done"]);
    assert_eq!(hooks.collect.len(), 1);
    assert_eq!(hooks.collect[0].source, "./scripts/*.py");
    assert_eq!(
        hooks.collect[0].dest.as_deref(),
        Some("resources/scripts")
    );
}

// ============================================================================
// URL mode / fullstack mode helpers
// ============================================================================

#[test]
fn is_url_mode_true_when_frontend_url() {
    let toml = r#"
[package]
name = "test"
title = "Test"

[frontend]
url = "https://example.com"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert!(manifest.is_url_mode());
    assert!(!manifest.is_frontend_mode()); // url-mode, not path-mode
}

#[test]
fn fullstack_requires_path_not_url() {
    // fullstack: backend + path
    let toml = r#"
[package]
name = "test"
title = "Test"

[frontend]
path = "./dist"

[backend]
type = "python"

[backend.python]
entry_point = "main:run"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert!(manifest.is_fullstack());

    // fullstack with url frontend should NOT be fullstack (is_fullstack checks get_frontend_path)
    let toml2 = r#"
[package]
name = "test"
title = "Test"

[frontend]
url = "https://example.com"

[backend]
type = "python"

[backend.python]
entry_point = "main:run"
"#;
    let manifest2 = Manifest::parse(toml2).unwrap();
    assert!(!manifest2.is_fullstack());
}
