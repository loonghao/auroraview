//! Navigation coverage: HistoryManager and BookmarkManager edge cases
//! Complements history_tests.rs and bookmarks_tests.rs

use auroraview_browser::navigation::{Bookmark, BookmarkFolder, BookmarkManager, HistoryManager};
use chrono::{Duration, Utc};
use tempfile::TempDir;
use rstest::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn history(max: usize) -> (HistoryManager, TempDir) {
    let tmp = TempDir::new().unwrap();
    let mgr = HistoryManager::new(Some(tmp.path().to_str().unwrap()), max, true);
    (mgr, tmp)
}

fn bookmarks() -> (BookmarkManager, TempDir) {
    let tmp = TempDir::new().unwrap();
    let mgr = BookmarkManager::new(Some(tmp.path().to_str().unwrap()));
    (mgr, tmp)
}

// ===========================================================================
// HistoryManager – additional coverage
// ===========================================================================

#[test]
fn history_default_constructor_enabled() {
    // default() must not panic
    let mgr = HistoryManager::default();
    assert!(mgr.is_enabled());
}

#[test]
fn history_is_enabled_reflects_constructor() {
    let tmp = TempDir::new().unwrap();
    let mgr = HistoryManager::new(Some(tmp.path().to_str().unwrap()), 100, false);
    assert!(!mgr.is_enabled());
}

#[test]
fn history_get_respects_limit() {
    let (mgr, _tmp) = history(100);
    for i in 0..10 {
        mgr.add(&format!("https://site{i}.com"), &format!("Site {i}"));
    }
    let entries = mgr.get(3);
    assert_eq!(entries.len(), 3);
}

#[test]
fn history_all_returns_all_entries() {
    let (mgr, _tmp) = history(100);
    mgr.add("https://a.com", "A");
    mgr.add("https://b.com", "B");
    let all = mgr.all();
    assert_eq!(all.len(), 2);
}

#[test]
fn history_remove_nonexistent_returns_false() {
    let (mgr, _tmp) = history(100);
    assert!(!mgr.remove("https://ghost.com"));
}

#[test]
fn history_search_by_title() {
    let (mgr, _tmp) = history(100);
    mgr.add("https://example.com", "AuroraView Documentation");
    mgr.add("https://other.com", "Unrelated Page");

    let results = mgr.search("auroraview", 10);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].url, "https://example.com");
}

#[test]
fn history_search_case_insensitive() {
    let (mgr, _tmp) = history(100);
    mgr.add("https://example.com", "My Title");

    let results = mgr.search("MY TITLE", 10);
    assert_eq!(results.len(), 1);
}

#[test]
fn history_search_empty_query_returns_all() {
    let (mgr, _tmp) = history(100);
    mgr.add("https://a.com", "A");
    mgr.add("https://b.com", "B");

    // empty string matches everything (contains "")
    let results = mgr.search("", 100);
    assert_eq!(results.len(), 2);
}

#[test]
fn history_clear_before_removes_old_entries() {
    let (mgr, _tmp) = history(100);
    mgr.add("https://old.com", "Old");
    // add a very old entry by first adding then we verify count
    let count_before = mgr.count();
    assert_eq!(count_before, 1);

    // clear_before a future timestamp should clear everything
    let future = Utc::now() + Duration::days(1);
    mgr.clear_before(future);
    assert_eq!(mgr.count(), 0);
}

#[test]
fn history_clear_before_keeps_recent_entries() {
    let (mgr, _tmp) = history(100);
    mgr.add("https://recent.com", "Recent");

    // clear_before a past timestamp should keep everything
    let past = Utc::now() - Duration::days(1);
    mgr.clear_before(past);
    assert_eq!(mgr.count(), 1);
}

#[test]
fn history_grouped_by_date_groups_correctly() {
    let (mgr, _tmp) = history(100);
    mgr.add("https://a.com", "A");
    mgr.add("https://b.com", "B");

    let groups = mgr.grouped_by_date();
    // Both added today, should be in one group
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].1.len(), 2);
}

#[test]
fn history_grouped_by_date_empty() {
    let (mgr, _tmp) = history(100);
    let groups = mgr.grouped_by_date();
    assert!(groups.is_empty());
}

#[test]
fn history_today_returns_todays_entries() {
    let (mgr, _tmp) = history(100);
    mgr.add("https://today.com", "Today");

    let today = mgr.today();
    assert!(!today.is_empty());
    assert!(today.iter().any(|e| e.url == "https://today.com"));
}

#[test]
fn history_count_reflects_entries() {
    let (mgr, _tmp) = history(100);
    assert_eq!(mgr.count(), 0);
    mgr.add("https://a.com", "A");
    assert_eq!(mgr.count(), 1);
    mgr.add("https://b.com", "B");
    assert_eq!(mgr.count(), 2);
}

#[test]
fn history_disabled_skips_internal_urls() {
    // Even enabled, about: and data: are skipped
    let (mgr, _tmp) = history(100);
    mgr.add("about:newtab", "New Tab");
    mgr.add("data:text/plain,hello", "Data");
    assert_eq!(mgr.count(), 0);
}

// revisit increments count
#[test]
fn history_revisit_increments_count_beyond_two() {
    let (mgr, _tmp) = history(100);
    mgr.add("https://repeated.com", "Repeated");
    mgr.add("https://repeated.com", "Repeated v2");
    mgr.add("https://repeated.com", "Repeated v3");

    let entries = mgr.all();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].visit_count, 3);
    assert_eq!(entries[0].title, "Repeated v3");
}

// ===========================================================================
// BookmarkManager – additional coverage
// ===========================================================================

#[test]
fn bookmark_default_constructor() {
    let mgr = BookmarkManager::default();
    assert_eq!(mgr.count(), 0);
}

#[test]
fn bookmark_with_id_sets_custom_id() {
    let b = Bookmark::with_id("my-id", "Title", "https://example.com");
    assert_eq!(b.id, "my-id");
    assert_eq!(b.title, "Title");
    assert_eq!(b.url, "https://example.com");
}

#[test]
fn bookmark_with_parent_sets_parent_id() {
    let b = Bookmark::new("Child", "https://child.com").with_parent("parent-folder-id".to_string());
    assert_eq!(b.parent_id, Some("parent-folder-id".to_string()));
}

#[test]
fn bookmark_all_returns_sorted_by_position() {
    let (mgr, _tmp) = bookmarks();
    let b0 = Bookmark::new("Third", "https://c.com").with_position(2);
    let b1 = Bookmark::new("First", "https://a.com").with_position(0);
    let b2 = Bookmark::new("Second", "https://b.com").with_position(1);

    mgr.add(b0);
    mgr.add(b1);
    mgr.add(b2);

    let all = mgr.all();
    assert_eq!(all.len(), 3);
    assert_eq!(all[0].title, "First");
    assert_eq!(all[1].title, "Second");
    assert_eq!(all[2].title, "Third");
}

#[test]
fn bookmark_bar_items_returns_root_level() {
    let (mgr, _tmp) = bookmarks();
    let root = Bookmark::new("Root", "https://root.com");
    let folder_id = "folder-1".to_string();
    let child =
        Bookmark::new("Child", "https://child.com").with_parent(folder_id.clone());

    mgr.add(root);
    mgr.add(child);

    let bar = mgr.bar_items();
    assert_eq!(bar.len(), 1);
    assert_eq!(bar[0].title, "Root");
}

#[test]
fn bookmark_in_folder_returns_only_folder_items() {
    let (mgr, _tmp) = bookmarks();
    let folder_id = "f1".to_string();

    mgr.add(Bookmark::new("Root", "https://root.com"));
    mgr.add(Bookmark::new("InFolder", "https://infolder.com").with_parent(folder_id.clone()));

    let in_f = mgr.in_folder(Some(&folder_id));
    assert_eq!(in_f.len(), 1);
    assert_eq!(in_f[0].title, "InFolder");
}

#[test]
fn bookmark_in_folder_none_returns_root_items() {
    let (mgr, _tmp) = bookmarks();
    mgr.add(Bookmark::new("A", "https://a.com"));
    mgr.add(Bookmark::new("B", "https://b.com").with_parent("f1".to_string()));

    let root = mgr.in_folder(None);
    assert_eq!(root.len(), 1);
    assert_eq!(root[0].title, "A");
}

#[test]
fn bookmark_add_folder_and_list_folders() {
    let (mgr, _tmp) = bookmarks();
    let folder = BookmarkFolder::new("My Folder");
    let folder_id = mgr.add_folder(folder);

    let folders = mgr.folders();
    assert_eq!(folders.len(), 1);
    assert_eq!(folders[0].id, folder_id);
    assert_eq!(folders[0].name, "My Folder");
}

#[test]
fn bookmark_clear_removes_folders_too() {
    let (mgr, _tmp) = bookmarks();
    mgr.add_bookmark("https://a.com", "A");
    mgr.add_folder(BookmarkFolder::new("Folder"));

    assert_eq!(mgr.count(), 1);
    assert_eq!(mgr.folders().len(), 1);

    mgr.clear();
    assert_eq!(mgr.count(), 0);
    assert!(mgr.folders().is_empty());
}

#[test]
fn bookmark_remove_nonexistent_returns_none() {
    let (mgr, _tmp) = bookmarks();
    let result = mgr.remove(&"ghost-id".to_string());
    assert!(result.is_none());
}

#[test]
fn bookmark_update_url_only() {
    let (mgr, _tmp) = bookmarks();
    let id = mgr.add_bookmark("https://old.com", "Old");

    let updated = mgr.update(&id, None, Some("https://new.com".to_string()));
    assert!(updated);

    let b = mgr.get(&id).unwrap();
    assert_eq!(b.url, "https://new.com");
    assert_eq!(b.title, "Old");
}

#[test]
fn bookmark_update_both_title_and_url() {
    let (mgr, _tmp) = bookmarks();
    let id = mgr.add_bookmark("https://old.com", "Old Title");

    let updated = mgr.update(
        &id,
        Some("New Title".to_string()),
        Some("https://new.com".to_string()),
    );
    assert!(updated);

    let b = mgr.get(&id).unwrap();
    assert_eq!(b.title, "New Title");
    assert_eq!(b.url, "https://new.com");
}

#[test]
fn bookmark_update_nonexistent_returns_false() {
    let (mgr, _tmp) = bookmarks();
    let updated = mgr.update(&"ghost".to_string(), Some("Title".to_string()), None);
    assert!(!updated);
}

#[test]
fn bookmark_get_nonexistent_returns_none() {
    let (mgr, _tmp) = bookmarks();
    assert!(mgr.get(&"nonexistent".to_string()).is_none());
}

// rstest: is_bookmarked parametric
#[rstest]
#[case("https://example.com", true)]
#[case("https://ghost.com", false)]
fn bookmark_is_bookmarked_parametric(#[case] url: &str, #[case] expected: bool) {
    let (mgr, _tmp) = bookmarks();
    mgr.add_bookmark("https://example.com", "Example");
    assert_eq!(mgr.is_bookmarked(url), expected);
}
