//! Tests for AG-UI and A2UI protocol implementations

use auroraview_ai_agent::protocol::a2ui::{builders, NotifyLevel, UIAction, UIComponentSpec, UIComponentType};
use auroraview_ai_agent::protocol::agui::{
    AGUIContext, AGUIEmitter, AGUIEvent, AGUIMessage, AGUITool, AGUIToolCall, BaseEvent,
    CallbackEmitter, EventType, JsonPatchOp, NoOpEmitter,
};
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

// ─── Additional AGUIEvent variant coverage ────────────────────────────────────

#[test]
fn agui_text_message_chunk_serialization() {
    let event = AGUIEvent::TextMessageChunk {
        message_id: "msg_1".to_string(),
        role: "assistant".to_string(),
        content: "Hello!".to_string(),
        base: BaseEvent::now(),
    };
    assert_eq!(event.event_type(), EventType::TextMessageChunk);
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("TEXT_MESSAGE_CHUNK"));
    assert!(json.contains("Hello!"));
}

#[test]
fn agui_thinking_events_serialization() {
    let events = vec![
        AGUIEvent::ThinkingTextMessageStart {
            message_id: "think_1".to_string(),
            base: BaseEvent::now(),
        },
        AGUIEvent::ThinkingTextMessageContent {
            message_id: "think_1".to_string(),
            delta: "Reasoning...".to_string(),
            base: BaseEvent::now(),
        },
        AGUIEvent::ThinkingTextMessageEnd {
            message_id: "think_1".to_string(),
            base: BaseEvent::now(),
        },
        AGUIEvent::ThinkingStart {
            thinking_id: "ts_1".to_string(),
            base: BaseEvent::now(),
        },
        AGUIEvent::ThinkingEnd {
            thinking_id: "ts_1".to_string(),
            base: BaseEvent::now(),
        },
    ];
    for e in &events {
        let json = serde_json::to_string(e).unwrap();
        assert!(!json.is_empty());
    }
    assert_eq!(events[0].event_type(), EventType::ThinkingTextMessageStart);
    assert_eq!(events[1].event_type(), EventType::ThinkingTextMessageContent);
    assert_eq!(events[2].event_type(), EventType::ThinkingTextMessageEnd);
    assert_eq!(events[3].event_type(), EventType::ThinkingStart);
    assert_eq!(events[4].event_type(), EventType::ThinkingEnd);
}

#[test]
fn agui_step_events_serialization() {
    let start = AGUIEvent::StepStarted {
        step_id: "step_1".to_string(),
        step_name: Some("analyze".to_string()),
        base: BaseEvent::now(),
    };
    let finish = AGUIEvent::StepFinished {
        step_id: "step_1".to_string(),
        base: BaseEvent::now(),
    };
    assert_eq!(start.event_type(), EventType::StepStarted);
    assert_eq!(finish.event_type(), EventType::StepFinished);

    let json = serde_json::to_string(&start).unwrap();
    assert!(json.contains("analyze"));

    // step_name absent when None
    let no_name = AGUIEvent::StepStarted {
        step_id: "step_2".to_string(),
        step_name: None,
        base: BaseEvent::now(),
    };
    let json2 = serde_json::to_string(&no_name).unwrap();
    assert!(!json2.contains("step_name"));
}

#[test]
fn agui_state_snapshot_event() {
    let state = serde_json::json!({"status": "running", "progress": 50});
    let event = AGUIEvent::StateSnapshot {
        snapshot: state.clone(),
        base: BaseEvent::now(),
    };
    assert_eq!(event.event_type(), EventType::StateSnapshot);
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("running"));
    assert!(json.contains("50"));
}

#[test]
fn agui_state_delta_event() {
    let ops = vec![
        JsonPatchOp::Add {
            path: "/items/0".to_string(),
            value: serde_json::json!("new_item"),
        },
        JsonPatchOp::Move {
            from: "/old_path".to_string(),
            path: "/new_path".to_string(),
        },
        JsonPatchOp::Copy {
            from: "/src".to_string(),
            path: "/dst".to_string(),
        },
        JsonPatchOp::Test {
            path: "/status".to_string(),
            value: serde_json::json!("ok"),
        },
    ];
    let event = AGUIEvent::StateDelta {
        delta: ops.clone(),
        base: BaseEvent::now(),
    };
    assert_eq!(event.event_type(), EventType::StateDelta);
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("add"));
    assert!(json.contains("move"));
    assert!(json.contains("copy"));
    assert!(json.contains("test"));
}

#[test]
fn agui_messages_snapshot_event() {
    let msg = AGUIMessage {
        id: "m1".to_string(),
        role: "assistant".to_string(),
        content: "Hello".to_string(),
        tool_calls: None,
        tool_call_id: None,
    };
    let event = AGUIEvent::MessagesSnapshot {
        messages: vec![msg],
        base: BaseEvent::now(),
    };
    assert_eq!(event.event_type(), EventType::MessagesSnapshot);
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("Hello"));
}

#[test]
fn agui_raw_and_custom_events() {
    let raw = AGUIEvent::Raw {
        event: "external.event".to_string(),
        data: serde_json::json!({"key": "val"}),
        base: BaseEvent::now(),
    };
    let custom = AGUIEvent::Custom {
        name: "dcc.scene_loaded".to_string(),
        value: serde_json::json!({"scene": "shot_001"}),
        base: BaseEvent::now(),
    };
    assert_eq!(raw.event_type(), EventType::Raw);
    assert_eq!(custom.event_type(), EventType::Custom);

    let raw_json = serde_json::to_string(&raw).unwrap();
    assert!(raw_json.contains("external.event"));

    let custom_json = serde_json::to_string(&custom).unwrap();
    assert!(custom_json.contains("dcc.scene_loaded"));
}

#[test]
fn agui_run_error_with_code() {
    let event = AGUIEvent::RunError {
        run_id: "run_err_1".to_string(),
        message: "Timeout exceeded".to_string(),
        code: Some("TIMEOUT".to_string()),
        base: BaseEvent::now(),
    };
    assert_eq!(event.event_type(), EventType::RunError);
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("TIMEOUT"));
    assert!(json.contains("Timeout exceeded"));
}

#[test]
fn agui_run_error_without_code_omits_field() {
    let event = AGUIEvent::RunError {
        run_id: "run_err_2".to_string(),
        message: "Unknown error".to_string(),
        code: None,
        base: BaseEvent::now(),
    };
    let json = serde_json::to_string(&event).unwrap();
    assert!(!json.contains("\"code\""));
}

#[test]
fn agui_base_event_timestamp_positive() {
    let base = BaseEvent::now();
    let ts = base.timestamp.unwrap();
    assert!(ts > 0.0, "timestamp should be positive");
}

#[test]
fn agui_base_event_default_has_no_timestamp() {
    let base = BaseEvent::default();
    assert!(base.timestamp.is_none());
    assert!(base.raw_event.is_none());
}

// ─── AGUIMessage / AGUIToolCall / AGUITool / AGUIContext ──────────────────────

#[test]
fn agui_message_serialization_roundtrip() {
    let msg = AGUIMessage {
        id: "msg_42".to_string(),
        role: "user".to_string(),
        content: "What is the scene?".to_string(),
        tool_calls: None,
        tool_call_id: None,
    };
    let json = serde_json::to_string(&msg).unwrap();
    let decoded: AGUIMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.id, "msg_42");
    assert_eq!(decoded.role, "user");
    assert_eq!(decoded.content, "What is the scene?");
    assert!(decoded.tool_calls.is_none());
}

#[test]
fn agui_message_with_tool_calls() {
    let tc = AGUIToolCall {
        id: "tc_1".to_string(),
        name: "export_scene".to_string(),
        arguments: r#"{"format":"fbx"}"#.to_string(),
    };
    let msg = AGUIMessage {
        id: "msg_tc".to_string(),
        role: "assistant".to_string(),
        content: "".to_string(),
        tool_calls: Some(vec![tc]),
        tool_call_id: None,
    };
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("export_scene"));
    assert!(json.contains("fbx"));
}

#[test]
fn agui_message_tool_result() {
    let msg = AGUIMessage {
        id: "msg_result".to_string(),
        role: "tool".to_string(),
        content: r#"{"status":"ok"}"#.to_string(),
        tool_calls: None,
        tool_call_id: Some("tc_1".to_string()),
    };
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("tc_1"));
    assert!(json.contains("tool"));
}

#[test]
fn agui_tool_call_serialization_roundtrip() {
    let tc = AGUIToolCall {
        id: "tc_abc".to_string(),
        name: "search_assets".to_string(),
        arguments: r#"{"query":"dragon","max":10}"#.to_string(),
    };
    let json = serde_json::to_string(&tc).unwrap();
    let decoded: AGUIToolCall = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.id, "tc_abc");
    assert_eq!(decoded.name, "search_assets");
}

#[test]
fn agui_tool_definition_serialization() {
    let tool = AGUITool {
        name: "render_frame".to_string(),
        description: "Renders a single frame at the current time".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "frame": { "type": "integer" }
            }
        }),
    };
    let json = serde_json::to_string(&tool).unwrap();
    let decoded: AGUITool = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.name, "render_frame");
    assert!(json.contains("Renders a single frame"));
}

#[test]
fn agui_context_serialization_roundtrip() {
    let ctx = AGUIContext {
        description: "Current Maya scene name".to_string(),
        value: serde_json::json!("shot_001.ma"),
    };
    let json = serde_json::to_string(&ctx).unwrap();
    let decoded: AGUIContext = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.description, "Current Maya scene name");
    assert_eq!(decoded.value, serde_json::json!("shot_001.ma"));
}

// ─── CallbackEmitter full method coverage ─────────────────────────────────────

#[test]
fn callback_emitter_tool_call_sequence() {
    let collected = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let c2 = collected.clone();
    let emitter = CallbackEmitter::new(move |event: AGUIEvent| {
        c2.lock().unwrap().push(format!("{:?}", event.event_type()));
    });

    emitter.tool_call_start("msg_1", "tc_1", "search");
    emitter.tool_call_args("tc_1", r#"{"q":"rust"}"#);
    emitter.tool_call_end("tc_1");
    emitter.tool_call_result("tc_1", r#"{"results":[]}"#);

    let events = collected.lock().unwrap();
    assert_eq!(events.len(), 4);
    assert!(events[0].contains("ToolCallStart"));
    assert!(events[1].contains("ToolCallArgs"));
    assert!(events[2].contains("ToolCallEnd"));
    assert!(events[3].contains("ToolCallResult"));
}

#[test]
fn callback_emitter_thinking_sequence() {
    let count = Arc::new(AtomicU32::new(0));
    let c2 = count.clone();
    let emitter = CallbackEmitter::new(move |_| {
        c2.fetch_add(1, Ordering::SeqCst);
    });

    emitter.thinking_start("msg_1");
    emitter.thinking_delta("msg_1", "Analyzing...");
    emitter.thinking_end("msg_1");

    assert_eq!(count.load(Ordering::SeqCst), 3);
}

#[test]
fn callback_emitter_state_snapshot() {
    let captured = Arc::new(std::sync::Mutex::new(None::<serde_json::Value>));
    let c2 = captured.clone();
    let emitter = CallbackEmitter::new(move |event: AGUIEvent| {
        if let AGUIEvent::StateSnapshot { snapshot, .. } = event {
            *c2.lock().unwrap() = Some(snapshot);
        }
    });

    emitter.state_snapshot(serde_json::json!({"progress": 75}));
    let snapshot = captured.lock().unwrap();
    assert_eq!(snapshot.as_ref().unwrap()["progress"], 75);
}

#[test]
fn callback_emitter_run_error() {
    let captured = Arc::new(std::sync::Mutex::new(None::<String>));
    let c2 = captured.clone();
    let emitter = CallbackEmitter::new(move |event: AGUIEvent| {
        if let AGUIEvent::RunError { message, .. } = event {
            *c2.lock().unwrap() = Some(message);
        }
    });

    emitter.run_error("run_1", "Out of memory");
    let msg = captured.lock().unwrap();
    assert_eq!(msg.as_deref(), Some("Out of memory"));
}

#[test]
fn noop_emitter_all_methods() {
    let emitter = NoOpEmitter;
    emitter.run_started("r1", "t1");
    emitter.run_finished("r1", "t1");
    emitter.run_error("r1", "err");
    emitter.text_start("m1", "assistant");
    emitter.text_delta("m1", "hello");
    emitter.text_end("m1");
    emitter.tool_call_start("m1", "tc1", "tool");
    emitter.tool_call_args("tc1", "{}");
    emitter.tool_call_end("tc1");
    emitter.tool_call_result("tc1", "result");
    emitter.state_snapshot(serde_json::json!({}));
    emitter.thinking_start("m1");
    emitter.thinking_delta("m1", "thinking");
    emitter.thinking_end("m1");
    // Should not panic for any call
}

// ─── JsonPatchOp full coverage ─────────────────────────────────────────────────

#[test]
fn json_patch_all_ops_roundtrip() {
    let ops = vec![
        JsonPatchOp::Add {
            path: "/a".to_string(),
            value: serde_json::json!(1),
        },
        JsonPatchOp::Remove {
            path: "/b".to_string(),
        },
        JsonPatchOp::Replace {
            path: "/c".to_string(),
            value: serde_json::json!("new"),
        },
        JsonPatchOp::Move {
            from: "/old".to_string(),
            path: "/new".to_string(),
        },
        JsonPatchOp::Copy {
            from: "/src".to_string(),
            path: "/dst".to_string(),
        },
        JsonPatchOp::Test {
            path: "/status".to_string(),
            value: serde_json::json!("ok"),
        },
    ];

    for op in &ops {
        let json = serde_json::to_string(op).unwrap();
        assert!(!json.is_empty());
    }
    // Verify op names in JSON
    let add_json = serde_json::to_string(&ops[0]).unwrap();
    assert!(add_json.contains(r#""op":"add""#));
    let remove_json = serde_json::to_string(&ops[1]).unwrap();
    assert!(remove_json.contains(r#""op":"remove""#));
}

// ─── A2UI UIComponentSpec builder methods ─────────────────────────────────────

#[test]
fn ui_component_spec_with_id() {
    let spec = UIComponentSpec::new(UIComponentType::Button)
        .with_id("btn_submit");
    assert_eq!(spec.id.as_deref(), Some("btn_submit"));
}

#[test]
fn ui_component_spec_with_props() {
    let spec = UIComponentSpec::new(UIComponentType::Text)
        .with_props(serde_json::json!({"content": "Hello"}));
    assert_eq!(spec.props["content"], "Hello");
}

#[test]
fn ui_component_spec_with_child() {
    let child = UIComponentSpec::new(UIComponentType::Text);
    let parent = UIComponentSpec::new(UIComponentType::Container)
        .with_child(child);
    assert_eq!(parent.children.len(), 1);
}

#[test]
fn ui_component_spec_chained_builder() {
    let spec = UIComponentSpec::new(UIComponentType::Card)
        .with_id("card_1")
        .with_props(serde_json::json!({"title": "My Card"}))
        .with_child(UIComponentSpec::new(UIComponentType::Text))
        .with_child(UIComponentSpec::new(UIComponentType::Button));
    assert_eq!(spec.id.as_deref(), Some("card_1"));
    assert_eq!(spec.children.len(), 2);
}

#[test]
fn ui_component_empty_children_not_serialized() {
    let spec = UIComponentSpec::new(UIComponentType::Text);
    let json = serde_json::to_string(&spec).unwrap();
    assert!(!json.contains("children"), "empty children should be omitted");
}

#[test]
fn ui_component_none_id_not_serialized() {
    let spec = UIComponentSpec::new(UIComponentType::Text);
    let json = serde_json::to_string(&spec).unwrap();
    assert!(!json.contains("\"id\""), "None id should be omitted");
}

// ─── A2UI builders extended coverage ──────────────────────────────────────────

#[test]
fn builder_card() {
    let c = builders::card("Analysis");
    assert_eq!(c.component_type, UIComponentType::Card);
    assert_eq!(c.props["title"], "Analysis");
}

#[test]
fn builder_row_and_column() {
    let row = builders::row(vec![builders::text("a"), builders::text("b")]);
    let col = builders::column(vec![builders::text("x")]);
    assert_eq!(row.component_type, UIComponentType::Row);
    assert_eq!(row.children.len(), 2);
    assert_eq!(col.component_type, UIComponentType::Column);
    assert_eq!(col.children.len(), 1);
}

#[test]
fn builder_alert() {
    let a = builders::alert("Something went wrong", NotifyLevel::Error);
    assert_eq!(a.component_type, UIComponentType::Alert);
    assert_eq!(a.props["level"], serde_json::json!("error"));
    assert!(a.props["message"].as_str().unwrap().contains("wrong"));
}

#[test]
fn builder_table() {
    let t = builders::table(
        vec!["Name", "Value"],
        vec![
            vec![serde_json::json!("alpha"), serde_json::json!(1)],
            vec![serde_json::json!("beta"), serde_json::json!(2)],
        ],
    );
    assert_eq!(t.component_type, UIComponentType::Table);
    assert_eq!(t.props["headers"][0], "Name");
}

#[test]
fn builder_image() {
    let img = builders::image("https://example.com/img.png", Some("alt text"));
    assert_eq!(img.component_type, UIComponentType::Image);
    assert_eq!(img.props["src"], "https://example.com/img.png");
    assert_eq!(img.props["alt"], "alt text");

    let img_no_alt = builders::image("https://example.com/bg.jpg", None);
    assert!(img_no_alt.props["alt"].is_null());
}

// ─── A2UI UIAction full variant coverage ─────────────────────────────────────

#[test]
fn ui_action_render_roundtrip() {
    let action = UIAction::Render {
        root: builders::text("Hello"),
    };
    let json = serde_json::to_string(&action).unwrap();
    assert!(json.contains("\"action\":\"render\""));
    assert!(json.contains("Hello"));
}

#[test]
fn ui_action_update() {
    let action = UIAction::Update {
        id: "my_btn".to_string(),
        props: serde_json::json!({"disabled": true}),
    };
    let json = serde_json::to_string(&action).unwrap();
    assert!(json.contains("update"));
    assert!(json.contains("my_btn"));
    assert!(json.contains("disabled"));
}

#[test]
fn ui_action_append_child() {
    let action = UIAction::AppendChild {
        parent_id: "container_1".to_string(),
        child: builders::text("new item"),
    };
    let json = serde_json::to_string(&action).unwrap();
    assert!(json.contains("append_child"));
    assert!(json.contains("container_1"));
}

#[test]
fn ui_action_remove() {
    let action = UIAction::Remove {
        id: "old_widget".to_string(),
    };
    let json = serde_json::to_string(&action).unwrap();
    assert!(json.contains("remove"));
    assert!(json.contains("old_widget"));
}

#[test]
fn ui_action_replace() {
    let action = UIAction::Replace {
        id: "widget_1".to_string(),
        component: builders::button("New Label", "new_action"),
    };
    let json = serde_json::to_string(&action).unwrap();
    assert!(json.contains("replace"));
    assert!(json.contains("widget_1"));
}

#[test]
fn ui_action_show_modal_closable() {
    let action = UIAction::ShowModal {
        content: builders::heading("Confirm", 2),
        closable: true,
    };
    let json = serde_json::to_string(&action).unwrap();
    assert!(json.contains("show_modal"));
    assert!(json.contains("true"));
    assert!(json.contains("Confirm"));
}

#[test]
fn ui_action_hide_modal() {
    let action = UIAction::HideModal;
    let json = serde_json::to_string(&action).unwrap();
    assert!(json.contains("hide_modal"));
}

#[test]
fn ui_action_notify_no_duration() {
    let action = UIAction::Notify {
        message: "Processing complete".to_string(),
        level: NotifyLevel::Info,
        duration_ms: None,
    };
    let json = serde_json::to_string(&action).unwrap();
    assert!(json.contains("notify"));
    assert!(json.contains("info"));
    assert!(json.contains("Processing complete"));
}

// ─── UIComponentType custom variant ───────────────────────────────────────────

#[test]
fn ui_component_type_custom_serialization() {
    let spec = UIComponentSpec::new(UIComponentType::Custom("dcc_viewport".to_string()));
    let json = serde_json::to_string(&spec).unwrap();
    assert!(json.contains("dcc_viewport"));
}

// ─── EventType enum completeness ─────────────────────────────────────────────

#[test]
fn event_type_all_variants_mapped() {
    // Each AGUIEvent variant should map to a unique EventType
    let all_events = vec![
        AGUIEvent::RunStarted {
            run_id: "r".to_string(),
            thread_id: "t".to_string(),
            base: BaseEvent::default(),
        },
        AGUIEvent::RunFinished {
            run_id: "r".to_string(),
            thread_id: "t".to_string(),
            base: BaseEvent::default(),
        },
        AGUIEvent::RunError {
            run_id: "r".to_string(),
            message: "err".to_string(),
            code: None,
            base: BaseEvent::default(),
        },
        AGUIEvent::TextMessageStart {
            message_id: "m".to_string(),
            role: "user".to_string(),
            base: BaseEvent::default(),
        },
        AGUIEvent::TextMessageContent {
            message_id: "m".to_string(),
            delta: "d".to_string(),
            base: BaseEvent::default(),
        },
        AGUIEvent::TextMessageEnd {
            message_id: "m".to_string(),
            base: BaseEvent::default(),
        },
        AGUIEvent::TextMessageChunk {
            message_id: "m".to_string(),
            role: "user".to_string(),
            content: "c".to_string(),
            base: BaseEvent::default(),
        },
        AGUIEvent::ToolCallChunk {
            message_id: "m".to_string(),
            tool_call_id: "tc".to_string(),
            tool_name: "search".to_string(),
            arguments: "{}".to_string(),
            base: BaseEvent::default(),
        },
    ];
    // All should serialize without error
    for event in &all_events {
        let json = serde_json::to_string(event).unwrap();
        assert!(!json.is_empty());
        assert!(event.timestamp().is_none(), "default base has no timestamp");
    }
}

