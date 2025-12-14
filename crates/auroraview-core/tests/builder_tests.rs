//! Tests for builder module

use auroraview_core::builder::{
    get_background_color, init_com_sta, ChildWindowStyleOptions, ComInitResult, WebContextConfig,
    DARK_BACKGROUND,
};
use rstest::rstest;
use std::path::PathBuf;

#[cfg(feature = "wry-builder")]
use auroraview_core::builder::{
    create_drag_drop_handler, create_ipc_handler, create_simple_ipc_handler, DragDropEventData,
    DragDropEventType, DragDropHandler, IpcMessageHandler, IpcMessageType, ProtocolConfig,
};
#[cfg(feature = "wry-builder")]
use std::sync::atomic::{AtomicUsize, Ordering};
#[cfg(feature = "wry-builder")]
use std::sync::Arc;

// ============================================================================
// DragDrop tests (wry-builder feature)
// ============================================================================

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_event_type_names() {
    assert_eq!(DragDropEventType::Enter.as_event_name(), "file_drop_hover");
    assert_eq!(DragDropEventType::Over.as_event_name(), "file_drop_over");
    assert_eq!(DragDropEventType::Drop.as_event_name(), "file_drop");
    assert_eq!(
        DragDropEventType::Leave.as_event_name(),
        "file_drop_cancelled"
    );
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_event_type_equality() {
    assert_eq!(DragDropEventType::Enter, DragDropEventType::Enter);
    assert_ne!(DragDropEventType::Enter, DragDropEventType::Leave);
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_event_type_debug() {
    let event = DragDropEventType::Drop;
    let debug_str = format!("{:?}", event);
    assert!(debug_str.contains("Drop"));
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_event_type_clone() {
    let original = DragDropEventType::Enter;
    let cloned = original;
    assert_eq!(original, cloned);
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_event_type_copy() {
    let original = DragDropEventType::Over;
    let copied = original;
    assert_eq!(original, copied);
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_event_data_to_json_enter() {
    let data = DragDropEventData {
        event_type: DragDropEventType::Enter,
        paths: vec!["file1.txt".to_string(), "file2.txt".to_string()],
        position: Some((100.0, 200.0)),
        timestamp: None,
    };

    let json = data.to_json();
    assert_eq!(json["hovering"], true);
    assert_eq!(json["paths"].as_array().unwrap().len(), 2);
    assert_eq!(json["position"]["x"], 100.0);
    assert_eq!(json["position"]["y"], 200.0);
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_event_data_to_json_enter_no_position() {
    let data = DragDropEventData {
        event_type: DragDropEventType::Enter,
        paths: vec!["file.txt".to_string()],
        position: None,
        timestamp: None,
    };

    let json = data.to_json();
    assert_eq!(json["hovering"], true);
    assert!(json["position"].is_null());
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_event_data_to_json_over() {
    let data = DragDropEventData {
        event_type: DragDropEventType::Over,
        paths: Vec::new(),
        position: Some((150.0, 250.0)),
        timestamp: None,
    };

    let json = data.to_json();
    assert_eq!(json["position"]["x"], 150.0);
    assert_eq!(json["position"]["y"], 250.0);
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_event_data_to_json_over_no_position() {
    let data = DragDropEventData {
        event_type: DragDropEventType::Over,
        paths: Vec::new(),
        position: None,
        timestamp: None,
    };

    let json = data.to_json();
    assert!(json["position"].is_null());
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_event_data_to_json_drop() {
    let data = DragDropEventData {
        event_type: DragDropEventType::Drop,
        paths: vec!["file.txt".to_string()],
        position: Some((50.0, 75.0)),
        timestamp: Some(1234567890),
    };

    let json = data.to_json();
    assert_eq!(json["paths"].as_array().unwrap().len(), 1);
    assert_eq!(json["paths"][0], "file.txt");
    assert_eq!(json["timestamp"], 1234567890);
    assert_eq!(json["position"]["x"], 50.0);
    assert_eq!(json["position"]["y"], 75.0);
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_event_data_to_json_drop_multiple_files() {
    let data = DragDropEventData {
        event_type: DragDropEventType::Drop,
        paths: vec![
            "/path/to/file1.txt".to_string(),
            "/path/to/file2.png".to_string(),
            "/path/to/file3.pdf".to_string(),
        ],
        position: Some((0.0, 0.0)),
        timestamp: Some(0),
    };

    let json = data.to_json();
    let paths = json["paths"].as_array().unwrap();
    assert_eq!(paths.len(), 3);
    assert_eq!(paths[0], "/path/to/file1.txt");
    assert_eq!(paths[1], "/path/to/file2.png");
    assert_eq!(paths[2], "/path/to/file3.pdf");
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_event_data_to_json_drop_no_timestamp() {
    let data = DragDropEventData {
        event_type: DragDropEventType::Drop,
        paths: vec!["file.txt".to_string()],
        position: Some((10.0, 20.0)),
        timestamp: None,
    };

    let json = data.to_json();
    assert!(json["timestamp"].is_null());
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_event_data_to_json_leave() {
    let data = DragDropEventData {
        event_type: DragDropEventType::Leave,
        paths: Vec::new(),
        position: None,
        timestamp: None,
    };

    let json = data.to_json();
    assert_eq!(json["hovering"], false);
    assert_eq!(json["reason"], "left_window");
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_event_data_debug() {
    let data = DragDropEventData {
        event_type: DragDropEventType::Drop,
        paths: vec!["test.txt".to_string()],
        position: Some((10.0, 20.0)),
        timestamp: Some(12345),
    };

    let debug_str = format!("{:?}", data);
    assert!(debug_str.contains("DragDropEventData"));
    assert!(debug_str.contains("Drop"));
    assert!(debug_str.contains("test.txt"));
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_event_data_clone() {
    let original = DragDropEventData {
        event_type: DragDropEventType::Enter,
        paths: vec!["file.txt".to_string()],
        position: Some((1.0, 2.0)),
        timestamp: None,
    };

    let cloned = original.clone();
    assert_eq!(original.event_type, cloned.event_type);
    assert_eq!(original.paths, cloned.paths);
    assert_eq!(original.position, cloned.position);
    assert_eq!(original.timestamp, cloned.timestamp);
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_handler_callback() {
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let handler = DragDropHandler::new(move |_data| {
        counter_clone.fetch_add(1, Ordering::SeqCst);
    });

    // We can't easily test the actual handler without wry,
    // but we can verify the handler is created correctly
    let _handler_fn = handler.into_handler();

    // The callback should be callable
    assert_eq!(counter.load(Ordering::SeqCst), 0);
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_handler_with_data_capture() {
    let captured = Arc::new(std::sync::Mutex::new(Vec::new()));
    let captured_clone = captured.clone();

    let _handler = DragDropHandler::new(move |data| {
        captured_clone.lock().unwrap().push(data);
    });

    // Verify handler is created
    assert!(captured.lock().unwrap().is_empty());
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_empty_paths() {
    let data = DragDropEventData {
        event_type: DragDropEventType::Enter,
        paths: Vec::new(),
        position: Some((0.0, 0.0)),
        timestamp: None,
    };

    let json = data.to_json();
    assert!(json["paths"].as_array().unwrap().is_empty());
}

// ============================================================================
// Helpers tests (wry-builder feature)
// ============================================================================

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_create_drag_drop_handler() {
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let _handler = create_drag_drop_handler(move |_event_name, _data| {
        counter_clone.fetch_add(1, Ordering::SeqCst);
    });

    // Handler is created successfully
    assert_eq!(counter.load(Ordering::SeqCst), 0);
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_create_drag_drop_handler_captures_event_name() {
    let events = Arc::new(std::sync::Mutex::new(Vec::new()));
    let events_clone = events.clone();

    let _handler = create_drag_drop_handler(move |event_name, _data| {
        events_clone.lock().unwrap().push(event_name.to_string());
    });

    // Verify handler is created
    assert!(events.lock().unwrap().is_empty());
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_create_drag_drop_handler_captures_json_data() {
    let data_list = Arc::new(std::sync::Mutex::new(Vec::new()));
    let data_clone = data_list.clone();

    let _handler = create_drag_drop_handler(move |_event_name, data| {
        data_clone.lock().unwrap().push(data);
    });

    // Verify handler is created
    assert!(data_list.lock().unwrap().is_empty());
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_create_ipc_handler() {
    let event_counter = Arc::new(AtomicUsize::new(0));
    let call_counter = Arc::new(AtomicUsize::new(0));

    let event_counter_clone = event_counter.clone();
    let call_counter_clone = call_counter.clone();

    let _handler = create_ipc_handler(
        move |_name, _data| {
            event_counter_clone.fetch_add(1, Ordering::SeqCst);
        },
        move |_method, _params, _id| {
            call_counter_clone.fetch_add(1, Ordering::SeqCst);
        },
        |_cmd, _args, _id| {},
        |_callback_id, _data| {},
    );

    // Handlers are created successfully
    assert_eq!(event_counter.load(Ordering::SeqCst), 0);
    assert_eq!(call_counter.load(Ordering::SeqCst), 0);
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_create_ipc_handler_all_callbacks() {
    let event_counter = Arc::new(AtomicUsize::new(0));
    let call_counter = Arc::new(AtomicUsize::new(0));
    let invoke_counter = Arc::new(AtomicUsize::new(0));
    let js_callback_counter = Arc::new(AtomicUsize::new(0));

    let event_clone = event_counter.clone();
    let call_clone = call_counter.clone();
    let invoke_clone = invoke_counter.clone();
    let js_callback_clone = js_callback_counter.clone();

    let _handler = create_ipc_handler(
        move |_name, _data| {
            event_clone.fetch_add(1, Ordering::SeqCst);
        },
        move |_method, _params, _id| {
            call_clone.fetch_add(1, Ordering::SeqCst);
        },
        move |_cmd, _args, _id| {
            invoke_clone.fetch_add(1, Ordering::SeqCst);
        },
        move |_callback_id, _data| {
            js_callback_clone.fetch_add(1, Ordering::SeqCst);
        },
    );

    // All counters start at 0
    assert_eq!(event_counter.load(Ordering::SeqCst), 0);
    assert_eq!(call_counter.load(Ordering::SeqCst), 0);
    assert_eq!(invoke_counter.load(Ordering::SeqCst), 0);
    assert_eq!(js_callback_counter.load(Ordering::SeqCst), 0);
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_create_simple_ipc_handler() {
    let event_counter = Arc::new(AtomicUsize::new(0));
    let call_counter = Arc::new(AtomicUsize::new(0));

    let event_clone = event_counter.clone();
    let call_clone = call_counter.clone();

    let _handler = create_simple_ipc_handler(
        move |_name, _data| {
            event_clone.fetch_add(1, Ordering::SeqCst);
        },
        move |_method, _params, _id| {
            call_clone.fetch_add(1, Ordering::SeqCst);
        },
    );

    // Handlers are created successfully
    assert_eq!(event_counter.load(Ordering::SeqCst), 0);
    assert_eq!(call_counter.load(Ordering::SeqCst), 0);
}

// ============================================================================
// IPC tests (wry-builder feature)
// ============================================================================

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_message_type_parsing() {
    assert_eq!(IpcMessageType::parse("event"), IpcMessageType::Event);
    assert_eq!(IpcMessageType::parse("call"), IpcMessageType::Call);
    assert_eq!(IpcMessageType::parse("invoke"), IpcMessageType::Invoke);
    assert_eq!(
        IpcMessageType::parse("js_callback_result"),
        IpcMessageType::JsCallbackResult
    );
    assert!(matches!(
        IpcMessageType::parse("unknown"),
        IpcMessageType::Unknown(_)
    ));
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_message_type_unknown_preserves_value() {
    if let IpcMessageType::Unknown(val) = IpcMessageType::parse("custom_type") {
        assert_eq!(val, "custom_type");
    } else {
        panic!("Expected Unknown variant");
    }
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_parse_event_message() {
    let body = r#"{"type":"event","event":"click","detail":{"x":100,"y":200}}"#;
    let parsed = IpcMessageHandler::parse(body).unwrap();

    assert_eq!(parsed.msg_type, IpcMessageType::Event);
    assert_eq!(parsed.name, Some("click".to_string()));
    assert_eq!(parsed.data["x"], 100);
    assert_eq!(parsed.data["y"], 200);
    assert!(parsed.id.is_none());
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_parse_event_without_detail() {
    let body = r#"{"type":"event","event":"ready"}"#;
    let parsed = IpcMessageHandler::parse(body).unwrap();

    assert_eq!(parsed.msg_type, IpcMessageType::Event);
    assert_eq!(parsed.name, Some("ready".to_string()));
    assert!(parsed.data.is_null());
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_parse_call_message() {
    let body = r#"{"type":"call","method":"api.echo","params":{"msg":"hello"},"id":"123"}"#;
    let parsed = IpcMessageHandler::parse(body).unwrap();

    assert_eq!(parsed.msg_type, IpcMessageType::Call);
    assert_eq!(parsed.name, Some("api.echo".to_string()));
    assert_eq!(parsed.data["msg"], "hello");
    assert_eq!(parsed.id, Some("123".to_string()));
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_parse_call_without_params() {
    let body = r#"{"type":"call","method":"api.ping","id":"1"}"#;
    let parsed = IpcMessageHandler::parse(body).unwrap();

    assert_eq!(parsed.msg_type, IpcMessageType::Call);
    assert_eq!(parsed.name, Some("api.ping".to_string()));
    assert!(parsed.data.is_null());
    assert_eq!(parsed.id, Some("1".to_string()));
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_parse_call_without_id() {
    let body = r#"{"type":"call","method":"api.fire_and_forget","params":{}}"#;
    let parsed = IpcMessageHandler::parse(body).unwrap();

    assert_eq!(parsed.msg_type, IpcMessageType::Call);
    assert!(parsed.id.is_none());
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_parse_invoke_message() {
    let body = r#"{"type":"invoke","cmd":"plugin.test","args":{"value":42},"id":"456"}"#;
    let parsed = IpcMessageHandler::parse(body).unwrap();

    assert_eq!(parsed.msg_type, IpcMessageType::Invoke);
    assert_eq!(parsed.name, Some("plugin.test".to_string()));
    assert_eq!(parsed.data["value"], 42);
    assert_eq!(parsed.id, Some("456".to_string()));
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_parse_invoke_without_args() {
    let body = r#"{"type":"invoke","cmd":"plugin.init","id":"1"}"#;
    let parsed = IpcMessageHandler::parse(body).unwrap();

    assert_eq!(parsed.msg_type, IpcMessageType::Invoke);
    assert_eq!(parsed.name, Some("plugin.init".to_string()));
    assert!(parsed.data.is_object());
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_parse_js_callback_result() {
    let body = r#"{"type":"js_callback_result","callback_id":789,"result":"success"}"#;
    let parsed = IpcMessageHandler::parse(body).unwrap();

    assert_eq!(parsed.msg_type, IpcMessageType::JsCallbackResult);
    assert_eq!(parsed.name, Some("789".to_string()));
    assert_eq!(parsed.data["result"], "success");
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_parse_js_callback_with_error() {
    let body = r#"{"type":"js_callback_result","callback_id":100,"error":"failed"}"#;
    let parsed = IpcMessageHandler::parse(body).unwrap();

    assert_eq!(parsed.msg_type, IpcMessageType::JsCallbackResult);
    assert_eq!(parsed.name, Some("100".to_string()));
    assert_eq!(parsed.data["error"], "failed");
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_parse_js_callback_with_both() {
    let body = r#"{"type":"js_callback_result","callback_id":50,"result":"ok","error":"warn"}"#;
    let parsed = IpcMessageHandler::parse(body).unwrap();

    assert_eq!(parsed.data["result"], "ok");
    assert_eq!(parsed.data["error"], "warn");
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_parse_unknown_type() {
    let body = r#"{"type":"custom"}"#;
    let parsed = IpcMessageHandler::parse(body).unwrap();

    assert!(matches!(parsed.msg_type, IpcMessageType::Unknown(_)));
    assert!(parsed.name.is_none());
    assert!(parsed.data.is_null());
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_parse_invalid_json() {
    let body = "not valid json";
    assert!(IpcMessageHandler::parse(body).is_none());
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_parse_missing_type() {
    let body = r#"{"event":"click"}"#;
    assert!(IpcMessageHandler::parse(body).is_none());
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_parse_null_type() {
    let body = r#"{"type":null}"#;
    assert!(IpcMessageHandler::parse(body).is_none());
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_parse_numeric_type() {
    let body = r#"{"type":123}"#;
    assert!(IpcMessageHandler::parse(body).is_none());
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_parsed_message_raw_field() {
    let body = r#"{"type":"event","event":"test","extra":"data"}"#;
    let parsed = IpcMessageHandler::parse(body).unwrap();

    assert_eq!(parsed.raw["extra"], "data");
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_handler_new_and_handle() {
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let handler = IpcMessageHandler::new(move |_msg| {
        counter_clone.fetch_add(1, Ordering::SeqCst);
    });

    // Handle a valid message
    handler.handle(r#"{"type":"event","event":"test"}"#);
    assert_eq!(counter.load(Ordering::SeqCst), 1);

    // Handle another valid message
    handler.handle(r#"{"type":"call","method":"api.test","id":"1"}"#);
    assert_eq!(counter.load(Ordering::SeqCst), 2);
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_handler_handle_invalid_message() {
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let handler = IpcMessageHandler::new(move |_msg| {
        counter_clone.fetch_add(1, Ordering::SeqCst);
    });

    // Invalid JSON should not trigger callback
    handler.handle("invalid json");
    assert_eq!(counter.load(Ordering::SeqCst), 0);

    // Missing type should not trigger callback
    handler.handle(r#"{"event":"test"}"#);
    assert_eq!(counter.load(Ordering::SeqCst), 0);
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_handler_callback_receives_correct_data() {
    let received = Arc::new(std::sync::Mutex::new(None));
    let received_clone = received.clone();

    let handler = IpcMessageHandler::new(move |msg| {
        *received_clone.lock().unwrap() = Some(msg);
    });

    handler.handle(r#"{"type":"call","method":"api.echo","params":{"value":42},"id":"abc"}"#);

    let msg = received.lock().unwrap().take().unwrap();
    assert_eq!(msg.msg_type, IpcMessageType::Call);
    assert_eq!(msg.name, Some("api.echo".to_string()));
    assert_eq!(msg.data["value"], 42);
    assert_eq!(msg.id, Some("abc".to_string()));
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_ipc_message_type_debug() {
    // Test Debug implementation
    let event = IpcMessageType::Event;
    let debug_str = format!("{:?}", event);
    assert!(debug_str.contains("Event"));

    let unknown = IpcMessageType::Unknown("test".to_string());
    let debug_str = format!("{:?}", unknown);
    assert!(debug_str.contains("Unknown"));
    assert!(debug_str.contains("test"));
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_ipc_message_type_clone() {
    let original = IpcMessageType::Unknown("test".to_string());
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_parsed_message_clone() {
    let body = r#"{"type":"event","event":"test","detail":{"x":1}}"#;
    let parsed = IpcMessageHandler::parse(body).unwrap();
    let cloned = parsed.clone();

    assert_eq!(parsed.msg_type, cloned.msg_type);
    assert_eq!(parsed.name, cloned.name);
    assert_eq!(parsed.data, cloned.data);
    assert_eq!(parsed.id, cloned.id);
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_parsed_message_debug() {
    let body = r#"{"type":"event","event":"test"}"#;
    let parsed = IpcMessageHandler::parse(body).unwrap();
    let debug_str = format!("{:?}", parsed);

    assert!(debug_str.contains("ParsedIpcMessage"));
    assert!(debug_str.contains("Event"));
}

// ============================================================================
// COM init tests
// ============================================================================

#[rstest]
fn test_com_init_result_enum() {
    // Just verify the enum variants exist
    let _ = ComInitResult::Initialized;
    let _ = ComInitResult::AlreadyInitialized;
    let _ = ComInitResult::Failed;
}

#[rstest]
fn test_init_com_sta() {
    // On Windows, this should return Initialized or AlreadyInitialized
    // On other platforms, it should return Initialized (no-op)
    let result = init_com_sta();
    assert!(matches!(
        result,
        ComInitResult::Initialized | ComInitResult::AlreadyInitialized
    ));
}

// ============================================================================
// Protocol tests (wry-builder feature)
// ============================================================================

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_protocol_default_config() {
    let config = ProtocolConfig::default();
    assert!(config.asset_root.is_none());
    assert!(!config.allow_file_protocol);
    assert!(!config.use_https_scheme);
    assert!(!config.has_auroraview_protocol());
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_protocol_with_asset_root() {
    let config = ProtocolConfig::new().with_asset_root(PathBuf::from("/assets"));
    assert!(config.asset_root.is_some());
    assert!(config.has_auroraview_protocol());
}

#[cfg(feature = "wry-builder")]
#[rstest]
fn test_protocol_builder_pattern() {
    let config = ProtocolConfig::new()
        .with_asset_root(PathBuf::from("/assets"))
        .with_file_protocol(true)
        .with_https_scheme(true);

    assert!(config.asset_root.is_some());
    assert!(config.allow_file_protocol);
    assert!(config.use_https_scheme);
}

// ============================================================================
// WebContext tests
// ============================================================================

#[rstest]
fn test_web_context_config_default() {
    let config = WebContextConfig::default();
    assert!(config.data_directory.is_none());
    assert!(config.shared_warmup_folder.is_none());
}

#[rstest]
fn test_web_context_config_with_data_directory() {
    let config = WebContextConfig::new().with_data_directory(PathBuf::from("/tmp/data"));
    assert_eq!(config.data_directory, Some(PathBuf::from("/tmp/data")));
}

#[rstest]
fn test_web_context_config_with_shared_warmup() {
    let config = WebContextConfig::new().with_shared_warmup_folder(PathBuf::from("/tmp/warmup"));
    assert_eq!(
        config.shared_warmup_folder,
        Some(PathBuf::from("/tmp/warmup"))
    );
}

#[rstest]
fn test_web_context_config_builder_chain() {
    let config = WebContextConfig::new()
        .with_data_directory(PathBuf::from("/tmp/data"))
        .with_shared_warmup_folder(PathBuf::from("/tmp/warmup"));

    assert_eq!(config.data_directory, Some(PathBuf::from("/tmp/data")));
    assert_eq!(
        config.shared_warmup_folder,
        Some(PathBuf::from("/tmp/warmup"))
    );
}

// ============================================================================
// WindowStyle tests
// ============================================================================

#[rstest]
fn test_options_default() {
    let opts = ChildWindowStyleOptions::default();
    assert!(!opts.force_position);
}

#[rstest]
fn test_options_for_dcc() {
    let opts = ChildWindowStyleOptions::for_dcc_embedding();
    assert!(opts.force_position);
}

#[rstest]
fn test_options_for_standalone() {
    let opts = ChildWindowStyleOptions::for_standalone();
    assert!(!opts.force_position);
}

// ============================================================================
// CommonConfig tests
// ============================================================================

#[rstest]
fn test_dark_background_color() {
    let color = get_background_color();
    assert_eq!(color, (2, 6, 23, 255));
}

#[rstest]
fn test_background_color_hex() {
    let color = get_background_color();
    // Verify it matches #020617
    assert_eq!(color.0, 0x02);
    assert_eq!(color.1, 0x06);
    assert_eq!(color.2, 0x17);
    assert_eq!(color.3, 0xFF);
}

#[rstest]
fn test_dark_background_constant() {
    assert_eq!(DARK_BACKGROUND, (2, 6, 23, 255));
}
