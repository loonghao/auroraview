//! Tests for auroraview-pack deps_collector module

use auroraview_pack::{DepsCollector, FileHashCache};
use std::path::PathBuf;
use tempfile::TempDir;

// Note: is_stdlib and default_excludes are private functions,
// so we test through the public DepsCollector API

#[test]
fn test_collector_builder() {
    let collector = DepsCollector::new()
        .python_exe("python3")
        .exclude(["test_pkg"])
        .include(["extra_pkg"]);

    // Verify construction works by using the collector
    drop(collector);
}

#[test]
fn test_collector_default() {
    let collector = DepsCollector::default();
    // Default collector should be constructible
    let _ = collector;
}

#[test]
fn test_collector_with_python_exe() {
    let collector = DepsCollector::new().python_exe(PathBuf::from("/usr/bin/python3"));
    let _ = collector;
}

#[test]
fn test_collector_with_multiple_excludes() {
    let collector = DepsCollector::new().exclude(["pkg1", "pkg2", "pkg3"]);
    let _ = collector;
}

#[test]
fn test_collector_with_multiple_includes() {
    let collector = DepsCollector::new().include(["requests", "pyyaml", "auroraview"]);
    let _ = collector;
}

#[test]
fn test_collector_chained_config() {
    let collector = DepsCollector::new()
        .python_exe("python")
        .exclude(["pytest", "coverage"])
        .include(["mypackage"]);
    let _ = collector;
}

// ============================================================================
// FileHashCache tests
// ============================================================================

#[test]
fn test_file_hash_cache_new_is_empty() {
    let cache = FileHashCache::new();
    assert!(cache.hashes.is_empty());
    assert_eq!(cache.version, 1);
}

#[test]
fn test_file_hash_cache_load_nonexistent_returns_empty() {
    let result = FileHashCache::load(std::path::Path::new("/nonexistent/path/cache.json"));
    assert!(result.is_ok());
    let cache = result.unwrap();
    assert!(cache.hashes.is_empty());
}

#[test]
fn test_file_hash_cache_save_and_load() {
    let temp = TempDir::new().unwrap();
    let cache_path = temp.path().join("cache.json");

    let mut cache = FileHashCache::new();
    cache.hashes.insert("key1".to_string(), "hash1".to_string());
    cache.hashes.insert("key2".to_string(), "hash2".to_string());

    cache.save(&cache_path).unwrap();
    assert!(cache_path.exists());

    let loaded = FileHashCache::load(&cache_path).unwrap();
    assert_eq!(loaded.hashes.get("key1").map(|s| s.as_str()), Some("hash1"));
    assert_eq!(loaded.hashes.get("key2").map(|s| s.as_str()), Some("hash2"));
}

#[test]
fn test_file_hash_cache_compute_hash_is_consistent() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("test.txt");
    std::fs::write(&file, b"hello world").unwrap();

    let hash1 = FileHashCache::compute_hash(&file).unwrap();
    let hash2 = FileHashCache::compute_hash(&file).unwrap();

    assert_eq!(hash1, hash2);
    assert!(!hash1.is_empty());
}

#[test]
fn test_file_hash_cache_different_contents_different_hashes() {
    let temp = TempDir::new().unwrap();
    let file1 = temp.path().join("a.txt");
    let file2 = temp.path().join("b.txt");
    std::fs::write(&file1, b"content A").unwrap();
    std::fs::write(&file2, b"content B").unwrap();

    let hash1 = FileHashCache::compute_hash(&file1).unwrap();
    let hash2 = FileHashCache::compute_hash(&file2).unwrap();

    assert_ne!(hash1, hash2);
}

#[test]
fn test_file_hash_cache_has_changed_new_file() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("file.py");
    std::fs::write(&file, b"import os").unwrap();

    let cache = FileHashCache::new();
    // File not in cache → has_changed returns true
    let changed = cache.has_changed(&file, "file.py").unwrap();
    assert!(changed);
}

#[test]
fn test_file_hash_cache_has_changed_same_content() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("file.py");
    std::fs::write(&file, b"import sys").unwrap();

    let mut cache = FileHashCache::new();
    cache.update("file.py", &file).unwrap();

    // Content unchanged → has_changed returns false
    let changed = cache.has_changed(&file, "file.py").unwrap();
    assert!(!changed);
}

#[test]
fn test_file_hash_cache_update_then_changed() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("script.py");
    std::fs::write(&file, b"x = 1").unwrap();

    let mut cache = FileHashCache::new();
    cache.update("script.py", &file).unwrap();

    // Modify the file
    std::fs::write(&file, b"x = 2").unwrap();

    let changed = cache.has_changed(&file, "script.py").unwrap();
    assert!(changed);
}

#[test]
fn test_file_hash_cache_remove() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("x.py");
    std::fs::write(&file, b"pass").unwrap();

    let mut cache = FileHashCache::new();
    cache.update("x.py", &file).unwrap();
    assert!(cache.hashes.contains_key("x.py"));

    cache.remove("x.py");
    assert!(!cache.hashes.contains_key("x.py"));
}

#[test]
fn test_file_hash_cache_save_creates_parent_dirs() {
    let temp = TempDir::new().unwrap();
    let nested = temp.path().join("a/b/c/cache.json");

    let cache = FileHashCache::new();
    cache.save(&nested).unwrap();

    assert!(nested.exists());
}
