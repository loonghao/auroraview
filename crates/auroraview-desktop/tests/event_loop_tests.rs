//! Tests for UserEvent enum

use auroraview_desktop::event_loop::UserEvent;
use rstest::rstest;

// ============================================================================
// UserEvent construction and Debug
// ============================================================================

#[rstest]
fn user_event_close_window() {
    let event = UserEvent::CloseWindow;
    let debug = format!("{:?}", event);
    assert!(debug.contains("CloseWindow"));
}

#[rstest]
fn user_event_show_window() {
    let event = UserEvent::ShowWindow;
    let debug = format!("{:?}", event);
    assert!(debug.contains("ShowWindow"));
}

#[rstest]
fn user_event_hide_window() {
    let event = UserEvent::HideWindow;
    let debug = format!("{:?}", event);
    assert!(debug.contains("HideWindow"));
}

#[rstest]
fn user_event_drag_window() {
    let event = UserEvent::DragWindow;
    let debug = format!("{:?}", event);
    assert!(debug.contains("DragWindow"));
}

#[rstest]
fn user_event_plugin_event() {
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
fn user_event_plugin_event_debug() {
    let event = UserEvent::PluginEvent {
        event: "test_event".to_string(),
        data: "{}".to_string(),
    };
    let debug = format!("{:?}", event);
    assert!(debug.contains("PluginEvent"));
    assert!(debug.contains("test_event"));
}

#[rstest]
fn user_event_eval_js() {
    let script = "console.log('hello');".to_string();
    let event = UserEvent::EvalJs(script.clone());
    match &event {
        UserEvent::EvalJs(s) => assert_eq!(s, &script),
        _ => panic!("expected EvalJs"),
    }
}

#[rstest]
fn user_event_eval_js_debug() {
    let event = UserEvent::EvalJs("alert(1)".to_string());
    let debug = format!("{:?}", event);
    assert!(debug.contains("EvalJs"));
    assert!(debug.contains("alert"));
}

#[rstest]
fn user_event_wake_up() {
    let event = UserEvent::WakeUp;
    let debug = format!("{:?}", event);
    assert!(debug.contains("WakeUp"));
}

// ============================================================================
// Clone semantics
// ============================================================================

#[rstest]
fn user_event_clone_close_window() {
    let event = UserEvent::CloseWindow;
    let cloned = event.clone();
    assert!(matches!(cloned, UserEvent::CloseWindow));
}

#[rstest]
fn user_event_clone_show_window() {
    let event = UserEvent::ShowWindow;
    let cloned = event.clone();
    assert!(matches!(cloned, UserEvent::ShowWindow));
}

#[rstest]
fn user_event_clone_hide_window() {
    let event = UserEvent::HideWindow;
    let cloned = event.clone();
    assert!(matches!(cloned, UserEvent::HideWindow));
}

#[rstest]
fn user_event_clone_drag_window() {
    let event = UserEvent::DragWindow;
    let cloned = event.clone();
    assert!(matches!(cloned, UserEvent::DragWindow));
}

#[rstest]
fn user_event_clone_wake_up() {
    let event = UserEvent::WakeUp;
    let cloned = event.clone();
    assert!(matches!(cloned, UserEvent::WakeUp));
}

#[rstest]
fn user_event_clone_plugin_event() {
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
fn user_event_clone_eval_js() {
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
fn user_event_debug_variants(#[case] event: UserEvent, #[case] expected: &str) {
    let debug = format!("{:?}", event);
    assert!(debug.contains(expected));
}

#[rstest]
#[case("scene_open", r#"{"file":"test.ma"}"#)]
#[case("frame_change", r#"{"frame":42}"#)]
#[case("selection_changed", r#"{"count":3}"#)]
fn user_event_plugin_event_data_roundtrip(#[case] event_name: &str, #[case] payload: &str) {
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
fn user_event_eval_js_scripts(#[case] script: &str) {
    let event = UserEvent::EvalJs(script.to_string());
    match event {
        UserEvent::EvalJs(s) => assert_eq!(s, script),
        _ => panic!("expected EvalJs"),
    }
}

// ============================================================================
// EvalJs edge cases
// ============================================================================

#[rstest]
fn user_event_eval_js_empty_string() {
    let event = UserEvent::EvalJs("".to_string());
    match event {
        UserEvent::EvalJs(s) => assert!(s.is_empty()),
        _ => panic!("expected EvalJs"),
    }
}

#[rstest]
fn user_event_eval_js_multiline_script() {
    let script = "var x = 1;\nvar y = 2;\nconsole.log(x + y);";
    let event = UserEvent::EvalJs(script.to_string());
    match &event {
        UserEvent::EvalJs(s) => {
            assert!(s.contains('\n'));
            assert_eq!(s, script);
        }
        _ => panic!("expected EvalJs"),
    }
}

#[rstest]
fn user_event_eval_js_unicode_content() {
    let script = "document.title = '中文标题';";
    let event = UserEvent::EvalJs(script.to_string());
    match event {
        UserEvent::EvalJs(s) => assert!(s.contains("中文标题")),
        _ => panic!("expected EvalJs"),
    }
}

#[rstest]
fn user_event_eval_js_json_payload() {
    let script = r#"auroraview.api.load({"key":"value","num":42});"#;
    let event = UserEvent::EvalJs(script.to_string());
    match event {
        UserEvent::EvalJs(s) => {
            assert!(s.contains("auroraview"));
            assert!(s.contains("value"));
        }
        _ => panic!("expected EvalJs"),
    }
}

// ============================================================================
// PluginEvent edge cases
// ============================================================================

#[rstest]
fn user_event_plugin_event_empty_data() {
    let event = UserEvent::PluginEvent {
        event: "ready".to_string(),
        data: "".to_string(),
    };
    match event {
        UserEvent::PluginEvent { event: e, data: d } => {
            assert_eq!(e, "ready");
            assert!(d.is_empty());
        }
        _ => panic!("expected PluginEvent"),
    }
}

#[rstest]
fn user_event_plugin_event_nested_json() {
    let data = r#"{"scene":{"name":"test","objects":[{"id":1},{"id":2}]}}"#;
    let event = UserEvent::PluginEvent {
        event: "scene_loaded".to_string(),
        data: data.to_string(),
    };
    match event {
        UserEvent::PluginEvent { event: e, data: d } => {
            assert_eq!(e, "scene_loaded");
            assert!(d.contains("objects"));
        }
        _ => panic!("expected PluginEvent"),
    }
}

#[rstest]
fn user_event_plugin_event_unicode_event_name() {
    let event = UserEvent::PluginEvent {
        event: "maya.模型_加载".to_string(),
        data: "{}".to_string(),
    };
    match event {
        UserEvent::PluginEvent { event: e, .. } => {
            assert!(e.contains("maya"));
        }
        _ => panic!("expected PluginEvent"),
    }
}

// ============================================================================
// WakeUp is lightweight
// ============================================================================

#[rstest]
fn user_event_wake_up_clone_eq() {
    let a = UserEvent::WakeUp;
    let b = a.clone();
    // Both should be WakeUp
    assert!(matches!(a, UserEvent::WakeUp));
    assert!(matches!(b, UserEvent::WakeUp));
}

// ============================================================================
// Parameterized EvalJs clone
// ============================================================================

#[rstest]
#[case("alert('x')")]
#[case("window.close()")]
#[case("document.getElementById('app').innerHTML = '';")]
fn user_event_eval_js_clone(#[case] script: &str) {
    let event = UserEvent::EvalJs(script.to_string());
    let cloned = event.clone();
    match (event, cloned) {
        (UserEvent::EvalJs(a), UserEvent::EvalJs(b)) => assert_eq!(a, b),
        _ => panic!("expected EvalJs"),
    }
}

// ============================================================================
// Box / Vec usage (UserEvent can be stored in containers)
// ============================================================================

#[rstest]
fn user_event_vec_of_events() {
    let events: Vec<UserEvent> = vec![
        UserEvent::ShowWindow,
        UserEvent::EvalJs("init()".to_string()),
        UserEvent::PluginEvent {
            event: "ready".to_string(),
            data: "{}".to_string(),
        },
        UserEvent::HideWindow,
        UserEvent::CloseWindow,
    ];
    assert_eq!(events.len(), 5);
    assert!(matches!(events[0], UserEvent::ShowWindow));
    assert!(matches!(events[4], UserEvent::CloseWindow));
}

#[rstest]
fn user_event_send_across_thread() {
    use std::sync::mpsc;
    use std::thread;

    let (tx, rx) = mpsc::channel::<UserEvent>();

    let events = vec![
        UserEvent::ShowWindow,
        UserEvent::EvalJs("ping()".to_string()),
        UserEvent::WakeUp,
    ];

    let handle = thread::spawn(move || {
        for event in events {
            tx.send(event).unwrap();
        }
    });

    handle.join().unwrap();

    let received: Vec<UserEvent> = rx.try_iter().collect();
    assert_eq!(received.len(), 3);
    assert!(matches!(received[0], UserEvent::ShowWindow));
    assert!(matches!(received[2], UserEvent::WakeUp));
}

// ─── Additional coverage R9 ──────────────────────────────────────────────────

#[rstest]
fn user_event_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<UserEvent>();
}

#[rstest]
fn user_event_close_window_is_close_window() {
    let event = UserEvent::CloseWindow;
    assert!(matches!(event, UserEvent::CloseWindow));
}

#[rstest]
fn user_event_show_window_is_show_window() {
    let event = UserEvent::ShowWindow;
    assert!(matches!(event, UserEvent::ShowWindow));
}

#[rstest]
fn user_event_hide_window_is_hide_window() {
    let event = UserEvent::HideWindow;
    assert!(matches!(event, UserEvent::HideWindow));
}

#[rstest]
fn user_event_wake_up_is_wake_up() {
    let event = UserEvent::WakeUp;
    assert!(matches!(event, UserEvent::WakeUp));
}

#[rstest]
fn user_event_drag_window_is_drag_window() {
    let event = UserEvent::DragWindow;
    assert!(matches!(event, UserEvent::DragWindow));
}

#[rstest]
fn user_event_eval_js_large_script() {
    let large_script = "console.log('x');".repeat(100);
    let event = UserEvent::EvalJs(large_script.clone());
    match event {
        UserEvent::EvalJs(s) => assert_eq!(s.len(), large_script.len()),
        _ => panic!("expected EvalJs"),
    }
}

#[rstest]
fn user_event_plugin_event_new_format() {
    let ev = UserEvent::PluginEvent {
        event: "selection_changed".to_string(),
        data: r#"{"count":5}"#.to_string(),
    };
    match ev {
        UserEvent::PluginEvent { event, data } => {
            assert_eq!(event, "selection_changed");
            assert!(data.contains("count"));
        }
        _ => panic!("expected PluginEvent"),
    }
}

#[rstest]
fn user_event_all_variants_can_be_constructed() {
    let _events: Vec<UserEvent> = vec![
        UserEvent::CloseWindow,
        UserEvent::ShowWindow,
        UserEvent::HideWindow,
        UserEvent::DragWindow,
        UserEvent::EvalJs("test".to_string()),
        UserEvent::WakeUp,
        UserEvent::PluginEvent { event: "e".to_string(), data: "{}".to_string() },
    ];
    assert_eq!(_events.len(), 7);
}

#[rstest]
fn user_event_eval_js_empty_then_nonempty() {
    let empty = UserEvent::EvalJs(String::new());
    let nonempty = UserEvent::EvalJs("code()".to_string());
    match empty {
        UserEvent::EvalJs(s) => assert!(s.is_empty()),
        _ => panic!()
    }
    match nonempty {
        UserEvent::EvalJs(s) => assert!(!s.is_empty()),
        _ => panic!()
    }
}

#[rstest]
fn user_event_plugin_event_empty_fields() {
    let ev = UserEvent::PluginEvent {
        event: String::new(),
        data: String::new(),
    };
    match ev {
        UserEvent::PluginEvent { event, data } => {
            assert!(event.is_empty());
            assert!(data.is_empty());
        }
        _ => panic!("expected PluginEvent"),
    }
}

#[rstest]
#[case("auroraview://init")]
#[case("menu.file.open")]
#[case("dcc.maya.selection")]
fn user_event_plugin_event_various_names(#[case] name: &str) {
    let ev = UserEvent::PluginEvent {
        event: name.to_string(),
        data: "{}".to_string(),
    };
    match ev {
        UserEvent::PluginEvent { event, .. } => assert_eq!(event, name),
        _ => panic!("expected PluginEvent"),
    }
}

#[rstest]
fn user_event_wake_up_can_be_in_vec() {
    let v: Vec<UserEvent> = vec![UserEvent::WakeUp, UserEvent::WakeUp, UserEvent::WakeUp];
    assert_eq!(v.len(), 3);
    assert!(v.iter().all(|e| matches!(e, UserEvent::WakeUp)));
}
