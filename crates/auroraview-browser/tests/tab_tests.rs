//! Tests for tab module

use std::sync::{Arc, Mutex};

use auroraview_browser::tab::{SecurityState, TabEvent, TabEventKind, TabListenerMap, TabState};

#[test]
fn test_tab_state_new() {
    let state = TabState::new("tab_1".to_string(), "https://example.com".to_string());

    assert_eq!(state.id, "tab_1");
    assert_eq!(state.url, "https://example.com");
    assert_eq!(state.title, "New Tab");
    assert!(state.is_loading);
    assert!(!state.can_go_back);
    assert!(!state.can_go_forward);
    assert!(!state.pinned);
    assert!(!state.muted);
    assert!(!state.audible);
}

#[test]
fn test_tab_state_setters() {
    let mut state = TabState::new("tab_1".to_string(), "https://example.com".to_string());

    state.set_title("Google".to_string());
    assert_eq!(state.title, "Google");

    state.set_url("https://google.com".to_string());
    assert_eq!(state.url, "https://google.com");

    state.set_loading(false);
    assert!(!state.is_loading);

    state.set_history_state(true, false);
    assert!(state.can_go_back);
    assert!(!state.can_go_forward);

    state.set_pinned(true);
    assert!(state.pinned);

    state.set_muted(true);
    assert!(state.muted);
}

#[test]
fn test_tab_state_security() {
    let mut state = TabState::new("tab_1".to_string(), "http://example.com".to_string());

    // Update to HTTPS URL
    state.set_url("https://secure.example.com".to_string());
    assert!(matches!(state.security_state, Some(SecurityState::Secure)));

    // Update to HTTP URL
    state.set_url("http://insecure.example.com".to_string());
    assert!(matches!(
        state.security_state,
        Some(SecurityState::Insecure)
    ));

    // Update to file URL
    state.set_url("file:///path/to/file.html".to_string());
    assert!(matches!(state.security_state, Some(SecurityState::Neutral)));
}

#[test]
fn test_tab_state_empty_title_not_updated() {
    let mut state = TabState::new("tab_1".to_string(), "https://example.com".to_string());
    state.set_title("Original Title".to_string());

    // Empty title should not update
    state.set_title("".to_string());
    assert_eq!(state.title, "Original Title");
}

// === TabEvent builder tests ===

#[test]
fn test_tab_event_new_tab_with_url() {
    let ev = TabEvent::new_tab(Some("https://example.com".to_string()));
    assert!(matches!(ev, TabEvent::NewTab { url: Some(ref u) } if u == "https://example.com"));
}

#[test]
fn test_tab_event_new_tab_without_url() {
    let ev = TabEvent::new_tab(None);
    assert!(matches!(ev, TabEvent::NewTab { url: None }));
}

#[test]
fn test_tab_event_close_tab_str_literal() {
    let ev = TabEvent::close_tab("tab_42");
    assert!(matches!(ev, TabEvent::CloseTab { ref tab_id } if tab_id == "tab_42"));
}

#[test]
fn test_tab_event_activate_tab() {
    let ev = TabEvent::activate_tab("tab_1");
    assert!(matches!(ev, TabEvent::ActivateTab { ref tab_id } if tab_id == "tab_1"));
}

#[test]
fn test_tab_event_navigate() {
    let ev = TabEvent::navigate("https://rust-lang.org");
    assert!(matches!(ev, TabEvent::Navigate { ref url } if url == "https://rust-lang.org"));
}

#[test]
fn test_tab_event_pin_tab() {
    let ev = TabEvent::pin_tab("tab_3", true);
    assert!(matches!(ev, TabEvent::PinTab { ref tab_id, pinned: true } if tab_id == "tab_3"));

    let ev2 = TabEvent::pin_tab("tab_3", false);
    assert!(matches!(ev2, TabEvent::PinTab { pinned: false, .. }));
}

#[test]
fn test_tab_event_mute_tab() {
    let ev = TabEvent::mute_tab("tab_5", true);
    assert!(matches!(ev, TabEvent::MuteTab { ref tab_id, muted: true } if tab_id == "tab_5"));
}

#[test]
fn test_tab_event_reorder_tab() {
    let ev = TabEvent::reorder_tab("tab_2", 0);
    assert!(matches!(ev, TabEvent::ReorderTab { ref tab_id, new_index: 0 } if tab_id == "tab_2"));
}

#[test]
fn test_tab_event_duplicate_tab() {
    let ev = TabEvent::duplicate_tab("tab_7");
    assert!(matches!(ev, TabEvent::DuplicateTab { ref tab_id } if tab_id == "tab_7"));
}

#[test]
fn test_tab_event_toggle_devtools_with_id() {
    let ev = TabEvent::toggle_devtools(Some("tab_1"));
    assert!(matches!(ev, TabEvent::ToggleDevTools { tab_id: Some(ref id) } if id == "tab_1"));
}

#[test]
fn test_tab_event_toggle_devtools_active() {
    // None means "use active tab"
    let ev = TabEvent::toggle_devtools(Option::<String>::None);
    assert!(matches!(ev, TabEvent::ToggleDevTools { tab_id: None }));
}

#[test]
fn test_tab_event_open_devtools() {
    let ev = TabEvent::open_devtools(Some("tab_2"));
    assert!(matches!(ev, TabEvent::OpenDevTools { tab_id: Some(ref id) } if id == "tab_2"));
}

#[test]
fn test_tab_event_close_devtools() {
    let ev = TabEvent::close_devtools(Some("tab_3"));
    assert!(matches!(ev, TabEvent::CloseDevTools { tab_id: Some(ref id) } if id == "tab_3"));
}

// === TabListenerMap tests ===

fn make_state(id: &str) -> TabState {
    TabState::new(id.to_string(), "https://example.com".to_string())
}

#[test]
fn test_listener_on_receives_event() {
    let map = TabListenerMap::new();
    let received = Arc::new(Mutex::new(Vec::<String>::new()));
    let recv = received.clone();

    map.on(TabEventKind::Activated, move |s| {
        recv.lock().unwrap().push(s.id.clone());
    });

    map.emit(&TabEventKind::Activated, &make_state("tab_1"));
    assert_eq!(*received.lock().unwrap(), vec!["tab_1"]);
}

#[test]
fn test_listener_off_removes_specific_listener() {
    let map = TabListenerMap::new();
    let count = Arc::new(Mutex::new(0u32));
    let c = count.clone();

    let id = map.on(TabEventKind::Created, move |_| {
        *c.lock().unwrap() += 1;
    });

    map.emit(&TabEventKind::Created, &make_state("t1"));
    assert_eq!(*count.lock().unwrap(), 1);

    let removed = map.off(&TabEventKind::Created, id);
    assert!(removed);

    map.emit(&TabEventKind::Created, &make_state("t2"));
    assert_eq!(*count.lock().unwrap(), 1, "should not receive after off()");
}

#[test]
fn test_listener_off_returns_false_for_unknown_id() {
    let map = TabListenerMap::new();
    let result = map.off(&TabEventKind::Closed, 9999);
    assert!(!result);
}

#[test]
fn test_listener_off_all_clears_all() {
    let map = TabListenerMap::new();
    let count = Arc::new(Mutex::new(0u32));

    for _ in 0..3 {
        let c = count.clone();
        map.on(TabEventKind::Updated, move |_| {
            *c.lock().unwrap() += 1;
        });
    }
    assert_eq!(map.listener_count(&TabEventKind::Updated), 3);

    let removed = map.off_all(&TabEventKind::Updated);
    assert_eq!(removed, 3);
    assert_eq!(map.listener_count(&TabEventKind::Updated), 0);

    map.emit(&TabEventKind::Updated, &make_state("t1"));
    assert_eq!(*count.lock().unwrap(), 0);
}

#[test]
fn test_listener_count_zero_when_none() {
    let map = TabListenerMap::new();
    assert_eq!(map.listener_count(&TabEventKind::LoadingChanged), 0);
}

#[test]
fn test_multiple_listeners_all_called() {
    let map = TabListenerMap::new();
    let ids = Arc::new(Mutex::new(Vec::<String>::new()));

    for suffix in ["a", "b", "c"] {
        let ids2 = ids.clone();
        let tag = suffix.to_string();
        map.on(TabEventKind::StateChanged, move |s| {
            ids2.lock().unwrap().push(format!("{}-{}", s.id, tag));
        });
    }

    map.emit(&TabEventKind::StateChanged, &make_state("tab_x"));
    let mut result = ids.lock().unwrap().clone();
    result.sort();
    assert_eq!(result, vec!["tab_x-a", "tab_x-b", "tab_x-c"]);
}

#[test]
fn test_listener_different_kinds_isolated() {
    let map = TabListenerMap::new();
    let activated = Arc::new(Mutex::new(0u32));
    let closed = Arc::new(Mutex::new(0u32));
    let a = activated.clone();
    let c = closed.clone();

    map.on(TabEventKind::Activated, move |_| {
        *a.lock().unwrap() += 1;
    });
    map.on(TabEventKind::Closed, move |_| {
        *c.lock().unwrap() += 1;
    });

    map.emit(&TabEventKind::Activated, &make_state("t1"));
    map.emit(&TabEventKind::Activated, &make_state("t2"));
    map.emit(&TabEventKind::Closed, &make_state("t3"));

    assert_eq!(*activated.lock().unwrap(), 2);
    assert_eq!(*closed.lock().unwrap(), 1);
}

#[test]
fn test_off_all_returns_zero_when_no_listeners() {
    let map = TabListenerMap::new();
    assert_eq!(map.off_all(&TabEventKind::DevToolsToggled), 0);
}

