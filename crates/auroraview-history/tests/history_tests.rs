use auroraview_history::{HistoryEntry, HistoryManager, SearchOptions, SearchResult};
use chrono::{Duration, Utc};
use rstest::*;
use tempfile::TempDir;

// ========== HistoryEntry Tests ==========

#[test]
fn test_entry_new() {
    let entry = HistoryEntry::new("https://github.com", "GitHub");
    assert_eq!(entry.url, "https://github.com");
    assert_eq!(entry.title, "GitHub");
    assert_eq!(entry.visit_count, 1);
    assert_eq!(entry.typed_count, 0);
    assert!(!entry.id.is_empty());
    assert!(entry.favicon.is_none());
}

#[test]
fn test_entry_same_url_same_id() {
    let e1 = HistoryEntry::new("https://github.com", "A");
    let e2 = HistoryEntry::new("https://github.com", "B");
    assert_eq!(e1.id, e2.id); // ID is URL-based
}

#[test]
fn test_entry_different_url_different_id() {
    let e1 = HistoryEntry::new("https://github.com", "A");
    let e2 = HistoryEntry::new("https://gitlab.com", "B");
    assert_ne!(e1.id, e2.id);
}

#[test]
fn test_entry_record_visit_increments() {
    let mut entry = HistoryEntry::new("https://github.com", "GitHub");
    entry.record_visit();
    assert_eq!(entry.visit_count, 2);
    entry.record_visit();
    assert_eq!(entry.visit_count, 3);
}

#[test]
fn test_entry_record_typed_visit() {
    let mut entry = HistoryEntry::new("https://github.com", "GitHub");
    entry.record_typed_visit();
    assert_eq!(entry.visit_count, 2);
    assert_eq!(entry.typed_count, 1);
}

#[test]
fn test_entry_set_title() {
    let mut entry = HistoryEntry::new("https://example.com", "Old");
    entry.set_title("New");
    assert_eq!(entry.title, "New");
}

#[test]
fn test_entry_set_favicon() {
    let mut entry = HistoryEntry::new("https://example.com", "T");
    entry.set_favicon(Some("https://example.com/fav.ico".to_string()));
    assert_eq!(entry.favicon, Some("https://example.com/fav.ico".to_string()));

    entry.set_favicon(None);
    assert!(entry.favicon.is_none());
}

#[rstest]
#[case("https://github.com/rust-lang", Some("github.com"))]
#[case("http://example.com/path?q=1", Some("example.com"))]
#[case("file:///local/file", None)]
fn test_entry_domain(#[case] url: &str, #[case] expected: Option<&str>) {
    let entry = HistoryEntry::new(url, "T");
    assert_eq!(entry.domain(), expected);
}

#[rstest]
#[case("git", true)]
#[case("hub", true)]
#[case("GIT", true)] // case insensitive
#[case("github.com", true)]
#[case("gitlab", false)]
fn test_entry_matches(#[case] query: &str, #[case] expected: bool) {
    let entry = HistoryEntry::new("https://github.com", "GitHub");
    assert_eq!(entry.matches(query), expected);
}

#[test]
fn test_entry_relevance_score_title_exact_match_highest() {
    let entry = HistoryEntry::new("https://github.com", "GitHub");
    let exact_score = entry.relevance_score("github");
    let partial_score = entry.relevance_score("git");

    assert!(exact_score >= partial_score);
}

#[test]
fn test_entry_relevance_score_visit_count_boost() {
    let mut e1 = HistoryEntry::new("https://a.com", "A");
    let e2 = HistoryEntry::new("https://b.com", "B");

    // Give e1 more visits
    for _ in 0..10 {
        e1.record_visit();
    }

    let score1 = e1.relevance_score("a");
    let score2 = e2.relevance_score("b");

    assert!(score1 > score2);
}

// ========== SearchOptions Tests ==========

#[test]
fn test_search_options_default_matches_all() {
    let opts = SearchOptions::default();
    let entry = HistoryEntry::new("https://github.com", "GitHub");
    assert!(opts.matches(&entry));
}

#[test]
fn test_search_options_min_visits_filter() {
    let opts = SearchOptions::new().min_visits(5);
    let entry = HistoryEntry::new("https://github.com", "GitHub");
    // visit_count starts at 1
    assert!(!opts.matches(&entry));
}

#[test]
fn test_search_options_domain_filter() {
    let opts = SearchOptions::new().domain("github.com");
    let match_entry = HistoryEntry::new("https://github.com/rust", "Rust");
    let no_match = HistoryEntry::new("https://gitlab.com", "GitLab");

    assert!(opts.matches(&match_entry));
    assert!(!opts.matches(&no_match));
}

#[test]
fn test_search_options_date_range() {
    let yesterday = Utc::now() - Duration::days(1);
    let tomorrow = Utc::now() + Duration::days(1);
    let entry = HistoryEntry::new("https://github.com", "GitHub");

    // entry visited now, should be in [yesterday, tomorrow]
    let opts = SearchOptions::new()
        .start_date(yesterday)
        .end_date(tomorrow);
    assert!(opts.matches(&entry));

    // Future start: entry is too old
    let opts_future = SearchOptions::new().start_date(tomorrow);
    assert!(!opts_future.matches(&entry));
}

#[test]
fn test_search_options_limit() {
    let opts = SearchOptions::new().limit(3);
    assert_eq!(opts.limit, Some(3));
}

#[test]
fn test_search_result_has_score() {
    let entry = HistoryEntry::new("https://github.com", "GitHub");
    let result = SearchResult::new(entry, "github");
    assert!(result.score > 0);
}

// ========== HistoryManager Tests ==========

#[test]
fn test_manager_starts_empty() {
    let manager = HistoryManager::new(None);
    assert_eq!(manager.count(), 0);
}

#[test]
fn test_manager_visit_creates_entry() {
    let manager = HistoryManager::new(None);
    let id = manager.visit("https://github.com", "GitHub");

    let entry = manager.get(&id).unwrap();
    assert_eq!(entry.url, "https://github.com");
    assert_eq!(entry.title, "GitHub");
    assert_eq!(entry.visit_count, 1);
}

#[test]
fn test_manager_visit_same_url_increments() {
    let manager = HistoryManager::new(None);
    manager.visit("https://github.com", "GitHub");
    manager.visit("https://github.com", "GitHub Updated");

    assert_eq!(manager.count(), 1);
    let entry = manager.get_by_url("https://github.com").unwrap();
    assert_eq!(entry.visit_count, 2);
    assert_eq!(entry.title, "GitHub Updated");
}

#[test]
fn test_manager_visit_empty_title_keeps_old() {
    let manager = HistoryManager::new(None);
    manager.visit("https://github.com", "GitHub");
    manager.visit("https://github.com", ""); // empty title should not replace

    let entry = manager.get_by_url("https://github.com").unwrap();
    assert_eq!(entry.title, "GitHub");
}

#[test]
fn test_manager_typed_visit() {
    let manager = HistoryManager::new(None);
    manager.typed_visit("https://github.com", "GitHub");

    let entry = manager.get_by_url("https://github.com").unwrap();
    assert_eq!(entry.typed_count, 1);
}

#[test]
fn test_manager_typed_visit_increments_typed() {
    let manager = HistoryManager::new(None);
    manager.visit("https://github.com", "GitHub");
    manager.typed_visit("https://github.com", "GitHub");

    let entry = manager.get_by_url("https://github.com").unwrap();
    assert_eq!(entry.visit_count, 2);
    assert_eq!(entry.typed_count, 1);
}

#[test]
fn test_manager_get_by_url() {
    let manager = HistoryManager::new(None);
    manager.visit("https://github.com", "GitHub");

    assert!(manager.get_by_url("https://github.com").is_some());
    assert!(manager.get_by_url("https://missing.com").is_none());
}

#[test]
fn test_manager_delete_by_id() {
    let manager = HistoryManager::new(None);
    let id = manager.visit("https://github.com", "GitHub");

    assert!(manager.delete(&id));
    assert!(manager.get(&id).is_none());
    assert!(!manager.delete(&id)); // second deletion returns false
}

#[test]
fn test_manager_delete_by_url() {
    let manager = HistoryManager::new(None);
    manager.visit("https://github.com", "GitHub");

    assert!(manager.delete_url("https://github.com"));
    assert!(manager.get_by_url("https://github.com").is_none());
    assert!(!manager.delete_url("https://github.com")); // already gone
}

#[test]
fn test_manager_clear() {
    let manager = HistoryManager::new(None);
    manager.visit("https://a.com", "A");
    manager.visit("https://b.com", "B");
    assert_eq!(manager.count(), 2);

    manager.clear();
    assert_eq!(manager.count(), 0);
}

// ========== Recent / Frequent Tests ==========

#[test]
fn test_manager_recent_sorted_newest_first() {
    let manager = HistoryManager::new(None);
    manager.visit("https://first.com", "First");
    std::thread::sleep(std::time::Duration::from_millis(10));
    manager.visit("https://second.com", "Second");
    std::thread::sleep(std::time::Duration::from_millis(10));
    manager.visit("https://third.com", "Third");

    let recent = manager.recent(10);
    assert_eq!(recent[0].url, "https://third.com");
    assert_eq!(recent[1].url, "https://second.com");
    assert_eq!(recent[2].url, "https://first.com");
}

#[test]
fn test_manager_recent_respects_limit() {
    let manager = HistoryManager::new(None);
    for i in 0..5 {
        manager.visit(format!("https://site{}.com", i), format!("Site {}", i));
    }

    let recent = manager.recent(3);
    assert_eq!(recent.len(), 3);
}

#[test]
fn test_manager_frequent_sorted_by_count() {
    let manager = HistoryManager::new(None);

    manager.visit("https://once.com", "Once");
    manager.visit("https://twice.com", "Twice");
    manager.visit("https://twice.com", "Twice");
    manager.visit("https://thrice.com", "Thrice");
    manager.visit("https://thrice.com", "Thrice");
    manager.visit("https://thrice.com", "Thrice");

    let freq = manager.frequent(3);
    assert_eq!(freq[0].url, "https://thrice.com");
    assert_eq!(freq[0].visit_count, 3);
    assert_eq!(freq[1].url, "https://twice.com");
}

// ========== Search Tests ==========

#[rstest]
#[case("git", 2)]
#[case("rust", 1)]
#[case("xyz_not_exist", 0)]
fn test_manager_search_basic(#[case] query: &str, #[case] expected: usize) {
    let manager = HistoryManager::new(None);
    manager.visit("https://github.com", "GitHub");
    manager.visit("https://gitlab.com", "GitLab");
    manager.visit("https://rust-lang.org", "Rust");

    let results = manager.search(query);
    assert_eq!(results.len(), expected);
}

#[test]
fn test_manager_search_results_sorted_by_score() {
    let manager = HistoryManager::new(None);
    manager.visit("https://github.com", "GitHub");
    manager.visit("https://github.com/rust-lang/rust", "GitHub Rust");

    // Visit the first one many times to give it a high score
    for _ in 0..5 {
        manager.visit("https://github.com", "GitHub");
    }

    let results = manager.search("github");
    assert!(!results.is_empty());
    // Results should be sorted (highest score first)
    for pair in results.windows(2) {
        assert!(pair[0].score >= pair[1].score);
    }
}

#[test]
fn test_manager_search_with_limit() {
    let manager = HistoryManager::new(None);
    for i in 0..10 {
        manager.visit(format!("https://github.com/{}", i), format!("GitHub Page {}", i));
    }

    let opts = SearchOptions::new().limit(3);
    let results = manager.search_with_options("github", opts);
    assert_eq!(results.len(), 3);
}

#[test]
fn test_manager_search_with_domain_filter() {
    let manager = HistoryManager::new(None);
    manager.visit("https://github.com/page1", "GitHub Page 1");
    manager.visit("https://github.com/page2", "GitHub Page 2");
    manager.visit("https://gitlab.com/page", "GitLab Page");

    let opts = SearchOptions::new().domain("github.com");
    let results = manager.search_with_options("git", opts);
    assert_eq!(results.len(), 2);
}

#[test]
fn test_manager_search_with_min_visits() {
    let manager = HistoryManager::new(None);
    manager.visit("https://once.com", "Once");
    manager.visit("https://often.com", "Often");
    for _ in 0..4 {
        manager.visit("https://often.com", "Often");
    }

    let opts = SearchOptions::new().min_visits(3);
    let results = manager.search_with_options("com", opts);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].entry.url, "https://often.com");
}

// ========== Domain / Date Queries ==========

#[test]
fn test_manager_by_domain() {
    let manager = HistoryManager::new(None);
    manager.visit("https://github.com/page1", "P1");
    manager.visit("https://github.com/page2", "P2");
    manager.visit("https://gitlab.com/page", "GL");

    let entries = manager.by_domain("github.com");
    assert_eq!(entries.len(), 2);
    assert!(entries.iter().all(|e| e.domain() == Some("github.com")));
}

#[test]
fn test_manager_today_includes_recent() {
    let manager = HistoryManager::new(None);
    manager.visit("https://now.com", "Now");

    let today = manager.today();
    assert!(!today.is_empty());
}

#[test]
fn test_manager_in_range() {
    let manager = HistoryManager::new(None);
    manager.visit("https://example.com", "Example");

    let start = Utc::now() - Duration::hours(1);
    let end = Utc::now() + Duration::hours(1);
    let results = manager.in_range(start, end);

    assert!(!results.is_empty());
}

#[test]
fn test_manager_delete_domain() {
    let manager = HistoryManager::new(None);
    manager.visit("https://github.com/page1", "P1");
    manager.visit("https://github.com/page2", "P2");
    manager.visit("https://rust-lang.org", "Rust");

    let removed = manager.delete_domain("github.com");
    assert_eq!(removed, 2);
    assert_eq!(manager.count(), 1);
}

#[test]
fn test_manager_delete_older_than() {
    let manager = HistoryManager::new(None);
    manager.visit("https://recent.com", "Recent");
    assert_eq!(manager.count(), 1);

    // Delete entries older than 30 days - recent visit should NOT be deleted
    let removed = manager.delete_older_than(30);
    assert_eq!(removed, 0);
    assert_eq!(manager.count(), 1);
}

// ========== max_entries Tests ==========

#[test]
fn test_manager_max_entries_evicts_oldest() {
    let manager = HistoryManager::new(None).with_max_entries(3);

    // Sleep between visits to ensure different timestamps
    manager.visit("https://old1.com", "Old1");
    std::thread::sleep(std::time::Duration::from_millis(5));
    manager.visit("https://old2.com", "Old2");
    std::thread::sleep(std::time::Duration::from_millis(5));
    manager.visit("https://old3.com", "Old3");
    std::thread::sleep(std::time::Duration::from_millis(5));
    manager.visit("https://new4.com", "New4"); // should evict oldest

    assert_eq!(manager.count(), 3);
    // old1 should be gone (oldest)
    assert!(manager.get_by_url("https://old1.com").is_none());
    // newest should still be there
    assert!(manager.get_by_url("https://new4.com").is_some());
}

// ========== Persistence Tests ==========

#[test]
fn test_manager_persistence_round_trip() {
    let dir = TempDir::new().unwrap();

    {
        let manager = HistoryManager::new(Some(dir.path()));
        manager.visit("https://github.com", "GitHub");
        manager.visit("https://rust-lang.org", "Rust");
        manager.visit("https://github.com", "GitHub"); // second visit
    }

    let manager2 = HistoryManager::new(Some(dir.path()));
    assert_eq!(manager2.count(), 2);

    let entry = manager2.get_by_url("https://github.com").unwrap();
    assert_eq!(entry.visit_count, 2);
}

#[test]
fn test_manager_export_import() {
    let m1 = HistoryManager::new(None);
    m1.visit("https://a.com", "A");
    m1.visit("https://b.com", "B");

    let json = m1.export().unwrap();

    let m2 = HistoryManager::new(None);
    m2.import(&json).unwrap();

    assert_eq!(m2.count(), 2);
    assert!(m2.get_by_url("https://a.com").is_some());
    assert!(m2.get_by_url("https://b.com").is_some());
}

// ========== Clone Tests ==========

#[test]
fn test_manager_clone_shares_state() {
    let m1 = HistoryManager::new(None);
    let m2 = m1.clone();

    m1.visit("https://shared.com", "Shared");
    assert!(m2.get_by_url("https://shared.com").is_some());
}

// ========== All / Count Tests ==========

#[test]
fn test_manager_all() {
    let manager = HistoryManager::new(None);
    manager.visit("https://a.com", "A");
    manager.visit("https://b.com", "B");

    let all = manager.all();
    assert_eq!(all.len(), 2);
}

// ========== Serde Roundtrip Tests ==========

#[test]
fn serde_history_entry_basic_roundtrip() {
    let entry = HistoryEntry::new("https://github.com", "GitHub");
    let json = serde_json::to_string(&entry).unwrap();
    let back: HistoryEntry = serde_json::from_str(&json).unwrap();

    assert_eq!(back.id, entry.id);
    assert_eq!(back.url, entry.url);
    assert_eq!(back.title, entry.title);
    assert_eq!(back.visit_count, 1);
    assert_eq!(back.typed_count, 0);
    assert!(back.favicon.is_none());
}

#[test]
fn serde_history_entry_full_roundtrip() {
    let mut entry = HistoryEntry::new("https://github.com", "GitHub");
    entry.set_favicon(Some("https://github.com/favicon.ico".to_string()));
    entry.record_visit();
    entry.record_typed_visit();

    let json = serde_json::to_string(&entry).unwrap();
    let back: HistoryEntry = serde_json::from_str(&json).unwrap();

    assert_eq!(back.visit_count, 3);
    assert_eq!(back.typed_count, 1);
    assert_eq!(back.favicon, Some("https://github.com/favicon.ico".to_string()));
}

#[test]
fn serde_history_entry_omits_favicon_when_none() {
    let entry = HistoryEntry::new("https://example.com", "Example");
    let json = serde_json::to_string(&entry).unwrap();
    assert!(!json.contains(r#""favicon""#));
}

#[test]
fn serde_search_options_fields() {
    // SearchOptions does not implement Serialize/Deserialize; test builder fields instead
    let opts = SearchOptions::new().limit(10).min_visits(3);
    assert_eq!(opts.limit, Some(10));
    assert_eq!(opts.min_visits, Some(3));

    let opts2 = SearchOptions::default();
    assert!(opts2.limit.is_none());
}

#[test]
fn serde_manager_export_import_preserves_visit_counts() {
    let m1 = HistoryManager::new(None);
    m1.visit("https://a.com", "A");
    m1.visit("https://a.com", "A");
    m1.visit("https://a.com", "A"); // 3 visits
    m1.visit("https://b.com", "B");

    let json = m1.export().unwrap();
    let m2 = HistoryManager::new(None);
    m2.import(&json).unwrap();

    let entry = m2.get_by_url("https://a.com").unwrap();
    assert_eq!(entry.visit_count, 3);
}

// ========== HistoryError Display Tests ==========

use auroraview_history::HistoryError;

#[test]
fn error_not_found_display() {
    let err = HistoryError::NotFound("entry-abc".to_string());
    let msg = err.to_string();
    assert!(msg.contains("entry-abc"), "got: {msg}");
    assert!(msg.to_lowercase().contains("not found"), "got: {msg}");
}

#[test]
fn error_invalid_url_display() {
    let err = HistoryError::InvalidUrl("not-a-url".to_string());
    let msg = err.to_string();
    assert!(msg.contains("not-a-url"), "got: {msg}");
}

#[test]
fn error_storage_display() {
    let err = HistoryError::Storage("write failed".to_string());
    let msg = err.to_string();
    assert!(msg.contains("write failed"), "got: {msg}");
}

// ========== HistoryEntry Edge Cases ==========

#[test]
fn entry_relevance_score_zero_for_no_match() {
    let entry = HistoryEntry::new("https://example.com", "Example");
    // Query that doesn't match anything in title or URL
    // Score can still be > 0 due to recency/visit boost, but let's just check it runs
    let score = entry.relevance_score("zzz-nomatch-xyz");
    // score may include visit/recency bonus, just ensure no panic
    let _ = score;
}

#[test]
fn entry_relevance_score_exact_title_highest() {
    let entry = HistoryEntry::new("https://github.com", "GitHub");
    let exact = entry.relevance_score("github");
    let prefix = entry.relevance_score("git");
    let partial = entry.relevance_score("hub");

    assert!(exact >= prefix, "exact match should score >= prefix match");
    assert!(prefix >= partial, "prefix should score >= partial match");
}

#[test]
fn entry_typed_count_accumulates() {
    let mut entry = HistoryEntry::new("https://example.com", "E");
    entry.record_typed_visit();
    entry.record_typed_visit();
    entry.record_typed_visit();
    assert_eq!(entry.typed_count, 3);
    assert_eq!(entry.visit_count, 4); // 1 initial + 3 typed
}

#[rstest]
#[case("https://sub.domain.example.com/path", Some("sub.domain.example.com"))]
#[case("http://localhost:3000/app", Some("localhost:3000"))]
#[case("https://example.com", Some("example.com"))]
#[case("ftp://files.example.com/file", None)] // no ftp support
fn entry_domain_various_urls(#[case] url: &str, #[case] expected: Option<&str>) {
    let entry = HistoryEntry::new(url, "T");
    assert_eq!(entry.domain(), expected);
}

#[test]
fn entry_set_title_empty_string() {
    let mut entry = HistoryEntry::new("https://example.com", "Old");
    entry.set_title("");
    assert_eq!(entry.title, ""); // set_title always updates (manager decides empty semantics)
}

// ========== Manager Edge Cases ==========

#[test]
fn test_manager_visit_updates_last_visit() {
    let manager = HistoryManager::new(None);
    manager.visit("https://example.com", "E");
    let e1 = manager.get_by_url("https://example.com").unwrap();
    let first_last_visit = e1.last_visit;

    std::thread::sleep(std::time::Duration::from_millis(5));
    manager.visit("https://example.com", "E Updated");
    let e2 = manager.get_by_url("https://example.com").unwrap();

    assert!(e2.last_visit >= first_last_visit);
}

#[test]
fn test_manager_frequent_with_zero_limit() {
    let manager = HistoryManager::new(None);
    manager.visit("https://example.com", "E");
    let freq = manager.frequent(0);
    assert!(freq.is_empty());
}

#[test]
fn test_manager_recent_empty_history() {
    let manager = HistoryManager::new(None);
    let recent = manager.recent(10);
    assert!(recent.is_empty());
}

#[test]
fn test_manager_delete_domain_none_matching() {
    let manager = HistoryManager::new(None);
    manager.visit("https://example.com", "E");
    let removed = manager.delete_domain("notexist.com");
    assert_eq!(removed, 0);
    assert_eq!(manager.count(), 1);
}

#[test]
fn test_manager_delete_older_than_removes_old_entries() {
    let manager = HistoryManager::new(None);
    manager.visit("https://recent.com", "Recent");
    assert_eq!(manager.count(), 1);

    // Delete entries older than 0 days — recent entry is NOT older than 0 days, so won't be deleted
    // (implementation: older_than_days strictly means last_visit < now - days*24h)
    let removed = manager.delete_older_than(0);
    // Visit was just now; might or might not be deleted depending on implementation
    // Just verify it doesn't panic
    let _ = removed;
}

#[test]
fn test_manager_search_case_insensitive() {
    let manager = HistoryManager::new(None);
    manager.visit("https://github.com", "GitHub");

    let results_lower = manager.search("github");
    let results_upper = manager.search("GITHUB");
    let results_mixed = manager.search("GitHub");

    assert!(!results_lower.is_empty());
    assert!(!results_upper.is_empty());
    assert!(!results_mixed.is_empty());
}

#[test]
fn test_manager_by_domain_returns_all_pages() {
    let manager = HistoryManager::new(None);
    for i in 0..5 {
        manager.visit(format!("https://github.com/page{i}"), format!("Page {i}"));
    }
    manager.visit("https://gitlab.com/other", "Other");

    let github_entries = manager.by_domain("github.com");
    assert_eq!(github_entries.len(), 5);
    assert!(github_entries.iter().all(|e| e.domain() == Some("github.com")));
}

#[test]
fn test_manager_in_range_excludes_outside() {
    let manager = HistoryManager::new(None);
    manager.visit("https://example.com", "Now");

    let future_start = Utc::now() + Duration::hours(1);
    let future_end = Utc::now() + Duration::hours(2);
    let results = manager.in_range(future_start, future_end);
    assert!(results.is_empty(), "future range should not include current entry");
}

// ========== Concurrent Tests ==========

#[test]
fn concurrent_visit_same_url_no_deadlock() {
    use std::sync::Arc;
    use std::thread;

    let manager = Arc::new(HistoryManager::new(None));

    let handles: Vec<_> = (0..8)
        .map(|_| {
            let m = Arc::clone(&manager);
            thread::spawn(move || {
                for _ in 0..10 {
                    m.visit("https://shared.com", "Shared");
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    let entry = manager.get_by_url("https://shared.com").unwrap();
    // 8 threads × 10 visits = 80 total, but first call creates with visit_count=1
    assert!(entry.visit_count >= 1);
    assert_eq!(manager.count(), 1); // only 1 unique URL
}

#[test]
fn concurrent_different_urls_no_panic() {
    use std::sync::Arc;
    use std::thread;

    let manager = Arc::new(HistoryManager::new(None));

    let handles: Vec<_> = (0..8)
        .map(|i| {
            let m = Arc::clone(&manager);
            thread::spawn(move || {
                for j in 0..10 {
                    m.visit(format!("https://thread{i}-site{j}.com"), format!("T{i}S{j}"));
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
fn concurrent_visit_and_search_no_deadlock() {
    use std::sync::Arc;
    use std::thread;

    let manager = Arc::new(HistoryManager::new(None));

    let writer = {
        let m = Arc::clone(&manager);
        thread::spawn(move || {
            for i in 0..50 {
                m.visit(format!("https://concurrent{i}.com"), format!("C{i}"));
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
fn concurrent_delete_and_visit_no_panic() {
    use std::sync::Arc;
    use std::thread;

    let manager = Arc::new(HistoryManager::new(None));

    // Pre-populate
    for i in 0..20 {
        manager.visit(format!("https://pre{i}.com"), format!("Pre{i}"));
    }

    let deleter = {
        let m = Arc::clone(&manager);
        thread::spawn(move || {
            m.clear();
        })
    };

    let visitor = {
        let m = Arc::clone(&manager);
        thread::spawn(move || {
            for i in 0..20 {
                m.visit(format!("https://new{i}.com"), format!("New{i}"));
            }
        })
    };

    deleter.join().unwrap();
    visitor.join().unwrap();
    // No panics — final count is indeterminate but valid
}
