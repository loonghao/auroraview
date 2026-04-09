use std::sync::{Arc, Mutex};
use std::thread;

use auroraview_bookmarks::{Bookmark, BookmarkError, BookmarkFolder, BookmarkManager};
use rstest::*;
use tempfile::TempDir;

// ========== Bookmark Struct Tests ==========


#[rstest]
fn bookmark_new() {
    let b = Bookmark::new("GitHub", "https://github.com");
    assert_eq!(b.title, "GitHub");
    assert_eq!(b.url, "https://github.com");
    assert!(!b.id.is_empty());
    assert!(b.parent_id.is_none());
    assert!(b.favicon.is_none());
    assert!(b.tags.is_empty());
}

#[rstest]
fn bookmark_with_id() {
    let b = Bookmark::with_id("custom-id", "Test", "https://test.com");
    assert_eq!(b.id, "custom-id");
}

#[rstest]
fn bookmark_builder_methods() {
    let b = Bookmark::new("Test", "https://test.com")
        .with_favicon("https://test.com/favicon.ico")
        .with_position(3)
        .with_tag("dev")
        .with_tag("rust");

    assert_eq!(b.favicon, Some("https://test.com/favicon.ico".to_string()));
    assert_eq!(b.position, 3);
    assert_eq!(b.tags, vec!["dev".to_string(), "rust".to_string()]);
}

#[rstest]
fn bookmark_with_tags_replaces_all() {
    let b = Bookmark::new("T", "https://t.com")
        .with_tag("old")
        .with_tags(vec!["a".to_string(), "b".to_string()]);
    assert_eq!(b.tags, vec!["a".to_string(), "b".to_string()]);
}

#[rstest]
fn bookmark_with_parent() {
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

#[rstest]
fn bookmark_set_title() {
    let mut b = Bookmark::new("Old", "https://example.com");
    b.set_title("New");
    assert_eq!(b.title, "New");
}

#[rstest]
fn bookmark_set_url() {
    let mut b = Bookmark::new("T", "https://old.com");
    b.set_url("https://new.com");
    assert_eq!(b.url, "https://new.com");
}

#[rstest]
fn bookmark_set_favicon() {
    let mut b = Bookmark::new("T", "https://t.com");
    b.set_favicon(Some("https://t.com/fav.ico".to_string()));
    assert_eq!(b.favicon, Some("https://t.com/fav.ico".to_string()));

    b.set_favicon(None);
    assert!(b.favicon.is_none());
}

#[rstest]
fn bookmark_set_parent() {
    let mut b = Bookmark::new("T", "https://t.com");
    b.set_parent(Some("folder-1".to_string()));
    assert_eq!(b.parent_id, Some("folder-1".to_string()));

    b.set_parent(None);
    assert!(b.parent_id.is_none());
}

// ========== BookmarkManager CRUD Tests ==========

#[rstest]
fn manager_add_and_get() {
    let manager = BookmarkManager::new(None);

    let id = manager.add("https://github.com", "GitHub");
    let b = manager.get(&id).unwrap();

    assert_eq!(b.title, "GitHub");
    assert_eq!(b.url, "https://github.com");
}

#[rstest]
fn manager_is_bookmarked() {
    let manager = BookmarkManager::new(None);

    manager.add("https://github.com", "GitHub");

    assert!(manager.is_bookmarked("https://github.com"));
    assert!(!manager.is_bookmarked("https://gitlab.com"));
}

#[rstest]
fn manager_find_by_url() {
    let manager = BookmarkManager::new(None);
    manager.add("https://rust-lang.org", "Rust");

    let b = manager.find_by_url("https://rust-lang.org").unwrap();
    assert_eq!(b.title, "Rust");

    assert!(manager.find_by_url("https://missing.com").is_none());
}

#[rstest]
fn manager_remove() {
    let manager = BookmarkManager::new(None);
    let id = manager.add("https://github.com", "GitHub");

    assert!(manager.remove(&id));
    assert!(!manager.is_bookmarked("https://github.com"));
    assert!(!manager.remove(&id)); // second removal returns false
}

#[rstest]
fn manager_update_title() {
    let manager = BookmarkManager::new(None);
    let id = manager.add("https://github.com", "GitHub");

    manager.update(&id, Some("GitHub Updated"), None).unwrap();
    let b = manager.get(&id).unwrap();
    assert_eq!(b.title, "GitHub Updated");
}

#[rstest]
fn manager_update_url() {
    let manager = BookmarkManager::new(None);
    let id = manager.add("https://old.com", "Old");

    manager.update(&id, None, Some("https://new.com")).unwrap();
    let b = manager.get(&id).unwrap();
    assert_eq!(b.url, "https://new.com");
}

#[rstest]
fn manager_update_nonexistent_returns_error() {
    let manager = BookmarkManager::new(None);
    let result = manager.update("bad-id", Some("Title"), None);
    assert!(matches!(result, Err(BookmarkError::NotFound(_))));
}

#[rstest]
fn manager_all_returns_all_bookmarks() {
    let manager = BookmarkManager::new(None);
    manager.add("https://a.com", "A");
    manager.add("https://b.com", "B");
    manager.add("https://c.com", "C");

    assert_eq!(manager.count(), 3);
    assert_eq!(manager.all().len(), 3);
}

#[rstest]
fn manager_clear() {
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

#[rstest]
fn special_folders_exist_on_creation() {
    let manager = BookmarkManager::new(None);

    // Special folder IDs defined in folder::special_folders
    let all_folders = manager.all_folders();
    // Should have at least 2 special folders (bookmarks bar + other bookmarks)
    assert!(all_folders.len() >= 2);
}

#[rstest]
fn create_folder() {
    let manager = BookmarkManager::new(None);
    let folder_id = manager.create_folder("Development");

    let folder = manager.get_folder(&folder_id).unwrap();
    assert_eq!(folder.name, "Development");
}

#[rstest]
fn add_to_folder() {
    let manager = BookmarkManager::new(None);
    let folder_id = manager.create_folder("Work");
    let bm_id = manager
        .add_to_folder("https://github.com", "GitHub", &folder_id)
        .unwrap();

    let b = manager.get(&bm_id).unwrap();
    assert_eq!(b.parent_id, Some(folder_id.clone()));

    let in_folder = manager.in_folder(&folder_id);
    assert_eq!(in_folder.len(), 1);
    assert_eq!(in_folder[0].title, "GitHub");
}

#[rstest]
fn add_to_nonexistent_folder_returns_error() {
    let manager = BookmarkManager::new(None);
    let result = manager.add_to_folder("https://a.com", "A", "bad-folder");
    assert!(matches!(result, Err(BookmarkError::FolderNotFound(_))));
}

#[rstest]
fn create_subfolder() {
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

#[rstest]
fn create_subfolder_bad_parent_returns_error() {
    let manager = BookmarkManager::new(None);
    let result = manager.create_subfolder("Child", "bad-parent");
    assert!(matches!(result, Err(BookmarkError::FolderNotFound(_))));
}

#[rstest]
fn rename_folder() {
    let manager = BookmarkManager::new(None);
    let folder_id = manager.create_folder("Old Name");

    manager.rename_folder(&folder_id, "New Name").unwrap();
    let folder = manager.get_folder(&folder_id).unwrap();
    assert_eq!(folder.name, "New Name");
}

#[rstest]
fn rename_nonexistent_folder_returns_error() {
    let manager = BookmarkManager::new(None);
    let result = manager.rename_folder("bad-id", "Name");
    assert!(matches!(result, Err(BookmarkError::FolderNotFound(_))));
}

#[rstest]
fn delete_folder_with_contents() {
    let manager = BookmarkManager::new(None);
    let folder_id = manager.create_folder("ToBe");
    manager
        .add_to_folder("https://a.com", "A", &folder_id)
        .unwrap();

    manager.delete_folder(&folder_id, true).unwrap();

    assert!(manager.get_folder(&folder_id).is_none());
    assert_eq!(manager.in_folder(&folder_id).len(), 0);
    assert!(!manager.is_bookmarked("https://a.com"));
}

#[rstest]
fn delete_folder_moves_contents_to_root() {
    let manager = BookmarkManager::new(None);
    let folder_id = manager.create_folder("ToBe");
    let bm_id = manager
        .add_to_folder("https://a.com", "A", &folder_id)
        .unwrap();

    manager.delete_folder(&folder_id, false).unwrap();

    assert!(manager.get_folder(&folder_id).is_none());
    // Bookmark should still exist, but without a parent
    let b = manager.get(&bm_id).unwrap();
    assert!(b.parent_id.is_none());
}

#[rstest]
fn delete_nonexistent_folder_returns_error() {
    let manager = BookmarkManager::new(None);
    let result = manager.delete_folder("bad-id", true);
    assert!(matches!(result, Err(BookmarkError::FolderNotFound(_))));
}

// ========== Move Tests ==========

#[rstest]
fn move_bookmark_to_folder() {
    let manager = BookmarkManager::new(None);
    let folder_id = manager.create_folder("Dest");
    let bm_id = manager.add("https://a.com", "A");

    manager.move_to_folder(&bm_id, Some(&folder_id)).unwrap();
    let b = manager.get(&bm_id).unwrap();
    assert_eq!(b.parent_id, Some(folder_id));
}

#[rstest]
fn move_bookmark_to_root() {
    let manager = BookmarkManager::new(None);
    let folder_id = manager.create_folder("Src");
    let bm_id = manager
        .add_to_folder("https://a.com", "A", &folder_id)
        .unwrap();

    manager.move_to_folder(&bm_id, None).unwrap();
    let b = manager.get(&bm_id).unwrap();
    assert!(b.parent_id.is_none());
}

#[rstest]
fn move_to_nonexistent_folder_returns_error() {
    let manager = BookmarkManager::new(None);
    let bm_id = manager.add("https://a.com", "A");
    let result = manager.move_to_folder(&bm_id, Some("bad-folder"));
    assert!(matches!(result, Err(BookmarkError::FolderNotFound(_))));
}

// ========== Root Bookmarks Tests ==========

#[rstest]
fn root_bookmarks_excludes_folder_items() {
    let manager = BookmarkManager::new(None);
    let folder_id = manager.create_folder("Folder");

    manager.add("https://root.com", "Root");
    manager
        .add_to_folder("https://in-folder.com", "InFolder", &folder_id)
        .unwrap();

    let root = manager.root_bookmarks();
    assert_eq!(root.len(), 1);
    assert_eq!(root[0].title, "Root");
}

// ========== Persistence Tests ==========

#[rstest]
fn persistence_save_and_reload() {
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

#[rstest]
fn export_import() {
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

#[rstest]
fn clone_shares_state() {
    let manager1 = BookmarkManager::new(None);
    let manager2 = manager1.clone();

    manager1.add("https://shared.com", "Shared");
    assert!(manager2.is_bookmarked("https://shared.com"));
}

// ========== BookmarkFolder Tests ==========

#[rstest]
fn bookmark_folder_new() {
    let folder = BookmarkFolder::new("My Folder");
    assert_eq!(folder.name, "My Folder");
    assert!(!folder.id.is_empty());
    assert!(folder.parent_id.is_none());
}

#[rstest]
fn bookmark_folder_set_name() {
    let mut folder = BookmarkFolder::new("Old");
    folder.set_name("New");
    assert_eq!(folder.name, "New");
}

// ========== Serde Roundtrip Tests ==========

#[test]
fn serde_bookmark_basic_roundtrip() {
    let b = Bookmark::new("GitHub", "https://github.com");
    let json = serde_json::to_string(&b).unwrap();
    let back: Bookmark = serde_json::from_str(&json).unwrap();

    assert_eq!(back.id, b.id);
    assert_eq!(back.title, "GitHub");
    assert_eq!(back.url, "https://github.com");
    assert!(back.parent_id.is_none());
    assert!(back.favicon.is_none());
    assert!(back.tags.is_empty());
}

#[test]
fn serde_bookmark_full_roundtrip() {
    let b = Bookmark::new("Test", "https://test.com")
        .with_favicon("https://test.com/fav.ico")
        .with_position(5)
        .with_tag("dev")
        .with_tag("rust")
        .with_parent("parent-folder-id");

    let json = serde_json::to_string(&b).unwrap();
    let back: Bookmark = serde_json::from_str(&json).unwrap();

    assert_eq!(back.title, "Test");
    assert_eq!(back.favicon, Some("https://test.com/fav.ico".to_string()));
    assert_eq!(back.position, 5);
    assert_eq!(back.tags, vec!["dev".to_string(), "rust".to_string()]);
    assert_eq!(back.parent_id, Some("parent-folder-id".to_string()));
}

#[test]
fn serde_bookmark_omits_optional_fields() {
    let b = Bookmark::new("T", "https://t.com");
    let json = serde_json::to_string(&b).unwrap();
    // Optional fields should be absent
    assert!(!json.contains(r#""favicon""#));
    assert!(!json.contains(r#""parent_id""#));
    // tags empty → should be absent too
    assert!(!json.contains(r#""tags""#));
}

#[test]
fn serde_bookmark_folder_roundtrip() {
    let folder = BookmarkFolder::new("My Folder");
    let json = serde_json::to_string(&folder).unwrap();
    let back: BookmarkFolder = serde_json::from_str(&json).unwrap();

    assert_eq!(back.name, "My Folder");
    assert_eq!(back.id, folder.id);
    assert!(back.parent_id.is_none());
}

// ========== BookmarkError Display Tests ==========

#[test]
fn error_not_found_display() {
    let err = BookmarkError::NotFound("bm-123".to_string());
    let msg = err.to_string();
    assert!(msg.contains("bm-123"), "got: {msg}");
    assert!(msg.to_lowercase().contains("not found"), "got: {msg}");
}

#[test]
fn error_folder_not_found_display() {
    let err = BookmarkError::FolderNotFound("folder-xyz".to_string());
    let msg = err.to_string();
    assert!(msg.contains("folder-xyz"), "got: {msg}");
}

#[test]
fn error_invalid_url_display() {
    let err = BookmarkError::InvalidUrl("not-a-url".to_string());
    let msg = err.to_string();
    assert!(msg.contains("not-a-url"), "got: {msg}");
}

#[test]
fn error_storage_display() {
    let err = BookmarkError::Storage("disk full".to_string());
    let msg = err.to_string();
    assert!(msg.contains("disk full"), "got: {msg}");
}

// ========== Position / Ordering Edge Cases ==========

#[rstest]
fn bookmark_position_zero_default() {
    let b = Bookmark::new("T", "https://t.com");
    assert_eq!(b.position, 0);
}

#[rstest]
fn bookmark_large_position() {
    let b = Bookmark::new("T", "https://t.com").with_position(u32::MAX);
    assert_eq!(b.position, u32::MAX);
}

#[rstest]
fn bookmark_with_id_is_stable() {
    let b = Bookmark::with_id("stable-id", "Title", "https://stable.com");
    assert_eq!(b.id, "stable-id");
    // Creating again with same id gives same id
    let b2 = Bookmark::with_id("stable-id", "Other", "https://other.com");
    assert_eq!(b2.id, "stable-id");
}

// ========== Search Edge Cases ==========

#[rstest]
fn manager_search_empty_query_matches_all() {
    let manager = BookmarkManager::new(None);
    manager.add("https://a.com", "A");
    manager.add("https://b.com", "B");
    // empty query matches everything
    let results = manager.search("");
    assert_eq!(results.len(), 2);
}

#[rstest]
fn manager_search_by_tag() {
    let manager = BookmarkManager::new(None);
    // Build a bookmark with a unique tag via a staging manager, then export/import
    let staging = BookmarkManager::new(None);
    staging.add("https://rust-lang.org", "Rust");
    let tagged_id = staging.add("https://tagged.com", "Tagged");
    // Update the bookmark to add a tag via a roundtrip through export
    // Since there's no direct "add tag" API on manager, use bookmark struct + import
    let b = staging.get(&tagged_id).unwrap();
    // Remove from staging and re-add with tag via export format
    staging.remove(&tagged_id);
    let b_with_tag = Bookmark::new(b.title.clone(), b.url.clone()).with_tag("unique-tag");
    let bm_id = b_with_tag.id.clone();

    // Build the export JSON manually in the correct HashMap<id, Bookmark> format
    let mut bm_map = std::collections::HashMap::new();
    bm_map.insert(bm_id.clone(), b_with_tag);
    let json = serde_json::json!({
        "bookmarks": bm_map,
        "folders": {}
    })
    .to_string();

    manager.import(&json).unwrap();

    let results = manager.search("unique-tag");
    assert!(
        results.iter().any(|bm| bm.id == bm_id),
        "tag search should find the bookmark"
    );
}

#[rstest]
#[case("https://github.com", true)]
#[case("https://GITHUB.COM", false)] // case sensitive URL check
#[case("https://gitlab.com", false)]
fn test_is_bookmarked_exact_url(#[case] url: &str, #[case] expected: bool) {
    let manager = BookmarkManager::new(None);
    manager.add("https://github.com", "GitHub");
    assert_eq!(manager.is_bookmarked(url), expected);
}

// ========== Multiple Folders ==========

#[rstest]
fn multiple_subfolders_under_same_parent() {
    let manager = BookmarkManager::new(None);
    let parent = manager.create_folder("Parent");
    let _c1 = manager.create_subfolder("Child1", &parent).unwrap();
    let _c2 = manager.create_subfolder("Child2", &parent).unwrap();
    let _c3 = manager.create_subfolder("Child3", &parent).unwrap();

    let subs = manager.subfolders(&parent);
    assert_eq!(subs.len(), 3);
}

#[rstest]
fn all_folders_includes_created() {
    let manager = BookmarkManager::new(None);
    let initial_count = manager.all_folders().len();
    manager.create_folder("Extra1");
    manager.create_folder("Extra2");

    assert_eq!(manager.all_folders().len(), initial_count + 2);
}

#[rstest]
fn in_folder_returns_empty_for_nonexistent_folder() {
    let manager = BookmarkManager::new(None);
    let items = manager.in_folder("nonexistent-folder-id");
    assert!(items.is_empty());
}

// ========== Concurrent Tests ==========

#[test]
fn concurrent_add_no_panic() {

    let manager = Arc::new(BookmarkManager::new(None));

    let handles: Vec<_> = (0..8)
        .map(|i| {
            let m = Arc::clone(&manager);
            thread::spawn(move || {
                for j in 0..10 {
                    m.add(format!("https://thread{i}-site{j}.com"), format!("Site{i}-{j}"));
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(manager.count(), 80);
}

#[test]
fn concurrent_add_and_search_no_deadlock() {

    let manager = Arc::new(BookmarkManager::new(None));

    let writer = {
        let m = Arc::clone(&manager);
        thread::spawn(move || {
            for i in 0..50 {
                m.add(format!("https://concurrent{i}.com"), format!("Concurrent{i}"));
            }
        })
    };

    let reader = {
        let m = Arc::clone(&manager);
        thread::spawn(move || {
            for _ in 0..50 {
                let _ = m.search("concurrent");
            }
        })
    };

    writer.join().unwrap();
    reader.join().unwrap();
}

#[test]
fn concurrent_remove_no_panic() {

    let manager = Arc::new(BookmarkManager::new(None));
    let ids: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    // Add 40 bookmarks first
    for i in 0..40 {
        let id = manager.add(format!("https://rm{i}.com"), format!("Rm{i}"));
        ids.lock().unwrap().push(id);
    }

    // Remove them concurrently
    let ids_snap: Vec<String> = ids.lock().unwrap().clone();
    let chunks: Vec<_> = ids_snap.chunks(10).map(|c| c.to_vec()).collect();

    let handles: Vec<_> = chunks
        .into_iter()
        .map(|chunk| {
            let m = Arc::clone(&manager);
            thread::spawn(move || {
                for id in chunk {
                    m.remove(&id);
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(manager.count(), 0);
}

// ========== Bookmark set_* mutation tests ==========

#[rstest]
fn bookmark_set_title_updates_modified_at() {
    let mut b = Bookmark::new("Old Title", "https://example.com");
    let original_modified = b.modified_at;
    std::thread::sleep(std::time::Duration::from_millis(5));
    b.set_title("New Title");
    assert_eq!(b.title, "New Title");
    assert!(b.modified_at >= original_modified);
}

#[rstest]
fn bookmark_set_url_mutation() {
    let mut b = Bookmark::new("Title", "https://old.com");
    b.set_url("https://new.com");
    assert_eq!(b.url, "https://new.com");
}

#[rstest]
fn bookmark_set_favicon_some_and_none() {
    let mut b = Bookmark::new("T", "https://example.com");
    b.set_favicon(Some("https://example.com/favicon.ico".to_string()));
    assert_eq!(b.favicon, Some("https://example.com/favicon.ico".to_string()));
    b.set_favicon(None);
    assert!(b.favicon.is_none());
}

#[rstest]
fn bookmark_set_parent_mutation() {
    let mut b = Bookmark::new("T", "https://example.com");
    b.set_parent(Some("folder-1".to_string()));
    assert_eq!(b.parent_id, Some("folder-1".to_string()));
    b.set_parent(None);
    assert!(b.parent_id.is_none());
}

#[rstest]
fn bookmark_matches_case_insensitive() {
    let b = Bookmark::new("GitHub", "https://github.com");
    assert!(b.matches("GITHUB"));
    assert!(b.matches("Github"));
    assert!(b.matches("GITHUB.COM"));
}

#[rstest]
fn bookmark_matches_by_tag_case_insensitive() {
    let b = Bookmark::new("T", "https://example.com").with_tag("RUST");
    assert!(b.matches("rust"));
    assert!(!b.matches("python"));
}

#[rstest]
fn bookmark_serde_roundtrip() {
    let b = Bookmark::new("GitHub", "https://github.com")
        .with_tag("code")
        .with_favicon("https://github.com/favicon.ico")
        .with_position(2)
        .with_parent("folder-1");
    let json = serde_json::to_string(&b).unwrap();
    let back: Bookmark = serde_json::from_str(&json).unwrap();
    assert_eq!(back.title, "GitHub");
    assert_eq!(back.tags, vec!["code"]);
    assert_eq!(back.position, 2);
    assert_eq!(back.parent_id, Some("folder-1".to_string()));
}

// ========== BookmarkFolder set_* and special_folders ==========

#[rstest]
fn folder_set_name() {
    let mut f = BookmarkFolder::new("Old");
    f.set_name("New");
    assert_eq!(f.name, "New");
}

#[rstest]
fn folder_set_parent_some_and_none() {
    let mut f = BookmarkFolder::new("F");
    f.set_parent(Some("parent-1".to_string()));
    assert_eq!(f.parent_id, Some("parent-1".to_string()));
    f.set_parent(None);
    assert!(f.parent_id.is_none());
}

#[rstest]
fn folder_with_icon() {
    let f = BookmarkFolder::new("F").with_icon("folder-icon.png");
    assert_eq!(f.icon, Some("folder-icon.png".to_string()));
}

#[rstest]
fn folder_with_id_stable() {
    let f = BookmarkFolder::with_id("stable-id", "Folder");
    assert_eq!(f.id, "stable-id");
}

#[rstest]
fn folder_serde_roundtrip() {
    let f = BookmarkFolder::new("Dev")
        .with_icon("dev.png")
        .with_position(3)
        .with_parent("root");
    let json = serde_json::to_string(&f).unwrap();
    let back: BookmarkFolder = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, "Dev");
    assert_eq!(back.icon, Some("dev.png".to_string()));
    assert_eq!(back.position, 3);
    assert_eq!(back.parent_id, Some("root".to_string()));
}

#[rstest]
fn manager_folder_count() {
    let manager = BookmarkManager::new(None);
    let initial = manager.folder_count(); // 2 special folders pre-created
    manager.create_folder("F1");
    manager.create_folder("F2");
    assert_eq!(manager.folder_count(), initial + 2);
}

#[rstest]
fn manager_root_bookmarks_only_root_items() {
    let manager = BookmarkManager::new(None);
    let folder_id = manager.create_folder("WorkFolder");
    let root_id = manager.add("https://root.com", "Root");
    manager.add_to_folder("https://in-folder.com", "InFolder", &folder_id).unwrap();

    let root_bms = manager.root_bookmarks();
    assert!(root_bms.iter().any(|b| b.id == root_id));
    assert!(!root_bms.iter().any(|b| b.url == "https://in-folder.com"));
}

#[rstest]
fn manager_update_title_only() {
    let manager = BookmarkManager::new(None);
    let id = manager.add("https://example.com", "Old Title");
    manager.update(&id, Some("New Title"), None).unwrap();
    let b = manager.get(&id).unwrap();
    assert_eq!(b.title, "New Title");
    assert_eq!(b.url, "https://example.com");
}

#[rstest]
fn manager_update_url_only() {
    let manager = BookmarkManager::new(None);
    let id = manager.add("https://old.com", "Title");
    manager.update(&id, None, Some("https://new.com")).unwrap();
    let b = manager.get(&id).unwrap();
    assert_eq!(b.url, "https://new.com");
    assert_eq!(b.title, "Title");
}

#[rstest]
fn manager_update_nonexistent_bookmark_error() {
    let manager = BookmarkManager::new(None);
    let result = manager.update("ghost", Some("T"), None);
    assert!(result.is_err());
}

#[rstest]
fn manager_find_by_url_returns_correct() {
    let manager = BookmarkManager::new(None);
    manager.add("https://rust-lang.org", "Rust");
    let found = manager.find_by_url("https://rust-lang.org");
    assert!(found.is_some());
    assert_eq!(found.unwrap().title, "Rust");
}

#[rstest]
fn manager_find_by_url_not_found() {
    let manager = BookmarkManager::new(None);
    assert!(manager.find_by_url("https://nonexistent.com").is_none());
}

#[rstest]
fn manager_move_to_folder_and_back() {
    let manager = BookmarkManager::new(None);
    let folder_id = manager.create_folder("Dev");
    let id = manager.add("https://github.com", "GitHub");

    manager.move_to_folder(&id, Some(&folder_id)).unwrap();
    let b = manager.get(&id).unwrap();
    assert_eq!(b.parent_id, Some(folder_id.clone()));

    manager.move_to_folder(&id, None).unwrap();
    let b = manager.get(&id).unwrap();
    assert!(b.parent_id.is_none());
}

#[rstest]
fn manager_move_nonexistent_returns_error() {
    let manager = BookmarkManager::new(None);
    let result = manager.move_to_folder("ghost", None);
    assert!(result.is_err());
}

#[rstest]
fn manager_rename_folder() {
    let manager = BookmarkManager::new(None);
    let id = manager.create_folder("OldName");
    manager.rename_folder(&id, "NewName").unwrap();
    let folder = manager.get_folder(&id).unwrap();
    assert_eq!(folder.name, "NewName");
}

#[rstest]
fn manager_rename_nonexistent_folder_error() {
    let manager = BookmarkManager::new(None);
    let result = manager.rename_folder("ghost_folder", "Name");
    assert!(result.is_err());
}

#[rstest]
fn manager_delete_folder_with_contents() {
    let manager = BookmarkManager::new(None);
    let folder_id = manager.create_folder("ToDelete");
    let _bm_id = manager.add_to_folder("https://a.com", "A", &folder_id).unwrap();
    manager.delete_folder(&folder_id, true).unwrap();
    assert!(manager.get_folder(&folder_id).is_none());
}

#[rstest]
fn manager_delete_folder_without_contents_keeps_bookmarks() {
    let manager = BookmarkManager::new(None);
    let folder_id = manager.create_folder("ToDelete");
    let bm_id = manager.add_to_folder("https://a.com", "A", &folder_id).unwrap();
    manager.delete_folder(&folder_id, false).unwrap();
    assert!(manager.get(&bm_id).is_some());
}

#[rstest]
fn manager_subfolders_returns_children() {
    let manager = BookmarkManager::new(None);
    let parent_id = manager.create_folder("Parent");
    manager.create_subfolder("Child1", &parent_id).unwrap();
    manager.create_subfolder("Child2", &parent_id).unwrap();
    let children = manager.subfolders(&parent_id);
    assert_eq!(children.len(), 2);
}

#[rstest]
fn manager_create_subfolder_nonexistent_parent_error() {
    let manager = BookmarkManager::new(None);
    let result = manager.create_subfolder("Child", "ghost_parent");
    assert!(result.is_err());
}

#[rstest]
fn manager_clear_removes_all() {
    let manager = BookmarkManager::new(None);
    manager.add("https://a.com", "A");
    manager.add("https://b.com", "B");
    assert_eq!(manager.count(), 2);
    manager.clear();
    assert_eq!(manager.count(), 0);
}

#[rstest]
fn manager_export_import_roundtrip() {
    let manager = BookmarkManager::new(None);
    manager.add("https://rust.rs", "Rust");
    manager.add("https://python.org", "Python");

    let json = manager.export().unwrap();
    let manager2 = BookmarkManager::new(None);
    manager2.import(&json).unwrap();
    assert_eq!(manager2.count(), 2);
}

// ========== BookmarkError display ==========

#[test]
fn bookmark_error_not_found_display() {
    let err = BookmarkError::NotFound("bm-99".to_string());
    assert!(err.to_string().contains("bm-99"));
}

#[test]
fn bookmark_error_folder_not_found_display() {
    let err = BookmarkError::FolderNotFound("folder-99".to_string());
    assert!(err.to_string().contains("folder-99"));
}

#[test]
fn bookmark_error_invalid_url_display() {
    let err = BookmarkError::InvalidUrl("not-a-url".to_string());
    assert!(err.to_string().contains("not-a-url"));
}

#[test]
fn bookmark_error_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<BookmarkError>();
}
