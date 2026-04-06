//! Tests for the ExtensionsPlugin callback system.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

use parking_lot::RwLock;
use rstest::*;
use serde_json::{json, Value};

use auroraview_plugins::extensions::*;
use auroraview_plugins::PluginHandler;

// ============================================================
// Fixtures
// ============================================================

#[fixture]
fn plugin() -> ExtensionsPlugin {
    let p = ExtensionsPlugin::new();
    p.register_extension(ExtensionInfo {
        id: "test-ext".to_string(),
        name: "Test Extension".to_string(),
        version: "1.0.0".to_string(),
        description: "Test".to_string(),
        enabled: true,
        side_panel_path: None,
        popup_path: Some("popup.html".to_string()),
        options_page: Some("options.html".to_string()),
        root_dir: "/tmp/test-ext".to_string(),
        permissions: vec!["storage".to_string(), "tabs".to_string()],
        host_permissions: vec![],
        manifest: Some(json!({
            "manifest_version": 3,
            "name": "Test Extension",
            "version": "1.0.0"
        })),
    });
    p
}

fn make_api_call(api: &str, method: &str, params: Value) -> Value {
    json!({
        "extensionId": "test-ext",
        "api": api,
        "method": method,
        "params": params
    })
}

// ============================================================
// Storage callback tests
// ============================================================

#[rstest]
fn test_storage_set_triggers_persist_callback(plugin: ExtensionsPlugin) {
    let persisted = Arc::new(AtomicBool::new(false));
    let persisted_clone = persisted.clone();

    plugin.set_on_storage_persist(move |_ext_id, _key, _data| {
        persisted_clone.store(true, Ordering::SeqCst);
    });

    let args = make_api_call(
        "storage",
        "set",
        json!({ "area": "local", "items": { "key1": "value1" } }),
    );

    let result = plugin.handle("api_call", args, &Default::default());
    assert!(result.is_ok());
    assert!(persisted.load(Ordering::SeqCst));
}

#[rstest]
fn test_storage_get_returns_set_values(plugin: ExtensionsPlugin) {
    // Set a value
    let set_args = make_api_call(
        "storage",
        "set",
        json!({ "area": "local", "items": { "foo": "bar" } }),
    );
    plugin
        .handle("api_call", set_args, &Default::default())
        .unwrap();

    // Get it back
    let get_args = make_api_call(
        "storage",
        "get",
        json!({ "area": "local", "keys": ["foo"] }),
    );
    let result = plugin
        .handle("api_call", get_args, &Default::default())
        .unwrap();
    assert_eq!(result["foo"], "bar");
}

// ============================================================
// Tabs callback tests
// ============================================================

#[rstest]
fn test_tabs_create_triggers_navigate_callback(plugin: ExtensionsPlugin) {
    let navigated_url = Arc::new(RwLock::new(String::new()));
    let url_clone = navigated_url.clone();

    plugin.set_on_navigate(move |url| {
        *url_clone.write() = url.to_string();
    });

    let args = make_api_call("tabs", "create", json!({ "url": "https://example.com" }));
    let result = plugin.handle("api_call", args, &Default::default());
    assert!(result.is_ok());
    assert_eq!(*navigated_url.read(), "https://example.com");
}

#[rstest]
fn test_tabs_reload_triggers_reload_callback(plugin: ExtensionsPlugin) {
    let reloaded = Arc::new(AtomicBool::new(false));
    let reloaded_clone = reloaded.clone();

    plugin.set_on_reload_page(move || {
        reloaded_clone.store(true, Ordering::SeqCst);
    });

    let args = make_api_call("tabs", "reload", json!({}));
    let result = plugin.handle("api_call", args, &Default::default());
    assert!(result.is_ok());
    assert!(reloaded.load(Ordering::SeqCst));
}

#[rstest]
fn test_tabs_send_message_triggers_callback(plugin: ExtensionsPlugin) {
    plugin.set_on_send_message(move |tab_id, msg| {
        assert_eq!(tab_id, 1);
        Some(json!({ "response": "ok", "received": msg }))
    });

    let args = make_api_call(
        "tabs",
        "sendMessage",
        json!({ "tabId": 1, "message": "hello" }),
    );
    let result = plugin
        .handle("api_call", args, &Default::default())
        .unwrap();
    assert_eq!(result["response"], "ok");
    assert_eq!(result["received"], "hello");
}

#[rstest]
fn test_tabs_send_message_without_callback(plugin: ExtensionsPlugin) {
    let args = make_api_call(
        "tabs",
        "sendMessage",
        json!({ "tabId": 1, "message": "hello" }),
    );
    let result = plugin
        .handle("api_call", args, &Default::default())
        .unwrap();
    // Without callback, returns the message itself
    assert_eq!(result, "hello");
}

#[rstest]
fn test_tabs_capture_visible_tab(plugin: ExtensionsPlugin) {
    let args = make_api_call("tabs", "captureVisibleTab", json!({}));
    let result = plugin
        .handle("api_call", args, &Default::default())
        .unwrap();
    assert_eq!(result, json!(""));
}

// ============================================================
// Runtime callback tests
// ============================================================

#[rstest]
fn test_runtime_send_message_triggers_callback(plugin: ExtensionsPlugin) {
    let received = Arc::new(AtomicBool::new(false));
    let received_clone = received.clone();

    plugin.set_on_runtime_message(move |ext_id, _msg| {
        assert_eq!(ext_id, "test-ext");
        received_clone.store(true, Ordering::SeqCst);
        Some(json!("ack"))
    });

    let args = make_api_call("runtime", "sendMessage", json!({ "message": "test" }));
    let result = plugin
        .handle("api_call", args, &Default::default())
        .unwrap();
    assert!(received.load(Ordering::SeqCst));
    assert_eq!(result, "ack");
}

#[rstest]
fn test_runtime_open_options_page(plugin: ExtensionsPlugin) {
    let opened = Arc::new(RwLock::new(String::new()));
    let opened_clone = opened.clone();

    plugin.set_on_open_options_page(move |ext_id, page| {
        *opened_clone.write() = format!("{}:{}", ext_id, page);
    });

    let args = make_api_call("runtime", "openOptionsPage", json!({}));
    let result = plugin.handle("api_call", args, &Default::default());
    assert!(result.is_ok());
    assert_eq!(*opened.read(), "test-ext:options.html");
}

#[rstest]
fn test_runtime_reload(plugin: ExtensionsPlugin) {
    let reloaded = Arc::new(RwLock::new(String::new()));
    let reloaded_clone = reloaded.clone();

    plugin.set_on_reload_extension(move |ext_id| {
        *reloaded_clone.write() = ext_id.to_string();
    });

    let args = make_api_call("runtime", "reload", json!({}));
    let result = plugin.handle("api_call", args, &Default::default());
    assert!(result.is_ok());
    assert_eq!(*reloaded.read(), "test-ext");
}

#[rstest]
fn test_runtime_connect(plugin: ExtensionsPlugin) {
    let args = make_api_call(
        "runtime",
        "connect",
        json!({ "portId": "port1", "name": "myport" }),
    );
    let result = plugin
        .handle("api_call", args, &Default::default())
        .unwrap();
    assert_eq!(result["portId"], "port1");
    assert_eq!(result["name"], "myport");
}

#[rstest]
fn test_runtime_port_post_message(plugin: ExtensionsPlugin) {
    let args = make_api_call("runtime", "portPostMessage", json!({}));
    let result = plugin.handle("api_call", args, &Default::default());
    assert!(result.is_ok());
}

// ============================================================
// Action callback tests
// ============================================================

#[rstest]
fn test_action_open_popup_triggers_callback(plugin: ExtensionsPlugin) {
    // First set a popup path via action.setPopup
    let set_popup_args = make_api_call("action", "setPopup", json!({ "popup": "popup.html" }));
    plugin
        .handle("api_call", set_popup_args, &Default::default())
        .unwrap();

    let popup_opened = Arc::new(RwLock::new(String::new()));
    let popup_clone = popup_opened.clone();

    plugin.set_on_open_popup(move |ext_id, popup_path| {
        *popup_clone.write() = format!("{}:{:?}", ext_id, popup_path);
    });

    let args = make_api_call("action", "openPopup", json!({}));
    let result = plugin.handle("api_call", args, &Default::default());
    assert!(result.is_ok());
    let opened = popup_opened.read();
    assert!(opened.contains("test-ext"));
    assert!(opened.contains("popup.html"));
}

#[rstest]
fn test_action_set_and_get_title(plugin: ExtensionsPlugin) {
    let set_args = make_api_call("action", "setTitle", json!({ "title": "My Title" }));
    plugin
        .handle("api_call", set_args, &Default::default())
        .unwrap();

    let get_args = make_api_call("action", "getTitle", json!({}));
    let result = plugin
        .handle("api_call", get_args, &Default::default())
        .unwrap();
    assert_eq!(result, "My Title");
}

// ============================================================
// Scripting callback tests
// ============================================================

#[rstest]
fn test_scripting_execute_script_triggers_callback(plugin: ExtensionsPlugin) {
    let call_count = Arc::new(AtomicU32::new(0));
    let count_clone = call_count.clone();

    plugin.set_on_execute_script(move |_ext_id, _params| {
        count_clone.fetch_add(1, Ordering::SeqCst);
        vec![json!(42)]
    });

    let args = make_api_call("scripting", "executeScript", json!({ "func": "() => 42" }));
    let result = plugin
        .handle("api_call", args, &Default::default())
        .unwrap();
    assert_eq!(call_count.load(Ordering::SeqCst), 1);
    assert_eq!(result[0]["result"], 42);
}

#[rstest]
fn test_scripting_execute_script_without_callback(plugin: ExtensionsPlugin) {
    let args = make_api_call(
        "scripting",
        "executeScript",
        json!({ "func": "() => null" }),
    );
    let result = plugin
        .handle("api_call", args, &Default::default())
        .unwrap();
    assert_eq!(result[0]["frameId"], 0);
}

#[rstest]
fn test_scripting_insert_css_triggers_callback(plugin: ExtensionsPlugin) {
    let inserted = Arc::new(AtomicBool::new(false));
    let inserted_clone = inserted.clone();

    plugin.set_on_insert_css(move |_ext_id, _params| {
        inserted_clone.store(true, Ordering::SeqCst);
    });

    let args = make_api_call(
        "scripting",
        "insertCSS",
        json!({ "css": "body { color: red; }" }),
    );
    let result = plugin.handle("api_call", args, &Default::default());
    assert!(result.is_ok());
    assert!(inserted.load(Ordering::SeqCst));
}

#[rstest]
fn test_scripting_remove_css_triggers_callback(plugin: ExtensionsPlugin) {
    let removed = Arc::new(AtomicBool::new(false));
    let removed_clone = removed.clone();

    plugin.set_on_remove_css(move |_ext_id, _params| {
        removed_clone.store(true, Ordering::SeqCst);
    });

    let args = make_api_call(
        "scripting",
        "removeCSS",
        json!({ "css": "body { color: red; }" }),
    );
    let result = plugin.handle("api_call", args, &Default::default());
    assert!(result.is_ok());
    assert!(removed.load(Ordering::SeqCst));
}

#[rstest]
fn test_scripting_update_content_scripts(plugin: ExtensionsPlugin) {
    // First register a script
    let register_args = make_api_call(
        "scripting",
        "registerContentScripts",
        json!({
            "scripts": [{
                "id": "script1",
                "matches": ["*://*.example.com/*"],
                "js": ["content.js"],
                "css": [],
                "runAt": "document_idle",
                "allFrames": false
            }]
        }),
    );
    plugin
        .handle("api_call", register_args, &Default::default())
        .unwrap();

    // Update it
    let update_args = make_api_call(
        "scripting",
        "updateContentScripts",
        json!({
            "scripts": [{
                "id": "script1",
                "matches": ["*://*.updated.com/*"],
                "js": ["updated.js"],
                "css": [],
                "runAt": "document_start",
                "allFrames": true
            }]
        }),
    );
    let result = plugin.handle("api_call", update_args, &Default::default());
    assert!(result.is_ok());

    // Verify update
    let get_args = make_api_call("scripting", "getRegisteredContentScripts", json!({}));
    let scripts = plugin
        .handle("api_call", get_args, &Default::default())
        .unwrap();
    assert_eq!(scripts[0]["matches"][0], "*://*.updated.com/*");
    assert_eq!(scripts[0]["js"][0], "updated.js");
}

// ============================================================
// Notifications callback tests
// ============================================================

#[rstest]
fn test_notifications_create_triggers_callback(plugin: ExtensionsPlugin) {
    let notified = Arc::new(RwLock::new(String::new()));
    let notified_clone = notified.clone();

    plugin.set_on_notification(move |info| {
        *notified_clone.write() = info.title.clone();
    });

    let args = make_api_call(
        "notifications",
        "create",
        json!({
            "notificationId": "notif1",
            "options": {
                "title": "Test Notification",
                "message": "Hello!",
                "type": "basic"
            }
        }),
    );
    let result = plugin
        .handle("api_call", args, &Default::default())
        .unwrap();
    assert_eq!(result, "notif1");
    assert_eq!(*notified.read(), "Test Notification");
}

// ============================================================
// Windows callback tests
// ============================================================

#[rstest]
fn test_windows_create_triggers_callback(plugin: ExtensionsPlugin) {
    plugin.set_on_create_window(move |params| {
        json!({
            "id": 2,
            "focused": true,
            "type": "normal",
            "state": "normal",
            "url": params.get("url").cloned()
        })
    });

    let args = make_api_call("windows", "create", json!({ "url": "https://example.com" }));
    let result = plugin
        .handle("api_call", args, &Default::default())
        .unwrap();
    assert_eq!(result["id"], 2);
}

#[rstest]
fn test_windows_create_without_callback(plugin: ExtensionsPlugin) {
    let args = make_api_call("windows", "create", json!({}));
    let result = plugin
        .handle("api_call", args, &Default::default())
        .unwrap();
    // Returns default window when no callback
    assert_eq!(result["id"], 1);
}

// ============================================================
// Event dispatch callback tests
// ============================================================

#[rstest]
fn test_event_dispatch_triggers_callback(plugin: ExtensionsPlugin) {
    let dispatched = Arc::new(RwLock::new(String::new()));
    let dispatched_clone = dispatched.clone();

    plugin.set_on_event_dispatch(move |ext_id, api, event, _args| {
        *dispatched_clone.write() = format!("{}.{} -> {}", api, event, ext_id);
    });

    let args = json!({
        "extensionId": "test-ext",
        "api": "tabs",
        "event": "onUpdated",
        "args": [1, {"status": "complete"}, {"url": "https://example.com"}]
    });
    let result = plugin.handle("dispatch_event", args, &Default::default());
    assert!(result.is_ok());
    assert_eq!(*dispatched.read(), "tabs.onUpdated -> test-ext");
}

// ============================================================
// Identity API tests
// ============================================================

#[rstest]
fn test_identity_get_auth_token_returns_error(plugin: ExtensionsPlugin) {
    let args = make_api_call("identity", "getAuthToken", json!({}));
    let result = plugin.handle("api_call", args, &Default::default());
    assert!(result.is_err());
}

#[rstest]
fn test_identity_get_redirect_url(plugin: ExtensionsPlugin) {
    let args = make_api_call("identity", "getRedirectURL", json!({ "path": "callback" }));
    let result = plugin
        .handle("api_call", args, &Default::default())
        .unwrap();
    let url = result.as_str().unwrap();
    assert!(url.contains("test-ext"));
    assert!(url.contains("callback"));
}

// ============================================================
// WebRequest API tests
// ============================================================

#[rstest]
fn test_web_request_add_listener(plugin: ExtensionsPlugin) {
    let args = make_api_call("webRequest", "addListener", json!({}));
    let result = plugin.handle("api_call", args, &Default::default());
    assert!(result.is_ok());
}

// ============================================================
// Callback registration tests
// ============================================================

#[rstest]
fn test_callbacks_ref_is_shared(plugin: ExtensionsPlugin) {
    let cbs = plugin.callbacks();
    assert!(cbs.read().on_navigate.is_none());

    plugin.set_on_navigate(|_url| {});
    assert!(cbs.read().on_navigate.is_some());
}

#[rstest]
fn test_state_ref_is_shared(plugin: ExtensionsPlugin) {
    let state = plugin.state();
    assert!(state.read().extensions.contains_key("test-ext"));
}

// ============================================================
// Polyfill and list commands
// ============================================================

#[rstest]
fn test_list_extensions(plugin: ExtensionsPlugin) {
    let result = plugin
        .handle("list_extensions", json!({}), &Default::default())
        .unwrap();
    let list = result.as_array().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["id"], "test-ext");
}

#[rstest]
fn test_get_extension(plugin: ExtensionsPlugin) {
    let result = plugin
        .handle(
            "get_extension",
            json!({ "extensionId": "test-ext" }),
            &Default::default(),
        )
        .unwrap();
    assert_eq!(result["id"], "test-ext");
    assert_eq!(result["name"], "Test Extension");
}

#[rstest]
fn test_get_extension_not_found(plugin: ExtensionsPlugin) {
    let result = plugin.handle(
        "get_extension",
        json!({ "extensionId": "nonexistent" }),
        &Default::default(),
    );
    assert!(result.is_err());
}
