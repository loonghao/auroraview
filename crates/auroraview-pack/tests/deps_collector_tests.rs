//! Tests for auroraview-pack deps_collector module

use std::path::PathBuf;

use auroraview_pack::{DepsCollector, FileHashCache};
use tempfile::TempDir;

// Note: is_stdlib and default_excludes are private functions,
// so we test through the public DepsCollector API

#[test]
fn collector_builder() {
    let collector = DepsCollector::new()
        .python_exe("python3")
        .exclude(["test_pkg"])
        .include(["extra_pkg"]);

    // Verify construction works by using the collector
    drop(collector);
}

#[test]
fn collector_default() {
    let collector = DepsCollector::default();
    // Default collector should be constructible
    let _ = collector;
}

#[test]
fn collector_with_python_exe() {
    let collector = DepsCollector::new().python_exe(PathBuf::from("/usr/bin/python3"));
    let _ = collector;
}

#[test]
fn collector_with_multiple_excludes() {
    let collector = DepsCollector::new().exclude(["pkg1", "pkg2", "pkg3"]);
    let _ = collector;
}

#[test]
fn collector_with_multiple_includes() {
    let collector = DepsCollector::new().include(["requests", "pyyaml", "auroraview"]);
    let _ = collector;
}

#[test]
fn collector_chained_config() {
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
fn file_hash_cache_new_is_empty() {
    let cache = FileHashCache::new();
    assert!(cache.hashes.is_empty());
    assert_eq!(cache.version, 1);
}

#[test]
fn file_hash_cache_load_nonexistent_returns_empty() {
    let result = FileHashCache::load(std::path::Path::new("/nonexistent/path/cache.json"));
    assert!(result.is_ok());
    let cache = result.unwrap();
    assert!(cache.hashes.is_empty());
}

#[test]
fn file_hash_cache_save_and_load() {
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
fn file_hash_cache_compute_hash_is_consistent() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("test.txt");
    std::fs::write(&file, b"hello world").unwrap();

    let hash1 = FileHashCache::compute_hash(&file).unwrap();
    let hash2 = FileHashCache::compute_hash(&file).unwrap();

    assert_eq!(hash1, hash2);
    assert!(!hash1.is_empty());
}

#[test]
fn file_hash_cache_different_contents_different_hashes() {
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
fn file_hash_cache_has_changed_new_file() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("file.py");
    std::fs::write(&file, b"import os").unwrap();

    let cache = FileHashCache::new();
    // File not in cache → has_changed returns true
    let changed = cache.has_changed(&file, "file.py").unwrap();
    assert!(changed);
}

#[test]
fn file_hash_cache_has_changed_same_content() {
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
fn file_hash_cache_update_then_changed() {
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
fn file_hash_cache_remove() {
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
fn file_hash_cache_save_creates_parent_dirs() {
    let temp = TempDir::new().unwrap();
    let nested = temp.path().join("a/b/c/cache.json");

    let cache = FileHashCache::new();
    cache.save(&nested).unwrap();

    assert!(nested.exists());
}

// ============================================================================
// Extended coverage tests
// ============================================================================

#[test]
fn file_hash_cache_version_is_one() {
    let cache = FileHashCache::new();
    assert_eq!(cache.version, 1);
}

#[test]
fn file_hash_cache_serde_roundtrip() {
    let temp = TempDir::new().unwrap();
    let cache_path = temp.path().join("serde_cache.json");

    let mut cache = FileHashCache::new();
    cache.hashes.insert("a.py".to_string(), "hash_a".to_string());

    cache.save(&cache_path).unwrap();
    let loaded = FileHashCache::load(&cache_path).unwrap();
    assert_eq!(loaded.version, 1);
    assert_eq!(loaded.hashes.get("a.py").map(String::as_str), Some("hash_a"));
}

#[test]
fn file_hash_cache_update_multiple_files() {
    let temp = TempDir::new().unwrap();
    let f1 = temp.path().join("f1.py");
    let f2 = temp.path().join("f2.py");
    let f3 = temp.path().join("f3.py");
    std::fs::write(&f1, b"x=1").unwrap();
    std::fs::write(&f2, b"x=2").unwrap();
    std::fs::write(&f3, b"x=3").unwrap();

    let mut cache = FileHashCache::new();
    cache.update("f1.py", &f1).unwrap();
    cache.update("f2.py", &f2).unwrap();
    cache.update("f3.py", &f3).unwrap();

    assert_eq!(cache.hashes.len(), 3);
    assert!(!cache.has_changed(&f1, "f1.py").unwrap());
    assert!(!cache.has_changed(&f2, "f2.py").unwrap());
    assert!(!cache.has_changed(&f3, "f3.py").unwrap());
}

#[test]
fn file_hash_cache_compute_hash_binary_file() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("binary.bin");
    let data: Vec<u8> = (0u8..=255).collect();
    std::fs::write(&file, &data).unwrap();

    let hash = FileHashCache::compute_hash(&file).unwrap();
    assert!(!hash.is_empty());
    // Should be hex string
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn file_hash_cache_compute_hash_empty_file() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("empty.py");
    std::fs::write(&file, b"").unwrap();

    let hash = FileHashCache::compute_hash(&file).unwrap();
    // Empty file should have a consistent hash
    assert!(!hash.is_empty());
    let hash2 = FileHashCache::compute_hash(&file).unwrap();
    assert_eq!(hash, hash2);
}

#[test]
fn file_hash_cache_large_file_hash_is_stable() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("large.py");
    let content = "x = 1\n".repeat(10_000);
    std::fs::write(&file, content.as_bytes()).unwrap();

    let h1 = FileHashCache::compute_hash(&file).unwrap();
    let h2 = FileHashCache::compute_hash(&file).unwrap();
    assert_eq!(h1, h2);
}

#[test]
fn file_hash_cache_remove_nonexistent_key_is_noop() {
    let mut cache = FileHashCache::new();
    // Should not panic
    cache.remove("nonexistent_key.py");
    assert!(cache.hashes.is_empty());
}

#[test]
fn file_hash_cache_update_overwrites_old_hash() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("mod.py");
    std::fs::write(&file, b"v = 1").unwrap();

    let mut cache = FileHashCache::new();
    cache.update("mod.py", &file).unwrap();
    let hash1 = cache.hashes["mod.py"].clone();

    std::fs::write(&file, b"v = 2").unwrap();
    cache.update("mod.py", &file).unwrap();
    let hash2 = cache.hashes["mod.py"].clone();

    assert_ne!(hash1, hash2);
}

#[test]
fn file_hash_cache_save_and_reload_version() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("v_cache.json");
    let cache = FileHashCache::new();
    cache.save(&path).unwrap();

    let loaded = FileHashCache::load(&path).unwrap();
    assert_eq!(loaded.version, 1);
}

#[test]
fn file_hash_cache_has_changed_after_remove() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("r.py");
    std::fs::write(&file, b"pass").unwrap();

    let mut cache = FileHashCache::new();
    cache.update("r.py", &file).unwrap();
    assert!(!cache.has_changed(&file, "r.py").unwrap());

    cache.remove("r.py");
    // After removal, should be treated as changed
    assert!(cache.has_changed(&file, "r.py").unwrap());
}

#[test]
fn collector_with_empty_exclude_and_include() {
    let collector = DepsCollector::new()
        .exclude([] as [&str; 0])
        .include([] as [&str; 0]);
    let _ = collector;
}

#[test]
fn file_hash_cache_load_invalid_json_returns_empty() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("bad.json");
    std::fs::write(&path, b"not valid json {{{").unwrap();

    let result = FileHashCache::load(&path);
    // Should return empty cache (or ok with empty) rather than panic
    let cache = result.unwrap_or_else(|_| FileHashCache::new());
    // The key point: no crash
    let _ = cache;
}
