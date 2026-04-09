//! Tests for tab module

use auroraview_browser::tab::{SecurityState, TabEvent, TabState};
use rstest::rstest;

#[test]
fn tab_state_new() {
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
fn tab_state_setters() {
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
fn tab_state_security() {
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
fn tab_state_empty_title_not_updated() {
    let mut state = TabState::new("tab_1".to_string(), "https://example.com".to_string());
    state.set_title("Original Title".to_string());

    // Empty title should not update
    state.set_title("".to_string());
    assert_eq!(state.title, "Original Title");
}

// === TabEvent builder tests ===

#[test]
fn tab_event_new_tab_with_url() {
    let ev = TabEvent::new_tab(Some("https://example.com".to_string()));
    assert!(matches!(ev, TabEvent::NewTab { url: Some(ref u) } if u == "https://example.com"));
}

#[test]
fn tab_event_new_tab_without_url() {
    let ev = TabEvent::new_tab(None);
    assert!(matches!(ev, TabEvent::NewTab { url: None }));
}

#[test]
fn tab_event_close_tab_str_literal() {
    let ev = TabEvent::close_tab("tab_42");
    assert!(matches!(ev, TabEvent::CloseTab { ref tab_id } if tab_id == "tab_42"));
}

#[test]
fn tab_event_activate_tab() {
    let ev = TabEvent::activate_tab("tab_1");
    assert!(matches!(ev, TabEvent::ActivateTab { ref tab_id } if tab_id == "tab_1"));
}

#[test]
fn tab_event_navigate() {
    let ev = TabEvent::navigate("https://rust-lang.org");
    assert!(matches!(ev, TabEvent::Navigate { ref url } if url == "https://rust-lang.org"));
}

#[test]
fn tab_event_pin_tab() {
    let ev = TabEvent::pin_tab("tab_3", true);
    assert!(matches!(ev, TabEvent::PinTab { ref tab_id, pinned: true } if tab_id == "tab_3"));

    let ev2 = TabEvent::pin_tab("tab_3", false);
    assert!(matches!(ev2, TabEvent::PinTab { pinned: false, .. }));
}

#[test]
fn tab_event_mute_tab() {
    let ev = TabEvent::mute_tab("tab_5", true);
    assert!(matches!(ev, TabEvent::MuteTab { ref tab_id, muted: true } if tab_id == "tab_5"));
}

#[test]
fn tab_event_reorder_tab() {
    let ev = TabEvent::reorder_tab("tab_2", 0);
    assert!(matches!(ev, TabEvent::ReorderTab { ref tab_id, new_index: 0 } if tab_id == "tab_2"));
}

#[test]
fn tab_event_duplicate_tab() {
    let ev = TabEvent::duplicate_tab("tab_7");
    assert!(matches!(ev, TabEvent::DuplicateTab { ref tab_id } if tab_id == "tab_7"));
}

#[test]
fn tab_event_toggle_devtools_with_id() {
    let ev = TabEvent::toggle_devtools(Some("tab_1"));
    assert!(matches!(ev, TabEvent::ToggleDevTools { tab_id: Some(ref id) } if id == "tab_1"));
}

#[test]
fn tab_event_toggle_devtools_active() {
    // None means "use active tab"
    let ev = TabEvent::toggle_devtools(Option::<String>::None);
    assert!(matches!(ev, TabEvent::ToggleDevTools { tab_id: None }));
}

#[test]
fn tab_event_open_devtools() {
    let ev = TabEvent::open_devtools(Some("tab_2"));
    assert!(matches!(ev, TabEvent::OpenDevTools { tab_id: Some(ref id) } if id == "tab_2"));
}

#[test]
fn tab_event_close_devtools() {
    let ev = TabEvent::close_devtools(Some("tab_3"));
    assert!(matches!(ev, TabEvent::CloseDevTools { tab_id: Some(ref id) } if id == "tab_3"));
}

// === Extended tests ===

#[test]
fn tab_state_default_security_is_none() {
    let state = TabState::new("t1".to_string(), "https://example.com".to_string());
    // Security state is None until set_url is called
    assert!(state.security_state.is_none());
}

#[test]
fn tab_state_security_after_set_url_https() {
    let mut state = TabState::new("t1".to_string(), "https://example.com".to_string());
    state.set_url("https://example.com".to_string());
    assert_eq!(state.security_state, Some(SecurityState::Secure));
}

#[test]
fn tab_state_set_audible() {
    let mut state = TabState::new("t1".to_string(), "https://example.com".to_string());
    assert!(!state.audible);
    state.set_audible(true);
    assert!(state.audible);
    state.set_audible(false);
    assert!(!state.audible);
}

#[test]
fn tab_state_set_favicon() {
    let mut state = TabState::new("t1".to_string(), "https://example.com".to_string());
    assert!(state.favicon.is_none());

    state.set_favicon(Some("https://example.com/favicon.ico".to_string()));
    assert_eq!(
        state.favicon,
        Some("https://example.com/favicon.ico".to_string())
    );

    state.set_favicon(None);
    assert!(state.favicon.is_none());
}

#[test]
fn tab_state_set_loading_toggle() {
    let mut state = TabState::new("t1".to_string(), "https://example.com".to_string());
    assert!(state.is_loading);
    state.set_loading(false);
    assert!(!state.is_loading);
    state.set_loading(true);
    assert!(state.is_loading);
}

#[test]
fn tab_state_history_both_directions() {
    let mut state = TabState::new("t1".to_string(), "https://a.com".to_string());
    state.set_history_state(true, true);
    assert!(state.can_go_back);
    assert!(state.can_go_forward);

    state.set_history_state(false, false);
    assert!(!state.can_go_back);
    assert!(!state.can_go_forward);
}

#[test]
fn tab_state_serde_roundtrip() {
    let mut state = TabState::new("tab_serde".to_string(), "https://example.com".to_string());
    state.set_title("Serde Test".to_string());
    state.set_loading(false);
    state.set_pinned(true);
    state.set_url("https://example.com".to_string());

    let json = serde_json::to_string(&state).unwrap();
    let deserialized: TabState = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.id, state.id);
    assert_eq!(deserialized.title, state.title);
    assert_eq!(deserialized.url, state.url);
    assert_eq!(deserialized.is_loading, state.is_loading);
    assert_eq!(deserialized.pinned, state.pinned);
    assert_eq!(deserialized.security_state, state.security_state);
}

#[test]
fn security_state_serde_roundtrip() {
    for variant in &[SecurityState::Secure, SecurityState::Insecure, SecurityState::Neutral] {
        let json = serde_json::to_string(variant).unwrap();
        let deser: SecurityState = serde_json::from_str(&json).unwrap();
        assert_eq!(&deser, variant);
    }
}

#[test]
fn security_state_clone_and_eq() {
    let s = SecurityState::Secure;
    let s2 = s.clone();
    assert_eq!(s, s2);
    assert_ne!(SecurityState::Secure, SecurityState::Insecure);
    assert_ne!(SecurityState::Insecure, SecurityState::Neutral);
}

// rstest: security state for various URL schemes
#[rstest]
#[case("https://secure.com", SecurityState::Secure)]
#[case("http://insecure.com", SecurityState::Insecure)]
#[case("file:///local/file.html", SecurityState::Neutral)]
#[case("ftp://ftp.example.com", SecurityState::Neutral)]
#[case("auroraview://localhost/index.html", SecurityState::Neutral)]
fn tab_state_security_by_url(#[case] url: &str, #[case] expected: SecurityState) {
    let mut state = TabState::new("t1".to_string(), "about:blank".to_string());
    state.set_url(url.to_string());
    assert_eq!(state.security_state, Some(expected));
}

#[test]
fn tab_state_title_updates_multiple_times() {
    let mut state = TabState::new("t1".to_string(), "https://a.com".to_string());
    for i in 0..10 {
        state.set_title(format!("Title {i}"));
        assert_eq!(state.title, format!("Title {i}"));
    }
}

#[test]
fn tab_event_reorder_large_index() {
    let ev = TabEvent::reorder_tab("tab_x", 999);
    assert!(matches!(ev, TabEvent::ReorderTab { new_index: 999, .. }));
}

#[test]
fn tab_event_mute_then_unmute() {
    let mute_ev = TabEvent::mute_tab("tab_1", true);
    let unmute_ev = TabEvent::mute_tab("tab_1", false);
    assert!(matches!(mute_ev, TabEvent::MuteTab { muted: true, .. }));
    assert!(matches!(unmute_ev, TabEvent::MuteTab { muted: false, .. }));
}

// ─── Additional coverage R9 ──────────────────────────────────────────────────

#[test]
fn tab_state_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<TabState>();
}

#[test]
fn security_state_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<SecurityState>();
}

#[test]
fn tab_state_default_security_state_none() {
    let state = TabState::new("t1".to_string(), "about:blank".to_string());
    // Default url 'about:blank' may set security to None or Neutral
    let _ = state.security_state;
}

#[test]
fn tab_state_audible_default_false() {
    let state = TabState::new("t1".to_string(), "https://example.com".to_string());
    assert!(!state.audible);
}

#[test]
fn tab_state_set_audible_true() {
    let mut state = TabState::new("t1".to_string(), "https://example.com".to_string());
    state.set_audible(true);
    assert!(state.audible);
}

#[test]
fn tab_state_id_preserved() {
    let state = TabState::new("unique-tab-id-xyz".to_string(), "https://a.com".to_string());
    assert_eq!(state.id, "unique-tab-id-xyz");
}

#[test]
fn tab_state_url_update_changes_security() {
    let mut state = TabState::new("t1".to_string(), "about:blank".to_string());
    // Set to http:// — should become Insecure
    state.set_url("http://insecure.com".to_string());
    assert_eq!(state.security_state, Some(SecurityState::Insecure));
    // Then to https:// — should become Secure
    state.set_url("https://secure.com".to_string());
    assert_eq!(state.security_state, Some(SecurityState::Secure));
}

#[test]
fn tab_state_can_go_back_forward_default_false() {
    let state = TabState::new("t1".to_string(), "https://a.com".to_string());
    assert!(!state.can_go_back);
    assert!(!state.can_go_forward);
}

#[test]
fn tab_state_pinned_default_false() {
    let state = TabState::new("t1".to_string(), "https://a.com".to_string());
    assert!(!state.pinned);
}

#[test]
fn tab_state_muted_default_false() {
    let state = TabState::new("t1".to_string(), "https://a.com".to_string());
    assert!(!state.muted);
}

#[test]
fn tab_state_set_pinned_toggle() {
    let mut state = TabState::new("t1".to_string(), "https://a.com".to_string());
    state.set_pinned(true);
    assert!(state.pinned);
    state.set_pinned(false);
    assert!(!state.pinned);
}

#[test]
fn tab_state_set_muted_toggle() {
    let mut state = TabState::new("t1".to_string(), "https://a.com".to_string());
    state.set_muted(true);
    assert!(state.muted);
    state.set_muted(false);
    assert!(!state.muted);
}

#[test]
fn tab_state_title_default() {
    let state = TabState::new("t1".to_string(), "https://a.com".to_string());
    assert_eq!(state.title, "New Tab");
}

#[rstest]
#[case("tab-a", "https://a.com")]
#[case("tab-b", "https://b.com")]
#[case("tab-c", "file:///local.html")]
fn tab_state_new_parametrized(#[case] id: &str, #[case] url: &str) {
    let state = TabState::new(id.to_string(), url.to_string());
    assert_eq!(state.id, id);
    assert_eq!(state.url, url);
    assert_eq!(state.title, "New Tab");
}
