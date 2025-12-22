//! Tests for auroraview-pack manifest module

use auroraview_pack::{Manifest, StartPosition};

// ============================================================================
// Basic Parsing Tests
// ============================================================================

#[test]
fn test_parse_minimal_manifest() {
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
fn test_parse_frontend_path() {
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
fn test_parse_full_manifest() {
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
fn test_validate_missing_frontend() {
    let toml = r#"
[package]
name = "test"
title = "Test"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert!(manifest.validate().is_err());
}

#[test]
fn test_validate_both_path_and_url() {
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
fn test_validate_valid_config() {
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
fn test_start_position_center() {
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
fn test_start_position_specific() {
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
fn test_backend_type_none() {
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
fn test_backend_type_python() {
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
fn test_backend_type_go() {
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

#[test]
fn test_backend_type_rust() {
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
fn test_backend_type_node() {
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
