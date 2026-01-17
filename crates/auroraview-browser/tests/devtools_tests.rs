//! DevTools module tests

use auroraview_browser::devtools::cdp::{CdpRequest, CdpResponse};
use auroraview_browser::{
    ConsoleMessage, ConsoleMessageType, DevToolsConfig, DevToolsManager, DockSide,
};
use rstest::rstest;
use serde_json::json;

#[rstest]
fn test_devtools_config_builder() {
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
fn test_devtools_manager_lifecycle() {
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
fn test_devtools_dock_side() {
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
fn test_console_message_types() {
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
fn test_console_messages_limit() {
    let mut manager = DevToolsManager::new(DevToolsConfig::default());

    // Add more than 1000 messages
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

    // Should be limited to 1000
    assert!(manager.console_messages().len() <= 1001);
}

#[rstest]
fn test_network_requests_tracking() {
    use auroraview_browser::devtools::NetworkRequestInfo;
    use std::collections::HashMap;

    let mut manager = DevToolsManager::new(DevToolsConfig::default());

    let request = NetworkRequestInfo {
        request_id: "req1".to_string(),
        url: "https://example.com/api".to_string(),
        method: "GET".to_string(),
        headers: HashMap::new(),
        post_data: None,
        resource_type: "fetch".to_string(),
        timestamp: 1234567890.0,
    };

    manager.add_network_request(request);

    assert_eq!(manager.network_requests().len(), 1);
    assert!(manager.network_requests().contains_key("req1"));

    manager.clear_network();
    assert!(manager.network_requests().is_empty());
}

#[rstest]
fn test_cdp_request_creation() {
    let req = CdpRequest::new(1, "Page.navigate", json!({"url": "https://example.com"}));

    assert_eq!(req.id, 1);
    assert_eq!(req.method, "Page.navigate");
    assert_eq!(req.params["url"], "https://example.com");
}

#[rstest]
fn test_cdp_request_simple() {
    let req = CdpRequest::simple(42, "Browser.getVersion");

    assert_eq!(req.id, 42);
    assert_eq!(req.method, "Browser.getVersion");
    assert!(req.params.is_null());
}

#[rstest]
fn test_cdp_response_success() {
    let resp = CdpResponse::success(1, json!({"frameId": "ABC123"}));

    assert!(resp.is_success());
    assert_eq!(resp.id, 1);
    assert_eq!(resp.result.unwrap()["frameId"], "ABC123");
    assert!(resp.error.is_none());
}

#[rstest]
fn test_cdp_response_error() {
    let resp = CdpResponse::error(2, -32601, "Method not found");

    assert!(!resp.is_success());
    assert_eq!(resp.id, 2);
    assert!(resp.result.is_none());

    let error = resp.error.unwrap();
    assert_eq!(error.code, -32601);
    assert_eq!(error.message, "Method not found");
}

#[rstest]
fn test_cdp_domains() {
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
fn test_cdp_methods() {
    use auroraview_browser::devtools::cdp::methods;

    assert_eq!(methods::PAGE_NAVIGATE, "Page.navigate");
    assert_eq!(methods::RUNTIME_EVALUATE, "Runtime.evaluate");
    assert_eq!(methods::NETWORK_ENABLE, "Network.enable");
    assert_eq!(methods::DOM_GET_DOCUMENT, "DOM.getDocument");
    assert_eq!(methods::BROWSER_GET_VERSION, "Browser.getVersion");
}

#[rstest]
fn test_dock_side_serialization() {
    let json_str = serde_json::to_string(&DockSide::Right).unwrap();
    assert_eq!(json_str, "\"right\"");

    let parsed: DockSide = serde_json::from_str("\"bottom\"").unwrap();
    assert_eq!(parsed, DockSide::Bottom);

    let parsed: DockSide = serde_json::from_str("\"undocked\"").unwrap();
    assert_eq!(parsed, DockSide::Undocked);
}
