//! Tests for chrome.runtime API

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde_json::json;

use auroraview_extensions::apis::runtime::{MessageSender, RuntimeApiHandler, RuntimeManager};
use auroraview_extensions::apis::ApiHandler;

#[test]
fn test_runtime_manager_new() {
    let manager = RuntimeManager::new();
    assert!(manager.get_uninstall_url("test-ext").is_none());
}

#[test]
fn test_runtime_manager_default() {
    let manager = RuntimeManager::default();
    assert!(manager.get_uninstall_url("test-ext").is_none());
}

#[test]
fn test_set_uninstall_url() {
    let manager = RuntimeManager::new();
    manager.set_uninstall_url("test-ext", "https://example.com/uninstall");
    assert_eq!(
        manager.get_uninstall_url("test-ext"),
        Some("https://example.com/uninstall".to_string())
    );
}

#[test]
fn test_set_uninstall_url_overwrite() {
    let manager = RuntimeManager::new();
    manager.set_uninstall_url("test-ext", "https://example.com/v1");
    manager.set_uninstall_url("test-ext", "https://example.com/v2");
    assert_eq!(
        manager.get_uninstall_url("test-ext"),
        Some("https://example.com/v2".to_string())
    );
}

#[test]
fn test_set_uninstall_url_per_extension() {
    let manager = RuntimeManager::new();
    manager.set_uninstall_url("ext-a", "https://a.com/uninstall");
    manager.set_uninstall_url("ext-b", "https://b.com/uninstall");
    assert_eq!(
        manager.get_uninstall_url("ext-a"),
        Some("https://a.com/uninstall".to_string())
    );
    assert_eq!(
        manager.get_uninstall_url("ext-b"),
        Some("https://b.com/uninstall".to_string())
    );
}

#[test]
fn test_send_message() {
    let manager = RuntimeManager::new();
    manager.add_message_handler(
        "test-ext",
        Box::new(|_ext_id, msg, _sender| {
            if msg == json!({"type": "ping"}) {
                Some(json!({"type": "pong"}))
            } else {
                None
            }
        }),
    );

    let sender = MessageSender {
        id: Some("test-ext".to_string()),
        url: None,
        tab: None,
        frame_id: None,
    };

    let response = manager.send_message("test-ext", json!({"type": "ping"}), sender.clone());
    assert_eq!(response, Some(json!({"type": "pong"})));

    let response = manager.send_message("test-ext", json!({"type": "other"}), sender);
    assert_eq!(response, None);
}

#[test]
fn test_send_message_to_unknown_extension() {
    let manager = RuntimeManager::new();
    let sender = MessageSender {
        id: Some("other".to_string()),
        url: None,
        tab: None,
        frame_id: None,
    };
    let response = manager.send_message("unknown-ext", json!("hello"), sender);
    assert_eq!(response, None);
}

#[test]
fn test_open_options_page_callback() {
    let called = Arc::new(AtomicBool::new(false));
    let called_clone = called.clone();
    let manager = RuntimeManager::new();
    manager.set_on_open_options_page(move |ext_id, page| {
        assert_eq!(ext_id, "test-ext");
        assert!(page.contains("options.html"));
        called_clone.store(true, Ordering::SeqCst);
    });
    manager.open_options_page("test-ext", "auroraview-extension://test-ext/options.html");
    assert!(called.load(Ordering::SeqCst));
}

#[test]
fn test_reload_callback() {
    let called = Arc::new(AtomicBool::new(false));
    let called_clone = called.clone();
    let manager = RuntimeManager::new();
    manager.set_on_reload(move |ext_id| {
        assert_eq!(ext_id, "test-ext");
        called_clone.store(true, Ordering::SeqCst);
    });
    manager.reload_extension("test-ext");
    assert!(called.load(Ordering::SeqCst));
}

#[test]
fn test_open_options_page_no_callback() {
    let manager = RuntimeManager::new();
    // Should not panic when no callback is set
    manager.open_options_page("test-ext", "options.html");
}

#[test]
fn test_reload_no_callback() {
    let manager = RuntimeManager::new();
    // Should not panic when no callback is set
    manager.reload_extension("test-ext");
}

// --- API Handler tests ---

#[test]
fn test_handler_get_url() {
    let manager = Arc::new(RuntimeManager::new());
    let handler = RuntimeApiHandler::new(manager);

    let result = handler.handle("getURL", json!({"path": "popup.html"}), "test-ext");
    assert!(result.is_ok());
    let url = result.unwrap();
    assert_eq!(url, json!("auroraview-extension://test-ext/popup.html"));
}

#[test]
fn test_handler_get_platform_info() {
    let manager = Arc::new(RuntimeManager::new());
    let handler = RuntimeApiHandler::new(manager);

    let result = handler.handle("getPlatformInfo", json!({}), "test-ext");
    assert!(result.is_ok());
    let info = result.unwrap();
    // Should have os and arch fields
    assert!(info.get("os").is_some());
    assert!(info.get("arch").is_some());
}

#[test]
fn test_handler_set_uninstall_url() {
    let manager = Arc::new(RuntimeManager::new());
    let handler = RuntimeApiHandler::new(manager.clone());

    let result = handler.handle(
        "setUninstallURL",
        json!({"url": "https://example.com/bye"}),
        "test-ext",
    );
    assert!(result.is_ok());
    assert_eq!(
        manager.get_uninstall_url("test-ext"),
        Some("https://example.com/bye".to_string())
    );
}

#[test]
fn test_handler_open_options_page() {
    let called = Arc::new(AtomicBool::new(false));
    let called_clone = called.clone();
    let manager = Arc::new(RuntimeManager::new());
    manager.set_on_open_options_page(move |_ext_id, _page| {
        called_clone.store(true, Ordering::SeqCst);
    });
    let handler = RuntimeApiHandler::new(manager);

    let result = handler.handle("openOptionsPage", json!({}), "test-ext");
    assert!(result.is_ok());
    assert!(called.load(Ordering::SeqCst));
}

#[test]
fn test_handler_reload() {
    let called = Arc::new(AtomicBool::new(false));
    let called_clone = called.clone();
    let manager = Arc::new(RuntimeManager::new());
    manager.set_on_reload(move |_ext_id| {
        called_clone.store(true, Ordering::SeqCst);
    });
    let handler = RuntimeApiHandler::new(manager);

    let result = handler.handle("reload", json!({}), "test-ext");
    assert!(result.is_ok());
    assert!(called.load(Ordering::SeqCst));
}

#[test]
fn test_handler_send_message() {
    let manager = Arc::new(RuntimeManager::new());
    manager.add_message_handler(
        "test-ext",
        Box::new(|_ext_id, _msg, _sender| Some(json!({"reply": true}))),
    );
    let handler = RuntimeApiHandler::new(manager);

    let result = handler.handle(
        "sendMessage",
        json!({"message": {"type": "hello"}}),
        "test-ext",
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), json!({"reply": true}));
}

#[test]
fn test_handler_get_background_page() {
    let manager = Arc::new(RuntimeManager::new());
    let handler = RuntimeApiHandler::new(manager);
    let result = handler.handle("getBackgroundPage", json!({}), "test-ext");
    assert!(result.is_ok());
    assert!(result.unwrap().is_null());
}

#[test]
fn test_handler_request_update_check() {
    let manager = Arc::new(RuntimeManager::new());
    let handler = RuntimeApiHandler::new(manager);
    let result = handler.handle("requestUpdateCheck", json!({}), "test-ext");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), json!({"status": "no_update"}));
}

#[test]
fn test_handler_unsupported_method() {
    let manager = Arc::new(RuntimeManager::new());
    let handler = RuntimeApiHandler::new(manager);
    let result = handler.handle("nonExistent", json!({}), "test-ext");
    assert!(result.is_err());
}

#[test]
fn test_handler_namespace() {
    let manager = Arc::new(RuntimeManager::new());
    let handler = RuntimeApiHandler::new(manager);
    assert_eq!(handler.namespace(), "runtime");
}

#[test]
fn test_handler_methods_list() {
    let manager = Arc::new(RuntimeManager::new());
    let handler = RuntimeApiHandler::new(manager);
    let methods = handler.methods();
    assert!(methods.contains(&"sendMessage"));
    assert!(methods.contains(&"getURL"));
    assert!(methods.contains(&"openOptionsPage"));
    assert!(methods.contains(&"setUninstallURL"));
    assert!(methods.contains(&"reload"));
}
