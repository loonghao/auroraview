/// AG-UI protocol event types for AuroraView MCP Server.
///
/// AG-UI is a protocol for streaming agent state updates to UI clients.
/// See: <https://docs.ag-ui.com>
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

/// AG-UI event types per the protocol spec.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AguiEvent {
    /// Run lifecycle: started
    RunStarted {
        run_id: String,
        thread_id: String,
    },
    /// Run lifecycle: finished
    RunFinished {
        run_id: String,
        thread_id: String,
    },
    /// Run lifecycle: error
    RunError {
        run_id: String,
        message: String,
        code: Option<String>,
    },
    /// Step (action) started
    StepStarted {
        run_id: String,
        step_name: String,
        step_id: String,
    },
    /// Step finished
    StepFinished {
        run_id: String,
        step_id: String,
    },
    /// Text message delta (streaming)
    TextMessageStart {
        run_id: String,
        message_id: String,
        role: String,
    },
    /// Text delta content
    TextMessageContent {
        run_id: String,
        message_id: String,
        delta: String,
    },
    /// Text message finished
    TextMessageEnd {
        run_id: String,
        message_id: String,
    },
    /// Tool call started
    ToolCallStart {
        run_id: String,
        tool_call_id: String,
        tool_name: String,
    },
    /// Tool call argument delta
    ToolCallArgs {
        run_id: String,
        tool_call_id: String,
        delta: String,
    },
    /// Tool call finished
    ToolCallEnd {
        run_id: String,
        tool_call_id: String,
    },
    /// State snapshot update
    StateSnapshot {
        run_id: String,
        snapshot: serde_json::Value,
    },
    /// State delta update
    StateDelta {
        run_id: String,
        delta: Vec<serde_json::Value>,
    },
    /// Raw custom event
    Custom {
        run_id: String,
        name: String,
        data: serde_json::Value,
    },
}

impl AguiEvent {
    /// Return the `run_id` associated with this event.
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
    pub fn to_sse_line(&self) -> String {
        let json = serde_json::to_string(self).unwrap_or_default();
        format!("data: {json}\n\n")
    }
}

/// Broadcast bus for AG-UI events.  
/// Cloning this is cheap — all clones share the same channel.
#[derive(Clone)]
pub struct AguiBus {
    tx: Arc<broadcast::Sender<AguiEvent>>,
}

impl AguiBus {
    /// Create a new bus with a buffer capacity of 256 events.
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        Self { tx: Arc::new(tx) }
    }

    /// Publish an event to all active subscribers.
    pub fn emit(&self, event: AguiEvent) {
        // If there are no receivers, send returns an error — we ignore it.
        let _result = self.tx.send(event);
    }

    /// Subscribe to receive events.  Returns a `Receiver` that receives
    /// events emitted *after* this call.
    pub fn subscribe(&self) -> broadcast::Receiver<AguiEvent> {
        self.tx.subscribe()
    }

    /// Return the number of active subscribers.
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
}
