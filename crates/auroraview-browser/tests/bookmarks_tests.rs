//! Tests for bookmarks module

use auroraview_browser::navigation::{Bookmark, BookmarkManager};

#[test]
fn test_bookmark_new() {
    let bookmark = Bookmark::new("Test Site", "https://test.com");

    assert!(!bookmark.id.is_empty());
    assert_eq!(bookmark.title, "Test Site");
    assert_eq!(bookmark.url, "https://test.com");
    assert!(bookmark.favicon.is_none());
    assert!(bookmark.parent_id.is_none());
}

#[test]
fn test_bookmark_builder() {
    let bookmark = Bookmark::new("Test Site", "https://test.com")
        .with_favicon("https://test.com/favicon.ico")
        .with_position(5);

    assert_eq!(
        bookmark.favicon,
        Some("https://test.com/favicon.ico".to_string())
    );
    assert_eq!(bookmark.position, 5);
}

#[test]
fn test_bookmark_manager_add_and_get() {
    let manager = BookmarkManager::new(None);

    let id = manager.add_bookmark("https://example.com", "Example");

    let bookmark = manager.get(&id);
    assert!(bookmark.is_some());

    let bookmark = bookmark.unwrap();
    assert_eq!(bookmark.title, "Example");
    assert_eq!(bookmark.url, "https://example.com");
}

#[test]
fn test_bookmark_manager_remove() {
    let manager = BookmarkManager::new(None);

    let id = manager.add_bookmark("https://example.com", "Example");
    assert_eq!(manager.count(), 1);

    let removed = manager.remove(&id);
    assert!(removed.is_some());
    assert_eq!(manager.count(), 0);
}

#[test]
fn test_bookmark_manager_update() {
    let manager = BookmarkManager::new(None);

    let id = manager.add_bookmark("https://example.com", "Example");

    let updated = manager.update(&id, Some("Updated Title".to_string()), None);
    assert!(updated);

    let bookmark = manager.get(&id).unwrap();
    assert_eq!(bookmark.title, "Updated Title");
    assert_eq!(bookmark.url, "https://example.com");
}

#[test]
fn test_bookmark_manager_is_bookmarked() {
    let manager = BookmarkManager::new(None);

    manager.add_bookmark("https://example.com", "Example");

    assert!(manager.is_bookmarked("https://example.com"));
    assert!(!manager.is_bookmarked("https://other.com"));
}

#[test]
fn test_bookmark_manager_find_by_url() {
    let manager = BookmarkManager::new(None);

    manager.add_bookmark("https://example.com", "Example");

    let found = manager.find_by_url("https://example.com");
    assert!(found.is_some());
    assert_eq!(found.unwrap().title, "Example");

    let not_found = manager.find_by_url("https://other.com");
    assert!(not_found.is_none());
}

#[test]
fn test_bookmark_manager_clear() {
    let manager = BookmarkManager::new(None);

    manager.add_bookmark("https://example1.com", "Example 1");
    manager.add_bookmark("https://example2.com", "Example 2");
    assert_eq!(manager.count(), 2);

    manager.clear();
    assert_eq!(manager.count(), 0);
}
