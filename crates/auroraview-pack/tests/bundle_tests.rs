//! Tests for auroraview-pack bundle module

use std::fs;

use auroraview_pack::BundleBuilder;
use tempfile::TempDir;

#[test]
fn bundle_builder() {
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
fn bundle_single_file() {
    let temp = TempDir::new().unwrap();
    let html_path = temp.path().join("page.html");
    fs::write(&html_path, "<html>test</html>").unwrap();

    let bundle = BundleBuilder::new(&html_path).build().unwrap();

    assert_eq!(bundle.len(), 1);
    assert_eq!(bundle.assets()[0].0, "index.html");
}

#[test]
fn bundle_excludes() {
    let temp = TempDir::new().unwrap();

    fs::write(temp.path().join("index.html"), "<html></html>").unwrap();
    fs::write(temp.path().join("app.js.map"), "sourcemap").unwrap();
    fs::write(temp.path().join(".DS_Store"), "").unwrap();

    let bundle = BundleBuilder::new(temp.path()).build().unwrap();

    // Should only include index.html
    assert_eq!(bundle.len(), 1);
    assert_eq!(bundle.assets()[0].0, "index.html");
}

#[test]
fn bundle_nonexistent_path_returns_error() {
    let result = BundleBuilder::new("/this/path/does/not/exist/at/all").build();
    assert!(result.is_err());
}

#[test]
fn bundle_empty_dir_returns_error() {
    let temp = TempDir::new().unwrap();
    // No files created — bundle should fail with empty error
    let result = BundleBuilder::new(temp.path()).build();
    assert!(result.is_err());
}

#[test]
fn bundle_with_extensions_filter() {
    let temp = TempDir::new().unwrap();

    fs::write(temp.path().join("index.html"), "<html></html>").unwrap();
    fs::write(temp.path().join("app.js"), "const x = 1;").unwrap();
    fs::write(temp.path().join("style.css"), "body {}").unwrap();
    fs::write(temp.path().join("data.json"), "{}").unwrap();

    // Only include JS and CSS
    let bundle = BundleBuilder::new(temp.path())
        .with_extensions(&["js", "css"])
        .build()
        .unwrap();

    assert_eq!(bundle.len(), 2);
    let names: Vec<&str> = bundle.assets().iter().map(|(n, _)| n.as_str()).collect();
    assert!(names.contains(&"app.js"));
    assert!(names.contains(&"style.css"));
    assert!(!names.contains(&"index.html"));
}

#[test]
fn bundle_custom_exclude_pattern() {
    let temp = TempDir::new().unwrap();

    fs::write(temp.path().join("index.html"), "<html></html>").unwrap();
    fs::write(temp.path().join("app.js"), "console.log('app')").unwrap();
    fs::write(temp.path().join("app.test.js"), "test()").unwrap();

    // Exclude *.test.js files
    let bundle = BundleBuilder::new(temp.path())
        .exclude(&["*.test.js"])
        .build()
        .unwrap();

    let names: Vec<&str> = bundle.assets().iter().map(|(n, _)| n.as_str()).collect();
    assert!(!names.contains(&"app.test.js"));
    assert!(names.contains(&"app.js"));
}

#[test]
fn bundle_nested_directories() {
    let temp = TempDir::new().unwrap();

    fs::create_dir_all(temp.path().join("assets/images")).unwrap();
    fs::create_dir_all(temp.path().join("src/components")).unwrap();
    fs::write(temp.path().join("index.html"), "<html></html>").unwrap();
    fs::write(
        temp.path().join("assets/images/logo.png"),
        b"\x89PNG".as_ref(),
    )
    .unwrap();
    fs::write(temp.path().join("src/components/app.js"), "export {}").unwrap();

    let bundle = BundleBuilder::new(temp.path()).build().unwrap();

    assert_eq!(bundle.len(), 3);
    let names: Vec<&str> = bundle.assets().iter().map(|(n, _)| n.as_str()).collect();
    assert!(names.iter().any(|n| n.contains("logo.png")));
    assert!(names.iter().any(|n| n.contains("app.js")));
}

#[test]
fn bundle_total_size_accumulates() {
    let temp = TempDir::new().unwrap();

    let content1 = "a".repeat(100);
    let content2 = "b".repeat(200);
    fs::write(temp.path().join("a.html"), &content1).unwrap();
    fs::write(temp.path().join("b.css"), &content2).unwrap();

    let bundle = BundleBuilder::new(temp.path()).build().unwrap();

    assert!(bundle.total_size() >= 300);
}

#[test]
fn bundle_into_assets() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("index.html"), "<html></html>").unwrap();

    let bundle = BundleBuilder::new(temp.path()).build().unwrap();
    let assets = bundle.into_assets();

    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0].0, "index.html");
}

#[test]
fn bundle_path_separators_normalized() {
    let temp = TempDir::new().unwrap();
    fs::create_dir_all(temp.path().join("sub/dir")).unwrap();
    fs::write(temp.path().join("sub/dir/file.js"), "x").unwrap();

    let bundle = BundleBuilder::new(temp.path()).build().unwrap();
    let name = &bundle.assets()[0].0;

    // Path separators must be forward slashes regardless of platform
    assert!(
        !name.contains('\\'),
        "path should use forward slashes: {name}"
    );
    assert!(name.contains('/'));
}

#[test]
fn bundle_is_empty_check() {
    // AssetBundle::new() directly — use a path that won't have files
    // We test through BundleBuilder returning an error on empty dir
    let temp = TempDir::new().unwrap();
    let result = BundleBuilder::new(temp.path()).build();
    assert!(result.is_err(), "empty directory should produce error");
}
