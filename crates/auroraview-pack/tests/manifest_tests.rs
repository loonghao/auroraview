//! Tests for auroraview-pack manifest module

use auroraview_pack::{Manifest, StartPosition};

#[test]
fn test_parse_minimal_manifest() {
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
fn test_parse_full_manifest() {
    let toml = r#"
[package]
name = "my-app"
version = "1.0.0"
description = "My awesome app"
authors = ["Test Author"]

[app]
title = "My Application"
frontend_path = "./dist"
backend_entry = "myapp.main:run"

[window]
width = 1280
height = 720
resizable = true
frameless = false

[bundle]
icon = "./assets/icon.png"
identifier = "com.example.myapp"

[bundle.windows]
icon = "./assets/icon.ico"

[python]
enabled = true
version = "3.11"
entry_point = "myapp.main:run"
packages = ["auroraview", "requests"]

[build]
before_build = ["npm run build"]
after_build = ["echo done"]

[debug]
enabled = true
devtools = true
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert_eq!(manifest.package.name, "my-app");
    assert_eq!(manifest.package.version, "1.0.0");
    assert!(manifest.python.is_some());
    assert!(manifest.is_fullstack());
}

#[test]
fn test_validate_manifest() {
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

#[test]
fn test_start_position_parsing() {
    // Center position
    let toml = r#"
[package]
name = "test"

[app]
title = "Test"
url = "https://example.com"

[window]
start_position = "center"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    assert!(manifest.window.start_position.is_center());

    // Specific position
    let toml = r#"
[package]
name = "test"

[app]
title = "Test"
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
