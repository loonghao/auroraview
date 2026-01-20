//! Tests for AG-UI and A2UI protocol implementations

use auroraview_ai_agent::protocol::agui::{
    AGUIEmitter, AGUIEvent, BaseEvent, CallbackEmitter, EventType, JsonPatchOp, NoOpEmitter,
};
use auroraview_ai_agent::protocol::a2ui::{builders, NotifyLevel, UIAction, UIComponentType};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

#[test]
fn test_agui_event_types() {
    let event = AGUIEvent::RunStarted {
        run_id: "run_1".to_string(),
        thread_id: "thread_1".to_string(),
        base: BaseEvent::now(),
    };

    assert_eq!(event.event_type(), EventType::RunStarted);
    assert!(event.timestamp().is_some());
}

#[test]
fn test_agui_event_serialization() {
    let event = AGUIEvent::TextMessageContent {
        message_id: "msg_123".to_string(),
        delta: "Hello, World!".to_string(),
        base: BaseEvent::now(),
    };

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("TEXT_MESSAGE_CONTENT"));
    assert!(json.contains("Hello, World!"));
    assert!(json.contains("msg_123"));
}

#[test]
fn test_callback_emitter() {
    let count = Arc::new(AtomicU32::new(0));
    let count_clone = count.clone();

    let emitter = CallbackEmitter::new(move |_| {
        count_clone.fetch_add(1, Ordering::SeqCst);
    });

    emitter.run_started("run_1", "thread_1");
    emitter.text_start("msg_1", "assistant");
    emitter.text_delta("msg_1", "Hello");
    emitter.text_end("msg_1");
    emitter.run_finished("run_1", "thread_1");

    assert_eq!(count.load(Ordering::SeqCst), 5);
}

#[test]
fn test_noop_emitter() {
    let emitter = NoOpEmitter;

    // Should not panic
    emitter.run_started("run_1", "thread_1");
    emitter.text_start("msg_1", "assistant");
    emitter.text_delta("msg_1", "Hello");
    emitter.text_end("msg_1");
    emitter.run_finished("run_1", "thread_1");
}

#[test]
fn test_tool_call_events() {
    let events = vec![
        AGUIEvent::ToolCallStart {
            message_id: "msg_1".to_string(),
            tool_call_id: "tc_1".to_string(),
            tool_name: "search".to_string(),
            base: BaseEvent::now(),
        },
        AGUIEvent::ToolCallArgs {
            tool_call_id: "tc_1".to_string(),
            delta: r#"{"query": "rust"}"#.to_string(),
            base: BaseEvent::now(),
        },
        AGUIEvent::ToolCallEnd {
            tool_call_id: "tc_1".to_string(),
            base: BaseEvent::now(),
        },
        AGUIEvent::ToolCallResult {
            tool_call_id: "tc_1".to_string(),
            role: "tool".to_string(),
            content: r#"{"results": []}"#.to_string(),
            base: BaseEvent::now(),
        },
    ];

    for event in events {
        let json = serde_json::to_string(&event).unwrap();
        assert!(!json.is_empty());
    }
}

#[test]
fn test_json_patch_operations() {
    let ops = vec![
        JsonPatchOp::Add {
            path: "/status".to_string(),
            value: serde_json::json!("pending"),
        },
        JsonPatchOp::Replace {
            path: "/status".to_string(),
            value: serde_json::json!("completed"),
        },
        JsonPatchOp::Remove {
            path: "/temp".to_string(),
        },
    ];

    for op in ops {
        let json = serde_json::to_string(&op).unwrap();
        assert!(!json.is_empty());
    }
}

// A2UI Protocol Tests

#[test]
fn test_ui_component_builder() {
    let container = builders::container(vec![
        builders::heading("Welcome", 1),
        builders::text("This is a test."),
        builders::button("Click me", "handle_click"),
    ]);

    assert_eq!(container.component_type, UIComponentType::Container);
    assert_eq!(container.children.len(), 3);
}

#[test]
fn test_ui_action_serialization() {
    let action = UIAction::Notify {
        message: "Operation completed!".to_string(),
        level: NotifyLevel::Success,
        duration_ms: Some(3000),
    };

    let json = serde_json::to_string(&action).unwrap();
    assert!(json.contains("notify"));
    assert!(json.contains("success"));
    assert!(json.contains("3000"));
}

#[test]
fn test_ui_component_types() {
    let components = vec![
        builders::text("Hello"),
        builders::heading("Title", 1),
        builders::button("Click", "action"),
        builders::input("Enter text", "field"),
        builders::code("let x = 1;", Some("rust")),
        builders::markdown("# Markdown"),
        builders::progress(50.0, 100.0),
    ];

    for component in components {
        let json = serde_json::to_string(&component).unwrap();
        assert!(!json.is_empty());
    }
}

#[test]
fn test_notify_levels() {
    let levels = vec![
        NotifyLevel::Info,
        NotifyLevel::Success,
        NotifyLevel::Warning,
        NotifyLevel::Error,
    ];

    for level in levels {
        let json = serde_json::to_string(&level).unwrap();
        assert!(!json.is_empty());
    }

    // Default should be Info
    assert_eq!(NotifyLevel::default(), NotifyLevel::Info);
}
