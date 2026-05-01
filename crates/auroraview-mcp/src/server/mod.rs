use crate::{agui::AguiBus, registry::WebViewRegistry, types::McpServerConfig};
use rmcp::tool_router;
use std::sync::Arc;

// Sub-modules
pub mod types;
pub mod tools;
pub mod helpers;
pub mod handler;

// Re-exports
pub use types::*;
pub use tools::AuroraViewMcpServer;
pub use helpers::*;
pub use handler::*;
