//! Integration tests for PackGenerator
//!
//! ## Cleanup Mechanism
//!
//! All tests in this module use `tempfile::tempdir()` for creating temporary directories.
//! The `TempDir` type implements `Drop` trait, which automatically removes the directory
//! and all its contents when the `TempDir` instance goes out of scope (test function ends).
//!
//! This ensures:
//! - No temporary files are left in the project directory after tests
//! - Failed tests still clean up their temporary directories
//! - No manual cleanup code is needed

use std::fs;
use std::process::Command;

use auroraview_pack::{PackConfig, PackGenerator};
use tempfile::tempdir;

// ============================================================================
// Configuration Validation Tests
// ============================================================================

#[test]
fn test_pack_config_url_mode() {
    let config = PackConfig::url("https://example.com")
        .with_output("test-app")
        .with_title("Test App")
        .with_size(1024, 768);

    let generator = PackGenerator::new(config);
    assert!(generator.validate().is_ok());
}

#[test]
fn test_pack_config_url_mode_invalid() {
    let config = PackConfig::url("invalid") // No dots, no scheme
        .with_output("test-app");

    let generator = PackGenerator::new(config);
    assert!(generator.validate().is_err());
}

#[test]
fn test_pack_config_frontend_mode_not_found() {
    let config = PackConfig::frontend("/nonexistent/path").with_output("test-app");

    let generator = PackGenerator::new(config);
    assert!(generator.validate().is_err());
}

// ============================================================================
// End-to-End URL Mode Tests
// ============================================================================

/// Test complete URL mode packaging flow
///
/// This test:
/// 1. Creates a temporary output directory
/// 2. Generates a URL mode project with custom settings
/// 3. Verifies all expected files are created
/// 4. Verifies file contents are correct
/// 5. Optionally runs cargo check to ensure the project compiles
#[test]
fn test_pack_generator_url_mode() {
    // Create temporary directory for output (auto-cleaned on drop)
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let output_dir = temp_dir.path();

    // Configure URL mode pack
    let config = PackConfig::url("https://github.com")
        .with_output("test-url-app")
        .with_output_dir(output_dir)
        .with_title("GitHub Viewer")
        .with_size(1280, 720);

    // Generate the project
    let generator = PackGenerator::new(config);
    let result = generator.generate();
    assert!(result.is_ok(), "Generation failed: {:?}", result.err());

    let project_dir = result.unwrap();

    // Verify project structure
    assert!(project_dir.exists(), "Project directory should exist");
    assert!(
        project_dir.join("Cargo.toml").exists(),
        "Cargo.toml should exist"
    );
    assert!(
        project_dir.join("src").exists(),
        "src directory should exist"
    );
    assert!(
        project_dir.join("src/main.rs").exists(),
        "src/main.rs should exist"
    );

    // URL mode should NOT have assets directory
    assert!(
        !project_dir.join("assets").exists(),
        "URL mode should not have assets directory"
    );

    // Verify Cargo.toml content
    let cargo_toml =
        fs::read_to_string(project_dir.join("Cargo.toml")).expect("Failed to read Cargo.toml");
    assert!(
        cargo_toml.contains("name = \"test-url-app\""),
        "Cargo.toml should contain project name"
    );
    assert!(
        cargo_toml.contains("wry"),
        "Cargo.toml should include wry dependency"
    );
    assert!(
        cargo_toml.contains("tao"),
        "Cargo.toml should include tao dependency"
    );
    // URL mode should NOT have rust-embed
    assert!(
        !cargo_toml.contains("rust-embed"),
        "URL mode Cargo.toml should not include rust-embed"
    );

    // Verify main.rs content
    let main_rs =
        fs::read_to_string(project_dir.join("src/main.rs")).expect("Failed to read main.rs");
    assert!(
        main_rs.contains("https://github.com"),
        "main.rs should contain the target URL"
    );
    assert!(
        main_rs.contains("GitHub Viewer"),
        "main.rs should contain window title"
    );
    assert!(
        main_rs.contains("1280") && main_rs.contains("720"),
        "main.rs should contain window dimensions"
    );

    // TempDir automatically cleaned up when `temp_dir` goes out of scope
}

/// Test URL mode with minimal configuration
#[test]
fn test_pack_generator_url_mode_minimal() {
    let temp_dir = tempdir().expect("Failed to create temp directory");

    let config = PackConfig::url("example.com")
        .with_output("minimal-app")
        .with_output_dir(temp_dir.path());

    let generator = PackGenerator::new(config);
    let result = generator.generate();
    assert!(result.is_ok(), "Minimal URL mode generation should work");

    let project_dir = result.unwrap();

    // Verify URL is normalized with https://
    let main_rs =
        fs::read_to_string(project_dir.join("src/main.rs")).expect("Failed to read main.rs");
    assert!(
        main_rs.contains("https://example.com"),
        "URL should be normalized with https:// prefix"
    );
}

// ============================================================================
// End-to-End Frontend Mode Tests
// ============================================================================

/// Test complete Frontend mode packaging flow with a directory
///
/// This test:
/// 1. Creates temporary directories for input (frontend) and output
/// 2. Creates test HTML/CSS/JS files
/// 3. Generates a Frontend mode project
/// 4. Verifies all expected files are created including assets
/// 5. Verifies assets are correctly copied
#[test]
fn test_pack_generator_frontend_mode_directory() {
    // Create temporary directories (auto-cleaned on drop)
    let input_temp = tempdir().expect("Failed to create input temp directory");
    let output_temp = tempdir().expect("Failed to create output temp directory");

    let frontend_dir = input_temp.path();
    let output_dir = output_temp.path();

    // Create test frontend files
    let index_html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Test App</title>
    <link rel="stylesheet" href="styles.css">
</head>
<body>
    <h1>Hello AuroraView!</h1>
    <script src="app.js"></script>
</body>
</html>"#;

    let styles_css = r#"body { font-family: sans-serif; margin: 0; padding: 20px; }
h1 { color: #333; }"#;

    let app_js = r#"console.log('AuroraView Frontend Mode');
document.addEventListener('DOMContentLoaded', () => {
    console.log('App loaded');
});"#;

    fs::write(frontend_dir.join("index.html"), index_html).expect("Failed to write index.html");
    fs::write(frontend_dir.join("styles.css"), styles_css).expect("Failed to write styles.css");
    fs::write(frontend_dir.join("app.js"), app_js).expect("Failed to write app.js");

    // Create a subdirectory with additional assets
    let img_dir = frontend_dir.join("images");
    fs::create_dir_all(&img_dir).expect("Failed to create images directory");
    fs::write(img_dir.join("logo.svg"), "<svg></svg>").expect("Failed to write logo.svg");

    // Configure Frontend mode pack
    let config = PackConfig::frontend(frontend_dir)
        .with_output("test-frontend-app")
        .with_output_dir(output_dir)
        .with_title("Frontend Test App")
        .with_size(1024, 768);

    // Generate the project
    let generator = PackGenerator::new(config);
    let result = generator.generate();
    assert!(result.is_ok(), "Generation failed: {:?}", result.err());

    let project_dir = result.unwrap();

    // Verify project structure
    assert!(project_dir.exists(), "Project directory should exist");
    assert!(
        project_dir.join("Cargo.toml").exists(),
        "Cargo.toml should exist"
    );
    assert!(
        project_dir.join("src/main.rs").exists(),
        "src/main.rs should exist"
    );
    assert!(
        project_dir.join("assets").exists(),
        "assets directory should exist"
    );

    // Verify assets are copied correctly
    let assets_dir = project_dir.join("assets");
    assert!(
        assets_dir.join("index.html").exists(),
        "index.html should be copied"
    );
    assert!(
        assets_dir.join("styles.css").exists(),
        "styles.css should be copied"
    );
    assert!(
        assets_dir.join("app.js").exists(),
        "app.js should be copied"
    );
    assert!(
        assets_dir.join("images/logo.svg").exists(),
        "Subdirectory assets should be copied"
    );

    // Verify copied content is correct
    let copied_html =
        fs::read_to_string(assets_dir.join("index.html")).expect("Failed to read copied index");
    assert!(
        copied_html.contains("Hello AuroraView!"),
        "Copied HTML should have correct content"
    );

    // Verify Cargo.toml has rust-embed
    let cargo_toml =
        fs::read_to_string(project_dir.join("Cargo.toml")).expect("Failed to read Cargo.toml");
    assert!(
        cargo_toml.contains("rust-embed"),
        "Frontend mode Cargo.toml should include rust-embed"
    );

    // Verify main.rs has embedded assets code
    let main_rs =
        fs::read_to_string(project_dir.join("src/main.rs")).expect("Failed to read main.rs");
    assert!(
        main_rs.contains("RustEmbed") || main_rs.contains("rust_embed"),
        "Frontend mode main.rs should use rust-embed"
    );
    assert!(
        main_rs.contains("Frontend Test App"),
        "main.rs should contain window title"
    );
}

/// Test Frontend mode with a single HTML file (not a directory)
#[test]
fn test_pack_generator_frontend_mode_single_file() {
    let input_temp = tempdir().expect("Failed to create input temp directory");
    let output_temp = tempdir().expect("Failed to create output temp directory");

    // Create a single HTML file
    let html_file = input_temp.path().join("single-page.html");
    let html_content = r#"<!DOCTYPE html>
<html>
<head><title>Single Page</title></head>
<body><h1>Single Page App</h1></body>
</html>"#;
    fs::write(&html_file, html_content).expect("Failed to write HTML file");

    let config = PackConfig::frontend(&html_file)
        .with_output("single-file-app")
        .with_output_dir(output_temp.path());

    let generator = PackGenerator::new(config);
    let result = generator.generate();
    assert!(result.is_ok(), "Single file mode should work");

    let project_dir = result.unwrap();

    // Single file should be copied as index.html in assets
    assert!(
        project_dir.join("assets/index.html").exists(),
        "Single file should be copied as index.html"
    );

    let copied_content = fs::read_to_string(project_dir.join("assets/index.html"))
        .expect("Failed to read copied file");
    assert!(
        copied_content.contains("Single Page App"),
        "Content should be preserved"
    );
}

// ============================================================================
// Cargo Check Compilation Tests (Optional - skipped if cargo not available)
// ============================================================================

/// Test that generated URL mode project can be checked by cargo
///
/// This test is more expensive as it runs cargo check, but ensures
/// the generated code is syntactically correct and compiles.
#[test]
fn test_pack_generator_url_mode_cargo_check() {
    // Skip if cargo is not available
    if Command::new("cargo").arg("--version").output().is_err() {
        eprintln!("Skipping cargo check test: cargo not available");
        return;
    }

    let temp_dir = tempdir().expect("Failed to create temp directory");

    let config = PackConfig::url("https://rust-lang.org")
        .with_output("cargo-check-app")
        .with_output_dir(temp_dir.path())
        .with_title("Cargo Check Test");

    let generator = PackGenerator::new(config);
    let project_dir = generator.generate().expect("Generation should succeed");

    // Run cargo check on the generated project
    let output = Command::new("cargo")
        .arg("check")
        .current_dir(&project_dir)
        .output()
        .expect("Failed to run cargo check");

    assert!(
        output.status.success(),
        "cargo check should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Test that generated Frontend mode project can be checked by cargo
#[test]
fn test_pack_generator_frontend_mode_cargo_check() {
    // Skip if cargo is not available
    if Command::new("cargo").arg("--version").output().is_err() {
        eprintln!("Skipping cargo check test: cargo not available");
        return;
    }

    let input_temp = tempdir().expect("Failed to create input temp directory");
    let output_temp = tempdir().expect("Failed to create output temp directory");

    // Create minimal frontend
    fs::write(
        input_temp.path().join("index.html"),
        "<!DOCTYPE html><html><body>Test</body></html>",
    )
    .expect("Failed to write index.html");

    let config = PackConfig::frontend(input_temp.path())
        .with_output("cargo-check-frontend")
        .with_output_dir(output_temp.path());

    let generator = PackGenerator::new(config);
    let project_dir = generator.generate().expect("Generation should succeed");

    // Run cargo check on the generated project
    let output = Command::new("cargo")
        .arg("check")
        .current_dir(&project_dir)
        .output()
        .expect("Failed to run cargo check");

    assert!(
        output.status.success(),
        "cargo check should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// ============================================================================
// Edge Cases and Error Handling Tests
// ============================================================================

/// Test that generator handles empty frontend directory gracefully
#[test]
fn test_pack_generator_frontend_mode_empty_directory() {
    let input_temp = tempdir().expect("Failed to create input temp directory");
    let output_temp = tempdir().expect("Failed to create output temp directory");

    // Empty directory - no index.html
    let config = PackConfig::frontend(input_temp.path())
        .with_output("empty-frontend")
        .with_output_dir(output_temp.path());

    let generator = PackGenerator::new(config);

    // Validation should fail because no index.html
    let result = generator.validate();
    assert!(
        result.is_err(),
        "Empty frontend directory should fail validation"
    );
}

/// Test URL mode with various URL formats
#[test]
fn test_pack_generator_url_mode_formats() {
    let temp_dir = tempdir().expect("Failed to create temp directory");

    // Test with http:// prefix
    let config = PackConfig::url("http://localhost:8080")
        .with_output("http-app")
        .with_output_dir(temp_dir.path());

    let generator = PackGenerator::new(config);
    let result = generator.generate();
    assert!(result.is_ok(), "http:// URL should work");

    let main_rs =
        fs::read_to_string(result.unwrap().join("src/main.rs")).expect("Failed to read main.rs");
    // Should preserve http:// (not convert to https://)
    assert!(
        main_rs.contains("http://localhost:8080"),
        "http:// URL should be preserved"
    );
}
