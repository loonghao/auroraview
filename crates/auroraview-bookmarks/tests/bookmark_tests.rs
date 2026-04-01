use auroraview_bookmarks::{Bookmark, BookmarkError, BookmarkFolder, BookmarkManager};
use rstest::*;
use tempfile::TempDir;

// ========== Bookmark Struct Tests ==========

#[test]
fn test_bookmark_new() {
    let b = Bookmark::new("GitHub", "https://github.com");
    assert_eq!(b.title, "GitHub");
    assert_eq!(b.url, "https://github.com");
    assert!(!b.id.is_empty());
    assert!(b.parent_id.is_none());
    assert!(b.favicon.is_none());
    assert!(b.tags.is_empty());
}

#[test]
fn test_bookmark_with_id() {
    let b = Bookmark::with_id("custom-id", "Test", "https://test.com");
    assert_eq!(b.id, "custom-id");
}

#[test]
fn test_bookmark_builder_methods() {
    let b = Bookmark::new("Test", "https://test.com")
        .with_favicon("https://test.com/favicon.ico")
        .with_position(3)
        .with_tag("dev")
        .with_tag("rust");

    assert_eq!(b.favicon, Some("https://test.com/favicon.ico".to_string()));
    assert_eq!(b.position, 3);
    assert_eq!(b.tags, vec!["dev".to_string(), "rust".to_string()]);
}

#[test]
fn test_bookmark_with_tags_replaces_all() {
    let b = Bookmark::new("T", "https://t.com")
        .with_tag("old")
        .with_tags(vec!["a".to_string(), "b".to_string()]);
    assert_eq!(b.tags, vec!["a".to_string(), "b".to_string()]);
}

#[test]
fn test_bookmark_with_parent() {
    let b = Bookmark::new("T", "https://t.com").with_parent("folder-1");
    assert_eq!(b.parent_id, Some("folder-1".to_string()));
}

#[rstest]
#[case("git", true)]
#[case("hub", true)]
#[case("GIT", true)] // case-insensitive
#[case("github.com", true)]
#[case("gitlab", false)]
#[case("dev", true)] // tag match
fn test_bookmark_matches(#[case] query: &str, #[case] expected: bool) {
    let b = Bookmark::new("GitHub", "https://github.com").with_tag("dev");
    assert_eq!(b.matches(query), expected);
}

#[test]
fn test_bookmark_set_title() {
    let mut b = Bookmark::new("Old", "https://example.com");
    b.set_title("New");
    assert_eq!(b.title, "New");
}

#[test]
fn test_bookmark_set_url() {
    let mut b = Bookmark::new("T", "https://old.com");
    b.set_url("https://new.com");
    assert_eq!(b.url, "https://new.com");
}

#[test]
fn test_bookmark_set_favicon() {
    let mut b = Bookmark::new("T", "https://t.com");
    b.set_favicon(Some("https://t.com/fav.ico".to_string()));
    assert_eq!(b.favicon, Some("https://t.com/fav.ico".to_string()));

    b.set_favicon(None);
    assert!(b.favicon.is_none());
}

#[test]
fn test_bookmark_set_parent() {
    let mut b = Bookmark::new("T", "https://t.com");
    b.set_parent(Some("folder-1".to_string()));
    assert_eq!(b.parent_id, Some("folder-1".to_string()));

    b.set_parent(None);
    assert!(b.parent_id.is_none());
}

// ========== BookmarkManager CRUD Tests ==========

#[test]
fn test_manager_add_and_get() {
    let manager = BookmarkManager::new(None);

    let id = manager.add("https://github.com", "GitHub");
    let b = manager.get(&id).unwrap();

    assert_eq!(b.title, "GitHub");
    assert_eq!(b.url, "https://github.com");
}

#[test]
fn test_manager_is_bookmarked() {
    let manager = BookmarkManager::new(None);

    manager.add("https://github.com", "GitHub");

    assert!(manager.is_bookmarked("https://github.com"));
    assert!(!manager.is_bookmarked("https://gitlab.com"));
}

#[test]
fn test_manager_find_by_url() {
    let manager = BookmarkManager::new(None);
    manager.add("https://rust-lang.org", "Rust");

    let b = manager.find_by_url("https://rust-lang.org").unwrap();
    assert_eq!(b.title, "Rust");

    assert!(manager.find_by_url("https://missing.com").is_none());
}

#[test]
fn test_manager_remove() {
    let manager = BookmarkManager::new(None);
    let id = manager.add("https://github.com", "GitHub");

    assert!(manager.remove(&id));
    assert!(!manager.is_bookmarked("https://github.com"));
    assert!(!manager.remove(&id)); // second removal returns false
}

#[test]
fn test_manager_update_title() {
    let manager = BookmarkManager::new(None);
    let id = manager.add("https://github.com", "GitHub");

    manager.update(&id, Some("GitHub Updated"), None).unwrap();
    let b = manager.get(&id).unwrap();
    assert_eq!(b.title, "GitHub Updated");
}

#[test]
fn test_manager_update_url() {
    let manager = BookmarkManager::new(None);
    let id = manager.add("https://old.com", "Old");

    manager.update(&id, None, Some("https://new.com")).unwrap();
    let b = manager.get(&id).unwrap();
    assert_eq!(b.url, "https://new.com");
}

#[test]
fn test_manager_update_nonexistent_returns_error() {
    let manager = BookmarkManager::new(None);
    let result = manager.update("bad-id", Some("Title"), None);
    assert!(matches!(result, Err(BookmarkError::NotFound(_))));
}

#[test]
fn test_manager_all_returns_all_bookmarks() {
    let manager = BookmarkManager::new(None);
    manager.add("https://a.com", "A");
    manager.add("https://b.com", "B");
    manager.add("https://c.com", "C");

    assert_eq!(manager.count(), 3);
    assert_eq!(manager.all().len(), 3);
}

#[test]
fn test_manager_clear() {
    let manager = BookmarkManager::new(None);
    manager.add("https://a.com", "A");
    manager.add("https://b.com", "B");
    assert_eq!(manager.count(), 2);

    manager.clear();
    assert_eq!(manager.count(), 0);
}

// ========== Search Tests ==========

#[rstest]
#[case("git", 2)]
#[case("rust", 1)]
#[case("xyz_not_found", 0)]
fn test_manager_search(#[case] query: &str, #[case] expected_count: usize) {
    let manager = BookmarkManager::new(None);
    manager.add("https://github.com", "GitHub");
    manager.add("https://gitlab.com", "GitLab");
    manager.add("https://rust-lang.org", "Rust Lang");

    let results = manager.search(query);
    assert_eq!(results.len(), expected_count);
}

// ========== Folder Tests ==========

#[test]
fn test_special_folders_exist_on_creation() {
    let manager = BookmarkManager::new(None);

    // Special folder IDs defined in folder::special_folders
    let all_folders = manager.all_folders();
    // Should have at least 2 special folders (bookmarks bar + other bookmarks)
    assert!(all_folders.len() >= 2);
}

#[test]
fn test_create_folder() {
    let manager = BookmarkManager::new(None);
    let folder_id = manager.create_folder("Development");

    let folder = manager.get_folder(&folder_id).unwrap();
    assert_eq!(folder.name, "Development");
}

#[test]
fn test_add_to_folder() {
    let manager = BookmarkManager::new(None);
    let folder_id = manager.create_folder("Work");
    let bm_id = manager.add_to_folder("https://github.com", "GitHub", &folder_id).unwrap();

    let b = manager.get(&bm_id).unwrap();
    assert_eq!(b.parent_id, Some(folder_id.clone()));

    let in_folder = manager.in_folder(&folder_id);
    assert_eq!(in_folder.len(), 1);
    assert_eq!(in_folder[0].title, "GitHub");
}

#[test]
fn test_add_to_nonexistent_folder_returns_error() {
    let manager = BookmarkManager::new(None);
    let result = manager.add_to_folder("https://a.com", "A", "bad-folder");
    assert!(matches!(result, Err(BookmarkError::FolderNotFound(_))));
}

#[test]
fn test_create_subfolder() {
    let manager = BookmarkManager::new(None);
    let parent_id = manager.create_folder("Parent");
    let child_id = manager.create_subfolder("Child", &parent_id).unwrap();

    let child = manager.get_folder(&child_id).unwrap();
    assert_eq!(child.name, "Child");
    assert_eq!(child.parent_id, Some(parent_id.clone()));

    let subs = manager.subfolders(&parent_id);
    assert_eq!(subs.len(), 1);
    assert_eq!(subs[0].id, child_id);
}

#[test]
fn test_create_subfolder_bad_parent_returns_error() {
    let manager = BookmarkManager::new(None);
    let result = manager.create_subfolder("Child", "bad-parent");
    assert!(matches!(result, Err(BookmarkError::FolderNotFound(_))));
}

#[test]
fn test_rename_folder() {
    let manager = BookmarkManager::new(None);
    let folder_id = manager.create_folder("Old Name");

    manager.rename_folder(&folder_id, "New Name").unwrap();
    let folder = manager.get_folder(&folder_id).unwrap();
    assert_eq!(folder.name, "New Name");
}

#[test]
fn test_rename_nonexistent_folder_returns_error() {
    let manager = BookmarkManager::new(None);
    let result = manager.rename_folder("bad-id", "Name");
    assert!(matches!(result, Err(BookmarkError::FolderNotFound(_))));
}

#[test]
fn test_delete_folder_with_contents() {
    let manager = BookmarkManager::new(None);
    let folder_id = manager.create_folder("ToBe");
    manager.add_to_folder("https://a.com", "A", &folder_id).unwrap();

    manager.delete_folder(&folder_id, true).unwrap();

    assert!(manager.get_folder(&folder_id).is_none());
    assert_eq!(manager.in_folder(&folder_id).len(), 0);
    assert!(!manager.is_bookmarked("https://a.com"));
}

#[test]
fn test_delete_folder_moves_contents_to_root() {
    let manager = BookmarkManager::new(None);
    let folder_id = manager.create_folder("ToBe");
    let bm_id = manager.add_to_folder("https://a.com", "A", &folder_id).unwrap();

    manager.delete_folder(&folder_id, false).unwrap();

    assert!(manager.get_folder(&folder_id).is_none());
    // Bookmark should still exist, but without a parent
    let b = manager.get(&bm_id).unwrap();
    assert!(b.parent_id.is_none());
}

#[test]
fn test_delete_nonexistent_folder_returns_error() {
    let manager = BookmarkManager::new(None);
    let result = manager.delete_folder("bad-id", true);
    assert!(matches!(result, Err(BookmarkError::FolderNotFound(_))));
}

// ========== Move Tests ==========

#[test]
fn test_move_bookmark_to_folder() {
    let manager = BookmarkManager::new(None);
    let folder_id = manager.create_folder("Dest");
    let bm_id = manager.add("https://a.com", "A");

    manager.move_to_folder(&bm_id, Some(&folder_id)).unwrap();
    let b = manager.get(&bm_id).unwrap();
    assert_eq!(b.parent_id, Some(folder_id));
}

#[test]
fn test_move_bookmark_to_root() {
    let manager = BookmarkManager::new(None);
    let folder_id = manager.create_folder("Src");
    let bm_id = manager.add_to_folder("https://a.com", "A", &folder_id).unwrap();

    manager.move_to_folder(&bm_id, None).unwrap();
    let b = manager.get(&bm_id).unwrap();
    assert!(b.parent_id.is_none());
}

#[test]
fn test_move_to_nonexistent_folder_returns_error() {
    let manager = BookmarkManager::new(None);
    let bm_id = manager.add("https://a.com", "A");
    let result = manager.move_to_folder(&bm_id, Some("bad-folder"));
    assert!(matches!(result, Err(BookmarkError::FolderNotFound(_))));
}

// ========== Root Bookmarks Tests ==========

#[test]
fn test_root_bookmarks_excludes_folder_items() {
    let manager = BookmarkManager::new(None);
    let folder_id = manager.create_folder("Folder");

    manager.add("https://root.com", "Root");
    manager.add_to_folder("https://in-folder.com", "InFolder", &folder_id).unwrap();

    let root = manager.root_bookmarks();
    assert_eq!(root.len(), 1);
    assert_eq!(root[0].title, "Root");
}

// ========== Persistence Tests ==========

#[test]
fn test_persistence_save_and_reload() {
    let dir = TempDir::new().unwrap();
    let path = dir.path();

    // Create and save
    {
        let manager = BookmarkManager::new(Some(path));
        manager.add("https://github.com", "GitHub");
        manager.add("https://rust-lang.org", "Rust");
        manager.create_folder("Dev");
    }

    // Reload
    let manager2 = BookmarkManager::new(Some(path));
    assert_eq!(manager2.count(), 2);
    assert!(manager2.is_bookmarked("https://github.com"));
    assert!(manager2.is_bookmarked("https://rust-lang.org"));
}

#[test]
fn test_export_import() {
    let manager1 = BookmarkManager::new(None);
    manager1.add("https://a.com", "A");
    manager1.add("https://b.com", "B");

    let json = manager1.export().unwrap();

    let manager2 = BookmarkManager::new(None);
    manager2.import(&json).unwrap();

    assert!(manager2.is_bookmarked("https://a.com"));
    assert!(manager2.is_bookmarked("https://b.com"));
}

// ========== Clone Tests ==========

#[test]
fn test_clone_shares_state() {
    let manager1 = BookmarkManager::new(None);
    let manager2 = manager1.clone();

    manager1.add("https://shared.com", "Shared");
    assert!(manager2.is_bookmarked("https://shared.com"));
}

// ========== BookmarkFolder Tests ==========

#[test]
fn test_bookmark_folder_new() {
    let folder = BookmarkFolder::new("My Folder");
    assert_eq!(folder.name, "My Folder");
    assert!(!folder.id.is_empty());
    assert!(folder.parent_id.is_none());
}

#[test]
fn test_bookmark_folder_set_name() {
    let mut folder = BookmarkFolder::new("Old");
    folder.set_name("New");
    assert_eq!(folder.name, "New");
}
