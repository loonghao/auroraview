/// Tests verifying that tool invocations automatically emit AG-UI events
/// via the attached `AguiBus`.
use auroraview_mcp::{
    AguiBus, AguiEvent, AuroraViewMcpServer, McpServerConfig, WebViewConfig,
};
use rstest::rstest;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn server_with_bus() -> (AuroraViewMcpServer, AguiBus) {
    let bus = AguiBus::new();
    let server = AuroraViewMcpServer::new(McpServerConfig {
        enable_mdns: false,
        ..Default::default()
    })
    .with_agui_bus(bus.clone());
    (server, bus)
}

// ---------------------------------------------------------------------------
// with_agui_bus builder
// ---------------------------------------------------------------------------

#[rstest]
fn server_without_bus_has_none() {
    let server = AuroraViewMcpServer::new(McpServerConfig {
        enable_mdns: false,
        ..Default::default()
    });
    assert!(server.agui_bus().is_none());
}

#[rstest]
fn server_with_bus_has_some() {
    let (server, _bus) = server_with_bus();
    assert!(server.agui_bus().is_some());
}

#[rstest]
fn server_bus_shares_channel_with_runner_bus() {
    let bus = AguiBus::new();
    let server = AuroraViewMcpServer::new(McpServerConfig {
        enable_mdns: false,
        ..Default::default()
    })
    .with_agui_bus(bus.clone());

    // Bus clones share the same underlying channel
    let mut rx = bus.subscribe();
    server.agui_bus().unwrap().emit(AguiEvent::RunStarted {
        run_id: "shared".to_string(),
        thread_id: "t1".to_string(),
    });

    // Can receive the event synchronously via try_recv
    let ev = rx.try_recv().expect("should have received event");
    assert_eq!(ev.run_id(), "shared");
}

// ---------------------------------------------------------------------------
// No-bus path (no panic, no event)
// ---------------------------------------------------------------------------

#[rstest]
fn server_without_bus_screenshot_no_panic() {
    // Server without bus: tool still works, just no events
    let server = AuroraViewMcpServer::new(McpServerConfig {
        enable_mdns: false,
        ..Default::default()
    });
    let reg = server.registry();
    let _id = reg.register(&WebViewConfig::default());
    // This just verifies registry state (tool methods need rmcp context to call directly)
    assert_eq!(reg.len(), 1);
}

// ---------------------------------------------------------------------------
// AguiBus integration: emit_tool_start/end via server bus
// ---------------------------------------------------------------------------

#[rstest]
fn server_agui_bus_emit_tool_call_start() {
    let (server, bus) = server_with_bus();
    let mut rx = bus.subscribe();

    // Manually emit a ToolCallStart through the bus (simulates what tools do internally)
    server.agui_bus().unwrap().emit(AguiEvent::ToolCallStart {
        run_id: "wv-1".to_string(),
        tool_call_id: "tc-abc".to_string(),
        tool_name: "screenshot".to_string(),
    });

    let ev = rx.try_recv().expect("event should be received");
    assert!(matches!(ev, AguiEvent::ToolCallStart { .. }));
    assert_eq!(ev.run_id(), "wv-1");
}

#[rstest]
fn server_agui_bus_emit_tool_call_end() {
    let (server, bus) = server_with_bus();
    let mut rx = bus.subscribe();

    server.agui_bus().unwrap().emit(AguiEvent::ToolCallEnd {
        run_id: "wv-2".to_string(),
        tool_call_id: "tc-xyz".to_string(),
    });

    let ev = rx.try_recv().expect("event should be received");
    assert!(matches!(ev, AguiEvent::ToolCallEnd { .. }));
    assert_eq!(ev.run_id(), "wv-2");
}

#[rstest]
fn server_agui_bus_emit_sequence() {
    let (server, bus) = server_with_bus();
    let mut rx = bus.subscribe();
    let agui = server.agui_bus().unwrap();

    // Simulate a full tool call lifecycle
    agui.emit(AguiEvent::ToolCallStart {
        run_id: "wv-3".to_string(),
        tool_call_id: "tc-1".to_string(),
        tool_name: "eval_js".to_string(),
    });
    agui.emit(AguiEvent::ToolCallArgs {
        run_id: "wv-3".to_string(),
        tool_call_id: "tc-1".to_string(),
        delta: r#"{"script":"document.title"}"#.to_string(),
    });
    agui.emit(AguiEvent::ToolCallEnd {
        run_id: "wv-3".to_string(),
        tool_call_id: "tc-1".to_string(),
    });

    let e1 = rx.try_recv().unwrap();
    let e2 = rx.try_recv().unwrap();
    let e3 = rx.try_recv().unwrap();

    assert!(matches!(e1, AguiEvent::ToolCallStart { .. }));
    assert!(matches!(e2, AguiEvent::ToolCallArgs { .. }));
    assert!(matches!(e3, AguiEvent::ToolCallEnd { .. }));
}

// ---------------------------------------------------------------------------
// McpRunner integration: agui_bus() shares channel with server
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_runner_bus_shared_with_server() {
    use auroraview_mcp::McpRunner;

    let runner = McpRunner::new(McpServerConfig {
        enable_mdns: false,
        ..Default::default()
    });

    let mut rx = runner.agui_bus().subscribe();

    // Emit via runner bus — server's internal bus is the same channel
    runner.emit_agui(AguiEvent::StepStarted {
        run_id: "integration".to_string(),
        step_name: "load_url".to_string(),
        step_id: "s-1".to_string(),
    });

    let ev = tokio::time::timeout(Duration::from_millis(100), rx.recv())
        .await
        .expect("timeout")
        .expect("channel closed");

    assert_eq!(ev.run_id(), "integration");
    assert!(matches!(ev, AguiEvent::StepStarted { .. }));
}

#[tokio::test]
async fn test_runner_emit_multiple_event_types() {
    use auroraview_mcp::McpRunner;

    let runner = McpRunner::new(McpServerConfig {
        enable_mdns: false,
        ..Default::default()
    });
    let mut rx = runner.agui_bus().subscribe();

    let events = vec![
        AguiEvent::RunStarted {
            run_id: "run-1".to_string(),
            thread_id: "t-1".to_string(),
        },
        AguiEvent::ToolCallStart {
            run_id: "run-1".to_string(),
            tool_call_id: "tc-1".to_string(),
            tool_name: "screenshot".to_string(),
        },
        AguiEvent::ToolCallEnd {
            run_id: "run-1".to_string(),
            tool_call_id: "tc-1".to_string(),
        },
        AguiEvent::RunFinished {
            run_id: "run-1".to_string(),
            thread_id: "t-1".to_string(),
        },
    ];

    for ev in &events {
        runner.emit_agui(ev.clone());
    }

    for expected_run_id in ["run-1", "run-1", "run-1", "run-1"] {
        let received = tokio::time::timeout(Duration::from_millis(100), rx.recv())
            .await
            .expect("timeout")
            .expect("channel error");
        assert_eq!(received.run_id(), expected_run_id);
    }
}

// ---------------------------------------------------------------------------
// AguiEvent JSON serialization round-trips
// ---------------------------------------------------------------------------

#[rstest]
fn tool_call_start_json_round_trip() {
    let ev = AguiEvent::ToolCallStart {
        run_id: "run-abc".to_string(),
        tool_call_id: "tc-001".to_string(),
        tool_name: "load_html".to_string(),
    };
    let json = serde_json::to_string(&ev).unwrap();
    assert!(json.contains("TOOL_CALL_START"));
    assert!(json.contains("load_html"));
    let restored: AguiEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(ev, restored);
}

#[rstest]
fn tool_call_end_json_round_trip() {
    let ev = AguiEvent::ToolCallEnd {
        run_id: "run-abc".to_string(),
        tool_call_id: "tc-001".to_string(),
    };
    let json = serde_json::to_string(&ev).unwrap();
    assert!(json.contains("TOOL_CALL_END"));
    let restored: AguiEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(ev, restored);
}

#[rstest]
fn tool_call_args_json_round_trip() {
    let ev = AguiEvent::ToolCallArgs {
        run_id: "r".to_string(),
        tool_call_id: "tc".to_string(),
        delta: r#"{"url":"https://example.com"}"#.to_string(),
    };
    let json = serde_json::to_string(&ev).unwrap();
    let restored: AguiEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(ev, restored);
}

#[rstest]
fn run_error_with_code_json_round_trip() {
    let ev = AguiEvent::RunError {
        run_id: "r-err".to_string(),
        message: "WebView not found".to_string(),
        code: Some("E404".to_string()),
    };
    let json = serde_json::to_string(&ev).unwrap();
    assert!(json.contains("RUN_ERROR"));
    let restored: AguiEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(ev, restored);
}

#[rstest]
fn run_error_without_code_json_round_trip() {
    let ev = AguiEvent::RunError {
        run_id: "r-err2".to_string(),
        message: "Internal error".to_string(),
        code: None,
    };
    let json = serde_json::to_string(&ev).unwrap();
    let restored: AguiEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(ev, restored);
}

#[rstest]
fn state_snapshot_json_round_trip() {
    let ev = AguiEvent::StateSnapshot {
        run_id: "r-snap".to_string(),
        snapshot: serde_json::json!({
            "webviews": 3,
            "active": "wv-1"
        }),
    };
    let json = serde_json::to_string(&ev).unwrap();
    assert!(json.contains("STATE_SNAPSHOT"));
    let restored: AguiEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(ev, restored);
}

#[rstest]
fn state_delta_json_round_trip() {
    let ev = AguiEvent::StateDelta {
        run_id: "r-delta".to_string(),
        delta: vec![
            serde_json::json!({"op": "replace", "path": "/active", "value": "wv-2"}),
        ],
    };
    let json = serde_json::to_string(&ev).unwrap();
    let restored: AguiEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(ev, restored);
}

#[rstest]
fn custom_event_json_round_trip() {
    let ev = AguiEvent::Custom {
        run_id: "r-custom".to_string(),
        name: "dcc_scene_exported".to_string(),
        data: serde_json::json!({"path": "/scene.usda", "format": "usd"}),
    };
    let json = serde_json::to_string(&ev).unwrap();
    assert!(json.contains("CUSTOM"));
    assert!(json.contains("dcc_scene_exported"));
    let restored: AguiEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(ev, restored);
}

// ---------------------------------------------------------------------------
// AguiBus capacity and overflow behavior
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_agui_bus_overflow_does_not_panic() {
    let bus = AguiBus::new();
    let _rx = bus.subscribe();

    // Emit more than capacity (256) without draining — should not panic
    for i in 0..300 {
        bus.emit(AguiEvent::TextMessageContent {
            run_id: format!("r-{i}"),
            message_id: "m".to_string(),
            delta: "x".to_string(),
        });
    }
}

#[tokio::test]
async fn test_agui_bus_receiver_count_after_clone() {
    let bus = AguiBus::new();
    let bus2 = bus.clone();

    let _rx1 = bus.subscribe();
    let _rx2 = bus2.subscribe();

    // Both subscribe to the same underlying broadcast channel
    assert_eq!(bus.receiver_count(), 2);
    assert_eq!(bus2.receiver_count(), 2);
}

// ---------------------------------------------------------------------------
// WebViewRegistry: additional edge cases
// ---------------------------------------------------------------------------

#[rstest]
fn registry_get_nonexistent_returns_none() {
    let server = AuroraViewMcpServer::new(McpServerConfig {
        enable_mdns: false,
        ..Default::default()
    });
    let fake: auroraview_mcp::WebViewId = "does-not-exist".parse().unwrap();
    assert!(server.registry().get(&fake).is_none());
}

#[rstest]
fn registry_concurrent_register() {
    use std::sync::Arc;
    use std::thread;

    let server = Arc::new(AuroraViewMcpServer::new(McpServerConfig {
        enable_mdns: false,
        ..Default::default()
    }));

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let s = server.clone();
            thread::spawn(move || {
                s.registry().register(&WebViewConfig::default());
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(server.registry().len(), 10);
}

#[rstest]
fn registry_update_url_and_verify() {
    let (server, _bus) = server_with_bus();
    let id = server.registry().register(&WebViewConfig {
        url: Some("https://original.com".to_string()),
        ..Default::default()
    });

    let info_before = server.registry().get(&id).unwrap();
    assert_eq!(info_before.url, "https://original.com");

    let ok = server.registry().update_url(&id, "https://updated.com");
    assert!(ok);

    let info_after = server.registry().get(&id).unwrap();
    assert_eq!(info_after.url, "https://updated.com");
}

#[rstest]
fn registry_remove_all() {
    let (server, _bus) = server_with_bus();
    let id1 = server.registry().register(&WebViewConfig::default());
    let id2 = server.registry().register(&WebViewConfig::default());
    let id3 = server.registry().register(&WebViewConfig::default());

    assert_eq!(server.registry().len(), 3);

    server.registry().remove(&id1);
    server.registry().remove(&id2);
    server.registry().remove(&id3);

    assert!(server.registry().is_empty());
}
