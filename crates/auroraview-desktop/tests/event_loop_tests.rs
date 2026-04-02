//! Tests for UserEvent enum

use auroraview_desktop::event_loop::UserEvent;
use rstest::rstest;

// ============================================================================
// UserEvent construction and Debug
// ============================================================================

#[rstest]
fn test_user_event_close_window() {
    let event = UserEvent::CloseWindow;
    let debug = format!("{:?}", event);
    assert!(debug.contains("CloseWindow"));
}

#[rstest]
fn test_user_event_show_window() {
    let event = UserEvent::ShowWindow;
    let debug = format!("{:?}", event);
    assert!(debug.contains("ShowWindow"));
}

#[rstest]
fn test_user_event_hide_window() {
    let event = UserEvent::HideWindow;
    let debug = format!("{:?}", event);
    assert!(debug.contains("HideWindow"));
}

#[rstest]
fn test_user_event_drag_window() {
    let event = UserEvent::DragWindow;
    let debug = format!("{:?}", event);
    assert!(debug.contains("DragWindow"));
}

#[rstest]
fn test_user_event_plugin_event() {
    let event = UserEvent::PluginEvent {
        event: "scene_loaded".to_string(),
        data: r#"{"path":"/tmp/scene.ma"}"#.to_string(),
    };
    match &event {
        UserEvent::PluginEvent { event: e, data: d } => {
            assert_eq!(e, "scene_loaded");
            assert!(d.contains("scene.ma"));
        }
        _ => panic!("expected PluginEvent"),
    }
}

#[rstest]
fn test_user_event_plugin_event_debug() {
    let event = UserEvent::PluginEvent {
        event: "test_event".to_string(),
        data: "{}".to_string(),
    };
    let debug = format!("{:?}", event);
    assert!(debug.contains("PluginEvent"));
    assert!(debug.contains("test_event"));
}

#[rstest]
fn test_user_event_eval_js() {
    let script = "console.log('hello');".to_string();
    let event = UserEvent::EvalJs(script.clone());
    match &event {
        UserEvent::EvalJs(s) => assert_eq!(s, &script),
        _ => panic!("expected EvalJs"),
    }
}

#[rstest]
fn test_user_event_eval_js_debug() {
    let event = UserEvent::EvalJs("alert(1)".to_string());
    let debug = format!("{:?}", event);
    assert!(debug.contains("EvalJs"));
    assert!(debug.contains("alert"));
}

#[rstest]
fn test_user_event_wake_up() {
    let event = UserEvent::WakeUp;
    let debug = format!("{:?}", event);
    assert!(debug.contains("WakeUp"));
}

// ============================================================================
// Clone semantics
// ============================================================================

#[rstest]
fn test_user_event_clone_close_window() {
    let event = UserEvent::CloseWindow;
    let cloned = event.clone();
    assert!(matches!(cloned, UserEvent::CloseWindow));
}

#[rstest]
fn test_user_event_clone_show_window() {
    let event = UserEvent::ShowWindow;
    let cloned = event.clone();
    assert!(matches!(cloned, UserEvent::ShowWindow));
}

#[rstest]
fn test_user_event_clone_hide_window() {
    let event = UserEvent::HideWindow;
    let cloned = event.clone();
    assert!(matches!(cloned, UserEvent::HideWindow));
}

#[rstest]
fn test_user_event_clone_drag_window() {
    let event = UserEvent::DragWindow;
    let cloned = event.clone();
    assert!(matches!(cloned, UserEvent::DragWindow));
}

#[rstest]
fn test_user_event_clone_wake_up() {
    let event = UserEvent::WakeUp;
    let cloned = event.clone();
    assert!(matches!(cloned, UserEvent::WakeUp));
}

#[rstest]
fn test_user_event_clone_plugin_event() {
    let event = UserEvent::PluginEvent {
        event: "e".to_string(),
        data: "d".to_string(),
    };
    let cloned = event.clone();
    match cloned {
        UserEvent::PluginEvent { event: e, data: d } => {
            assert_eq!(e, "e");
            assert_eq!(d, "d");
        }
        _ => panic!("expected PluginEvent"),
    }
}

#[rstest]
fn test_user_event_clone_eval_js() {
    let event = UserEvent::EvalJs("1+1".to_string());
    let cloned = event.clone();
    match cloned {
        UserEvent::EvalJs(s) => assert_eq!(s, "1+1"),
        _ => panic!("expected EvalJs"),
    }
}

// ============================================================================
// Parameterized variant coverage
// ============================================================================

#[rstest]
#[case(UserEvent::CloseWindow, "CloseWindow")]
#[case(UserEvent::ShowWindow, "ShowWindow")]
#[case(UserEvent::HideWindow, "HideWindow")]
#[case(UserEvent::DragWindow, "DragWindow")]
#[case(UserEvent::WakeUp, "WakeUp")]
fn test_user_event_debug_variants(#[case] event: UserEvent, #[case] expected: &str) {
    let debug = format!("{:?}", event);
    assert!(debug.contains(expected));
}

#[rstest]
#[case("scene_open", r#"{"file":"test.ma"}"#)]
#[case("frame_change", r#"{"frame":42}"#)]
#[case("selection_changed", r#"{"count":3}"#)]
fn test_user_event_plugin_event_data_roundtrip(#[case] event_name: &str, #[case] payload: &str) {
    let event = UserEvent::PluginEvent {
        event: event_name.to_string(),
        data: payload.to_string(),
    };
    match event {
        UserEvent::PluginEvent { event: e, data: d } => {
            assert_eq!(e, event_name);
            assert_eq!(d, payload);
        }
        _ => panic!("expected PluginEvent"),
    }
}

#[rstest]
#[case("document.title")]
#[case("window.location.href")]
#[case("auroraview.api.echo('test')")]
fn test_user_event_eval_js_scripts(#[case] script: &str) {
    let event = UserEvent::EvalJs(script.to_string());
    match event {
        UserEvent::EvalJs(s) => assert_eq!(s, script),
        _ => panic!("expected EvalJs"),
    }
}
