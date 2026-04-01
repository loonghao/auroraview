//! Tests for chrome.tabs API

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use serde_json::{json, Value};

use auroraview_extensions::apis::tabs::{TabState, TabsApiHandler};
use auroraview_extensions::apis::ApiHandler;

#[test]
fn test_tab_state_new() {
    let state = TabState::new();
    let tab = state.get_current();
    assert_eq!(tab.id, 1);
    assert_eq!(tab.window_id, 1);
    assert!(tab.active);
    assert!(tab.url.is_none());
}

#[test]
fn test_tab_state_default() {
    let state = TabState::default();
    let tab = state.get_current();
    assert_eq!(tab.id, 1);
}

#[test]
fn test_tab_state_set_url() {
    let state = TabState::new();
    state.set_url("https://example.com");
    let tab = state.get_current();
    assert_eq!(tab.url, Some("https://example.com".to_string()));
}

#[test]
fn test_tab_state_set_title() {
    let state = TabState::new();
    state.set_title("My Page");
    let tab = state.get_current();
    assert_eq!(tab.title, Some("My Page".to_string()));
}

#[test]
fn test_tab_state_navigate_with_callback() {
    let called = Arc::new(AtomicBool::new(false));
    let called_clone = called.clone();
    let state = TabState::new();
    state.set_on_navigate(move |url| {
        assert_eq!(url, "https://example.com/page");
        called_clone.store(true, Ordering::SeqCst);
    });

    state.navigate("https://example.com/page");
    assert!(called.load(Ordering::SeqCst));
    // URL should also be set
    let tab = state.get_current();
    assert_eq!(tab.url, Some("https://example.com/page".to_string()));
}

#[test]
fn test_tab_state_navigate_without_callback() {
    let state = TabState::new();
    // Should not panic when no callback is set
    state.navigate("https://example.com");
    let tab = state.get_current();
    assert_eq!(tab.url, Some("https://example.com".to_string()));
}

#[test]
fn test_tab_state_send_message_with_callback() {
    let state = TabState::new();
    state.set_on_send_message(|tab_id, msg| {
        if tab_id == 1 {
            Some(json!({"response": msg}))
        } else {
            None
        }
    });

    let response = state.send_message(1, json!("hello"));
    assert_eq!(response, Some(json!({"response": "hello"})));

    let response = state.send_message(999, json!("hello"));
    assert_eq!(response, None);
}

#[test]
fn test_tab_state_send_message_without_callback() {
    let state = TabState::new();
    let response = state.send_message(1, json!("hello"));
    assert_eq!(response, None);
}

// --- API Handler tests ---

#[test]
fn test_handler_query() {
    let state = Arc::new(TabState::new());
    state.set_url("https://example.com");
    let handler = TabsApiHandler::new(state);

    let result = handler.handle("query", json!({"active": true}), "test-ext");
    assert!(result.is_ok());
    let tabs: Vec<Value> = serde_json::from_value(result.unwrap()).unwrap();
    assert_eq!(tabs.len(), 1);
    assert_eq!(tabs[0]["url"], "https://example.com");
}

#[test]
fn test_handler_get() {
    let state = Arc::new(TabState::new());
    let handler = TabsApiHandler::new(state);

    let result = handler.handle("get", json!({"tabId": 1}), "test-ext");
    assert!(result.is_ok());
    let tab = result.unwrap();
    assert_eq!(tab["id"], 1);
}

#[test]
fn test_handler_get_not_found() {
    let state = Arc::new(TabState::new());
    let handler = TabsApiHandler::new(state);

    let result = handler.handle("get", json!({"tabId": 999}), "test-ext");
    assert!(result.is_err());
}

#[test]
fn test_handler_get_missing_tab_id() {
    let state = Arc::new(TabState::new());
    let handler = TabsApiHandler::new(state);

    let result = handler.handle("get", json!({}), "test-ext");
    assert!(result.is_err());
}

#[test]
fn test_handler_get_current() {
    let state = Arc::new(TabState::new());
    let handler = TabsApiHandler::new(state);

    let result = handler.handle("getCurrent", json!({}), "test-ext");
    assert!(result.is_ok());
    let tab = result.unwrap();
    assert_eq!(tab["id"], 1);
    assert_eq!(tab["active"], true);
}

#[test]
fn test_handler_create_navigates() {
    let called = Arc::new(AtomicBool::new(false));
    let called_clone = called.clone();
    let state = Arc::new(TabState::new());
    state.set_on_navigate(move |url| {
        assert_eq!(url, "https://new-page.com");
        called_clone.store(true, Ordering::SeqCst);
    });
    let handler = TabsApiHandler::new(state);

    let result = handler.handle("create", json!({"url": "https://new-page.com"}), "test-ext");
    assert!(result.is_ok());
    assert!(called.load(Ordering::SeqCst));
    let tab = result.unwrap();
    assert_eq!(tab["url"], "https://new-page.com");
}

#[test]
fn test_handler_create_no_url() {
    let state = Arc::new(TabState::new());
    let handler = TabsApiHandler::new(state);

    let result = handler.handle("create", json!({}), "test-ext");
    assert!(result.is_ok());
}

#[test]
fn test_handler_update_navigates() {
    let called = Arc::new(AtomicBool::new(false));
    let called_clone = called.clone();
    let state = Arc::new(TabState::new());
    state.set_on_navigate(move |url| {
        assert_eq!(url, "https://updated.com");
        called_clone.store(true, Ordering::SeqCst);
    });
    let handler = TabsApiHandler::new(state);

    let result = handler.handle("update", json!({"url": "https://updated.com"}), "test-ext");
    assert!(result.is_ok());
    assert!(called.load(Ordering::SeqCst));
}

#[test]
fn test_handler_remove() {
    let state = Arc::new(TabState::new());
    let handler = TabsApiHandler::new(state);

    let result = handler.handle("remove", json!({}), "test-ext");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), json!({}));
}

#[test]
fn test_handler_send_message() {
    let state = Arc::new(TabState::new());
    state.set_on_send_message(|_tab_id, msg| Some(json!({"echo": msg})));
    let handler = TabsApiHandler::new(state);

    let result = handler.handle(
        "sendMessage",
        json!({"tabId": 1, "message": {"type": "test"}}),
        "test-ext",
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), json!({"echo": {"type": "test"}}));
}

#[test]
fn test_handler_send_message_missing_tab_id() {
    let state = Arc::new(TabState::new());
    let handler = TabsApiHandler::new(state);

    let result = handler.handle("sendMessage", json!({"message": "hello"}), "test-ext");
    assert!(result.is_err());
}

#[test]
fn test_handler_send_message_no_callback() {
    let state = Arc::new(TabState::new());
    let handler = TabsApiHandler::new(state);

    let result = handler.handle(
        "sendMessage",
        json!({"tabId": 1, "message": "hello"}),
        "test-ext",
    );
    assert!(result.is_ok());
    assert!(result.unwrap().is_null());
}

#[test]
fn test_handler_unsupported_method() {
    let state = Arc::new(TabState::new());
    let handler = TabsApiHandler::new(state);

    let result = handler.handle("nonExistent", json!({}), "test-ext");
    assert!(result.is_err());
}

#[test]
fn test_handler_namespace() {
    let state = Arc::new(TabState::new());
    let handler = TabsApiHandler::new(state);
    assert_eq!(handler.namespace(), "tabs");
}

#[test]
fn test_handler_methods_list() {
    let state = Arc::new(TabState::new());
    let handler = TabsApiHandler::new(state);
    let methods = handler.methods();
    assert!(methods.contains(&"query"));
    assert!(methods.contains(&"get"));
    assert!(methods.contains(&"getCurrent"));
    assert!(methods.contains(&"create"));
    assert!(methods.contains(&"update"));
    assert!(methods.contains(&"remove"));
    assert!(methods.contains(&"sendMessage"));
}
