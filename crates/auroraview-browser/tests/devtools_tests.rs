//! DevTools module tests

use std::collections::HashMap;

use auroraview_browser::devtools::cdp::{CdpError, CdpEvent, CdpRequest, CdpResponse};
use auroraview_browser::devtools::{DevToolsState, NetworkRequestInfo, NetworkResponseInfo};
use auroraview_browser::{
    ConsoleMessage, ConsoleMessageType, DevToolsConfig, DevToolsManager, DockSide,
};
use rstest::rstest;
use serde_json::json;

// ─── DevToolsConfig ───────────────────────────────────────────────────────────

#[rstest]
fn devtools_config_default_values() {
    let config = DevToolsConfig::default();
    assert!(config.enabled);
    assert_eq!(config.remote_debugging_port, 0);
    assert!(!config.auto_open);
    assert_eq!(config.dock_side, DockSide::Right);
}

#[rstest]
fn devtools_config_builder() {
    let config = DevToolsConfig {
        enabled: true,
        remote_debugging_port: 9222,
        auto_open: true,
        dock_side: DockSide::Bottom,
    };

    assert!(config.enabled);
    assert_eq!(config.remote_debugging_port, 9222);
    assert!(config.auto_open);
    assert_eq!(config.dock_side, DockSide::Bottom);
}

#[rstest]
fn devtools_config_disabled() {
    let config = DevToolsConfig {
        enabled: false,
        remote_debugging_port: 0,
        auto_open: false,
        dock_side: DockSide::Right,
    };
    assert!(!config.enabled);
}

#[rstest]
#[case(DockSide::Right)]
#[case(DockSide::Bottom)]
#[case(DockSide::Left)]
#[case(DockSide::Undocked)]
fn devtools_config_all_dock_sides(#[case] side: DockSide) {
    let config = DevToolsConfig {
        enabled: true,
        remote_debugging_port: 0,
        auto_open: false,
        dock_side: side,
    };
    assert_eq!(config.dock_side, side);
}

// ─── DevToolsManager lifecycle ────────────────────────────────────────────────

#[rstest]
fn devtools_manager_lifecycle() {
    let config = DevToolsConfig {
        enabled: true,
        remote_debugging_port: 0,
        auto_open: false,
        dock_side: DockSide::Right,
    };

    let mut manager = DevToolsManager::new(config);

    assert!(manager.is_enabled());
    assert!(!manager.is_open());

    manager.open();
    assert!(manager.is_open());

    manager.close();
    assert!(!manager.is_open());

    manager.toggle();
    assert!(manager.is_open());

    manager.toggle();
    assert!(!manager.is_open());
}

#[rstest]
fn devtools_manager_disabled_can_still_call() {
    let config = DevToolsConfig {
        enabled: false,
        remote_debugging_port: 0,
        auto_open: false,
        dock_side: DockSide::Right,
    };
    let mut manager = DevToolsManager::new(config);
    assert!(!manager.is_enabled());
    // open/close/toggle should not panic even when disabled
    manager.open();
    manager.close();
    manager.toggle();
}

#[rstest]
fn devtools_manager_remote_debugging_port() {
    let config = DevToolsConfig {
        enabled: true,
        remote_debugging_port: 9229,
        auto_open: false,
        dock_side: DockSide::Right,
    };
    let manager = DevToolsManager::new(config);
    assert_eq!(manager.remote_debugging_port(), 9229);
}

#[rstest]
fn devtools_manager_remote_debugging_port_zero() {
    let manager = DevToolsManager::new(DevToolsConfig::default());
    assert_eq!(manager.remote_debugging_port(), 0);
}

// ─── DockSide ─────────────────────────────────────────────────────────────────

#[rstest]
fn devtools_dock_side_change() {
    let config = DevToolsConfig::default();
    let mut manager = DevToolsManager::new(config);

    manager.set_dock_side(DockSide::Bottom);
    assert_eq!(manager.state().dock_side, Some(DockSide::Bottom));

    manager.set_dock_side(DockSide::Left);
    assert_eq!(manager.state().dock_side, Some(DockSide::Left));

    manager.set_dock_side(DockSide::Undocked);
    assert_eq!(manager.state().dock_side, Some(DockSide::Undocked));
}

#[rstest]
#[case(DockSide::Right, "\"right\"")]
#[case(DockSide::Bottom, "\"bottom\"")]
#[case(DockSide::Left, "\"left\"")]
#[case(DockSide::Undocked, "\"undocked\"")]
fn dock_side_serialization_roundtrip(#[case] side: DockSide, #[case] expected_json: &str) {
    let json_str = serde_json::to_string(&side).unwrap();
    assert_eq!(json_str, expected_json);

    let parsed: DockSide = serde_json::from_str(expected_json).unwrap();
    assert_eq!(parsed, side);
}

#[rstest]
fn dock_side_clone_equality() {
    let a = DockSide::Bottom;
    let b = a;
    assert_eq!(a, b);
}

// ─── DevToolsState ────────────────────────────────────────────────────────────

#[rstest]
fn devtools_state_default() {
    let state = DevToolsState::default();
    assert!(!state.is_open);
    assert!(state.dock_side.is_none());
    assert!(state.selected_panel.is_none());
}

#[rstest]
fn devtools_state_selected_panel() {
    let manager = DevToolsManager::new(DevToolsConfig::default());
    // Initially no panel selected
    assert!(manager.state().selected_panel.is_none());
}

#[rstest]
fn devtools_state_open_reflected() {
    let mut manager = DevToolsManager::new(DevToolsConfig::default());
    assert!(!manager.state().is_open);
    manager.open();
    assert!(manager.state().is_open);
    manager.close();
    assert!(!manager.state().is_open);
}

// ─── Console messages ─────────────────────────────────────────────────────────

#[rstest]
fn console_message_types_all() {
    let mut manager = DevToolsManager::new(DevToolsConfig::default());

    let types = [
        ConsoleMessageType::Log,
        ConsoleMessageType::Debug,
        ConsoleMessageType::Info,
        ConsoleMessageType::Warning,
        ConsoleMessageType::Error,
    ];

    for msg_type in &types {
        manager.add_console_message(ConsoleMessage {
            message_type: *msg_type,
            text: format!("Test {:?} message", msg_type),
            source: Some("test.js".to_string()),
            line: Some(10),
            column: Some(5),
            stack_trace: None,
            timestamp: 0,
        });
    }

    assert_eq!(manager.console_messages().len(), 5);
}

#[rstest]
fn console_messages_limit() {
    let mut manager = DevToolsManager::new(DevToolsConfig::default());

    for i in 0..1100 {
        manager.add_console_message(ConsoleMessage {
            message_type: ConsoleMessageType::Log,
            text: format!("Message {}", i),
            source: None,
            line: None,
            column: None,
            stack_trace: None,
            timestamp: i as i64,
        });
    }

    assert!(manager.console_messages().len() <= 1001);
}

#[rstest]
fn console_clear_empties_all() {
    let mut manager = DevToolsManager::new(DevToolsConfig::default());

    for i in 0..5 {
        manager.add_console_message(ConsoleMessage {
            message_type: ConsoleMessageType::Log,
            text: format!("msg {}", i),
            source: None,
            line: None,
            column: None,
            stack_trace: None,
            timestamp: 0,
        });
    }
    assert_eq!(manager.console_messages().len(), 5);
    manager.clear_console();
    assert!(manager.console_messages().is_empty());
}

#[rstest]
fn console_message_serde_roundtrip() {
    let msg = ConsoleMessage {
        message_type: ConsoleMessageType::Error,
        text: "Uncaught TypeError".to_string(),
        source: Some("app.js".to_string()),
        line: Some(42),
        column: Some(8),
        stack_trace: Some("at foo (app.js:42:8)".to_string()),
        timestamp: 1_700_000_000,
    };
    let json_str = serde_json::to_string(&msg).unwrap();
    let parsed: ConsoleMessage = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed.text, "Uncaught TypeError");
    assert_eq!(parsed.line, Some(42));
    assert_eq!(parsed.stack_trace, Some("at foo (app.js:42:8)".to_string()));
}

#[rstest]
#[case(ConsoleMessageType::Log)]
#[case(ConsoleMessageType::Debug)]
#[case(ConsoleMessageType::Info)]
#[case(ConsoleMessageType::Warning)]
#[case(ConsoleMessageType::Error)]
fn console_message_type_serde(#[case] msg_type: ConsoleMessageType) {
    let json_str = serde_json::to_string(&msg_type).unwrap();
    let parsed: ConsoleMessageType = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed, msg_type);
}

// ─── Network requests ─────────────────────────────────────────────────────────

#[rstest]
fn network_requests_tracking() {
    let mut manager = DevToolsManager::new(DevToolsConfig::default());

    let request = NetworkRequestInfo {
        request_id: "req1".to_string(),
        url: "https://example.com/api".to_string(),
        method: "GET".to_string(),
        headers: HashMap::new(),
        post_data: None,
        resource_type: "fetch".to_string(),
        timestamp: 1_234_567_890.0,
    };

    manager.add_network_request(request);

    assert_eq!(manager.network_requests().len(), 1);
    assert!(manager.network_requests().contains_key("req1"));

    manager.clear_network();
    assert!(manager.network_requests().is_empty());
}

#[rstest]
fn network_multiple_requests() {
    let mut manager = DevToolsManager::new(DevToolsConfig::default());

    for i in 0..5 {
        manager.add_network_request(NetworkRequestInfo {
            request_id: format!("req-{}", i),
            url: format!("https://example.com/{}", i),
            method: "GET".to_string(),
            headers: HashMap::new(),
            post_data: None,
            resource_type: "document".to_string(),
            timestamp: i as f64 * 1000.0,
        });
    }

    assert_eq!(manager.network_requests().len(), 5);
}

#[rstest]
fn network_request_serde_roundtrip() {
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    let req = NetworkRequestInfo {
        request_id: "test-req".to_string(),
        url: "https://api.example.com/data".to_string(),
        method: "POST".to_string(),
        headers,
        post_data: Some("{\"key\":\"value\"}".to_string()),
        resource_type: "fetch".to_string(),
        timestamp: 9_999_999.5,
    };

    let json_str = serde_json::to_string(&req).unwrap();
    let parsed: NetworkRequestInfo = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed.request_id, "test-req");
    assert_eq!(parsed.method, "POST");
    assert!(parsed.post_data.is_some());
}

#[rstest]
fn network_response_serde_roundtrip() {
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    let resp = NetworkResponseInfo {
        request_id: "resp-1".to_string(),
        status: 200,
        status_text: "OK".to_string(),
        headers,
        mime_type: "application/json".to_string(),
        content_length: Some(1024),
        from_cache: false,
        timestamp: 1_000_000.0,
    };

    let json_str = serde_json::to_string(&resp).unwrap();
    let parsed: NetworkResponseInfo = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed.status, 200);
    assert_eq!(parsed.status_text, "OK");
    assert_eq!(parsed.content_length, Some(1024));
    assert!(!parsed.from_cache);
}

#[rstest]
fn network_response_from_cache() {
    let resp = NetworkResponseInfo {
        request_id: "cached".to_string(),
        status: 304,
        status_text: "Not Modified".to_string(),
        headers: HashMap::new(),
        mime_type: "text/html".to_string(),
        content_length: None,
        from_cache: true,
        timestamp: 500.0,
    };
    let json_str = serde_json::to_string(&resp).unwrap();
    let parsed: NetworkResponseInfo = serde_json::from_str(&json_str).unwrap();
    assert!(parsed.from_cache);
    assert_eq!(parsed.status, 304);
    assert!(parsed.content_length.is_none());
}

// ─── CDP request/response ─────────────────────────────────────────────────────

#[rstest]
fn cdp_request_creation() {
    let req = CdpRequest::new(1, "Page.navigate", json!({"url": "https://example.com"}));
    assert_eq!(req.id, 1);
    assert_eq!(req.method, "Page.navigate");
    assert_eq!(req.params["url"], "https://example.com");
}

#[rstest]
fn cdp_request_simple() {
    let req = CdpRequest::simple(42, "Browser.getVersion");
    assert_eq!(req.id, 42);
    assert_eq!(req.method, "Browser.getVersion");
    assert!(req.params.is_null());
}

#[rstest]
fn cdp_request_serde_roundtrip() {
    let req = CdpRequest::new(100, "Runtime.evaluate", json!({"expression": "1+1"}));
    let json_str = serde_json::to_string(&req).unwrap();
    let parsed: CdpRequest = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed.id, 100);
    assert_eq!(parsed.method, "Runtime.evaluate");
    assert_eq!(parsed.params["expression"], "1+1");
}

#[rstest]
fn cdp_response_success() {
    let resp = CdpResponse::success(1, json!({"frameId": "ABC123"}));
    assert!(resp.is_success());
    assert_eq!(resp.id, 1);
    assert_eq!(resp.result.unwrap()["frameId"], "ABC123");
    assert!(resp.error.is_none());
}

#[rstest]
fn cdp_response_error() {
    let resp = CdpResponse::error(2, -32601, "Method not found");
    assert!(!resp.is_success());
    assert_eq!(resp.id, 2);
    assert!(resp.result.is_none());

    let error = resp.error.unwrap();
    assert_eq!(error.code, -32601);
    assert_eq!(error.message, "Method not found");
}

#[rstest]
fn cdp_response_serde_roundtrip_success() {
    let resp = CdpResponse::success(99, json!({"result": 42}));
    let json_str = serde_json::to_string(&resp).unwrap();
    let parsed: CdpResponse = serde_json::from_str(&json_str).unwrap();
    assert!(parsed.is_success());
    assert_eq!(parsed.id, 99);
}

#[rstest]
fn cdp_response_serde_roundtrip_error() {
    let resp = CdpResponse::error(7, -32700, "Parse error");
    let json_str = serde_json::to_string(&resp).unwrap();
    let parsed: CdpResponse = serde_json::from_str(&json_str).unwrap();
    assert!(!parsed.is_success());
    let err = parsed.error.unwrap();
    assert_eq!(err.code, -32700);
    assert_eq!(err.message, "Parse error");
}

#[rstest]
fn cdp_error_with_data() {
    let err = CdpError {
        code: -32000,
        message: "Server error".to_string(),
        data: Some("extra info".to_string()),
    };
    assert_eq!(err.code, -32000);
    assert_eq!(err.data, Some("extra info".to_string()));
}

#[rstest]
fn cdp_event_serde_roundtrip() {
    let event = CdpEvent {
        method: "Page.loadEventFired".to_string(),
        params: json!({"timestamp": 1234.5}),
    };
    let json_str = serde_json::to_string(&event).unwrap();
    let parsed: CdpEvent = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed.method, "Page.loadEventFired");
    assert_eq!(parsed.params["timestamp"], 1234.5);
}

#[rstest]
fn cdp_event_default_params() {
    // CdpEvent with empty params (default)
    let json_str = r#"{"method":"Runtime.exceptionThrown"}"#;
    let parsed: CdpEvent = serde_json::from_str(json_str).unwrap();
    assert_eq!(parsed.method, "Runtime.exceptionThrown");
    assert!(parsed.params.is_null());
}

// ─── CDP domains and methods ─────────────────────────────────────────────────

#[rstest]
fn cdp_domains_values() {
    use auroraview_browser::devtools::cdp::domains;

    assert_eq!(domains::PAGE, "Page");
    assert_eq!(domains::RUNTIME, "Runtime");
    assert_eq!(domains::NETWORK, "Network");
    assert_eq!(domains::DOM, "DOM");
    assert_eq!(domains::CONSOLE, "Console");
    assert_eq!(domains::DEBUGGER, "Debugger");
    assert_eq!(domains::TARGET, "Target");
    assert_eq!(domains::BROWSER, "Browser");
}

#[rstest]
fn cdp_methods_values() {
    use auroraview_browser::devtools::cdp::methods;

    assert_eq!(methods::PAGE_NAVIGATE, "Page.navigate");
    assert_eq!(methods::RUNTIME_EVALUATE, "Runtime.evaluate");
    assert_eq!(methods::NETWORK_ENABLE, "Network.enable");
    assert_eq!(methods::DOM_GET_DOCUMENT, "DOM.getDocument");
    assert_eq!(methods::BROWSER_GET_VERSION, "Browser.getVersion");
}

#[rstest]
fn cdp_methods_page_domain() {
    use auroraview_browser::devtools::cdp::methods;
    assert!(methods::PAGE_NAVIGATE.starts_with("Page."));
    assert!(methods::PAGE_RELOAD.starts_with("Page."));
    assert!(methods::PAGE_GET_FRAME_TREE.starts_with("Page."));
    assert!(methods::PAGE_CAPTURE_SCREENSHOT.starts_with("Page."));
}

#[rstest]
fn cdp_methods_network_domain() {
    use auroraview_browser::devtools::cdp::methods;
    assert!(methods::NETWORK_ENABLE.starts_with("Network."));
    assert!(methods::NETWORK_DISABLE.starts_with("Network."));
    assert!(methods::NETWORK_SET_EXTRA_HEADERS.starts_with("Network."));
}

#[rstest]
fn cdp_methods_runtime_domain() {
    use auroraview_browser::devtools::cdp::methods;
    assert!(methods::RUNTIME_EVALUATE.starts_with("Runtime."));
    assert!(methods::RUNTIME_CALL_FUNCTION_ON.starts_with("Runtime."));
    assert!(methods::RUNTIME_GET_PROPERTIES.starts_with("Runtime."));
}

#[rstest]
fn cdp_methods_target_domain() {
    use auroraview_browser::devtools::cdp::methods;
    assert!(methods::TARGET_GET_TARGETS.starts_with("Target."));
    assert!(methods::TARGET_CREATE_TARGET.starts_with("Target."));
    assert!(methods::TARGET_CLOSE_TARGET.starts_with("Target."));
}

#[rstest]
fn cdp_methods_dom_domain() {
    use auroraview_browser::devtools::cdp::methods;
    assert!(methods::DOM_GET_DOCUMENT.starts_with("DOM."));
    assert!(methods::DOM_QUERY_SELECTOR.starts_with("DOM."));
    assert!(methods::DOM_QUERY_SELECTOR_ALL.starts_with("DOM."));
}
