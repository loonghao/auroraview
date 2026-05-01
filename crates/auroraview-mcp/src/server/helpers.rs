// Helper methods for AuroraViewMcpServer.
// Extracted from server.rs to keep files under 1000 lines.

use super::AuroraViewMcpServer;
use crate::{
    agui::{AguiBus, AguiEvent},
    registry::WebViewRegistry,
    types::McpServerConfig,
};
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

impl AuroraViewMcpServer {
    /// Attach an `AguiBus` so tool invocations automatically emit AG-UI events.
    #[must_use] 
    pub fn with_agui_bus(mut self, bus: AguiBus) -> Self {
        self.agui_bus = Some(bus);
        self
    }

    #[must_use] 
    pub fn registry(&self) -> &WebViewRegistry {
        &self.registry
    }

    #[must_use] 
    pub fn config(&self) -> &McpServerConfig {
        &self.config
    }

    /// Return a reference to the attached AG-UI bus, if any.
    #[must_use] 
    pub fn agui_bus(&self) -> Option<&AguiBus> {
        self.agui_bus.as_ref()
    }

    /// Resolve a `WebView` ID: use provided string or fall back to first registered.
    pub fn resolve_id(&self, id: Option<&str>) -> String {
        if let Some(s) = id {
            return s.to_string();
        }
        self.registry()
            .list()
            .into_iter()
            .next()
            .map_or_else(|| "default".to_string(), |v| v.id.0)
    }

    /// Emit `ToolCallStart` when a tool begins execution.
    pub fn emit_tool_start(&self, tool_name: &str, call_id: &str, run_id: &str) {
        if let Some(bus) = &self.agui_bus {
            bus.emit(AguiEvent::ToolCallStart {
                run_id: run_id.to_string(),
                tool_call_id: call_id.to_string(),
                tool_name: tool_name.to_string(),
            });
        }
    }

    /// Emit `ToolCallEnd` when a tool finishes execution.
    pub fn emit_tool_end(&self, call_id: &str, run_id: &str) {
        if let Some(bus) = &self.agui_bus {
            bus.emit(AguiEvent::ToolCallEnd {
                run_id: run_id.to_string(),
                tool_call_id: call_id.to_string(),
            });
        }
    }
}
