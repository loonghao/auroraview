//! Tests for tab module

use auroraview_browser::tab::{SecurityState, TabState};

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
    assert!(matches!(
        state.security_state,
        Some(SecurityState::Secure)
    ));

    // Update to HTTP URL
    state.set_url("http://insecure.example.com".to_string());
    assert!(matches!(
        state.security_state,
        Some(SecurityState::Insecure)
    ));

    // Update to file URL
    state.set_url("file:///path/to/file.html".to_string());
    assert!(matches!(
        state.security_state,
        Some(SecurityState::Neutral)
    ));
}

#[test]
fn test_tab_state_empty_title_not_updated() {
    let mut state = TabState::new("tab_1".to_string(), "https://example.com".to_string());
    state.set_title("Original Title".to_string());

    // Empty title should not update
    state.set_title("".to_string());
    assert_eq!(state.title, "Original Title");
}
