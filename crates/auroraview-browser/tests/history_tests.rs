//! Tests for history module

use auroraview_browser::navigation::{HistoryEntry, HistoryManager};
use tempfile::TempDir;

/// Helper to create isolated HistoryManager with temp directory
fn create_test_manager(max_entries: usize, enabled: bool) -> (HistoryManager, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let manager = HistoryManager::new(
        Some(temp_dir.path().to_str().unwrap()),
        max_entries,
        enabled,
    );
    (manager, temp_dir)
}

#[test]
fn test_history_entry_new() {
    let entry = HistoryEntry::new("https://example.com", "Example Site");

    assert_eq!(entry.url, "https://example.com");
    assert_eq!(entry.title, "Example Site");
    assert_eq!(entry.visit_count, 1);
    assert!(entry.favicon.is_none());
}

#[test]
fn test_history_entry_with_favicon() {
    let entry = HistoryEntry::new("https://example.com", "Example Site")
        .with_favicon("https://example.com/favicon.ico");

    assert_eq!(entry.favicon, Some("https://example.com/favicon.ico".to_string()));
}

#[test]
fn test_history_manager_add_and_get() {
    let (manager, _temp_dir) = create_test_manager(100, true);

    manager.add("https://example.com", "Example");
    manager.add("https://google.com", "Google");

    let entries = manager.get(10);
    assert_eq!(entries.len(), 2);

    // Most recent first
    assert_eq!(entries[0].url, "https://google.com");
    assert_eq!(entries[1].url, "https://example.com");
}

#[test]
fn test_history_manager_revisit_updates_count() {
    let (manager, _temp_dir) = create_test_manager(100, true);

    manager.add("https://example.com", "Example");
    manager.add("https://example.com", "Example - Updated");

    let entries = manager.all();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].visit_count, 2);
    assert_eq!(entries[0].title, "Example - Updated");
}

#[test]
fn test_history_manager_search() {
    let (manager, _temp_dir) = create_test_manager(100, true);

    manager.add("https://google.com", "Google Search");
    manager.add("https://github.com", "GitHub");
    manager.add("https://golang.org", "Go Programming Language");

    // "go" appears in URL "google", URL "golang", and title "Go Programming Language"
    // This matches 2 distinct entries: google.com and golang.org
    let results = manager.search("go", 10);
    assert_eq!(results.len(), 2);

    let results = manager.search("github", 10);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].url, "https://github.com");
}

#[test]
fn test_history_manager_remove() {
    let (manager, _temp_dir) = create_test_manager(100, true);

    manager.add("https://example.com", "Example");
    manager.add("https://google.com", "Google");

    let removed = manager.remove("https://example.com");
    assert!(removed);

    let entries = manager.all();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].url, "https://google.com");
}

#[test]
fn test_history_manager_clear() {
    let (manager, _temp_dir) = create_test_manager(100, true);

    manager.add("https://example1.com", "Example 1");
    manager.add("https://example2.com", "Example 2");

    manager.clear();

    assert_eq!(manager.count(), 0);
}

#[test]
fn test_history_manager_disabled() {
    let (manager, _temp_dir) = create_test_manager(100, false);

    manager.add("https://example.com", "Example");

    // Should not add when disabled
    assert_eq!(manager.count(), 0);
}

#[test]
fn test_history_manager_max_entries() {
    let (manager, _temp_dir) = create_test_manager(3, true);

    manager.add("https://site1.com", "Site 1");
    manager.add("https://site2.com", "Site 2");
    manager.add("https://site3.com", "Site 3");
    manager.add("https://site4.com", "Site 4");

    // Should only keep 3 entries (oldest removed)
    let entries = manager.all();
    assert_eq!(entries.len(), 3);

    // Site 1 should be removed
    assert!(!entries.iter().any(|e| e.url == "https://site1.com"));
}

#[test]
fn test_history_manager_skips_internal_urls() {
    let (manager, _temp_dir) = create_test_manager(100, true);

    manager.add("about:blank", "New Tab");
    manager.add("data:text/html,<h1>Test</h1>", "Data URL");
    manager.add("https://example.com", "Example");

    // Should only have the external URL
    let entries = manager.all();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].url, "https://example.com");
}
