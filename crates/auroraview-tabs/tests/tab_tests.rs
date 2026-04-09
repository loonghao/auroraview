use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use auroraview_tabs::{Result, Session, SessionManager, TabError, TabEvent, TabManager, TabState};
use rstest::*;
use tempfile::TempDir;

// ========== TabState Tests ==========

#[rstest]
#[case("https://example.com", true)]
#[case("http://example.com", false)]
#[case("file:///path/to/file", false)]
fn test_tab_is_secure(#[case] url: &str, #[case] expected: bool) {
    let state = TabState::new("t1".to_string(), url);
    assert_eq!(state.is_secure(), expected);
}

#[rstest]
#[case("https://github.com/rust-lang/rust", Some("github.com"))]
#[case("http://example.com/path?q=1", Some("example.com"))]
#[case("file:///local/file.html", None)]
fn test_tab_domain(#[case] url: &str, #[case] expected: Option<&str>) {
    let state = TabState::new("t1".to_string(), url);
    assert_eq!(state.domain(), expected);
}

#[test]
fn test_tab_state_with_title() {
    let state = TabState::with_title("t1".to_string(), "https://rust-lang.org", "Rust");
    assert_eq!(state.title, "Rust");
    assert_eq!(state.url, "https://rust-lang.org");
}

#[test]
fn test_tab_state_update_url_changes_security() {
    let mut state = TabState::new("t1".to_string(), "https://secure.com");
    assert!(state.is_secure());

    state.set_url("http://insecure.com");
    assert!(!state.is_secure());
}

#[test]
fn test_tab_state_set_title_ignores_empty() {
    let mut state = TabState::new("t1".to_string(), "https://example.com");
    state.set_title("Original");
    state.set_title(""); // should be ignored
    assert_eq!(state.title, "Original");
}

#[test]
fn test_tab_state_set_title_updates_nonempty() {
    let mut state = TabState::new("t1".to_string(), "https://example.com");
    state.set_title("New Title");
    assert_eq!(state.title, "New Title");
}

#[test]
fn test_tab_state_pinned_muted_audible() {
    let mut state = TabState::new("t1".to_string(), "https://example.com");
    assert!(!state.pinned);
    assert!(!state.muted);
    assert!(!state.audible);

    state.set_pinned(true);
    state.set_muted(true);
    state.set_audible(true);

    assert!(state.pinned);
    assert!(state.muted);
    assert!(state.audible);
}

#[test]
fn test_tab_state_history() {
    let mut state = TabState::new("t1".to_string(), "https://example.com");
    assert!(!state.can_go_back);
    assert!(!state.can_go_forward);

    state.set_history_state(true, true);
    assert!(state.can_go_back);
    assert!(state.can_go_forward);
}

// ========== TabManager Creation Tests ==========

#[test]
fn test_manager_starts_empty() {
    let manager = TabManager::new();
    assert!(manager.is_empty());
    assert_eq!(manager.count(), 0);
    assert!(manager.active_id().is_none());
}

#[test]
fn test_manager_default_equals_new() {
    let m = TabManager::default();
    assert!(m.is_empty());
}

// ========== TabManager CRUD Tests ==========

#[test]
fn test_create_first_tab_becomes_active() {
    let manager = TabManager::new();
    let id = manager.create("https://github.com");

    assert_eq!(manager.count(), 1);
    assert_eq!(manager.active_id(), Some(id.clone()));

    let tab = manager.get(&id).unwrap();
    assert_eq!(tab.url, "https://github.com");
    assert!(tab.is_loading);
}

#[test]
fn test_create_second_tab_does_not_change_active() {
    let manager = TabManager::new();
    let id1 = manager.create("https://a.com");
    let _id2 = manager.create("https://b.com");

    assert_eq!(manager.active_id(), Some(id1));
}

#[test]
fn test_create_with_state() {
    let manager = TabManager::new();
    let state = TabState::with_title("custom-id".to_string(), "https://custom.com", "Custom");
    let id = manager.create_with_state(state);

    assert_eq!(id, "custom-id");
    let tab = manager.get(&id).unwrap();
    assert_eq!(tab.title, "Custom");
}

#[test]
fn test_close_existing_tab() {
    let manager = TabManager::new();
    let id = manager.create("https://example.com");

    assert!(manager.close(&id).is_ok());
    assert_eq!(manager.count(), 0);
    assert!(manager.get(&id).is_none());
}

#[test]
fn test_close_nonexistent_returns_error() {
    let manager = TabManager::new();
    let result = manager.close(&"nonexistent".to_string());
    assert!(matches!(result, Err(TabError::NotFound(_))));
}

#[test]
fn test_close_active_tab_advances_active() {
    let manager = TabManager::new();
    let id1 = manager.create("https://a.com");
    let id2 = manager.create("https://b.com");

    // id1 is active
    manager.close(&id1).unwrap();

    // id2 becomes active
    assert_eq!(manager.active_id(), Some(id2));
}

#[test]
fn test_close_last_tab_clears_active() {
    let manager = TabManager::new();
    let id = manager.create("https://solo.com");
    manager.close(&id).unwrap();
    assert!(manager.active_id().is_none());
}

// ========== Activate Tests ==========

#[test]
fn test_activate_tab() {
    let manager = TabManager::new();
    let _id1 = manager.create("https://a.com");
    let id2 = manager.create("https://b.com");

    manager.activate(&id2).unwrap();
    assert_eq!(manager.active_id(), Some(id2.clone()));
}

#[test]
fn test_activate_nonexistent_returns_error() {
    let manager = TabManager::new();
    let result = manager.activate(&"bad-id".to_string());
    assert!(matches!(result, Err(TabError::NotFound(_))));
}

// ========== Ordering Tests ==========

#[test]
fn test_tab_order_matches_creation_order() {
    let manager = TabManager::new();
    let id1 = manager.create("https://a.com");
    let id2 = manager.create("https://b.com");
    let id3 = manager.create("https://c.com");

    assert_eq!(manager.order(), vec![id1, id2, id3]);
}

#[test]
fn test_reorder_move_to_front() {
    let manager = TabManager::new();
    let id1 = manager.create("https://a.com");
    let id2 = manager.create("https://b.com");
    let id3 = manager.create("https://c.com");

    manager.reorder(&id3, 0);
    assert_eq!(manager.order(), vec![id3.clone(), id1, id2]);
}

#[test]
fn test_reorder_move_to_middle() {
    let manager = TabManager::new();
    let id1 = manager.create("https://a.com");
    let id2 = manager.create("https://b.com");
    let id3 = manager.create("https://c.com");

    manager.reorder(&id1, 1);
    assert_eq!(manager.order(), vec![id2.clone(), id1.clone(), id3.clone()]);
}

#[test]
fn test_all_returns_in_order() {
    let manager = TabManager::new();
    let id1 = manager.create("https://a.com");
    let id2 = manager.create("https://b.com");

    let tabs = manager.all();
    assert_eq!(tabs.len(), 2);
    assert_eq!(tabs[0].id, id1);
    assert_eq!(tabs[1].id, id2);
}

// ========== State Update Tests ==========

#[test]
fn test_update_title() {
    let manager = TabManager::new();
    let id = manager.create("https://example.com");

    manager.update_title(&id, "Example");
    let tab = manager.get(&id).unwrap();
    assert_eq!(tab.title, "Example");
}

#[test]
fn test_update_url_changes_security() {
    let manager = TabManager::new();
    let id = manager.create("https://secure.com");

    manager.update_url(&id, "http://insecure.com");
    let tab = manager.get(&id).unwrap();
    assert_eq!(tab.url, "http://insecure.com");
    assert!(!tab.is_secure());
}

#[test]
fn test_update_loading() {
    let manager = TabManager::new();
    let id = manager.create("https://example.com");
    assert!(manager.get(&id).unwrap().is_loading);

    manager.update_loading(&id, false);
    assert!(!manager.get(&id).unwrap().is_loading);
}

#[test]
fn test_update_history_state() {
    let manager = TabManager::new();
    let id = manager.create("https://example.com");

    manager.update_history(&id, true, true);
    let tab = manager.get(&id).unwrap();
    assert!(tab.can_go_back);
    assert!(tab.can_go_forward);
}

#[test]
fn test_update_favicon() {
    let manager = TabManager::new();
    let id = manager.create("https://example.com");

    manager.update_favicon(&id, "https://example.com/favicon.ico");
    let tab = manager.get(&id).unwrap();
    assert_eq!(
        tab.favicon,
        Some("https://example.com/favicon.ico".to_string())
    );
}

#[test]
fn test_set_pinned_muted() {
    let manager = TabManager::new();
    let id = manager.create("https://example.com");

    manager.set_pinned(&id, true);
    manager.set_muted(&id, true);

    let tab = manager.get(&id).unwrap();
    assert!(tab.pinned);
    assert!(tab.muted);
}

#[test]
fn test_update_closure() {
    let manager = TabManager::new();
    let id = manager.create("https://example.com");

    let old_title = manager.update(&id, |tab| {
        let old = tab.title.clone();
        tab.set_title("Updated");
        old
    });
    assert_eq!(old_title, Some("New Tab".to_string()));
    assert_eq!(manager.get(&id).unwrap().title, "Updated");
}

// ========== Duplicate Tests ==========

#[test]
fn test_duplicate_tab() {
    let manager = TabManager::new();
    let id = manager.create("https://example.com");
    manager.update_title(&id, "Original");

    let new_id = manager.duplicate(&id).unwrap();
    assert_ne!(new_id, id);
    assert_eq!(manager.count(), 2);

    let new_tab = manager.get(&new_id).unwrap();
    assert_eq!(new_tab.url, "https://example.com");
}

#[test]
fn test_duplicate_nonexistent_returns_error() {
    let manager = TabManager::new();
    let result = manager.duplicate(&"bad-id".to_string());
    assert!(matches!(result, Err(TabError::NotFound(_))));
}

// ========== Tab Group Tests ==========

#[test]
fn test_create_group_with_tabs() {
    let manager = TabManager::new();
    let tab1 = manager.create("https://github.com");
    let tab2 = manager.create("https://gitlab.com");

    let group_id = manager.create_group_with_tabs("Dev", vec![tab1.clone(), tab2.clone()]);

    let group = manager.get_group(&group_id).unwrap();
    assert_eq!(group.len(), 2);
    assert!(group.contains(&tab1));
    assert!(group.contains(&tab2));
}

#[test]
fn test_add_and_remove_from_group() {
    let manager = TabManager::new();
    let tab = manager.create("https://example.com");
    let group_id = manager.create_group("Test");

    manager.add_to_group(&tab, &group_id).unwrap();

    let state = manager.get(&tab).unwrap();
    assert_eq!(state.group_id, Some(group_id.clone()));

    manager.remove_from_group(&tab).unwrap();
    let state = manager.get(&tab).unwrap();
    assert!(state.group_id.is_none());
}

#[test]
fn test_add_to_nonexistent_group_returns_error() {
    let manager = TabManager::new();
    let tab = manager.create("https://example.com");
    let result = manager.add_to_group(&tab, &"bad-group".to_string());
    assert!(matches!(result, Err(TabError::GroupNotFound(_))));
}

#[test]
fn test_delete_group_ungroups_tabs() {
    let manager = TabManager::new();
    let tab = manager.create("https://example.com");
    let group_id = manager.create_group_with_tabs("G1", vec![tab.clone()]);

    manager.delete_group(&group_id).unwrap();

    let state = manager.get(&tab).unwrap();
    assert!(state.group_id.is_none());
    assert!(manager.get_group(&group_id).is_none());
}

#[test]
fn test_group_collapse_expand() {
    let manager = TabManager::new();
    let group_id = manager.create_group("Collapsible");

    manager.set_group_collapsed(&group_id, true).unwrap();
    assert!(manager.get_group(&group_id).unwrap().collapsed);

    manager.set_group_collapsed(&group_id, false).unwrap();
    assert!(!manager.get_group(&group_id).unwrap().collapsed);
}

#[test]
fn test_close_tab_removes_from_group() {
    let manager = TabManager::new();
    let tab = manager.create("https://example.com");
    let group_id = manager.create_group_with_tabs("G1", vec![tab.clone()]);

    manager.close(&tab).unwrap();

    let group = manager.get_group(&group_id).unwrap();
    assert!(!group.contains(&tab));
}

// ========== Event Tests ==========

#[test]
fn test_event_created_fires_on_create() {
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = Arc::clone(&counter);

    let manager = TabManager::new();
    manager.on_event(move |event| {
        if matches!(event, TabEvent::Created { .. }) {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }
    });

    manager.create("https://example.com");
    manager.create("https://other.com");

    assert_eq!(counter.load(Ordering::SeqCst), 2);
}

#[test]
fn test_event_closed_fires_on_close() {
    let received = Arc::new(AtomicUsize::new(0));
    let received_clone = Arc::clone(&received);

    let manager = TabManager::new();
    manager.on_event(move |event| {
        if matches!(event, TabEvent::Closed { .. }) {
            received_clone.fetch_add(1, Ordering::SeqCst);
        }
    });

    let id = manager.create("https://example.com");
    manager.close(&id).unwrap();

    assert_eq!(received.load(Ordering::SeqCst), 1);
}

#[test]
fn test_event_activated_fires_on_activate() {
    let received = Arc::new(AtomicUsize::new(0));
    let received_clone = Arc::clone(&received);

    let manager = TabManager::new();
    manager.on_event(move |event| {
        if matches!(event, TabEvent::Activated { .. }) {
            received_clone.fetch_add(1, Ordering::SeqCst);
        }
    });

    let _id1 = manager.create("https://a.com");
    let id2 = manager.create("https://b.com");
    manager.activate(&id2).unwrap();

    assert_eq!(received.load(Ordering::SeqCst), 1);
}

#[test]
fn test_event_title_changed() {
    let received = Arc::new(AtomicUsize::new(0));
    let received_clone = Arc::clone(&received);

    let manager = TabManager::new();
    manager.on_event(move |event| {
        if matches!(event, TabEvent::TitleChanged { .. }) {
            received_clone.fetch_add(1, Ordering::SeqCst);
        }
    });

    let id = manager.create("https://example.com");
    manager.update_title(&id, "New Title");

    assert_eq!(received.load(Ordering::SeqCst), 1);
}

// ========== Session Tests ==========

#[test]
fn test_session_empty_by_default() {
    let session = Session::new();
    assert!(session.is_empty());
    assert_eq!(session.tab_count(), 0);
    assert!(session.active_tab_id.is_none());
}

#[test]
fn test_session_from_state() {
    let tabs = vec![
        TabState::new("t1".to_string(), "https://a.com"),
        TabState::new("t2".to_string(), "https://b.com"),
    ];
    let session = Session::from_state(tabs, Some("t1".to_string()), vec![]);

    assert_eq!(session.tab_count(), 2);
    assert_eq!(session.active_tab_id, Some("t1".to_string()));
    assert!(!session.is_empty());
}

#[test]
fn test_session_manager_round_trip() {
    let dir = TempDir::new().unwrap();
    let manager = SessionManager::new(dir.path());

    let tabs = vec![TabState::new("t1".to_string(), "https://github.com")];
    let original = Session::from_state(tabs, Some("t1".to_string()), vec![]);

    manager.save(&original).unwrap();
    assert!(manager.exists());

    let loaded = manager.load().unwrap();
    assert_eq!(loaded.tab_count(), 1);
    assert_eq!(loaded.tabs[0].url, "https://github.com");
    assert_eq!(loaded.active_tab_id, Some("t1".to_string()));
}

#[test]
fn test_session_manager_load_missing_returns_empty() {
    let dir = TempDir::new().unwrap();
    let manager = SessionManager::new(dir.path());

    let session = manager.load().unwrap();
    assert!(session.is_empty());
}

#[test]
fn test_session_manager_backup_restore() {
    let dir = TempDir::new().unwrap();
    let manager = SessionManager::new(dir.path());

    let tabs = vec![TabState::new("t1".to_string(), "https://example.com")];
    let session = Session::from_state(tabs, None, vec![]);

    manager.save(&session).unwrap();
    let backup_path = manager.backup().unwrap();
    assert!(backup_path.exists());

    let restored = manager.restore_backup().unwrap();
    assert_eq!(restored.tab_count(), 1);
}

#[test]
fn test_session_manager_delete() {
    let dir = TempDir::new().unwrap();
    let manager = SessionManager::new(dir.path());

    let session = Session::new();
    manager.save(&session).unwrap();
    assert!(manager.exists());

    manager.delete().unwrap();
    assert!(!manager.exists());
}

#[test]
fn test_session_manager_auto_save_flag() {
    let dir = TempDir::new().unwrap();
    let mut manager = SessionManager::new(dir.path());

    assert!(manager.auto_save());
    manager.set_auto_save(false);
    assert!(!manager.auto_save());
}

// ========== Concurrency Tests ==========

#[test]
fn test_concurrent_tab_creation() {
    use std::sync::Arc;
    use std::thread;

    let manager = Arc::new(TabManager::new());
    let mut handles = vec![];

    for i in 0..10 {
        let m = Arc::clone(&manager);
        handles.push(thread::spawn(move || {
            m.create(format!("https://site-{}.com", i))
        }));
    }

    let ids: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(manager.count(), 10);

    // All IDs should be unique
    let mut seen = std::collections::HashSet::new();
    for id in &ids {
        assert!(seen.insert(id.clone()), "Duplicate ID: {}", id);
    }
}

// ========== Result type Test ==========

#[test]
fn test_result_type_alias() {
    let ok: Result<i32> = Ok(42);
    assert!(ok.is_ok());
}

// ========== TabEvent Factory Methods ==========

#[test]
fn test_event_new_tab_with_url() {
    let ev = TabEvent::new_tab(Some("https://example.com".to_string()));
    assert!(matches!(ev, TabEvent::NewTab { url: Some(_) }));
}

#[test]
fn test_event_new_tab_no_url() {
    let ev = TabEvent::new_tab(None);
    assert!(matches!(ev, TabEvent::NewTab { url: None }));
}

#[test]
fn test_event_close_tab() {
    let ev = TabEvent::close_tab("tab_1".to_string());
    assert!(matches!(ev, TabEvent::CloseTab { .. }));
}

#[test]
fn test_event_activate_tab() {
    let ev = TabEvent::activate_tab("tab_2".to_string());
    assert!(matches!(ev, TabEvent::ActivateTab { .. }));
}

#[test]
fn test_event_navigate() {
    let ev = TabEvent::navigate("https://github.com");
    assert!(matches!(ev, TabEvent::Navigate { url } if url == "https://github.com"));
}

#[test]
fn test_event_title_changed_factory_method() {
    let ev = TabEvent::title_changed("t1".to_string(), "My Title");
    assert!(matches!(ev, TabEvent::TitleChanged { title, .. } if title == "My Title"));
}

#[test]
fn test_event_url_changed() {
    let ev = TabEvent::url_changed("t1".to_string(), "https://new.com");
    assert!(matches!(ev, TabEvent::UrlChanged { url, .. } if url == "https://new.com"));
}

#[test]
fn test_event_loading_changed_true() {
    let ev = TabEvent::loading_changed("t1".to_string(), true);
    assert!(matches!(ev, TabEvent::LoadingChanged { is_loading: true, .. }));
}

#[test]
fn test_event_loading_changed_false() {
    let ev = TabEvent::loading_changed("t1".to_string(), false);
    assert!(matches!(ev, TabEvent::LoadingChanged { is_loading: false, .. }));
}

#[rstest]
#[case(TabEvent::Created { tab_id: "1".to_string() }, true)]
#[case(TabEvent::Closed { tab_id: "1".to_string() }, true)]
#[case(TabEvent::Activated { tab_id: "1".to_string() }, true)]
#[case(TabEvent::Deactivated { tab_id: "1".to_string() }, true)]
#[case(TabEvent::Navigate { url: "https://x.com".to_string() }, false)]
fn test_event_is_lifecycle(#[case] ev: TabEvent, #[case] expected: bool) {
    assert_eq!(ev.is_lifecycle_event(), expected);
}

#[rstest]
#[case(TabEvent::TitleChanged { tab_id: "t".to_string(), title: "T".to_string() }, true)]
#[case(TabEvent::UrlChanged { tab_id: "t".to_string(), url: "u".to_string() }, true)]
#[case(TabEvent::LoadingChanged { tab_id: "t".to_string(), is_loading: false }, true)]
#[case(TabEvent::HistoryChanged { tab_id: "t".to_string(), can_go_back: false, can_go_forward: false }, true)]
#[case(TabEvent::FaviconChanged { tab_id: "t".to_string(), favicon_url: "f".to_string() }, true)]
#[case(TabEvent::Created { tab_id: "t".to_string() }, false)]
fn test_event_is_state_update(#[case] ev: TabEvent, #[case] expected: bool) {
    assert_eq!(ev.is_state_update(), expected);
}

#[rstest]
#[case(TabEvent::AddedToGroup { tab_id: "t".to_string(), group_id: "g".to_string() }, true)]
#[case(TabEvent::RemovedFromGroup { tab_id: "t".to_string(), group_id: "g".to_string() }, true)]
#[case(TabEvent::GroupCreated { group_id: "g".to_string(), name: "n".to_string() }, true)]
#[case(TabEvent::GroupDeleted { group_id: "g".to_string() }, true)]
#[case(TabEvent::GroupCollapsed { group_id: "g".to_string(), collapsed: true }, true)]
#[case(TabEvent::Navigate { url: "u".to_string() }, false)]
fn test_event_is_group_event(#[case] ev: TabEvent, #[case] expected: bool) {
    assert_eq!(ev.is_group_event(), expected);
}

#[test]
fn test_event_serde_roundtrip() {
    let ev = TabEvent::TitleChanged {
        tab_id: "tab_1".to_string(),
        title: "Hello".to_string(),
    };
    let json = serde_json::to_string(&ev).unwrap();
    let back: TabEvent = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, TabEvent::TitleChanged { title, .. } if title == "Hello"));
}

// ========== TabGroup Extended Tests ==========

#[test]
fn test_group_with_id() {
    let group = auroraview_tabs::TabGroup::with_id("group-1", "Work");
    assert_eq!(group.id, "group-1");
    assert_eq!(group.name, "Work");
    assert!(group.color.is_none());
}

#[test]
fn test_group_with_color() {
    let group = auroraview_tabs::TabGroup::new("Work").with_color("#ff0000");
    assert_eq!(group.color, Some("#ff0000".to_string()));
}

#[test]
fn test_group_with_tabs() {
    let group = auroraview_tabs::TabGroup::new("Work")
        .with_tabs(vec!["t1".to_string(), "t2".to_string()]);
    assert_eq!(group.len(), 2);
    assert!(group.contains(&"t1".to_string()));
}

#[test]
fn test_group_add_tab_at_position() {
    let mut group = auroraview_tabs::TabGroup::new("Work")
        .with_tabs(vec!["t1".to_string(), "t2".to_string()]);
    group.add_tab_at("t0".to_string(), 0);
    assert_eq!(group.tab_ids[0], "t0");
}

#[test]
fn test_group_add_tab_no_duplicate() {
    let mut group = auroraview_tabs::TabGroup::new("Work");
    group.add_tab("t1".to_string());
    group.add_tab("t1".to_string()); // should not duplicate
    assert_eq!(group.len(), 1);
}

#[test]
fn test_group_remove_nonexistent_tab() {
    let mut group = auroraview_tabs::TabGroup::new("Work");
    group.add_tab("t1".to_string());
    let removed = group.remove_tab(&"t99".to_string());
    assert!(!removed);
    assert_eq!(group.len(), 1);
}

#[test]
fn test_group_set_name() {
    let mut group = auroraview_tabs::TabGroup::new("Old");
    group.set_name("New");
    assert_eq!(group.name, "New");
}

#[test]
fn test_group_set_color() {
    let mut group = auroraview_tabs::TabGroup::new("G");
    group.set_color(Some("blue".to_string()));
    assert_eq!(group.color, Some("blue".to_string()));
    group.set_color(None);
    assert!(group.color.is_none());
}

#[test]
fn test_group_toggle_collapsed() {
    let mut group = auroraview_tabs::TabGroup::new("G");
    assert!(!group.collapsed);
    group.toggle_collapsed();
    assert!(group.collapsed);
    group.toggle_collapsed();
    assert!(!group.collapsed);
}

#[test]
fn test_group_reorder_tab_to_end() {
    let mut group = auroraview_tabs::TabGroup::new("G")
        .with_tabs(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
    group.reorder_tab(&"a".to_string(), 100); // beyond end
    assert_eq!(group.tab_ids, vec!["b", "c", "a"]);
}

#[test]
fn test_group_serde_roundtrip() {
    let group = auroraview_tabs::TabGroup::new("Work")
        .with_color("red")
        .with_tabs(vec!["t1".to_string()]);
    let json = serde_json::to_string(&group).unwrap();
    let back: auroraview_tabs::TabGroup = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, "Work");
    assert_eq!(back.color, Some("red".to_string()));
    assert_eq!(back.tab_ids, vec!["t1"]);
}

// ========== TabError Display Tests ==========

#[test]
fn test_error_not_found_display() {
    let err = TabError::NotFound("tab_99".to_string());
    assert!(err.to_string().contains("tab_99"));
}

#[test]
fn test_error_group_not_found_display() {
    let err = TabError::GroupNotFound("group_x".to_string());
    assert!(err.to_string().contains("group_x"));
}

#[test]
fn test_error_invalid_operation_display() {
    let err = TabError::InvalidOperation("cannot pin active tab".to_string());
    assert!(err.to_string().contains("cannot pin active tab"));
}

#[test]
fn test_error_session_display() {
    let err = TabError::Session("file not found".to_string());
    assert!(err.to_string().contains("file not found"));
}

// ========== TabState Serde Tests ==========

#[test]
fn test_tab_state_serde_roundtrip() {
    let state = TabState::with_title("t1".to_string(), "https://example.com", "Example");
    let json = serde_json::to_string(&state).unwrap();
    let back: TabState = serde_json::from_str(&json).unwrap();
    assert_eq!(back.id, "t1");
    assert_eq!(back.title, "Example");
    assert_eq!(back.url, "https://example.com");
}

#[test]
fn test_tab_state_serde_with_favicon() {
    let mut state = TabState::new("t1".to_string(), "https://example.com");
    state.set_favicon(Some("https://example.com/favicon.ico".to_string()));
    let json = serde_json::to_string(&state).unwrap();
    let back: TabState = serde_json::from_str(&json).unwrap();
    assert_eq!(back.favicon, Some("https://example.com/favicon.ico".to_string()));
}

#[test]
fn test_tab_state_set_audible() {
    let mut state = TabState::new("t1".to_string(), "https://example.com");
    assert!(!state.audible);
    state.set_audible(true);
    assert!(state.audible);
    state.set_audible(false);
    assert!(!state.audible);
}

#[test]
fn test_tab_state_set_position() {
    let mut state = TabState::new("t1".to_string(), "https://example.com");
    assert_eq!(state.position, 0);
    state.set_position(3);
    assert_eq!(state.position, 3);
}

#[test]
fn test_tab_state_generate_id_unique() {
    let id1 = TabState::generate_id();
    let id2 = TabState::generate_id();
    assert_ne!(id1, id2);
    assert!(!id1.is_empty());
}

// ========== Manager Extra Scenarios ==========

#[test]
fn test_manager_count_and_is_empty() {
    let manager = TabManager::new();
    assert!(manager.is_empty());
    assert_eq!(manager.count(), 0);

    manager.create("https://a.com");
    assert!(!manager.is_empty());
    assert_eq!(manager.count(), 1);
}

#[test]
fn test_manager_active_state_after_multiple_creates() {
    let manager = TabManager::new();
    let id1 = manager.create("https://a.com");
    manager.create("https://b.com");
    manager.create("https://c.com");

    // Active should still be the first
    assert_eq!(manager.active_id(), Some(id1));
}

#[test]
fn test_manager_active_returns_state() {
    let manager = TabManager::new();
    let id = manager.create("https://example.com");
    let active = manager.active().unwrap();
    assert_eq!(active.id, id);
}

#[test]
fn test_manager_get_nonexistent_returns_none() {
    let manager = TabManager::new();
    assert!(manager.get(&"nonexistent".to_string()).is_none());
}

#[test]
fn test_manager_update_closure_returns_value() {
    let manager = TabManager::new();
    let id = manager.create("https://example.com");
    let result = manager.update(&id, |tab| {
        tab.set_title("Updated");
        tab.title.clone()
    });
    assert_eq!(result, Some("Updated".to_string()));
}

#[test]
fn test_manager_update_nonexistent_returns_none() {
    let manager = TabManager::new();
    let result = manager.update::<_, ()>(&"ghost".to_string(), |_| ());
    assert!(result.is_none());
}

#[test]
fn test_manager_event_url_changed_fires() {
    let manager = TabManager::new();
    let url_events = Arc::new(std::sync::Mutex::new(Vec::new()));
    let ev_clone = url_events.clone();

    manager.on_event(move |ev| {
        if let TabEvent::UrlChanged { url, .. } = ev {
            ev_clone.lock().unwrap().push(url.clone());
        }
    });

    let id = manager.create("https://old.com");
    manager.update_url(&id, "https://new.com");

    let events = url_events.lock().unwrap();
    assert!(events.iter().any(|u| u == "https://new.com"));
}

#[test]
fn test_manager_event_history_changed_fires() {
    let manager = TabManager::new();
    let back_states = Arc::new(std::sync::Mutex::new(Vec::new()));
    let bs = back_states.clone();

    manager.on_event(move |ev| {
        if let TabEvent::HistoryChanged { can_go_back, .. } = ev {
            bs.lock().unwrap().push(*can_go_back);
        }
    });

    let id = manager.create("https://example.com");
    manager.update_history(&id, true, false);

    let states = back_states.lock().unwrap();
    assert!(!states.is_empty());
    assert!(states[0]);
}

#[test]
fn test_manager_event_favicon_changed_fires() {
    let manager = TabManager::new();
    let favicons = Arc::new(std::sync::Mutex::new(Vec::new()));
    let fv = favicons.clone();

    manager.on_event(move |ev| {
        if let TabEvent::FaviconChanged { favicon_url, .. } = ev {
            fv.lock().unwrap().push(favicon_url.clone());
        }
    });

    let id = manager.create("https://example.com");
    manager.update_favicon(&id, "https://example.com/favicon.ico");

    let fv_list = favicons.lock().unwrap();
    assert!(!fv_list.is_empty());
}

#[test]
fn test_manager_event_deactivated_fires() {
    let manager = TabManager::new();
    let deactivated = Arc::new(std::sync::Mutex::new(Vec::new()));
    let dv = deactivated.clone();

    manager.on_event(move |ev| {
        if let TabEvent::Deactivated { tab_id } = ev {
            dv.lock().unwrap().push(tab_id.clone());
        }
    });

    let id1 = manager.create("https://a.com");
    let id2 = manager.create("https://b.com");
    manager.activate(&id2).unwrap();

    let deact = deactivated.lock().unwrap();
    assert!(deact.contains(&id1));
}

#[test]
fn test_manager_all_returns_order() {
    let manager = TabManager::new();
    let id1 = manager.create("https://a.com");
    let id2 = manager.create("https://b.com");
    let id3 = manager.create("https://c.com");

    let all = manager.all();
    assert_eq!(all.len(), 3);
    assert_eq!(all[0].id, id1);
    assert_eq!(all[1].id, id2);
    assert_eq!(all[2].id, id3);
}

#[test]
fn test_manager_all_groups() {
    let manager = TabManager::new();
    manager.create_group("Group A");
    manager.create_group("Group B");

    let groups = manager.all_groups();
    assert_eq!(groups.len(), 2);
}

#[test]
fn test_manager_get_group_nonexistent() {
    let manager = TabManager::new();
    assert!(manager.get_group(&"nonexistent".to_string()).is_none());
}

#[test]
fn test_manager_delete_nonexistent_group_returns_err() {
    let manager = TabManager::new();
    let result = manager.delete_group(&"ghost_group".to_string());
    assert!(result.is_err());
}

#[test]
fn test_manager_set_group_collapsed_nonexistent_returns_err() {
    let manager = TabManager::new();
    let result = manager.set_group_collapsed(&"ghost_group".to_string(), true);
    assert!(result.is_err());
}

#[test]
fn test_manager_add_to_nonexistent_tab_returns_err() {
    let manager = TabManager::new();
    let group_id = manager.create_group("G");
    let result = manager.add_to_group(&"ghost_tab".to_string(), &group_id);
    assert!(result.is_err());
}

#[test]
fn test_manager_remove_from_group_tab_not_in_group() {
    let manager = TabManager::new();
    let tab_id = manager.create("https://a.com");
    // Tab not in any group, remove_from_group should succeed (no-op)
    let result = manager.remove_from_group(&tab_id);
    assert!(result.is_ok());
}

#[test]
fn test_manager_create_with_state_sets_active_if_first() {
    let manager = TabManager::new();
    let state = TabState::with_title("custom-1".to_string(), "https://example.com", "Custom");
    let id = manager.create_with_state(state);
    assert_eq!(manager.active_id(), Some(id));
}

#[test]
fn test_manager_reorder_to_end() {
    let manager = TabManager::new();
    let id1 = manager.create("https://a.com");
    let id2 = manager.create("https://b.com");
    let id3 = manager.create("https://c.com");

    manager.reorder(&id1, 100); // beyond end
    let order = manager.order();
    assert_eq!(order, vec![id2, id3, id1]);
}

#[test]
fn test_manager_set_pinned_muted_via_manager() {
    let manager = TabManager::new();
    let id = manager.create("https://example.com");

    manager.set_pinned(&id, true);
    let tab = manager.get(&id).unwrap();
    assert!(tab.pinned);

    manager.set_muted(&id, true);
    let tab = manager.get(&id).unwrap();
    assert!(tab.muted);
}

#[test]
fn test_manager_multiple_event_handlers() {
    let manager = TabManager::new();
    let counter1 = Arc::new(AtomicUsize::new(0));
    let counter2 = Arc::new(AtomicUsize::new(0));

    let c1 = counter1.clone();
    let c2 = counter2.clone();

    manager.on_event(move |_| { c1.fetch_add(1, Ordering::SeqCst); });
    manager.on_event(move |_| { c2.fetch_add(1, Ordering::SeqCst); });

    manager.create("https://example.com");
    // Both handlers should fire on Created event
    assert!(counter1.load(Ordering::SeqCst) >= 1);
    assert!(counter2.load(Ordering::SeqCst) >= 1);
}
