//! Tests for bookmarks module

use auroraview_browser::navigation::{Bookmark, BookmarkFolder, BookmarkManager};
use rstest::rstest;
use tempfile::TempDir;

fn create_test_manager() -> (BookmarkManager, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let manager = BookmarkManager::new(Some(temp_dir.path().to_str().unwrap()));
    (manager, temp_dir)
}

// ─── Bookmark struct ─────────────────────────────────────────────────────────

#[test]
fn bookmark_new() {
    let bookmark = Bookmark::new("Test Site", "https://test.com");

    assert!(!bookmark.id.is_empty());
    assert_eq!(bookmark.title, "Test Site");
    assert_eq!(bookmark.url, "https://test.com");
    assert!(bookmark.favicon.is_none());
    assert!(bookmark.parent_id.is_none());
    assert_eq!(bookmark.position, 0);
}

#[test]
fn bookmark_builder() {
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
fn bookmark_with_id() {
    let bm = Bookmark::with_id("custom-id-123", "Title", "https://url.com");
    assert_eq!(bm.id, "custom-id-123");
    assert_eq!(bm.title, "Title");
    assert_eq!(bm.url, "https://url.com");
}

#[test]
fn bookmark_with_parent() {
    let bm = Bookmark::new("Child", "https://child.com")
        .with_parent("parent-folder-id".to_string());
    assert_eq!(bm.parent_id, Some("parent-folder-id".to_string()));
}

#[test]
fn bookmark_ids_unique() {
    let a = Bookmark::new("A", "https://a.com");
    let b = Bookmark::new("B", "https://b.com");
    assert_ne!(a.id, b.id);
}

#[test]
fn bookmark_clone() {
    let bm = Bookmark::new("Clone", "https://clone.com")
        .with_favicon("https://clone.com/fav.ico")
        .with_position(3);
    let cloned = bm.clone();
    assert_eq!(cloned.id, bm.id);
    assert_eq!(cloned.title, bm.title);
    assert_eq!(cloned.position, bm.position);
}

#[test]
fn bookmark_serde_roundtrip() {
    let bm = Bookmark::new("Serde", "https://serde.com").with_position(7);
    let json = serde_json::to_string(&bm).unwrap();
    let restored: Bookmark = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.id, bm.id);
    assert_eq!(restored.title, bm.title);
    assert_eq!(restored.url, bm.url);
    assert_eq!(restored.position, 7);
}

// ─── BookmarkFolder ──────────────────────────────────────────────────────────

#[test]
fn folder_new() {
    let folder = BookmarkFolder::new("Favorites");
    assert!(!folder.id.is_empty());
    assert_eq!(folder.name, "Favorites");
    assert!(folder.parent_id.is_none());
    assert_eq!(folder.position, 0);
}

#[test]
fn folder_ids_unique() {
    let a = BookmarkFolder::new("A");
    let b = BookmarkFolder::new("B");
    assert_ne!(a.id, b.id);
}

#[test]
fn folder_serde_roundtrip() {
    let folder = BookmarkFolder::new("Work");
    let json = serde_json::to_string(&folder).unwrap();
    let restored: BookmarkFolder = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.id, folder.id);
    assert_eq!(restored.name, "Work");
}

// ─── BookmarkManager: basic CRUD ─────────────────────────────────────────────

#[test]
fn manager_add_and_get() {
    let (manager, _temp_dir) = create_test_manager();

    let id = manager.add_bookmark("https://example.com", "Example");

    let bookmark = manager.get(&id);
    assert!(bookmark.is_some());

    let bookmark = bookmark.unwrap();
    assert_eq!(bookmark.title, "Example");
    assert_eq!(bookmark.url, "https://example.com");
}

#[test]
fn manager_remove() {
    let (manager, _temp_dir) = create_test_manager();

    assert_eq!(manager.count(), 0);

    let id = manager.add_bookmark("https://example.com", "Example");
    assert_eq!(manager.count(), 1);

    let removed = manager.remove(&id);
    assert!(removed.is_some());
    assert_eq!(manager.count(), 0);
}

#[test]
fn manager_remove_returns_bookmark() {
    let (manager, _temp_dir) = create_test_manager();
    let id = manager.add_bookmark("https://ret.com", "ReturnMe");
    let removed = manager.remove(&id).unwrap();
    assert_eq!(removed.title, "ReturnMe");
}

#[test]
fn manager_remove_nonexistent_returns_none() {
    let (manager, _temp_dir) = create_test_manager();
    let result = manager.remove(&"nonexistent-id".to_string());
    assert!(result.is_none());
}

#[test]
fn manager_get_nonexistent_returns_none() {
    let (manager, _temp_dir) = create_test_manager();
    let result = manager.get(&"nonexistent".to_string());
    assert!(result.is_none());
}

#[test]
fn manager_update() {
    let (manager, _temp_dir) = create_test_manager();

    let id = manager.add_bookmark("https://example.com", "Example");

    let updated = manager.update(&id, Some("Updated Title".to_string()), None);
    assert!(updated);

    let bookmark = manager.get(&id).unwrap();
    assert_eq!(bookmark.title, "Updated Title");
    assert_eq!(bookmark.url, "https://example.com");
}

#[test]
fn manager_update_url() {
    let (manager, _temp_dir) = create_test_manager();
    let id = manager.add_bookmark("https://old.com", "Page");
    manager.update(&id, None, Some("https://new.com".to_string()));
    let bm = manager.get(&id).unwrap();
    assert_eq!(bm.url, "https://new.com");
    assert_eq!(bm.title, "Page");
}

#[test]
fn manager_update_nonexistent_returns_false() {
    let (manager, _temp_dir) = create_test_manager();
    let result = manager.update(&"bad-id".to_string(), Some("title".to_string()), None);
    assert!(!result);
}

#[test]
fn manager_is_bookmarked() {
    let (manager, _temp_dir) = create_test_manager();

    manager.add_bookmark("https://example.com", "Example");

    assert!(manager.is_bookmarked("https://example.com"));
    assert!(!manager.is_bookmarked("https://other.com"));
}

#[test]
fn manager_find_by_url() {
    let (manager, _temp_dir) = create_test_manager();

    manager.add_bookmark("https://example.com", "Example");

    let found = manager.find_by_url("https://example.com");
    assert!(found.is_some());
    assert_eq!(found.unwrap().title, "Example");

    let not_found = manager.find_by_url("https://other.com");
    assert!(not_found.is_none());
}

#[test]
fn manager_clear() {
    let (manager, _temp_dir) = create_test_manager();

    manager.add_bookmark("https://example1.com", "Example 1");
    manager.add_bookmark("https://example2.com", "Example 2");
    assert_eq!(manager.count(), 2);

    manager.clear();
    assert_eq!(manager.count(), 0);
}

#[test]
fn manager_clear_also_removes_folders() {
    let (manager, _temp_dir) = create_test_manager();
    let folder = BookmarkFolder::new("Work");
    manager.add_folder(folder);
    assert!(!manager.folders().is_empty());

    manager.clear();
    assert!(manager.folders().is_empty());
}

// ─── BookmarkManager: all / bar_items / in_folder ────────────────────────────

#[test]
fn manager_all_returns_all_bookmarks() {
    let (manager, _temp_dir) = create_test_manager();
    manager.add_bookmark("https://a.com", "A");
    manager.add_bookmark("https://b.com", "B");
    manager.add_bookmark("https://c.com", "C");

    let all = manager.all();
    assert_eq!(all.len(), 3);
}

#[test]
fn manager_bar_items_returns_root_bookmarks() {
    let (manager, _temp_dir) = create_test_manager();
    manager.add_bookmark("https://root.com", "Root");

    // Add one in a subfolder
    let folder = BookmarkFolder::new("Sub");
    let folder_id = manager.add_folder(folder);
    let bm = Bookmark::new("In Folder", "https://infolder.com")
        .with_parent(folder_id.clone());
    manager.add(bm);

    let bar = manager.bar_items();
    assert_eq!(bar.len(), 1);
    assert_eq!(bar[0].url, "https://root.com");
}

#[test]
fn manager_in_folder_returns_children() {
    let (manager, _temp_dir) = create_test_manager();
    let folder = BookmarkFolder::new("TestFolder");
    let folder_id = manager.add_folder(folder);

    let bm = Bookmark::new("Child", "https://child.com")
        .with_parent(folder_id.clone());
    manager.add(bm);

    let children = manager.in_folder(Some(&folder_id));
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].url, "https://child.com");
}

// ─── BookmarkManager: folders ─────────────────────────────────────────────────

#[test]
fn manager_add_folder() {
    let (manager, _temp_dir) = create_test_manager();
    let folder = BookmarkFolder::new("My Folder");
    let id = manager.add_folder(folder);

    let folders = manager.folders();
    assert_eq!(folders.len(), 1);
    assert_eq!(folders[0].id, id);
    assert_eq!(folders[0].name, "My Folder");
}

#[test]
fn manager_multiple_folders() {
    let (manager, _temp_dir) = create_test_manager();
    manager.add_folder(BookmarkFolder::new("A"));
    manager.add_folder(BookmarkFolder::new("B"));

    assert_eq!(manager.folders().len(), 2);
}

// ─── BookmarkManager: add with Bookmark struct ────────────────────────────────

#[test]
fn manager_add_with_position() {
    let (manager, _temp_dir) = create_test_manager();
    let bm = Bookmark::new("Positioned", "https://pos.com").with_position(10);
    let id = manager.add(bm);
    let got = manager.get(&id).unwrap();
    assert_eq!(got.position, 10);
}

// ─── BookmarkManager: default ─────────────────────────────────────────────────

#[test]
fn manager_default_creates_instance() {
    let manager = BookmarkManager::default();
    assert_eq!(manager.count(), 0);
}

// ─── rstest parametrized ─────────────────────────────────────────────────────

#[rstest]
#[case("https://google.com", "Google")]
#[case("https://github.com", "GitHub")]
#[case("https://rust-lang.org", "Rust")]
#[case("https://example.com", "Example")]
fn add_various_bookmarks(#[case] url: &str, #[case] title: &str) {
    let (manager, _temp_dir) = create_test_manager();
    let id = manager.add_bookmark(url, title);

    let bm = manager.get(&id).unwrap();
    assert_eq!(bm.url, url);
    assert_eq!(bm.title, title);
    assert!(manager.is_bookmarked(url));
}

#[rstest]
#[case("https://a.com", "A", 1)]
#[case("https://b.com", "B", 2)]
#[case("https://c.com", "C", 3)]
fn count_increases_per_add(#[case] url: &str, #[case] title: &str, #[case] expected: usize) {
    let (manager, _temp_dir) = create_test_manager();
    for i in 0..expected {
        manager.add_bookmark(&format!("{}/{}", url, i), title);
    }
    assert_eq!(manager.count(), expected);
}
