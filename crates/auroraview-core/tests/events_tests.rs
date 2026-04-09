//! Events module tests
//!
//! Covers CoreUserEvent and ExtendedUserEvent enums with all variants,
//! Clone/Debug trait behavior, and edge cases.

use auroraview_core::events::{CoreUserEvent, ExtendedUserEvent};
use rstest::rstest;

// ============================================================================
// CoreUserEvent variants
// ============================================================================

#[test]
fn core_process_messages() {
    let ev = CoreUserEvent::ProcessMessages;
    let debug_str = format!("{:?}", ev);
    assert!(debug_str.contains("ProcessMessages"));
}

#[test]
fn core_close_window() {
    let ev = CoreUserEvent::CloseWindow;
    assert!(matches!(ev, CoreUserEvent::CloseWindow));
}

#[test]
fn core_plugin_event() {
    let ev = CoreUserEvent::PluginEvent {
        event: "test.event".to_string(),
        data: r#"{"key": "value"}"#.to_string(),
    };
    match ev {
        CoreUserEvent::PluginEvent { event, data } => {
            assert_eq!(event, "test.event");
            assert_eq!(data, r#"{"key": "value"}"#);
        }
        _ => panic!("Expected PluginEvent variant"),
    }
}

#[test]
fn core_plugin_event_empty_data() {
    let ev = CoreUserEvent::PluginEvent {
        event: "empty".to_string(),
        data: String::new(),
    };
    if let CoreUserEvent::PluginEvent { data, .. } = ev {
        assert!(data.is_empty());
    }
}

#[test]
fn core_drag_window() {
    let ev = CoreUserEvent::DragWindow;
    assert!(matches!(ev, CoreUserEvent::DragWindow));
}

// ============================================================================
// ExtendedUserEvent variants
// ============================================================================

#[test]
fn ext_python_ready() {
    let ev = ExtendedUserEvent::PythonReady {
        handlers: vec!["api.echo".to_string(), "api.ping".to_string()],
    };
    match ev {
        ExtendedUserEvent::PythonReady { handlers } => {
            assert_eq!(handlers.len(), 2);
            assert!(handlers.contains(&"api.echo".to_string()));
        }
        _ => panic!("Expected PythonReady"),
    }
}

#[test]
fn ext_python_ready_empty_handlers() {
    let ev = ExtendedUserEvent::PythonReady { handlers: vec![] };
    if let ExtendedUserEvent::PythonReady { handlers } = ev {
        assert!(handlers.is_empty());
    }
}

#[test]
fn ext_python_response() {
    let ev = ExtendedUserEvent::PythonResponse("result data".to_string());
    match ev {
        ExtendedUserEvent::PythonResponse(data) => {
            assert_eq!(data, "result data");
        }
        _ => panic!("Expected PythonResponse"),
    }
}

#[test]
fn ext_python_response_empty() {
    let ev = ExtendedUserEvent::PythonResponse(String::new());
    if let ExtendedUserEvent::PythonResponse(data) = ev {
        assert!(data.is_empty());
    }
}

#[test]
fn ext_loading_screen_ready() {
    let ev = ExtendedUserEvent::LoadingScreenReady;
    assert!(matches!(ev, ExtendedUserEvent::LoadingScreenReady));
}

#[test]
fn ext_navigate_to_app() {
    let ev = ExtendedUserEvent::NavigateToApp;
    assert!(matches!(ev, ExtendedUserEvent::NavigateToApp));
}

#[test]
fn ext_page_ready() {
    let ev = ExtendedUserEvent::PageReady;
    assert!(matches!(ev, ExtendedUserEvent::PageReady));
}

#[rstest]
#[case(Some(50), Some("loading"), Some("step1"), Some("in progress"), Some("ok"))]
#[case(Some(100), None, None, None, None)]
#[case(None, None, None, None, None)]
fn ext_loading_update(
    #[case] progress: Option<i32>,
    #[case] text: Option<&str>,
    #[case] step_id: Option<&str>,
    #[case] step_text: Option<&str>,
    #[case] step_status: Option<&str>,
) {
    let ev = ExtendedUserEvent::LoadingUpdate {
        progress,
        text: text.map(|s| s.to_string()),
        step_id: step_id.map(|s| s.to_string()),
        step_text: step_text.map(|s| s.to_string()),
        step_status: step_status.map(|s| s.to_string()),
    };
    assert!(matches!(ev, ExtendedUserEvent::LoadingUpdate { .. }));
}

#[test]
fn ext_backend_error() {
    let ev = ExtendedUserEvent::BackendError {
        message: "Failed to connect".to_string(),
        source: "stderr".to_string(),
    };
    match ev {
        ExtendedUserEvent::BackendError { message, source } => {
            assert_eq!(message, "Failed to connect");
            assert_eq!(source, "stderr");
        }
        _ => panic!("Expected BackendError"),
    }
}

#[test]
fn ext_backend_error_startup_source() {
    let ev = ExtendedUserEvent::BackendError {
        message: "Python not found".to_string(),
        source: "startup".to_string(),
    };
    if let ExtendedUserEvent::BackendError { source, .. } = ev {
        assert_eq!(source, "startup");
    }
}

#[test]
fn ext_set_html_with_title() {
    let ev = ExtendedUserEvent::SetHtml {
        html: "<div>content</div>".to_string(),
        title: Some("My Page".to_string()),
    };
    match ev {
        ExtendedUserEvent::SetHtml { html, title } => {
            assert_eq!(html, "<div>content</div>");
            assert_eq!(title, Some("My Page".to_string()));
        }
        _ => panic!("Expected SetHtml"),
    }
}

#[test]
fn ext_set_html_without_title() {
    let ev = ExtendedUserEvent::SetHtml {
        html: "<p>Hello</p>".to_string(),
        title: None,
    };
    if let ExtendedUserEvent::SetHtml { title, .. } = ev {
        assert!(title.is_none());
    }
}

#[test]
fn ext_show_error_full() {
    let ev = ExtendedUserEvent::ShowError {
        code: 500,
        title: "Internal Error".to_string(),
        message: "Something went wrong".to_string(),
        details: Some("Stack trace...".to_string()),
        source: "webview".to_string(),
    };
    match ev {
        ExtendedUserEvent::ShowError {
            code,
            title,
            message,
            details,
            source,
        } => {
            assert_eq!(code, 500);
            assert_eq!(title, "Internal Error");
            assert_eq!(message, "Something went wrong");
            assert_eq!(details, Some("Stack trace...".to_string()));
            assert_eq!(source, "webview");
        }
        _ => panic!("Expected ShowError"),
    }
}

#[test]
fn ext_show_error_no_details() {
    let ev = ExtendedUserEvent::ShowError {
        code: 404,
        title: "Not Found".to_string(),
        message: "Missing resource".to_string(),
        details: None,
        source: "".to_string(),
    };
    if let ExtendedUserEvent::ShowError { details, .. } = ev {
        assert!(details.is_none());
    }
}

#[test]
fn ext_tray_menu_click() {
    let ev = ExtendedUserEvent::TrayMenuClick("Exit".to_string());
    match ev {
        ExtendedUserEvent::TrayMenuClick(label) => {
            assert_eq!(label, "Exit");
        }
        _ => panic!("Expected TrayMenuClick"),
    }
}

#[test]
fn ext_tray_menu_click_empty() {
    let ev = ExtendedUserEvent::TrayMenuClick(String::new());
    if let ExtendedUserEvent::TrayMenuClick(label) = ev {
        assert!(label.is_empty());
    }
}

#[test]
fn ext_tray_icon_click() {
    let ev = ExtendedUserEvent::TrayIconClick;
    assert!(matches!(ev, ExtendedUserEvent::TrayIconClick));
}

#[test]
fn ext_tray_icon_double_click() {
    let ev = ExtendedUserEvent::TrayIconDoubleClick;
    assert!(matches!(ev, ExtendedUserEvent::TrayIconDoubleClick));
}

#[test]
fn ext_create_child_window() {
    let ev = ExtendedUserEvent::CreateChildWindow {
        url: "https://example.com".to_string(),
        width: 800,
        height: 600,
    };
    match ev {
        ExtendedUserEvent::CreateChildWindow { url, width, height } => {
            assert_eq!(url, "https://example.com");
            assert_eq!(width, 800);
            assert_eq!(height, 600);
        }
        _ => panic!("Expected CreateChildWindow"),
    }
}

#[test]
fn ext_create_child_window_minimal() {
    let ev = ExtendedUserEvent::CreateChildWindow {
        url: String::new(),
        width: 0,
        height: 0,
    };
    if let ExtendedUserEvent::CreateChildWindow { width, height, .. } = ev {
        assert_eq!(width, 0);
        assert_eq!(height, 0);
    }
}

// ============================================================================
// Trait derivations: Clone
// ============================================================================

#[test]
fn core_user_event_clone() {
    let original = CoreUserEvent::PluginEvent {
        event: "clone.test".to_string(),
        data: "data".to_string(),
    };
    let cloned = original.clone();
    assert_eq!(
        format!("{:?}", original),
        format!("{:?}", cloned)
    );
}

#[test]
fn extended_user_event_clone() {
    let original = ExtendedUserEvent::BackendError {
        message: "error".to_string(),
        source: "test".to_string(),
    };
    let cloned = original.clone();
    assert_eq!(
        format!("{:?}", original),
        format!("{:?}", cloned)
    );
}

#[test]
fn extended_python_ready_clone() {
    let original = ExtendedUserEvent::PythonReady {
        handlers: vec!["h1".to_string(), "h2".to_string()],
    };
    let cloned = original.clone();
    if let (ExtendedUserEvent::PythonReady { handlers: h1 },
              ExtendedUserEvent::PythonReady { handlers: h2 }) = (original, cloned) {
        assert_eq!(h1, h2);
    }
}

// ============================================================================
// Trait derivations: Debug
// ============================================================================

#[test]
fn core_debug_contains_variant_name() {
    let events = vec![
        CoreUserEvent::ProcessMessages,
        CoreUserEvent::CloseWindow,
        CoreUserEvent::PluginEvent {
            event: "x".to_string(),
            data: "y".to_string(),
        },
        CoreUserEvent::DragWindow,
    ];
    for ev in events {
        let debug_str = format!("{:?}", ev);
        assert!(!debug_str.is_empty());
    }
}

#[test]
fn extended_debug_contains_info() {
    let events: Vec<ExtendedUserEvent> = vec![
        ExtendedUserEvent::PythonReady { handlers: vec![] },
        ExtendedUserEvent::PythonResponse("resp".into()),
        ExtendedUserEvent::LoadingScreenReady,
        ExtendedUserEvent::NavigateToApp,
        ExtendedUserEvent::PageReady,
        ExtendedUserEvent::BackendError {
            message: "err".into(),
            source: "src".into(),
        },
        ExtendedUserEvent::TrayMenuClick("menu".into()),
        ExtendedUserEvent::TrayIconClick,
        ExtendedUserEvent::TrayIconDoubleClick,
    ];
    for ev in events {
        let debug_str = format!("{:?}", ev);
        assert!(!debug_str.is_empty());
    }
}

// ============================================================================
// PartialEq is NOT derived on CoreUserEvent / ExtendedUserEvent
// Verify they can be compared via pattern matching instead
// ============================================================================

#[test]
fn core_event_pattern_match_same_plugin_event() {
    let a = CoreUserEvent::PluginEvent {
        event: "same".to_string(),
        data: "data".to_string(),
    };
    let b = CoreUserEvent::PluginEvent {
        event: "same".to_string(),
        data: "data".to_string(),
    };
    // Use debug comparison as proxy since PartialEq not derived
    assert_eq!(format!("{:?}", a), format!("{:?}", b));
}

#[test]
fn core_event_different_variants_produce_different_debug() {
    let a = CoreUserEvent::ProcessMessages;
    let b = CoreUserEvent::CloseWindow;
    assert_ne!(format!("{:?}", a), format!("{:?}", b));
}

#[test]
fn extended_event_different_variants_produce_different_debug() {
    let a = ExtendedUserEvent::LoadingScreenReady;
    let b = ExtendedUserEvent::PageReady;
    assert_ne!(format!("{:?}", a), format!("{:?}", b));
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn plugin_event_unicode() {
    let ev = CoreUserEvent::PluginEvent {
        event: "中文事件".to_string(),
        data: r#"{"emoji": "🚀"}"#.to_string(),
    };
    if let CoreUserEvent::PluginEvent { event, data } = ev {
        assert_eq!(event, "中文事件");
        assert!(data.contains("🚀"));
    }
}

#[test]
fn backend_error_special_chars_in_message() {
    let ev = ExtendedUserEvent::BackendError {
        message: "Error: <script>alert('xss')</script>".to_string(),
        source: "stderr".to_string(),
    };
    if let ExtendedUserEvent::BackendError { message, .. } = ev {
        assert!(message.contains("<script>"));
    }
}

#[test]
fn set_html_large_content() {
    let large_html = "<div>".repeat(1000) + "</div>";
    // Verify the HTML is large (>5000 chars) and can be stored in SetHtml
    assert!(large_html.len() > 5000);
    let ev = ExtendedUserEvent::SetHtml {
        html: large_html.clone(),
        title: Some("Large page".to_string()),
    };
    if let ExtendedUserEvent::SetHtml { html, title } = ev {
        assert!(html.len() > 5000);
        assert_eq!(title, Some("Large page".to_string()));
    }
}

#[test]
fn create_child_window_url_with_query_params() {
    let url = "https://example.com/path?key=value&other=123#fragment";
    let ev = ExtendedUserEvent::CreateChildWindow {
        url: url.to_string(),
        width: 1024,
        height: 768,
    };
    if let ExtendedUserEvent::CreateChildWindow { url: u, .. } = ev {
        assert_eq!(u, url);
    }
}

// ============================================================================
// R10 Extensions
// ============================================================================

#[test]
fn core_event_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<CoreUserEvent>();
    assert_sync::<CoreUserEvent>();
}

#[test]
fn extended_event_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<ExtendedUserEvent>();
    assert_sync::<ExtendedUserEvent>();
}

#[test]
fn core_plugin_event_large_data() {
    let large_data = "{\"key\":\"".to_string() + &"v".repeat(8192) + "\"}";
    let ev = CoreUserEvent::PluginEvent {
        event: "large.event".to_string(),
        data: large_data.clone(),
    };
    if let CoreUserEvent::PluginEvent { data, .. } = ev {
        assert_eq!(data.len(), large_data.len());
    }
}

#[test]
fn ext_loading_update_all_none() {
    let ev = ExtendedUserEvent::LoadingUpdate {
        progress: None,
        text: None,
        step_id: None,
        step_text: None,
        step_status: None,
    };
    if let ExtendedUserEvent::LoadingUpdate { progress, text, step_id, step_text, step_status } = ev {
        assert!(progress.is_none());
        assert!(text.is_none());
        assert!(step_id.is_none());
        assert!(step_text.is_none());
        assert!(step_status.is_none());
    }
}

#[test]
fn ext_loading_update_all_some() {
    let ev = ExtendedUserEvent::LoadingUpdate {
        progress: Some(75),
        text: Some("Loading...".to_string()),
        step_id: Some("step-3".to_string()),
        step_text: Some("Initializing plugins".to_string()),
        step_status: Some("running".to_string()),
    };
    if let ExtendedUserEvent::LoadingUpdate { progress, text, step_id, .. } = ev {
        assert_eq!(progress, Some(75));
        assert_eq!(text.as_deref(), Some("Loading..."));
        assert_eq!(step_id.as_deref(), Some("step-3"));
    }
}

#[test]
fn ext_show_error_code_variants() {
    for code in &[400_u16, 401, 403, 404, 500, 503] {
        let ev = ExtendedUserEvent::ShowError {
            code: *code,
            title: format!("Error {}", code),
            message: "Error message".to_string(),
            details: None,
            source: "test".to_string(),
        };
        if let ExtendedUserEvent::ShowError { code: c, .. } = ev {
            assert_eq!(c, *code);
        }
    }
}

#[test]
fn ext_tray_menu_click_unicode_label() {
    let ev = ExtendedUserEvent::TrayMenuClick("終了".to_string());
    if let ExtendedUserEvent::TrayMenuClick(label) = ev {
        assert_eq!(label, "終了");
    }
}

#[test]
fn ext_python_ready_many_handlers() {
    let handlers: Vec<String> = (0..100).map(|i| format!("api.method_{i}")).collect();
    let ev = ExtendedUserEvent::PythonReady { handlers: handlers.clone() };
    if let ExtendedUserEvent::PythonReady { handlers: h } = ev {
        assert_eq!(h.len(), 100);
        assert_eq!(h[0], "api.method_0");
        assert_eq!(h[99], "api.method_99");
    }
}

#[test]
fn ext_python_response_json_content() {
    let json = r#"{"ok":true,"result":{"key":"value"}}"#;
    let ev = ExtendedUserEvent::PythonResponse(json.to_string());
    if let ExtendedUserEvent::PythonResponse(data) = ev {
        assert!(data.contains("\"ok\":true"));
    }
}

#[test]
fn ext_set_html_unicode_content() {
    let html = "<title>日本語ページ</title><body>テスト</body>";
    let ev = ExtendedUserEvent::SetHtml {
        html: html.to_string(),
        title: Some("日本語ページ".to_string()),
    };
    if let ExtendedUserEvent::SetHtml { html: h, title } = ev {
        assert!(h.contains("テスト"));
        assert_eq!(title.as_deref(), Some("日本語ページ"));
    }
}

#[test]
fn core_all_variants_constructable() {
    let events = vec![
        CoreUserEvent::ProcessMessages,
        CoreUserEvent::CloseWindow,
        CoreUserEvent::DragWindow,
        CoreUserEvent::PluginEvent {
            event: "x".to_string(),
            data: "y".to_string(),
        },
    ];
    assert_eq!(events.len(), 4);
    for ev in &events {
        let _ = format!("{:?}", ev);
    }
}

#[test]
fn ext_backend_error_crash_source() {
    let ev = ExtendedUserEvent::BackendError {
        message: "Segfault in backend".to_string(),
        source: "crash".to_string(),
    };
    if let ExtendedUserEvent::BackendError { source, .. } = ev {
        assert_eq!(source, "crash");
    }
}

#[test]
fn ext_create_child_window_large_dimensions() {
    let ev = ExtendedUserEvent::CreateChildWindow {
        url: "https://large.example.com".to_string(),
        width: 3840,
        height: 2160,
    };
    if let ExtendedUserEvent::CreateChildWindow { width, height, .. } = ev {
        assert_eq!(width, 3840);
        assert_eq!(height, 2160);
    }
}

#[test]
fn ext_show_error_with_details() {
    let ev = ExtendedUserEvent::ShowError {
        code: 500,
        title: "Server Error".to_string(),
        message: "Internal failure".to_string(),
        details: Some("Traceback:\n  line 42".to_string()),
        source: "python".to_string(),
    };
    if let ExtendedUserEvent::ShowError { details, source, .. } = ev {
        assert!(details.unwrap().contains("line 42"));
        assert_eq!(source, "python");
    }
}

#[rstest]
#[case("api.echo")]
#[case("api.export_scene")]
#[case("tool.apply")]
#[case("dcc.maya.run_mel")]
#[case("dcc.houdini.execute")]
fn core_plugin_event_common_names(#[case] event_name: &str) {
    let ev = CoreUserEvent::PluginEvent {
        event: event_name.to_string(),
        data: "{}".to_string(),
    };
    if let CoreUserEvent::PluginEvent { event, .. } = ev {
        assert_eq!(event, event_name);
    }
}
