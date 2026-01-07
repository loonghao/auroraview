//! MCP Sidecar Service - implements rmcp ServerHandler trait.
//!
//! This service forwards all tool calls to the main process via IPC.

use std::sync::Arc;

use rmcp::{
    handler::server::ServerHandler,
    model::{
        CallToolResult, CallToolRequestParam, Content, Implementation, InitializeResult,
        ListToolsResult, PaginatedRequestParam, ProtocolVersion, ServerCapabilities,
        Tool as RmcpToolInfo,
    },
    service::{RequestContext, RoleServer},
    ErrorData as RmcpError,
};
use serde_json::Value;

use crate::ipc::IpcClient;
use crate::protocol::ToolDefinition;

/// MCP Sidecar Service that forwards requests to the main process.
#[derive(Clone)]
pub struct McpSidecarService {
    /// Server name
    name: String,
    /// Server version
    version: String,
    /// IPC client for communicating with main process
    ipc_client: Arc<IpcClient>,
    /// Cached tool definitions from main process
    tools: Arc<parking_lot::RwLock<Vec<ToolDefinition>>>,
}

impl McpSidecarService {
    /// Create a new sidecar service.
    pub fn new(name: String, version: String, ipc_client: Arc<IpcClient>) -> Self {
        Self {
            name,
            version,
            ipc_client,
            tools: Arc::new(parking_lot::RwLock::new(Vec::new())),
        }
    }

    /// Refresh tool list from main process.
    pub fn refresh_tools(&self) -> Result<(), String> {
        match self.ipc_client.get_tool_list() {
            Ok(tools) => {
                *self.tools.write() = tools;
                tracing::info!("[McpSidecarService] Refreshed {} tools", self.tools.read().len());
                Ok(())
            }
            Err(e) => {
                tracing::error!("[McpSidecarService] Failed to refresh tools: {}", e);
                Err(e.to_string())
            }
        }
    }

    /// Convert internal tool definition to rmcp format.
    fn to_rmcp_tool(tool: &ToolDefinition) -> RmcpToolInfo {
        let schema = tool
            .input_schema
            .clone()
            .and_then(|v| v.as_object().cloned())
            .map(Arc::new)
            .unwrap_or_else(|| Arc::new(serde_json::Map::new()));

        RmcpToolInfo::new(
            tool.name.clone(),
            tool.description.clone().unwrap_or_default(),
            schema,
        )
    }
}

impl ServerHandler for McpSidecarService {
    fn get_info(&self) -> InitializeResult {
        InitializeResult {
            protocol_version: ProtocolVersion::LATEST,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation {
                name: self.name.clone(),
                version: self.version.clone(),
                title: Some("AuroraView MCP Sidecar".to_string()),
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "This MCP server provides access to AuroraView tools. \
                Use tools/list to see available tools and tools/call to execute them."
                    .to_string(),
            ),
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, RmcpError> {
        tracing::info!("[McpSidecarService] Received list_tools request");

        // Check if IPC is still connected
        if !self.ipc_client.is_connected() {
            tracing::error!("[McpSidecarService] IPC client disconnected, cannot list tools");
            return Err(RmcpError::internal_error(
                "IPC connection lost - main process may have exited",
                None,
            ));
        }

        // Try to refresh tools from main process
        if let Err(e) = self.refresh_tools() {
            tracing::warn!("[McpSidecarService] Could not refresh tools: {}", e);
        }

        let tools = self.tools.read();
        let rmcp_tools: Vec<RmcpToolInfo> = tools.iter().map(Self::to_rmcp_tool).collect();

        tracing::info!(
            "[McpSidecarService] Returning {} tools",
            rmcp_tools.len()
        );

        Ok(ListToolsResult {
            tools: rmcp_tools,
            next_cursor: None,
            meta: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, RmcpError> {
        let tool_name: &str = &request.name;
        let arguments = request
            .arguments
            .map(|m| Value::Object(m.into_iter().collect()))
            .unwrap_or(Value::Null);

        tracing::info!(
            "[McpSidecarService] Calling tool: {} with args: {:?}",
            tool_name,
            arguments
        );

        // Check if IPC is still connected
        if !self.ipc_client.is_connected() {
            tracing::error!("[McpSidecarService] IPC client disconnected, cannot call tool");
            return Err(RmcpError::internal_error(
                "IPC connection lost - main process may have exited",
                None,
            ));
        }

        match self.ipc_client.call_tool(tool_name, arguments, None) {
            Ok(result) => {
                tracing::info!("[McpSidecarService] Tool {} returned successfully", tool_name);
                let content = Content::text(serde_json::to_string_pretty(&result).unwrap_or_default());
                Ok(CallToolResult::success(vec![content]))
            }
            Err(e) => {
                tracing::error!("[McpSidecarService] Tool call failed: {}", e);
                let content = Content::text(format!("Error: {}", e));
                Ok(CallToolResult::error(vec![content]))
            }
        }
    }
}

