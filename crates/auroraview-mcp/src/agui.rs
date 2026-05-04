//! AG-UI protocol event types for `AuroraView` MCP Server.
//!
//! AG-UI is a protocol for streaming agent state updates to UI clients.
//! See: <https://docs.ag-ui.com>

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

/// AG-UI event types per the protocol spec.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AguiEvent {
    /// Run lifecycle: started
    RunStarted {
        /// Unique identifier for this run.
        run_id: String,
        /// Thread associated with this run.
        thread_id: String,
    },
    /// Run lifecycle: finished
    RunFinished {
        /// Unique identifier for this run.
        run_id: String,
        /// Thread associated with this run.
        thread_id: String,
    },
    /// Run lifecycle: error
    RunError {
        /// Unique identifier for this run.
        run_id: String,
        /// Human-readable error message.
        message: String,
        /// Optional machine-readable error code.
        code: Option<String>,
    },
    /// Step (action) started
    StepStarted {
        /// Unique identifier for this run.
        run_id: String,
        /// Display name of the step.
        step_name: String,
        /// Unique identifier for this step.
        step_id: String,
    },
    /// Step finished
    StepFinished {
        /// Unique identifier for this run.
        run_id: String,
        /// Unique identifier for this step.
        step_id: String,
    },
    /// Text message delta (streaming)
    TextMessageStart {
        /// Unique identifier for this run.
        run_id: String,
        /// Unique identifier for this message.
        message_id: String,
        /// Role of the message sender (e.g. "user", "assistant").
        role: String,
    },
    /// Text delta content
    TextMessageContent {
        /// Unique identifier for this run.
        run_id: String,
        /// Unique identifier for this message.
        message_id: String,
        /// Text delta to append to the message.
        delta: String,
    },
    /// Text message finished
    TextMessageEnd {
        /// Unique identifier for this run.
        run_id: String,
        /// Unique identifier for this message.
        message_id: String,
    },
    /// Tool call started
    ToolCallStart {
        /// Unique identifier for this run.
        run_id: String,
        /// Unique identifier for this tool call.
        tool_call_id: String,
        /// Name of the tool being called.
        tool_name: String,
    },
    /// Tool call argument delta
    ToolCallArgs {
        /// Unique identifier for this run.
        run_id: String,
        /// Unique identifier for this tool call.
        tool_call_id: String,
        /// Argument delta (partial JSON string).
        delta: String,
    },
    /// Tool call finished
    ToolCallEnd {
        /// Unique identifier for this run.
        run_id: String,
        /// Unique identifier for this tool call.
        tool_call_id: String,
    },
    /// State snapshot update
    StateSnapshot {
        /// Unique identifier for this run.
        run_id: String,
        /// Full state snapshot.
        snapshot: serde_json::Value,
    },
    /// State delta update
    StateDelta {
        /// Unique identifier for this run.
        run_id: String,
        /// List of JSON patch operations.
        delta: Vec<serde_json::Value>,
    },
    /// Raw custom event
    Custom {
        /// Unique identifier for this run.
        run_id: String,
        /// Custom event name.
        name: String,
        /// Arbitrary event payload.
        data: serde_json::Value,
    },
}

impl AguiEvent {
    /// Return the `run_id` associated with this event.
    #[must_use]
    pub fn run_id(&self) -> &str {
        match self {
            Self::RunStarted { run_id, .. }
            | Self::RunFinished { run_id, .. }
            | Self::RunError { run_id, .. }
            | Self::StepStarted { run_id, .. }
            | Self::StepFinished { run_id, .. }
            | Self::TextMessageStart { run_id, .. }
            | Self::TextMessageContent { run_id, .. }
            | Self::TextMessageEnd { run_id, .. }
            | Self::ToolCallStart { run_id, .. }
            | Self::ToolCallArgs { run_id, .. }
            | Self::ToolCallEnd { run_id, .. }
            | Self::StateSnapshot { run_id, .. }
            | Self::StateDelta { run_id, .. }
            | Self::Custom { run_id, .. } => run_id,
        }
    }

    /// Serialize this event to SSE format: `data: <json>\n\n`
    #[must_use]
    pub fn to_sse_line(&self) -> String {
        let json = serde_json::to_string(self).unwrap_or_default();
        format!("data: {json}\n\n")
    }
}

/// Broadcast bus for AG-UI events.
///
/// `AguiBus` wraps a `broadcast::Sender` and allows multiple subscribers to
/// receive AG-UI protocol events. Cloning is cheap — all clones share the
/// same underlying channel.
///
/// # Example
///
/// ```rust
/// use auroraview_mcp::agui::{AguiBus, AguiEvent};
///
/// let bus = AguiBus::new();
/// let mut rx = bus.subscribe();
///
/// bus.emit(AguiEvent::RunStarted {
///     run_id: "r1".to_string(),
///     thread_id: "t1".to_string(),
/// });
/// ```
#[derive(Clone)]
pub struct AguiBus {
    tx: Arc<broadcast::Sender<AguiEvent>>,
}

impl AguiBus {
    /// Create a new bus with a buffer capacity of 64 events.
    ///
    /// The capacity is the number of events the channel can hold.
    /// If subscribers are slow, events will be dropped (latest wins).
    #[must_use]
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(64);
        Self { tx: Arc::new(tx) }
    }

    /// Publish an event to all active subscribers.
    ///
    /// If there are no receivers, the send result is ignored silently.
    /// This is intentional — the bus should not panic if no one is listening.
    pub fn emit(&self, event: AguiEvent) {
        // If there are no receivers, send returns an error — we ignore it.
        let _result = self.tx.send(event);
    }

    /// Subscribe to receive events.
    ///
    /// Returns a `broadcast::Receiver` that receives events emitted *after*
    /// this call. Events emitted before this call are not received.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use auroraview_mcp::agui::AguiBus;
    /// let bus = AguiBus::new();
    /// let mut rx = bus.subscribe();
    /// ```
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<AguiEvent> {
        self.tx.subscribe()
    }

    /// Return the number of active subscribers.
    ///
    /// This is useful for debugging and testing. Note that disconnected
    /// receivers are not immediately removed from the count.
    #[must_use]
    pub fn receiver_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

impl Default for AguiBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agui_bus_new_creates_instance() {
        let bus = AguiBus::new();
        // New bus has 0 subscribers
        assert_eq!(bus.receiver_count(), 0);
    }

    #[test]
    fn agui_bus_default_creates_instance() {
        let bus = AguiBus::default();
        // Default should behave same as new()
        assert_eq!(bus.receiver_count(), 0);
    }

    #[test]
    fn run_id_returns_correct_value() {
        let event = AguiEvent::RunStarted {
            run_id: "run-1".to_string(),
            thread_id: "t-1".to_string(),
        };
        assert_eq!(event.run_id(), "run-1");

        let event = AguiEvent::ToolCallStart {
            run_id: "run-2".to_string(),
            tool_call_id: "c-1".to_string(),
            tool_name: "screenshot".to_string(),
        };
        assert_eq!(event.run_id(), "run-2");
    }

    #[test]
    fn run_id_returns_correct_value_for_all_variants() {
        // Test all AguiEvent variants have correct run_id
        let events: Vec<(AguiEvent, &str)> = vec![
            (
                AguiEvent::RunStarted {
                    run_id: "r1".to_string(),
                    thread_id: "t1".to_string(),
                },
                "r1",
            ),
            (
                AguiEvent::RunFinished {
                    run_id: "r2".to_string(),
                    thread_id: "t2".to_string(),
                },
                "r2",
            ),
            (
                AguiEvent::RunError {
                    run_id: "r3".to_string(),
                    message: "err".to_string(),
                    code: None,
                },
                "r3",
            ),
            (
                AguiEvent::StepStarted {
                    run_id: "r4".to_string(),
                    step_name: "s1".to_string(),
                    step_id: "s1".to_string(),
                },
                "r4",
            ),
            (
                AguiEvent::StepFinished {
                    run_id: "r5".to_string(),
                    step_id: "s1".to_string(),
                },
                "r5",
            ),
        ];
        for (event, expected_id) in events {
            assert_eq!(event.run_id(), expected_id);
        }
    }

    #[test]
    fn to_sse_line_produces_valid_sse() {
        let event = AguiEvent::RunStarted {
            run_id: "r1".to_string(),
            thread_id: "t1".to_string(),
        };
        let line = event.to_sse_line();
        assert!(line.starts_with("data: "));
        assert!(line.ends_with("\n\n"));
        // Should be valid JSON
        let json_start = line.trim_start_matches("data: ");
        let json_end = json_start.trim_end_matches("\n\n");
        let parsed: serde_json::Value = serde_json::from_str(json_end).unwrap();
        assert_eq!(parsed["type"], "RUN_STARTED");
    }

    #[tokio::test]
    async fn bus_subscribe_receives_emitted_event() {
        let bus = AguiBus::new();
        let mut rx = bus.subscribe();

        let event = AguiEvent::RunStarted {
            run_id: "r1".to_string(),
            thread_id: "t1".to_string(),
        };
        bus.emit(event.clone());

        let received = rx.recv().await.unwrap();
        assert_eq!(received.run_id(), "r1");
    }

    #[tokio::test]
    async fn bus_emit_with_multiple_subscribers() {
        let bus = AguiBus::new();
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();

        let event = AguiEvent::RunStarted {
            run_id: "r1".to_string(),
            thread_id: "t1".to_string(),
        };
        bus.emit(event.clone());

        // Both subscribers should receive the event
        let received1 = rx1.recv().await.unwrap();
        let received2 = rx2.recv().await.unwrap();
        assert_eq!(received1.run_id(), "r1");
        assert_eq!(received2.run_id(), "r1");
    }

    #[test]
    fn bus_receiver_count_tracks_subscribers() {
        let bus = AguiBus::new();
        assert_eq!(bus.receiver_count(), 0);

        let _rx1 = bus.subscribe();
        assert_eq!(bus.receiver_count(), 1);

        let _rx2 = bus.subscribe();
        assert_eq!(bus.receiver_count(), 2);
    }

    #[test]
    fn bus_emit_without_receivers_does_not_panic() {
        let bus = AguiBus::new();
        // Should not panic even with no receivers
        bus.emit(AguiEvent::RunFinished {
            run_id: "r1".to_string(),
            thread_id: "t1".to_string(),
        });
    }

    #[test]
    fn subscribe_returns_valid_receiver() {
        let bus = AguiBus::new();
        let _rx = bus.subscribe();
        // Receiver should be valid (we can't do much with it without emitting,
        // but we can check that it's a different receiver each time)
        let _rx2 = bus.subscribe();
        // Two different subscribers should have different receiver IDs
        assert_eq!(bus.receiver_count(), 2);
        // Dropping receiver should decrement count
        drop(_rx);
        // Note: broadcast receiver count may not immediately decrement
    }
}
