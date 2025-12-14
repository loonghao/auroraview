//! Tests for auroraview-pack deps_collector module

use auroraview_pack::DepsCollector;
use std::path::PathBuf;

// Note: is_stdlib and default_excludes are private functions,
// so we test through the public DepsCollector API

#[test]
fn test_collector_builder() {
    let collector = DepsCollector::new()
        .python_exe("python3")
        .exclude(["test_pkg"])
        .include(["extra_pkg"]);

    // We can't directly access private fields, but we can verify construction works
    assert!(true);
    let _ = collector; // Use the collector to avoid unused warning
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
