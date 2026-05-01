// ServerHandler trait implementation for AuroraViewMcpServer.
// Extracted from server.rs to keep files under 1000 lines.

use crate::server::tools::AuroraViewMcpServer;
use rmcp::{
    RoleServer,
    ServerHandler,
    handler::server::tool::ToolCallContext,
    model::{CallToolResult, CallToolRequestParams, InitializeResult, ListToolsResult,
        PaginatedRequestParams, ServerCapabilities},
    service::RequestContext,
    ErrorData,
};

impl ServerHandler for AuroraViewMcpServer {
    fn get_info(&self) -> InitializeResult {
        let capabilities = ServerCapabilities::builder().enable_tools().build();
        InitializeResult::new(capabilities)
            .with_instructions(
                "AuroraView MCP Server: manage WebView windows in DCC applications (Maya, Houdini, Blender, UE, etc.)".to_string()
            )
    }

    fn call_tool(
        &self,
        req: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, ErrorData>> + Send + '_ {
        let ctx = ToolCallContext::new(self, req, context);
        self.tool_router.call(ctx)
    }

    fn list_tools(
        &self,
        _req: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, ErrorData>> + Send + '_
    {
        let tools = self.tool_router.list_all();
        async move {
            Ok(ListToolsResult {
                tools,
                ..Default::default()
            })
        }
    }
}
