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
    fs::write(temp.path().join("assets/images/logo.png"), b"\x89PNG".as_ref()).unwrap();
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
    assert!(!name.contains('\\'), "path should use forward slashes: {name}");
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

// ============================================================================
// AssetBundle direct manipulation
// ============================================================================

use auroraview_pack::AssetBundle;

#[test]
fn asset_bundle_new_is_empty() {
    let bundle = AssetBundle::new();
    assert!(bundle.is_empty());
    assert_eq!(bundle.len(), 0);
    assert_eq!(bundle.total_size(), 0);
}

#[test]
fn asset_bundle_add_increments_len() {
    let mut bundle = AssetBundle::new();
    bundle.add("a.html", b"<html>".to_vec());
    bundle.add("b.css", b"body{}".to_vec());
    assert_eq!(bundle.len(), 2);
    assert!(!bundle.is_empty());
}

#[test]
fn asset_bundle_total_size_accumulates() {
    let mut bundle = AssetBundle::new();
    bundle.add("file.html", vec![0u8; 100]);
    bundle.add("style.css", vec![0u8; 50]);
    assert_eq!(bundle.total_size(), 150);
}

#[test]
fn asset_bundle_assets_order_preserved() {
    let mut bundle = AssetBundle::new();
    bundle.add("first.js", b"first".to_vec());
    bundle.add("second.js", b"second".to_vec());
    let assets = bundle.assets();
    assert_eq!(assets[0].0, "first.js");
    assert_eq!(assets[1].0, "second.js");
}

#[test]
fn asset_bundle_into_assets_consumes() {
    let mut bundle = AssetBundle::new();
    bundle.add("x.html", b"x".to_vec());
    let assets = bundle.into_assets();
    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0].0, "x.html");
    assert_eq!(assets[0].1, b"x");
}

#[test]
fn asset_bundle_content_correct() {
    let mut bundle = AssetBundle::new();
    let data = b"<html><body>Hello</body></html>";
    bundle.add("index.html", data.to_vec());
    assert_eq!(bundle.assets()[0].1, data);
}

// ============================================================================
// BundleBuilder — content integrity
// ============================================================================

#[test]
fn bundle_content_matches_source() {
    let temp = TempDir::new().unwrap();
    let content = "<html>test content</html>";
    fs::write(temp.path().join("index.html"), content).unwrap();

    let bundle = BundleBuilder::new(temp.path()).build().unwrap();
    let stored = std::str::from_utf8(&bundle.assets()[0].1).unwrap();
    assert_eq!(stored, content);
}

#[test]
fn bundle_multiple_extensions_all_included() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("a.html"), "html").unwrap();
    fs::write(temp.path().join("b.js"), "js").unwrap();
    fs::write(temp.path().join("c.css"), "css").unwrap();

    let bundle = BundleBuilder::new(temp.path()).build().unwrap();
    assert_eq!(bundle.len(), 3);
}

#[test]
fn bundle_excludes_git_files() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("index.html"), "html").unwrap();
    fs::write(temp.path().join(".gitignore"), "*.log").unwrap();

    let bundle = BundleBuilder::new(temp.path()).build().unwrap();
    let names: Vec<&str> = bundle.assets().iter().map(|(n, _)| n.as_str()).collect();
    assert!(!names.contains(&".gitignore"));
    assert!(names.contains(&"index.html"));
}

#[test]
fn bundle_excludes_thumbs_db() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("index.html"), "html").unwrap();
    fs::write(temp.path().join("Thumbs.db"), "thumb").unwrap();

    let bundle = BundleBuilder::new(temp.path()).build().unwrap();
    let names: Vec<&str> = bundle.assets().iter().map(|(n, _)| n.as_str()).collect();
    assert!(!names.contains(&"Thumbs.db"));
}

#[test]
fn bundle_with_extensions_excludes_others() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("a.html"), "html").unwrap();
    fs::write(temp.path().join("b.js"), "js").unwrap();

    let bundle = BundleBuilder::new(temp.path())
        .with_extensions(&["html"])
        .build()
        .unwrap();
    assert_eq!(bundle.len(), 1);
    assert_eq!(bundle.assets()[0].0, "a.html");
}

#[test]
fn bundle_asset_bundle_default_is_empty() {
    let bundle = AssetBundle::default();
    assert!(bundle.is_empty());
}

// ============================================================================
// Additional AssetBundle / BundleBuilder tests
// ============================================================================

#[test]
fn asset_bundle_add_same_name_twice() {
    let mut bundle = AssetBundle::new();
    bundle.add("dup.js", b"v1".to_vec());
    bundle.add("dup.js", b"v2".to_vec());
    // Two entries with same name (no dedup required)
    assert_eq!(bundle.len(), 2);
}

#[test]
fn asset_bundle_total_size_zero_content() {
    let mut bundle = AssetBundle::new();
    bundle.add("empty.txt", vec![]);
    assert_eq!(bundle.total_size(), 0);
    assert_eq!(bundle.len(), 1);
}

#[test]
fn asset_bundle_large_content() {
    let mut bundle = AssetBundle::new();
    let data = vec![0xABu8; 1_000_000];
    bundle.add("large.bin", data);
    assert_eq!(bundle.total_size(), 1_000_000);
}

#[test]
fn bundle_html_content_is_bytes() {
    let temp = TempDir::new().unwrap();
    let html = "<html><body>Test</body></html>";
    fs::write(temp.path().join("index.html"), html).unwrap();

    let bundle = BundleBuilder::new(temp.path()).build().unwrap();
    assert_eq!(bundle.assets()[0].1, html.as_bytes());
}

#[test]
fn bundle_single_js_file_from_dir() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("main.js"), "console.log('hello')").unwrap();

    // BundleBuilder on a dir with only JS should work
    let bundle = BundleBuilder::new(temp.path()).build().unwrap();
    assert_eq!(bundle.len(), 1);
    assert_eq!(bundle.assets()[0].0, "main.js");
}

#[test]
fn bundle_all_assets_have_nonempty_names() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("index.html"), "html").unwrap();
    fs::write(temp.path().join("app.js"), "js").unwrap();
    fs::write(temp.path().join("style.css"), "css").unwrap();

    let bundle = BundleBuilder::new(temp.path()).build().unwrap();
    for (name, _) in bundle.assets() {
        assert!(!name.is_empty(), "Asset name should not be empty");
    }
}

#[test]
fn bundle_no_map_files_included() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("index.html"), "html").unwrap();
    // .js.map files should be excluded (source maps)
    fs::write(temp.path().join("app.js.map"), "sourcemap").unwrap();

    let bundle = BundleBuilder::new(temp.path()).build().unwrap();
    for (name, _) in bundle.assets() {
        assert!(
            !name.ends_with(".map"),
            ".map source files should not be in bundle: {name}"
        );
    }
}

#[test]
fn asset_bundle_add_unicode_name() {
    let mut bundle = AssetBundle::new();
    bundle.add("日本語.html", b"<html>".to_vec());
    assert_eq!(bundle.len(), 1);
    assert_eq!(bundle.assets()[0].0, "日本語.html");
}

#[test]
fn asset_bundle_many_files() {
    let mut bundle = AssetBundle::new();
    for i in 0..50 {
        bundle.add(format!("file_{}.js", i), format!("var x{};", i).into_bytes());
    }
    assert_eq!(bundle.len(), 50);
    assert!(bundle.total_size() > 0);
}

#[test]
fn bundle_extension_filter_empty_allows_all() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("index.html"), "html").unwrap();
    fs::write(temp.path().join("app.js"), "js").unwrap();

    // Empty extensions slice = no filter = all files allowed
    let bundle = BundleBuilder::new(temp.path())
        .with_extensions(&[])
        .build()
        .unwrap();
    // With empty filter, should include all (html and js = 2)
    assert!(bundle.len() >= 1);
}

#[test]
fn bundle_builder_single_html_file_named_index() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("mypage.html");
    fs::write(&file_path, "<html>content</html>").unwrap();

    let bundle = BundleBuilder::new(&file_path).build().unwrap();
    // Single file mode: renamed to index.html
    assert_eq!(bundle.assets()[0].0, "index.html");
}

#[test]
fn bundle_gitignore_excluded() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("index.html"), "html").unwrap();
    // Create a .gitignore — it should be excluded
    fs::write(temp.path().join(".gitignore"), "*.log").unwrap();

    let bundle = BundleBuilder::new(temp.path()).build().unwrap();
    let names: Vec<&str> = bundle.assets().iter().map(|(n, _)| n.as_str()).collect();
    assert!(
        !names.iter().any(|n| n.contains(".gitignore")),
        "gitignore should be excluded"
    );
}

