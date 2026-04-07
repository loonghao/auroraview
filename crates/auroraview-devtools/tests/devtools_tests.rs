//! Integration tests for auroraview-devtools
//!
//! Tests cover: DevToolsConfig, DockSide, DevToolsManager, ConsoleMessage,
//! NetworkRequestInfo, NetworkResponseInfo, CdpRequest, CdpResponse, CdpEvent,
//! DevToolsState, DevToolsError, serialization round-trips.

use auroraview_devtools::cdp::{domains, methods, CdpEvent, CdpRequest, CdpResponse};
use auroraview_devtools::{
    CdpSessionInfo, ConsoleMessage, ConsoleMessageType, DevToolsConfig, DevToolsError,
    DevToolsManager, DevToolsState, DockSide, NetworkRequestInfo, NetworkResponseInfo,
};
use rstest::*;
use serde_json::{json, Value};

// ── Fixtures ─────────────────────────────────────────────────────────────────

#[fixture]
fn default_manager() -> DevToolsManager {
    DevToolsManager::default()
}

#[fixture]
fn debug_manager() -> DevToolsManager {
    DevToolsManager::new(
        DevToolsConfig::enabled()
            .with_remote_debugging_port(9222)
            .with_auto_open(false)
            .with_dock_side(DockSide::Bottom),
    )
}

// ── DevToolsConfig ───────────────────────────────────────────────────────────

#[test]
fn config_default_values() {
    let cfg = DevToolsConfig::default();
    assert!(cfg.enabled);
    assert_eq!(cfg.remote_debugging_port, 0);
    assert!(!cfg.auto_open);
    assert_eq!(cfg.dock_side, DockSide::Right);
    assert!(!cfg.is_remote_debugging_enabled());
}

#[test]
fn config_disabled_factory() {
    let cfg = DevToolsConfig::disabled();
    assert!(!cfg.enabled);
}

#[test]
fn config_builder_chain() {
    let cfg = DevToolsConfig::enabled()
        .with_remote_debugging_port(9222)
        .with_auto_open(true)
        .with_dock_side(DockSide::Undocked);
    assert!(cfg.enabled);
    assert_eq!(cfg.remote_debugging_port, 9222);
    assert!(cfg.auto_open);
    assert_eq!(cfg.dock_side, DockSide::Undocked);
    assert!(cfg.is_remote_debugging_enabled());
}

#[test]
fn config_port_zero_means_no_remote_debug() {
    let cfg = DevToolsConfig::enabled().with_remote_debugging_port(0);
    assert!(!cfg.is_remote_debugging_enabled());
}

// ── DockSide ─────────────────────────────────────────────────────────────────

#[rstest]
#[case(DockSide::Right)]
#[case(DockSide::Bottom)]
#[case(DockSide::Left)]
#[case(DockSide::Undocked)]
fn dock_side_serde_round_trip(#[case] side: DockSide) {
    let json = serde_json::to_string(&side).unwrap();
    let back: DockSide = serde_json::from_str(&json).unwrap();
    assert_eq!(back, side);
}

#[test]
fn dock_side_default_is_right() {
    assert_eq!(DockSide::default(), DockSide::Right);
}

// ── DevToolsState ────────────────────────────────────────────────────────────

#[test]
fn state_default() {
    let state = DevToolsState::default();
    assert!(!state.is_open);
    assert!(state.dock_side.is_none());
    assert!(state.selected_panel.is_none());
}

#[test]
fn state_serde_round_trip() {
    let state = DevToolsState {
        is_open: true,
        dock_side: Some(DockSide::Left),
        selected_panel: Some("console".to_string()),
    };
    let json = serde_json::to_string(&state).unwrap();
    let back: DevToolsState = serde_json::from_str(&json).unwrap();
    assert!(back.is_open);
    assert_eq!(back.dock_side, Some(DockSide::Left));
    assert_eq!(back.selected_panel.as_deref(), Some("console"));
}

#[test]
fn state_skip_none_fields_in_serialization() {
    let state = DevToolsState {
        is_open: false,
        dock_side: None,
        selected_panel: None,
    };
    let json = serde_json::to_string(&state).unwrap();
    // Optional None fields should not appear
    assert!(!json.contains("dock_side"));
    assert!(!json.contains("selected_panel"));
}

// ── DevToolsManager – open/close/toggle ──────────────────────────────────────

#[rstest]
fn manager_initial_state(default_manager: DevToolsManager) {
    assert!(!default_manager.is_open());
    assert!(default_manager.is_enabled());
}

#[rstest]
fn manager_open_close(mut default_manager: DevToolsManager) {
    default_manager.open();
    assert!(default_manager.is_open());
    default_manager.close();
    assert!(!default_manager.is_open());
}

#[rstest]
fn manager_toggle(mut default_manager: DevToolsManager) {
    assert!(!default_manager.is_open());
    default_manager.toggle();
    assert!(default_manager.is_open());
    default_manager.toggle();
    assert!(!default_manager.is_open());
}

#[rstest]
fn manager_disabled_config() {
    let manager = DevToolsManager::new(DevToolsConfig::disabled());
    assert!(!manager.is_enabled());
}

// ── DevToolsManager – dock_side / selected_panel ─────────────────────────────

#[rstest]
fn manager_default_dock_side(debug_manager: DevToolsManager) {
    assert_eq!(debug_manager.dock_side(), Some(DockSide::Bottom));
}

#[rstest]
fn manager_set_dock_side(mut default_manager: DevToolsManager) {
    assert_eq!(default_manager.dock_side(), Some(DockSide::Right));
    default_manager.set_dock_side(DockSide::Left);
    assert_eq!(default_manager.dock_side(), Some(DockSide::Left));
}

#[rstest]
fn manager_selected_panel(mut default_manager: DevToolsManager) {
    assert!(default_manager.selected_panel().is_none());
    default_manager.set_selected_panel("network");
    assert_eq!(default_manager.selected_panel(), Some("network"));
    default_manager.set_selected_panel("console");
    assert_eq!(default_manager.selected_panel(), Some("console"));
}

// ── DevToolsManager – remote debugging ──────────────────────────────────────

#[rstest]
fn manager_remote_debugging_disabled(default_manager: DevToolsManager) {
    assert_eq!(default_manager.remote_debugging_port(), 0);
    assert!(!default_manager.is_remote_debugging_enabled());
}

#[rstest]
fn manager_remote_debugging_enabled(debug_manager: DevToolsManager) {
    assert_eq!(debug_manager.remote_debugging_port(), 9222);
    assert!(debug_manager.is_remote_debugging_enabled());
}

// ── DevToolsManager – console messages ──────────────────────────────────────

#[rstest]
fn manager_add_console_message(mut default_manager: DevToolsManager) {
    assert_eq!(default_manager.console_message_count(), 0);
    default_manager.add_console_message(ConsoleMessage::log("hello"));
    assert_eq!(default_manager.console_message_count(), 1);
}

#[rstest]
fn manager_console_filter_errors(mut default_manager: DevToolsManager) {
    default_manager.add_console_message(ConsoleMessage::log("info"));
    default_manager.add_console_message(ConsoleMessage::error("err1"));
    default_manager.add_console_message(ConsoleMessage::warning("warn"));
    default_manager.add_console_message(ConsoleMessage::error("err2"));

    assert_eq!(default_manager.error_messages().len(), 2);
    assert_eq!(default_manager.warning_messages().len(), 1);
    assert_eq!(default_manager.console_message_count(), 4);
}

#[rstest]
fn manager_console_clear(mut default_manager: DevToolsManager) {
    default_manager.add_console_message(ConsoleMessage::log("a"));
    default_manager.add_console_message(ConsoleMessage::log("b"));
    default_manager.clear_console();
    assert_eq!(default_manager.console_message_count(), 0);
}

#[test]
fn manager_console_max_messages_evicts_oldest() {
    let mut manager = DevToolsManager::default().with_max_console_messages(3);
    for i in 0..6u32 {
        manager.add_console_message(ConsoleMessage::log(format!("msg{}", i)));
    }
    assert_eq!(manager.console_message_count(), 3);
    // Oldest (msg0..msg2) evicted; remaining should be msg3, msg4, msg5
    let msgs = manager.console_messages();
    assert!(msgs[0].text.contains("3"));
    assert!(msgs[1].text.contains("4"));
    assert!(msgs[2].text.contains("5"));
}

#[test]
fn manager_console_max_messages_exact_boundary() {
    let mut manager = DevToolsManager::default().with_max_console_messages(2);
    manager.add_console_message(ConsoleMessage::log("a"));
    manager.add_console_message(ConsoleMessage::log("b"));
    // At capacity; next evicts "a"
    manager.add_console_message(ConsoleMessage::log("c"));
    assert_eq!(manager.console_message_count(), 2);
    assert_eq!(manager.console_messages()[0].text, "b");
    assert_eq!(manager.console_messages()[1].text, "c");
}

// ── DevToolsManager – network requests ──────────────────────────────────────

#[rstest]
fn manager_add_network_request(mut default_manager: DevToolsManager) {
    let req = NetworkRequestInfo::new("req-1", "https://example.com", "GET");
    default_manager.add_network_request(req);
    assert_eq!(default_manager.network_request_count(), 1);
    assert!(default_manager.get_network_request("req-1").is_some());
    assert!(default_manager.get_network_request("req-none").is_none());
}

#[rstest]
fn manager_network_clear(mut default_manager: DevToolsManager) {
    default_manager.add_network_request(NetworkRequestInfo::new("r1", "https://a.com", "GET"));
    default_manager.add_network_request(NetworkRequestInfo::new("r2", "https://b.com", "POST"));
    default_manager.clear_network();
    assert_eq!(default_manager.network_request_count(), 0);
}

#[rstest]
fn manager_network_list(mut default_manager: DevToolsManager) {
    default_manager.add_network_request(NetworkRequestInfo::new("r1", "https://a.com", "GET"));
    default_manager.add_network_request(NetworkRequestInfo::new("r2", "https://b.com", "POST"));
    assert_eq!(default_manager.network_requests_list().len(), 2);
}

#[rstest]
fn manager_clear_all(mut default_manager: DevToolsManager) {
    default_manager.add_console_message(ConsoleMessage::log("x"));
    default_manager.add_network_request(NetworkRequestInfo::new("r1", "https://a.com", "GET"));
    default_manager.clear_all();
    assert_eq!(default_manager.console_message_count(), 0);
    assert_eq!(default_manager.network_request_count(), 0);
}

// ── DevToolsManager – state/config accessors ─────────────────────────────────

#[rstest]
fn manager_state_accessor(debug_manager: DevToolsManager) {
    let state = debug_manager.state();
    assert!(!state.is_open);
    assert_eq!(state.dock_side, Some(DockSide::Bottom));
}

#[rstest]
fn manager_config_accessor(debug_manager: DevToolsManager) {
    let cfg = debug_manager.config();
    assert_eq!(cfg.remote_debugging_port, 9222);
}

// ── ConsoleMessage ───────────────────────────────────────────────────────────

#[rstest]
#[case(ConsoleMessage::log("l"), ConsoleMessageType::Log, false, false)]
#[case(ConsoleMessage::debug("d"), ConsoleMessageType::Debug, false, false)]
#[case(ConsoleMessage::info("i"), ConsoleMessageType::Info, false, false)]
#[case(ConsoleMessage::warning("w"), ConsoleMessageType::Warning, false, true)]
#[case(ConsoleMessage::error("e"), ConsoleMessageType::Error, true, false)]
fn console_message_factories(
    #[case] msg: ConsoleMessage,
    #[case] expected_type: ConsoleMessageType,
    #[case] is_error: bool,
    #[case] is_warning: bool,
) {
    assert_eq!(msg.message_type, expected_type);
    assert_eq!(msg.is_error(), is_error);
    assert_eq!(msg.is_warning(), is_warning);
}

#[test]
fn console_message_with_source() {
    let msg = ConsoleMessage::error("fail")
        .with_source("app.js", 42, 7)
        .with_stack_trace("at foo (app.js:42)");
    assert_eq!(msg.source.as_deref(), Some("app.js"));
    assert_eq!(msg.line, Some(42));
    assert_eq!(msg.column, Some(7));
    assert!(msg.stack_trace.is_some());
}

#[test]
fn console_message_serde_round_trip() {
    let msg = ConsoleMessage::error("oops").with_source("x.js", 1, 0);
    let json = serde_json::to_string(&msg).unwrap();
    let back: ConsoleMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(back.text, "oops");
    assert_eq!(back.source.as_deref(), Some("x.js"));
    assert!(back.is_error());
}

#[test]
fn console_message_timestamp_is_positive() {
    let msg = ConsoleMessage::log("ts");
    assert!(msg.timestamp >= 0);
}

// ── NetworkRequestInfo ───────────────────────────────────────────────────────

#[test]
fn network_request_new() {
    let req = NetworkRequestInfo::new("r1", "https://api.example.com/v1/data", "POST");
    assert_eq!(req.request_id, "r1");
    assert_eq!(req.method, "POST");
    assert_eq!(req.domain(), Some("api.example.com"));
    assert!(req.headers.is_empty());
    assert!(req.post_data.is_none());
}

#[test]
fn network_request_with_header() {
    let req = NetworkRequestInfo::new("r2", "https://x.com", "GET")
        .with_header("Authorization", "Bearer token")
        .with_header("Accept", "application/json");
    assert_eq!(req.headers.len(), 2);
    assert_eq!(
        req.headers.get("Authorization").map(|s| s.as_str()),
        Some("Bearer token")
    );
}

#[test]
fn network_request_with_post_data() {
    let req =
        NetworkRequestInfo::new("r3", "https://x.com", "POST").with_post_data(r#"{"key":"value"}"#);
    assert!(req.post_data.is_some());
}

#[test]
fn network_request_with_resource_type() {
    let req = NetworkRequestInfo::new("r4", "https://x.com/style.css", "GET")
        .with_resource_type("Stylesheet");
    assert_eq!(req.resource_type, "Stylesheet");
}

#[rstest]
#[case("https://example.com/path", Some("example.com"))]
#[case("http://localhost:8080/", Some("localhost:8080"))]
#[case("ftp://example.com", None)]
fn network_request_domain_extraction(#[case] url: &str, #[case] expected: Option<&str>) {
    let req = NetworkRequestInfo::new("x", url, "GET");
    assert_eq!(req.domain(), expected);
}

#[test]
fn network_request_serde_round_trip() {
    let req =
        NetworkRequestInfo::new("r5", "https://example.com", "DELETE").with_header("X-Auth", "abc");
    let json = serde_json::to_string(&req).unwrap();
    let back: NetworkRequestInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(back.request_id, "r5");
    assert_eq!(back.method, "DELETE");
    assert_eq!(back.headers.get("X-Auth").map(|s| s.as_str()), Some("abc"));
}

// ── NetworkResponseInfo ──────────────────────────────────────────────────────

#[rstest]
#[case(200, true, false, false, false)]
#[case(201, true, false, false, false)]
#[case(301, false, true, false, false)]
#[case(302, false, true, false, false)]
#[case(400, false, false, true, false)]
#[case(404, false, false, true, false)]
#[case(500, false, false, false, true)]
#[case(503, false, false, false, true)]
fn network_response_status_categories(
    #[case] status: u16,
    #[case] success: bool,
    #[case] redirect: bool,
    #[case] client_err: bool,
    #[case] server_err: bool,
) {
    let resp = NetworkResponseInfo::new("r", status, "");
    assert_eq!(resp.is_success(), success);
    assert_eq!(resp.is_redirect(), redirect);
    assert_eq!(resp.is_client_error(), client_err);
    assert_eq!(resp.is_server_error(), server_err);
}

#[test]
fn network_response_builder() {
    let resp = NetworkResponseInfo::new("r1", 200, "OK")
        .with_mime_type("application/json")
        .with_content_length(2048)
        .with_from_cache(true)
        .with_header("ETag", "\"abc\"");
    assert_eq!(resp.mime_type, "application/json");
    assert_eq!(resp.content_length, Some(2048));
    assert!(resp.from_cache);
    assert_eq!(
        resp.headers.get("ETag").map(|s| s.as_str()),
        Some("\"abc\"")
    );
}

#[test]
fn network_response_serde_round_trip() {
    let resp = NetworkResponseInfo::new("r2", 404, "Not Found").with_content_length(100);
    let json = serde_json::to_string(&resp).unwrap();
    let back: NetworkResponseInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(back.status, 404);
    assert_eq!(back.content_length, Some(100));
    assert!(back.is_client_error());
}

// ── CDP types ────────────────────────────────────────────────────────────────

#[test]
fn cdp_request_new() {
    let req = CdpRequest::new(1, "Page.navigate", json!({"url": "https://example.com"}));
    assert_eq!(req.id, 1);
    assert_eq!(req.method, "Page.navigate");
}

#[test]
fn cdp_request_simple_has_null_params() {
    let req = CdpRequest::simple(5, "Page.reload");
    assert_eq!(req.params, Value::Null);
}

#[test]
fn cdp_request_page_navigate() {
    let req = CdpRequest::page_navigate(2, "https://example.com");
    assert_eq!(req.method, methods::PAGE_NAVIGATE);
    assert_eq!(req.params["url"], "https://example.com");
}

#[test]
fn cdp_request_page_reload() {
    let req = CdpRequest::page_reload(3);
    assert_eq!(req.method, methods::PAGE_RELOAD);
}

#[test]
fn cdp_request_runtime_evaluate() {
    let req = CdpRequest::runtime_evaluate(4, "1+1");
    assert_eq!(req.method, methods::RUNTIME_EVALUATE);
    assert_eq!(req.params["expression"], "1+1");
}

#[test]
fn cdp_request_capture_screenshot() {
    let req = CdpRequest::capture_screenshot(6, "png");
    assert_eq!(req.method, methods::PAGE_CAPTURE_SCREENSHOT);
    assert_eq!(req.params["format"], "png");
}

#[test]
fn cdp_request_serde_round_trip() {
    let req = CdpRequest::page_navigate(1, "https://example.com");
    let json = serde_json::to_string(&req).unwrap();
    let back: CdpRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.id, 1);
    assert_eq!(back.method, methods::PAGE_NAVIGATE);
}

#[test]
fn cdp_response_success() {
    let resp = CdpResponse::success(1, json!({"frameId": "123"}));
    assert!(resp.is_success());
    assert!(resp.result.is_some());
    assert!(resp.error.is_none());
}

#[test]
fn cdp_response_error() {
    let resp = CdpResponse::error(2, -32601, "Method not found");
    assert!(!resp.is_success());
    assert!(resp.result.is_none());
    let err = resp.error.as_ref().unwrap();
    assert_eq!(err.code, -32601);
    assert_eq!(err.message, "Method not found");
}

#[test]
fn cdp_response_into_result_ok() {
    let resp = CdpResponse::success(1, json!(42));
    let result = resp.into_result();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), json!(42));
}

#[test]
fn cdp_response_into_result_err() {
    let resp = CdpResponse::error(2, -32600, "Invalid request");
    let result = resp.into_result();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid request"));
}

#[test]
fn cdp_response_success_null_result() {
    // success with Null result should still succeed
    let resp = CdpResponse::success(7, Value::Null);
    let result = resp.into_result();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Null);
}

#[test]
fn cdp_response_serde_round_trip() {
    let resp = CdpResponse::success(10, json!({"ok": true}));
    let json = serde_json::to_string(&resp).unwrap();
    let back: CdpResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(back.id, 10);
    assert!(back.is_success());
}

#[rstest]
#[case("Page.loadEventFired", true, false, false)]
#[case("Network.requestWillBeSent", false, true, false)]
#[case("Console.messageAdded", false, false, true)]
#[case("Debugger.paused", false, false, false)]
fn cdp_event_domain_detection(
    #[case] method: &str,
    #[case] is_page: bool,
    #[case] is_network: bool,
    #[case] is_console: bool,
) {
    let event = CdpEvent::new(method, json!({}));
    assert_eq!(event.is_page_event(), is_page);
    assert_eq!(event.is_network_event(), is_network);
    assert_eq!(event.is_console_event(), is_console);
}

#[test]
fn cdp_event_serde_round_trip() {
    let event = CdpEvent::new("Page.loadEventFired", json!({"timestamp": 1234.5}));
    let json_str = serde_json::to_string(&event).unwrap();
    let back: CdpEvent = serde_json::from_str(&json_str).unwrap();
    assert_eq!(back.method, "Page.loadEventFired");
    assert_eq!(back.params["timestamp"], 1234.5);
}

// ── CdpSessionInfo ───────────────────────────────────────────────────────────

#[test]
fn cdp_session_info_serde() {
    let json_str = r#"{
        "websocketDebuggerUrl": "ws://localhost:9222/devtools/browser/abc",
        "devtoolsFrontendUrl": "/devtools/inspector.html?ws=localhost:9222/devtools/browser/abc",
        "browser": "Chrome/109.0.0.0",
        "protocolVersion": "1.3",
        "userAgent": "Mozilla/5.0",
        "v8Version": "10.9.193.0",
        "webkitVersion": "537.36"
    }"#;
    let session: CdpSessionInfo = serde_json::from_str(json_str).unwrap();
    assert!(session.websocket_debugger_url.starts_with("ws://"));
    assert_eq!(session.protocol_version, "1.3");
    assert!(!session.user_agent.is_empty());
}

// ── Domains & Methods constants ──────────────────────────────────────────────

#[test]
fn domain_constants_non_empty() {
    assert!(!domains::PAGE.is_empty());
    assert!(!domains::RUNTIME.is_empty());
    assert!(!domains::NETWORK.is_empty());
    assert!(!domains::DOM.is_empty());
    assert!(!domains::CONSOLE.is_empty());
    assert!(!domains::DEBUGGER.is_empty());
    assert!(!domains::PROFILER.is_empty());
    assert!(!domains::TARGET.is_empty());
    assert!(!domains::BROWSER.is_empty());
}

#[test]
fn method_constants_contain_dot() {
    // All CDP methods follow "Domain.method" naming convention
    let all_methods = [
        methods::PAGE_NAVIGATE,
        methods::PAGE_RELOAD,
        methods::PAGE_GET_FRAME_TREE,
        methods::PAGE_CAPTURE_SCREENSHOT,
        methods::PAGE_ENABLE,
        methods::PAGE_DISABLE,
        methods::RUNTIME_EVALUATE,
        methods::RUNTIME_CALL_FUNCTION_ON,
        methods::RUNTIME_GET_PROPERTIES,
        methods::RUNTIME_ENABLE,
        methods::RUNTIME_DISABLE,
        methods::NETWORK_ENABLE,
        methods::NETWORK_DISABLE,
        methods::NETWORK_SET_EXTRA_HEADERS,
        methods::NETWORK_GET_RESPONSE_BODY,
        methods::DOM_GET_DOCUMENT,
        methods::DOM_QUERY_SELECTOR,
        methods::DOM_QUERY_SELECTOR_ALL,
        methods::DOM_ENABLE,
        methods::DOM_DISABLE,
        methods::CONSOLE_ENABLE,
        methods::CONSOLE_DISABLE,
        methods::CONSOLE_CLEAR_MESSAGES,
        methods::TARGET_GET_TARGETS,
        methods::TARGET_CREATE_TARGET,
        methods::TARGET_CLOSE_TARGET,
        methods::TARGET_ATTACH_TO_TARGET,
        methods::BROWSER_GET_VERSION,
        methods::BROWSER_CLOSE,
    ];
    for m in &all_methods {
        assert!(m.contains('.'), "method constant '{}' missing '.'", m);
    }
}

// ── DevToolsError ────────────────────────────────────────────────────────────

#[test]
fn error_display_messages() {
    let err = DevToolsError::NotEnabled;
    assert!(err.to_string().contains("not enabled"));

    let err = DevToolsError::AlreadyOpen;
    assert!(err.to_string().contains("already open"));

    let err = DevToolsError::CdpConnection("timeout".to_string());
    assert!(err.to_string().contains("timeout"));

    let err = DevToolsError::CdpCommand("unknown domain".to_string());
    assert!(err.to_string().contains("unknown domain"));
}

#[test]
fn error_from_serde_json() {
    let invalid_json = r#"{"id": }"#;
    let serde_err = serde_json::from_str::<CdpRequest>(invalid_json).unwrap_err();
    let err = DevToolsError::Serialization(serde_err);
    assert!(err.to_string().to_lowercase().contains("serial"));
}

// ── Manager default() == default_config() ────────────────────────────────────

#[test]
fn manager_default_equals_default_config() {
    let a = DevToolsManager::default();
    let b = DevToolsManager::default_config();
    // Both are enabled with port 0
    assert_eq!(a.remote_debugging_port(), b.remote_debugging_port());
    assert_eq!(a.is_enabled(), b.is_enabled());
    assert_eq!(a.is_open(), b.is_open());
}

// ── Multi-message ordering guarantees ────────────────────────────────────────

#[test]
fn manager_console_messages_preserve_order() {
    let mut manager = DevToolsManager::default();
    for i in 0..10u32 {
        manager.add_console_message(ConsoleMessage::info(format!("line{}", i)));
    }
    let messages = manager.console_messages();
    for (i, msg) in messages.iter().enumerate() {
        assert!(msg.text.contains(&i.to_string()));
    }
}

// ── Network request overwrite ────────────────────────────────────────────────

#[test]
fn network_request_overwrite_same_id() {
    let mut manager = DevToolsManager::default();
    manager.add_network_request(NetworkRequestInfo::new("dup", "https://a.com", "GET"));
    manager.add_network_request(NetworkRequestInfo::new("dup", "https://b.com", "POST"));
    // Only 1 entry since same request_id
    assert_eq!(manager.network_request_count(), 1);
    let req = manager.get_network_request("dup").unwrap();
    assert_eq!(req.url, "https://b.com");
    assert_eq!(req.method, "POST");
}
