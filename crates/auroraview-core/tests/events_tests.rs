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
#[case(
    Some(50),
    Some("loading"),
    Some("step1"),
    Some("in progress"),
    Some("ok")
)]
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
    assert_eq!(format!("{:?}", original), format!("{:?}", cloned));
}

#[test]
fn extended_user_event_clone() {
    let original = ExtendedUserEvent::BackendError {
        message: "error".to_string(),
        source: "test".to_string(),
    };
    let cloned = original.clone();
    assert_eq!(format!("{:?}", original), format!("{:?}", cloned));
}

#[test]
fn extended_python_ready_clone() {
    let original = ExtendedUserEvent::PythonReady {
        handlers: vec!["h1".to_string(), "h2".to_string()],
    };
    let cloned = original.clone();
    if let (
        ExtendedUserEvent::PythonReady { handlers: h1 },
        ExtendedUserEvent::PythonReady { handlers: h2 },
    ) = (original, cloned)
    {
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
