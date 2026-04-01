//! Tests for chrome.scripting API

use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

use serde_json::json;

use auroraview_extensions::apis::scripting::{
    CssInjection, InjectionResult, InjectionTarget, RegisteredContentScript, ScriptInjection,
    ScriptingApiHandler, ScriptingManager,
};
use auroraview_extensions::apis::ApiHandler;

#[test]
fn test_scripting_manager_new() {
    let manager = ScriptingManager::new();
    let scripts = manager.get_registered_content_scripts("test-ext", None);
    assert!(scripts.is_empty());
}

#[test]
fn test_scripting_manager_default() {
    let manager = ScriptingManager::default();
    let scripts = manager.get_registered_content_scripts("test-ext", None);
    assert!(scripts.is_empty());
}

#[test]
fn test_register_content_scripts() {
    let manager = ScriptingManager::new();
    manager.register_content_scripts(
        "test-ext",
        vec![RegisteredContentScript {
            id: "script1".to_string(),
            matches: vec!["*://*.example.com/*".to_string()],
            exclude_matches: vec![],
            js: vec!["content.js".to_string()],
            css: vec![],
            all_frames: false,
            run_at: Some("document_idle".to_string()),
            world: None,
            persist_across_sessions: true,
        }],
    );

    let scripts = manager.get_registered_content_scripts("test-ext", None);
    assert_eq!(scripts.len(), 1);
    assert_eq!(scripts[0].id, "script1");
}

#[test]
fn test_unregister_content_scripts_by_id() {
    let manager = ScriptingManager::new();
    manager.register_content_scripts(
        "test-ext",
        vec![
            RegisteredContentScript {
                id: "s1".to_string(),
                matches: vec![],
                exclude_matches: vec![],
                js: vec![],
                css: vec![],
                all_frames: false,
                run_at: None,
                world: None,
                persist_across_sessions: false,
            },
            RegisteredContentScript {
                id: "s2".to_string(),
                matches: vec![],
                exclude_matches: vec![],
                js: vec![],
                css: vec![],
                all_frames: false,
                run_at: None,
                world: None,
                persist_across_sessions: false,
            },
        ],
    );

    manager.unregister_content_scripts("test-ext", Some(vec!["s1".to_string()]));
    let scripts = manager.get_registered_content_scripts("test-ext", None);
    assert_eq!(scripts.len(), 1);
    assert_eq!(scripts[0].id, "s2");
}

#[test]
fn test_unregister_all_content_scripts() {
    let manager = ScriptingManager::new();
    manager.register_content_scripts(
        "test-ext",
        vec![RegisteredContentScript {
            id: "s1".to_string(),
            matches: vec![],
            exclude_matches: vec![],
            js: vec![],
            css: vec![],
            all_frames: false,
            run_at: None,
            world: None,
            persist_across_sessions: false,
        }],
    );

    manager.unregister_content_scripts("test-ext", None);
    let scripts = manager.get_registered_content_scripts("test-ext", None);
    assert!(scripts.is_empty());
}

#[test]
fn test_get_registered_content_scripts_by_ids() {
    let manager = ScriptingManager::new();
    manager.register_content_scripts(
        "test-ext",
        vec![
            RegisteredContentScript {
                id: "s1".to_string(),
                matches: vec![],
                exclude_matches: vec![],
                js: vec![],
                css: vec![],
                all_frames: false,
                run_at: None,
                world: None,
                persist_across_sessions: false,
            },
            RegisteredContentScript {
                id: "s2".to_string(),
                matches: vec![],
                exclude_matches: vec![],
                js: vec![],
                css: vec![],
                all_frames: false,
                run_at: None,
                world: None,
                persist_across_sessions: false,
            },
        ],
    );

    let scripts =
        manager.get_registered_content_scripts("test-ext", Some(vec!["s2".to_string()]));
    assert_eq!(scripts.len(), 1);
    assert_eq!(scripts[0].id, "s2");
}

#[test]
fn test_execute_script_with_callback() {
    let manager = ScriptingManager::new();
    manager.set_on_execute(|ext_id, injection| {
        vec![InjectionResult {
            frame_id: injection.target.tab_id,
            result: Some(json!({"ext": ext_id})),
            error: None,
        }]
    });

    let results = manager.execute_script(
        "test-ext",
        &ScriptInjection {
            target: InjectionTarget {
                tab_id: 1,
                frame_ids: None,
                all_frames: false,
            },
            files: None,
            func: Some("() => 42".to_string()),
            args: None,
            world: None,
            inject_immediately: false,
        },
    );

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].frame_id, 1);
    assert!(results[0].error.is_none());
}

#[test]
fn test_execute_script_without_callback() {
    let manager = ScriptingManager::new();
    let results = manager.execute_script(
        "test-ext",
        &ScriptInjection {
            target: InjectionTarget {
                tab_id: 1,
                frame_ids: None,
                all_frames: false,
            },
            files: None,
            func: None,
            args: None,
            world: None,
            inject_immediately: false,
        },
    );
    assert_eq!(results.len(), 1);
    assert!(results[0].error.is_some());
}

#[test]
fn test_insert_css_with_callback() {
    let called = Arc::new(AtomicBool::new(false));
    let called_clone = called.clone();
    let manager = ScriptingManager::new();
    manager.set_on_insert_css(move |_ext_id, _injection| {
        called_clone.store(true, Ordering::SeqCst);
    });

    manager.insert_css(
        "test-ext",
        &CssInjection {
            target: InjectionTarget {
                tab_id: 1,
                frame_ids: None,
                all_frames: false,
            },
            files: None,
            css: Some("body { color: red; }".to_string()),
            origin: None,
        },
    );
    assert!(called.load(Ordering::SeqCst));
}

#[test]
fn test_remove_css_with_callback() {
    let called = Arc::new(AtomicBool::new(false));
    let called_clone = called.clone();
    let manager = ScriptingManager::new();
    manager.set_on_remove_css(move |_ext_id, _injection| {
        called_clone.store(true, Ordering::SeqCst);
    });

    manager.remove_css(
        "test-ext",
        &CssInjection {
            target: InjectionTarget {
                tab_id: 1,
                frame_ids: None,
                all_frames: false,
            },
            files: None,
            css: Some("body { color: red; }".to_string()),
            origin: None,
        },
    );
    assert!(called.load(Ordering::SeqCst));
}

#[test]
fn test_remove_css_without_callback() {
    let manager = ScriptingManager::new();
    // Should not panic
    manager.remove_css(
        "test-ext",
        &CssInjection {
            target: InjectionTarget {
                tab_id: 1,
                frame_ids: None,
                all_frames: false,
            },
            files: None,
            css: Some("body { color: red; }".to_string()),
            origin: None,
        },
    );
}

// --- API Handler tests ---

#[test]
fn test_handler_execute_script() {
    let manager = Arc::new(ScriptingManager::new());
    manager.set_on_execute(|_ext_id, _injection| {
        vec![InjectionResult {
            frame_id: 0,
            result: Some(json!(42)),
            error: None,
        }]
    });
    let handler = ScriptingApiHandler::new(manager);

    let result = handler.handle(
        "executeScript",
        json!({
            "target": {"tabId": 1},
            "func": "() => 42"
        }),
        "test-ext",
    );
    assert!(result.is_ok());
}

#[test]
fn test_handler_insert_css() {
    let call_count = Arc::new(AtomicUsize::new(0));
    let count_clone = call_count.clone();
    let manager = Arc::new(ScriptingManager::new());
    manager.set_on_insert_css(move |_ext_id, _injection| {
        count_clone.fetch_add(1, Ordering::SeqCst);
    });
    let handler = ScriptingApiHandler::new(manager);

    let result = handler.handle(
        "insertCSS",
        json!({
            "target": {"tabId": 1},
            "css": "body { background: blue; }"
        }),
        "test-ext",
    );
    assert!(result.is_ok());
    assert_eq!(call_count.load(Ordering::SeqCst), 1);
}

#[test]
fn test_handler_remove_css() {
    let call_count = Arc::new(AtomicUsize::new(0));
    let count_clone = call_count.clone();
    let manager = Arc::new(ScriptingManager::new());
    manager.set_on_remove_css(move |_ext_id, _injection| {
        count_clone.fetch_add(1, Ordering::SeqCst);
    });
    let handler = ScriptingApiHandler::new(manager);

    let result = handler.handle(
        "removeCSS",
        json!({
            "target": {"tabId": 1},
            "css": "body { background: blue; }"
        }),
        "test-ext",
    );
    assert!(result.is_ok());
    assert_eq!(call_count.load(Ordering::SeqCst), 1);
}

#[test]
fn test_handler_register_content_scripts() {
    let manager = Arc::new(ScriptingManager::new());
    let handler = ScriptingApiHandler::new(manager.clone());

    let result = handler.handle(
        "registerContentScripts",
        json!({
            "scripts": [{
                "id": "my-script",
                "matches": ["*://*.example.com/*"],
                "js": ["script.js"]
            }]
        }),
        "test-ext",
    );
    assert!(result.is_ok());
    let scripts = manager.get_registered_content_scripts("test-ext", None);
    assert_eq!(scripts.len(), 1);
}

#[test]
fn test_handler_unregister_content_scripts() {
    let manager = Arc::new(ScriptingManager::new());
    manager.register_content_scripts(
        "test-ext",
        vec![RegisteredContentScript {
            id: "s1".to_string(),
            matches: vec![],
            exclude_matches: vec![],
            js: vec![],
            css: vec![],
            all_frames: false,
            run_at: None,
            world: None,
            persist_across_sessions: false,
        }],
    );
    let handler = ScriptingApiHandler::new(manager.clone());

    let result = handler.handle(
        "unregisterContentScripts",
        json!({"ids": ["s1"]}),
        "test-ext",
    );
    assert!(result.is_ok());
    let scripts = manager.get_registered_content_scripts("test-ext", None);
    assert!(scripts.is_empty());
}

#[test]
fn test_handler_get_registered_content_scripts() {
    let manager = Arc::new(ScriptingManager::new());
    manager.register_content_scripts(
        "test-ext",
        vec![RegisteredContentScript {
            id: "s1".to_string(),
            matches: vec!["*://*.example.com/*".to_string()],
            exclude_matches: vec![],
            js: vec!["content.js".to_string()],
            css: vec![],
            all_frames: false,
            run_at: None,
            world: None,
            persist_across_sessions: true,
        }],
    );
    let handler = ScriptingApiHandler::new(manager);

    let result = handler.handle("getRegisteredContentScripts", json!({}), "test-ext");
    assert!(result.is_ok());
    let scripts: Vec<serde_json::Value> = serde_json::from_value(result.unwrap()).unwrap();
    assert_eq!(scripts.len(), 1);
}

#[test]
fn test_handler_unsupported_method() {
    let manager = Arc::new(ScriptingManager::new());
    let handler = ScriptingApiHandler::new(manager);
    let result = handler.handle("nonExistent", json!({}), "test-ext");
    assert!(result.is_err());
}

#[test]
fn test_handler_namespace() {
    let manager = Arc::new(ScriptingManager::new());
    let handler = ScriptingApiHandler::new(manager);
    assert_eq!(handler.namespace(), "scripting");
}

#[test]
fn test_handler_methods_list() {
    let manager = Arc::new(ScriptingManager::new());
    let handler = ScriptingApiHandler::new(manager);
    let methods = handler.methods();
    assert!(methods.contains(&"executeScript"));
    assert!(methods.contains(&"insertCSS"));
    assert!(methods.contains(&"removeCSS"));
    assert!(methods.contains(&"registerContentScripts"));
    assert!(methods.contains(&"unregisterContentScripts"));
    assert!(methods.contains(&"getRegisteredContentScripts"));
}
