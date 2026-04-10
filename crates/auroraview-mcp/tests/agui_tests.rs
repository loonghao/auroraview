use auroraview_mcp::{AguiBus, AguiEvent};
use rstest::rstest;

// ---------------------------------------------------------------------------
// AguiEvent construction
// ---------------------------------------------------------------------------

#[rstest]
fn run_started_run_id() {
    let ev = AguiEvent::RunStarted {
        run_id: "r1".to_string(),
        thread_id: "t1".to_string(),
    };
    assert_eq!(ev.run_id(), "r1");
}

#[rstest]
fn run_finished_run_id() {
    let ev = AguiEvent::RunFinished {
        run_id: "r2".to_string(),
        thread_id: "t2".to_string(),
    };
    assert_eq!(ev.run_id(), "r2");
}

#[rstest]
fn run_error_run_id() {
    let ev = AguiEvent::RunError {
        run_id: "r3".to_string(),
        message: "something failed".to_string(),
        code: Some("E001".to_string()),
    };
    assert_eq!(ev.run_id(), "r3");
}

#[rstest]
fn step_started_run_id() {
    let ev = AguiEvent::StepStarted {
        run_id: "r4".to_string(),
        step_name: "load_url".to_string(),
        step_id: "s1".to_string(),
    };
    assert_eq!(ev.run_id(), "r4");
}

#[rstest]
fn step_finished_run_id() {
    let ev = AguiEvent::StepFinished {
        run_id: "r5".to_string(),
        step_id: "s2".to_string(),
    };
    assert_eq!(ev.run_id(), "r5");
}

#[rstest]
fn text_message_start_run_id() {
    let ev = AguiEvent::TextMessageStart {
        run_id: "r6".to_string(),
        message_id: "m1".to_string(),
        role: "assistant".to_string(),
    };
    assert_eq!(ev.run_id(), "r6");
}

#[rstest]
fn text_message_content_run_id() {
    let ev = AguiEvent::TextMessageContent {
        run_id: "r7".to_string(),
        message_id: "m1".to_string(),
        delta: "hello".to_string(),
    };
    assert_eq!(ev.run_id(), "r7");
}

#[rstest]
fn text_message_end_run_id() {
    let ev = AguiEvent::TextMessageEnd {
        run_id: "r8".to_string(),
        message_id: "m1".to_string(),
    };
    assert_eq!(ev.run_id(), "r8");
}

#[rstest]
fn tool_call_start_run_id() {
    let ev = AguiEvent::ToolCallStart {
        run_id: "r9".to_string(),
        tool_call_id: "tc1".to_string(),
        tool_name: "screenshot".to_string(),
    };
    assert_eq!(ev.run_id(), "r9");
}

#[rstest]
fn tool_call_args_run_id() {
    let ev = AguiEvent::ToolCallArgs {
        run_id: "r10".to_string(),
        tool_call_id: "tc1".to_string(),
        delta: r#"{"id": "wv1"}"#.to_string(),
    };
    assert_eq!(ev.run_id(), "r10");
}

#[rstest]
fn tool_call_end_run_id() {
    let ev = AguiEvent::ToolCallEnd {
        run_id: "r11".to_string(),
        tool_call_id: "tc1".to_string(),
    };
    assert_eq!(ev.run_id(), "r11");
}

#[rstest]
fn state_snapshot_run_id() {
    let ev = AguiEvent::StateSnapshot {
        run_id: "r12".to_string(),
        snapshot: serde_json::json!({"key": "value"}),
    };
    assert_eq!(ev.run_id(), "r12");
}

#[rstest]
fn state_delta_run_id() {
    let ev = AguiEvent::StateDelta {
        run_id: "r13".to_string(),
        delta: vec![serde_json::json!({"op": "add", "path": "/key", "value": 1})],
    };
    assert_eq!(ev.run_id(), "r13");
}

#[rstest]
fn custom_event_run_id() {
    let ev = AguiEvent::Custom {
        run_id: "r14".to_string(),
        name: "webview_ready".to_string(),
        data: serde_json::json!({"hwnd": 12345}),
    };
    assert_eq!(ev.run_id(), "r14");
}

// ---------------------------------------------------------------------------
// SSE serialization
// ---------------------------------------------------------------------------

#[rstest]
fn to_sse_line_starts_with_data() {
    let ev = AguiEvent::RunStarted {
        run_id: "x".to_string(),
        thread_id: "y".to_string(),
    };
    let line = ev.to_sse_line();
    assert!(line.starts_with("data: "), "SSE line must start with 'data: '");
    assert!(line.ends_with("\n\n"), "SSE line must end with double newline");
}

#[rstest]
fn to_sse_line_contains_json() {
    let ev = AguiEvent::ToolCallStart {
        run_id: "run1".to_string(),
        tool_call_id: "tc1".to_string(),
        tool_name: "eval_js".to_string(),
    };
    let line = ev.to_sse_line();
    assert!(line.contains("\"tool_name\""), "SSE line should contain tool_name field");
    assert!(line.contains("eval_js"), "SSE line should contain tool name value");
}

#[rstest]
fn event_round_trip_json() {
    let ev = AguiEvent::TextMessageContent {
        run_id: "run-abc".to_string(),
        message_id: "msg-1".to_string(),
        delta: "Hello, world".to_string(),
    };
    let json = serde_json::to_string(&ev).unwrap();
    let restored: AguiEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(ev, restored);
}

// ---------------------------------------------------------------------------
// AguiBus
// ---------------------------------------------------------------------------

#[rstest]
fn agui_bus_default() {
    let bus = AguiBus::default();
    assert_eq!(bus.receiver_count(), 0);
}

#[rstest]
fn agui_bus_subscribe_increments_count() {
    let bus = AguiBus::new();
    let _rx1 = bus.subscribe();
    assert_eq!(bus.receiver_count(), 1);
    let _rx2 = bus.subscribe();
    assert_eq!(bus.receiver_count(), 2);
}

#[rstest]
fn agui_bus_emit_no_subscribers_no_panic() {
    let bus = AguiBus::new();
    // Should not panic even with no subscribers
    bus.emit(AguiEvent::RunStarted {
        run_id: "r".to_string(),
        thread_id: "t".to_string(),
    });
}

#[tokio::test]
async fn test_agui_bus_emit_received_by_subscriber() {
    let bus = AguiBus::new();
    let mut rx = bus.subscribe();

    let sent = AguiEvent::RunStarted {
        run_id: "my-run".to_string(),
        thread_id: "my-thread".to_string(),
    };
    bus.emit(sent.clone());

    let received = tokio::time::timeout(
        std::time::Duration::from_millis(100),
        rx.recv(),
    )
    .await
    .expect("timeout")
    .expect("channel error");

    assert_eq!(received, sent);
}

#[tokio::test]
async fn test_agui_bus_multiple_subscribers() {
    let bus = AguiBus::new();
    let mut rx1 = bus.subscribe();
    let mut rx2 = bus.subscribe();

    bus.emit(AguiEvent::StepStarted {
        run_id: "r".to_string(),
        step_name: "export".to_string(),
        step_id: "s".to_string(),
    });

    let e1 = tokio::time::timeout(
        std::time::Duration::from_millis(100),
        rx1.recv(),
    )
    .await
    .unwrap()
    .unwrap();
    let e2 = tokio::time::timeout(
        std::time::Duration::from_millis(100),
        rx2.recv(),
    )
    .await
    .unwrap()
    .unwrap();

    assert_eq!(e1.run_id(), "r");
    assert_eq!(e2.run_id(), "r");
}

#[tokio::test]
async fn test_agui_bus_clone_shares_channel() {
    let bus = AguiBus::new();
    let bus_clone = bus.clone();

    let mut rx = bus.subscribe();
    bus_clone.emit(AguiEvent::RunFinished {
        run_id: "clone-run".to_string(),
        thread_id: "t".to_string(),
    });

    let ev = tokio::time::timeout(
        std::time::Duration::from_millis(100),
        rx.recv(),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(ev.run_id(), "clone-run");
}

#[tokio::test]
async fn test_agui_bus_drop_subscriber_reduces_count() {
    let bus = AguiBus::new();
    {
        let _rx = bus.subscribe();
        assert_eq!(bus.receiver_count(), 1);
    }
    // After drop, count is 0 (broadcast channel lazy cleanup)
    assert_eq!(bus.receiver_count(), 0);
}
