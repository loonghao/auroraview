//! Tests for auroraview-pack bundle module

use auroraview_pack::BundleBuilder;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_bundle_builder() {
    let temp = TempDir::new().unwrap();

    // Create test files
    fs::write(temp.path().join("index.html"), "<html></html>").unwrap();
    fs::write(temp.path().join("style.css"), "body { }").unwrap();
    fs::create_dir(temp.path().join("js")).unwrap();
    fs::write(temp.path().join("js/app.js"), "console.log('hi')").unwrap();

    let bundle = BundleBuilder::new(temp.path()).build().unwrap();

    assert_eq!(bundle.len(), 3);
    assert!(bundle.total_size() > 0);
}

#[test]
fn test_bundle_single_file() {
    let temp = TempDir::new().unwrap();
    let html_path = temp.path().join("page.html");
    fs::write(&html_path, "<html>test</html>").unwrap();

    let bundle = BundleBuilder::new(&html_path).build().unwrap();

    assert_eq!(bundle.len(), 1);
    assert_eq!(bundle.assets()[0].0, "index.html");
}

#[test]
fn test_bundle_excludes() {
    let temp = TempDir::new().unwrap();

    fs::write(temp.path().join("index.html"), "<html></html>").unwrap();
    fs::write(temp.path().join("app.js.map"), "sourcemap").unwrap();
    fs::write(temp.path().join(".DS_Store"), "").unwrap();

    let bundle = BundleBuilder::new(temp.path()).build().unwrap();

    // Should only include index.html
    assert_eq!(bundle.len(), 1);
    assert_eq!(bundle.assets()[0].0, "index.html");
}
