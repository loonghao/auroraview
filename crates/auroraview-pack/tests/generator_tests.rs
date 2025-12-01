//! Integration tests for PackGenerator

use auroraview_pack::{PackConfig, PackGenerator};

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
