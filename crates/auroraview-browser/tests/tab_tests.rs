//! Tests for tab module

use auroraview_browser::tab::{SecurityState, TabEvent, TabState};

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
