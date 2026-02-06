//! Tests for chrome.action API

use auroraview_extensions::apis::action::{ActionApiHandler, ActionManager, ActionState};
use auroraview_extensions::apis::ApiHandler;
use serde_json::json;
use std::sync::Arc;

#[test]
fn test_action_manager_new() {
    let manager = ActionManager::new();
    let state = manager.get_state("test-ext", None);
    assert!(state.enabled);
    assert!(state.badge_text.is_none());
}

#[test]
fn test_action_manager_default() {
    let manager = ActionManager::default();
    let state = manager.get_state("test-ext", None);
    assert!(state.enabled);
}

#[test]
fn test_update_global_state() {
    let manager = ActionManager::new();

    manager.update_global_state("test-ext", |state| {
        state.badge_text = Some("5".to_string());
        state.title = Some("My Extension".to_string());
    });

    let state = manager.get_state("test-ext", None);
    assert_eq!(state.badge_text, Some("5".to_string()));
    assert_eq!(state.title, Some("My Extension".to_string()));
}

#[test]
fn test_update_tab_state() {
    let manager = ActionManager::new();

    // Set global state
    manager.update_global_state("test-ext", |state| {
        state.badge_text = Some("global".to_string());
    });

    // Set tab-specific state
    manager.update_tab_state("test-ext", 1, |state| {
        state.badge_text = Some("tab1".to_string());
    });

    // Global state should be unchanged
    let global_state = manager.get_state("test-ext", None);
    assert_eq!(global_state.badge_text, Some("global".to_string()));

    // Tab state should override
    let tab_state = manager.get_state("test-ext", Some(1));
    assert_eq!(tab_state.badge_text, Some("tab1".to_string()));

    // Other tabs should use global
    let other_tab_state = manager.get_state("test-ext", Some(2));
    assert_eq!(other_tab_state.badge_text, Some("global".to_string()));
}

#[test]
fn test_action_click_callback() {
    use std::sync::atomic::{AtomicBool, Ordering};

    let manager = ActionManager::new();
    let clicked = Arc::new(AtomicBool::new(false));
    let clicked_clone = clicked.clone();

    manager.set_on_clicked(move |ext_id, tab_id| {
        assert_eq!(ext_id, "test-ext");
        assert_eq!(tab_id, 42);
        clicked_clone.store(true, Ordering::SeqCst);
    });

    manager.trigger_click("test-ext", 42);
    assert!(clicked.load(Ordering::SeqCst));
}

#[test]
fn test_action_api_handler_namespace() {
    let manager = Arc::new(ActionManager::new());
    let handler = ActionApiHandler::new(manager);
    assert_eq!(handler.namespace(), "action");
}

#[test]
fn test_action_api_handler_methods() {
    let manager = Arc::new(ActionManager::new());
    let handler = ActionApiHandler::new(manager);
    let methods = handler.methods();

    assert!(methods.contains(&"setTitle"));
    assert!(methods.contains(&"getTitle"));
    assert!(methods.contains(&"setBadgeText"));
    assert!(methods.contains(&"getBadgeText"));
    assert!(methods.contains(&"enable"));
    assert!(methods.contains(&"disable"));
    assert!(methods.contains(&"isEnabled"));
}

#[test]
fn test_action_api_set_get_title() {
    let manager = Arc::new(ActionManager::new());
    let handler = ActionApiHandler::new(manager);

    // Set title
    let result = handler.handle("setTitle", json!({"title": "Test Title"}), "test-ext");
    assert!(result.is_ok());

    // Get title
    let result = handler.handle("getTitle", json!({}), "test-ext");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), json!("Test Title"));
}

#[test]
fn test_action_api_set_get_badge_text() {
    let manager = Arc::new(ActionManager::new());
    let handler = ActionApiHandler::new(manager);

    // Set badge text
    let result = handler.handle("setBadgeText", json!({"text": "99"}), "test-ext");
    assert!(result.is_ok());

    // Get badge text
    let result = handler.handle("getBadgeText", json!({}), "test-ext");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), json!("99"));
}

#[test]
fn test_action_api_set_badge_color_string() {
    let manager = Arc::new(ActionManager::new());
    let handler = ActionApiHandler::new(manager);

    // Set badge color as string
    let result = handler.handle(
        "setBadgeBackgroundColor",
        json!({"color": "#FF0000"}),
        "test-ext",
    );
    assert!(result.is_ok());

    // Get badge color
    let result = handler.handle("getBadgeBackgroundColor", json!({}), "test-ext");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), json!("#FF0000"));
}

#[test]
fn test_action_api_set_badge_color_array() {
    let manager = Arc::new(ActionManager::new());
    let handler = ActionApiHandler::new(manager);

    // Set badge color as RGBA array
    let result = handler.handle(
        "setBadgeBackgroundColor",
        json!({"color": [255, 0, 0, 255]}),
        "test-ext",
    );
    assert!(result.is_ok());

    // Get badge color
    let result = handler.handle("getBadgeBackgroundColor", json!({}), "test-ext");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), json!("rgba(255, 0, 0, 1)"));
}

#[test]
fn test_action_api_enable_disable() {
    let manager = Arc::new(ActionManager::new());
    let handler = ActionApiHandler::new(manager);

    // Initially enabled
    let result = handler.handle("isEnabled", json!({}), "test-ext");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), json!(true));

    // Disable
    let result = handler.handle("disable", json!({}), "test-ext");
    assert!(result.is_ok());

    // Check disabled
    let result = handler.handle("isEnabled", json!({}), "test-ext");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), json!(false));

    // Enable
    let result = handler.handle("enable", json!({}), "test-ext");
    assert!(result.is_ok());

    // Check enabled
    let result = handler.handle("isEnabled", json!({}), "test-ext");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), json!(true));
}

#[test]
fn test_action_api_set_get_popup() {
    let manager = Arc::new(ActionManager::new());
    let handler = ActionApiHandler::new(manager);

    // Set popup
    let result = handler.handle("setPopup", json!({"popup": "popup.html"}), "test-ext");
    assert!(result.is_ok());

    // Get popup
    let result = handler.handle("getPopup", json!({}), "test-ext");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), json!("popup.html"));
}

#[test]
fn test_action_api_unsupported_method() {
    let manager = Arc::new(ActionManager::new());
    let handler = ActionApiHandler::new(manager);

    let result = handler.handle("unknownMethod", json!({}), "test-ext");
    assert!(result.is_err());
}

#[test]
fn test_action_state_default() {
    let state = ActionState::default();
    assert!(!state.enabled); // Default is false
    assert!(state.badge_text.is_none());
    assert!(state.title.is_none());
    assert!(state.popup.is_none());
}

#[test]
fn test_action_api_tab_specific() {
    let manager = Arc::new(ActionManager::new());
    let handler = ActionApiHandler::new(manager);

    // Set global title
    let _ = handler.handle("setTitle", json!({"title": "Global"}), "test-ext");

    // Set tab-specific title
    let _ = handler.handle(
        "setTitle",
        json!({"title": "Tab 1", "tabId": 1}),
        "test-ext",
    );

    // Get global title
    let result = handler.handle("getTitle", json!({}), "test-ext");
    assert_eq!(result.unwrap(), json!("Global"));

    // Get tab-specific title
    let result = handler.handle("getTitle", json!({"tabId": 1}), "test-ext");
    assert_eq!(result.unwrap(), json!("Tab 1"));

    // Other tabs should get global
    let result = handler.handle("getTitle", json!({"tabId": 2}), "test-ext");
    assert_eq!(result.unwrap(), json!("Global"));
}
