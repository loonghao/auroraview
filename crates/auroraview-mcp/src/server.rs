//! MCP Server implementation with Streamable HTTP transport (rmcp SDK 0.12+)

use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::Arc;
use std::time::Duration;

use axum::{http::Method, Router};
use parking_lot::RwLock;
use rmcp::{
    handler::server::ServerHandler,
    model::{
        AnnotateAble, CallToolResult, Content, GetPromptResult as RmcpGetPromptResult,
        Implementation, InitializeResult, ListPromptsResult, ListToolsResult,
        PaginatedRequestParam, Prompt as RmcpPrompt, PromptMessage as RmcpPromptMessage,
        PromptMessageContent, PromptMessageRole, ProtocolVersion, ServerCapabilities,
        Tool as RmcpToolInfo,
    },
    service::RequestContext,
    transport::streamable_http_server::{
        session::local::LocalSessionManager,
        tower::{StreamableHttpServerConfig, StreamableHttpService},
    },
    ErrorData as RmcpError,
};
use serde_json::Value;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tower_http::cors::{Any as CorsAny, CorsLayer};
use tracing::{debug, info, warn};

use crate::config::McpConfig;
use crate::error::{McpError, McpResult};
use crate::tool::{Prompt, PromptRegistry, Tool, ToolRegistry};

/// AuroraView MCP Service Handler
///
/// This struct implements the rmcp ServerHandler trait to handle MCP requests
#[derive(Clone)]
pub struct AuroraViewService {
    /// Server configuration
    config: McpConfig,
    /// Registered tools
    tools: Arc<ToolRegistry>,
    /// Registered prompts
    prompts: Arc<RwLock<PromptRegistry>>,
}

impl AuroraViewService {
    /// Create a new service instance
    pub fn new(
        config: McpConfig,
        tools: Arc<ToolRegistry>,
        prompts: Arc<RwLock<PromptRegistry>>,
    ) -> Self {
        Self {
            config,
            tools,
            prompts,
        }
    }
}

/// Implement the rmcp ServerHandler trait for AuroraViewService
impl ServerHandler for AuroraViewService {
    fn get_info(&self) -> InitializeResult {
        let prompts = self.prompts.read();
        let _has_prompts = !prompts.is_empty();

        InitializeResult {
            protocol_version: ProtocolVersion::LATEST,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .build(),
            server_info: Implementation {
                name: self.config.name.clone(),
                version: self.config.version.clone(),
                title: None,
                icons: None,
                website_url: None,
            },
            instructions: None,
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<rmcp::service::RoleServer>,
    ) -> Result<ListToolsResult, RmcpError> {
        let tool_defs = self.tools.definitions();
        info!("[MCP] list_tools: {} tools registered", tool_defs.len());

        let rmcp_tools: Vec<RmcpToolInfo> = tool_defs
            .into_iter()
            .map(|def| {
                debug!("[MCP] tool: {}", def.name);
                // Convert serde_json::Value to Arc<JsonObject>
                let schema = match def.input_schema {
                    Value::Object(obj) => Arc::new(obj),
                    _ => Arc::new(serde_json::Map::new()),
                };
                RmcpToolInfo::new(def.name, def.description, schema)
            })
            .collect();

        Ok(ListToolsResult {
            tools: rmcp_tools,
            next_cursor: None,
            meta: None,
        })
    }

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<rmcp::service::RoleServer>,
    ) -> Result<ListPromptsResult, RmcpError> {
        let prompt_defs = self.prompts.read().definitions();
        info!(
            "[MCP] list_prompts: {} prompts registered",
            prompt_defs.len()
        );

        // Convert PromptDefinition to RmcpPrompt
        let prompts: Vec<RmcpPrompt> = prompt_defs
            .into_iter()
            .map(|def| {
                let arguments = def.arguments.map(|args| {
                    args.into_iter()
                        .map(|arg| rmcp::model::PromptArgument {
                            name: arg.name,
                            description: Some(arg.description),
                            required: arg.required,
                            title: None,
                        })
                        .collect()
                });
                RmcpPrompt {
                    name: def.name,
                    description: Some(def.description),
                    arguments,
                    title: None,
                    icons: None,
                    meta: None,
                }
            })
            .collect();

        Ok(ListPromptsResult {
            prompts,
            next_cursor: None,
            meta: None,
        })
    }

    async fn get_prompt(
        &self,
        request: rmcp::model::GetPromptRequestParam,
        _context: RequestContext<rmcp::service::RoleServer>,
    ) -> Result<RmcpGetPromptResult, RmcpError> {
        let name = request.name.as_ref();
        // arguments is Option<JsonObject> where JsonObject = Map<String, Value>
        let args = match request.arguments {
            Some(obj) => obj.clone(),
            None => serde_json::Map::new(),
        };

        info!(
            "[MCP] get_prompt: name={}, args={}",
            name,
            serde_json::to_string(&args).unwrap_or_else(|_| "{?}".to_string())
        );

        // Convert our GetPromptResult to rmcp's GetPromptResult
        let prompt_result = self
            .prompts
            .read()
            .execute(name, args)
            .map_err(|e| RmcpError::invalid_params(e.to_string(), None))?;

        // Convert our PromptMessage enum to rmcp's PromptMessage
        let messages: Vec<RmcpPromptMessage> = match prompt_result.messages {
            Some(msgs) => msgs
                .into_iter()
                .map(|msg| {
                    let (role, inner_content) = match msg {
                        crate::protocol::PromptMessage::User { content } => {
                            (PromptMessageRole::User, content)
                        }
                        crate::protocol::PromptMessage::Assistant { content } => {
                            (PromptMessageRole::Assistant, content)
                        }
                        crate::protocol::PromptMessage::System { content } => {
                            // rmcp doesn't have System role, map to User
                            (PromptMessageRole::User, content)
                        }
                    };

                    let rmcp_content = match inner_content {
                        crate::protocol::Content::Text { text: ref t } => {
                            PromptMessageContent::Text { text: t.clone() }
                        }
                        crate::protocol::Content::Image {
                            data: ref d,
                            mime_type: ref m,
                        } => PromptMessageContent::Image {
                            image: rmcp::model::RawImageContent {
                                data: d.clone(),
                                mime_type: m.clone(),
                                meta: None,
                            }
                            .optional_annotate(None),
                        },
                        crate::protocol::Content::Resource { uri, .. } => {
                            // For now, convert resource to text with URI
                            PromptMessageContent::Text {
                                text: format!("resource:{}", uri),
                            }
                        }
                    };

                    RmcpPromptMessage {
                        role,
                        content: rmcp_content,
                    }
                })
                .collect(),
            None => vec![],
        };

        Ok(RmcpGetPromptResult {
            description: Some(prompt_result.prompt.description),
            messages,
        })
    }

    async fn call_tool(
        &self,
        request: rmcp::model::CallToolRequestParam,
        _context: RequestContext<rmcp::service::RoleServer>,
    ) -> Result<CallToolResult, RmcpError> {
        info!("[MCP] call_tool ENTRY - request received");
        let name = request.name.as_ref();
        // arguments is Option<JsonObject> where JsonObject = Map<String, Value>
        let args = match request.arguments {
            Some(obj) => Value::Object(obj),
            None => Value::Object(Default::default()),
        };

        info!("[MCP] call_tool: name={}, args={}", name, args);

        // Log before calling async handler
        info!("[MCP] call_tool: about to call tool registry for {}", name);

        match self.tools.call_async(name, args).await {
            Ok(result) => {
                let text =
                    serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string());
                info!(
                    "[MCP] call_tool success: name={}, result_len={}",
                    name,
                    text.len()
                );
                debug!("[MCP] call_tool result: {}", &text[..text.len().min(500)]);
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(e) => {
                warn!("[MCP] call_tool error: name={}, error={}", name, e);
                Ok(CallToolResult::error(vec![Content::text(e.to_string())]))
            }
        }
    }
}

/// MCP Server with Streamable HTTP transport
pub struct McpServer {
    config: McpConfig,
    tools: Arc<ToolRegistry>,
    prompts: Arc<RwLock<PromptRegistry>>,
    port: AtomicU16,
    running: AtomicBool,
    shutdown_tx: RwLock<Option<mpsc::Sender<()>>>,
    cancellation_token: CancellationToken,
    /// Handle to the server task for graceful shutdown
    server_task: RwLock<Option<tokio::task::JoinHandle<()>>>,
}

impl McpServer {
    /// Create a new MCP Server
    pub fn new(config: McpConfig) -> Self {
        Self {
            config,
            tools: Arc::new(ToolRegistry::new()),
            prompts: Arc::new(RwLock::new(PromptRegistry::new())),
            port: AtomicU16::new(0),
            running: AtomicBool::new(false),
            shutdown_tx: RwLock::new(None),
            cancellation_token: CancellationToken::new(),
            server_task: RwLock::new(None),
        }
    }

    /// Create with default config
    pub fn with_default_config() -> Self {
        Self::new(McpConfig::default())
    }

    /// Get the tool registry
    pub fn tools(&self) -> &Arc<ToolRegistry> {
        &self.tools
    }

    /// Register a tool
    pub fn register_tool(&self, tool: Tool) {
        self.tools.register(tool);
    }

    /// Register a prompt
    pub fn register_prompt(&self, prompt: Prompt) -> McpResult<()> {
        let mut prompts = self.prompts.write();
        prompts.register(prompt)
    }

    /// Get the prompt registry
    pub fn prompts(&self) -> Arc<RwLock<PromptRegistry>> {
        Arc::clone(&self.prompts)
    }

    /// Check if server is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Get the server port (0 if not running)
    pub fn port(&self) -> u16 {
        self.port.load(Ordering::SeqCst)
    }

    /// Start the server with Streamable HTTP transport
    pub async fn start(&self) -> McpResult<u16> {
        if self.running.load(Ordering::SeqCst) {
            let port = self.port.load(Ordering::SeqCst);
            warn!("MCP Server is already running on port {}", port);
            return Ok(port);
        }

        // Find available port
        let port = if self.config.port == 0 {
            find_free_port()?
        } else {
            self.config.port
        };

        let addr: SocketAddr = format!("{}:{}", self.config.host, port)
            .parse()
            .map_err(|e| McpError::StartFailed {
                port: port.to_string(),
                reason: format!("Invalid address: {}", e),
            })?;

        // Create service factory for rmcp
        let config = self.config.clone();
        let tools = Arc::clone(&self.tools);
        let prompts = Arc::clone(&self.prompts);
        let service_factory = move || {
            Ok(AuroraViewService::new(
                config.clone(),
                Arc::clone(&tools),
                Arc::clone(&prompts),
            ))
        };

        // Configure Streamable HTTP server
        let http_config = StreamableHttpServerConfig {
            sse_keep_alive: Some(Duration::from_secs(self.config.heartbeat_interval)),
            stateful_mode: false,
            cancellation_token: self.cancellation_token.clone(),
        };

        // Create session manager and HTTP service
        let session_manager = Arc::new(LocalSessionManager::default());
        let mcp_service = StreamableHttpService::new(service_factory, session_manager, http_config);

        // Build CORS layer
        let cors = CorsLayer::new()
            .allow_origin(CorsAny)
            .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
            .allow_headers(tower_http::cors::Any)
            .max_age(Duration::from_secs(86400));

        // Build router with MCP endpoint
        // Use nest_service to properly route all /mcp/* paths to the MCP service
        // This is the pattern used in the official rmcp examples
        info!("Registering MCP endpoint at /mcp (using nest_service)");
        let tools_state = Arc::clone(&self.tools);
        let app = Router::new()
            .nest_service("/mcp", mcp_service)
            .route("/health", axum::routing::get(health_handler))
            .route(
                "/tools",
                axum::routing::get(move || {
                    let tools = Arc::clone(&tools_state);
                    async move { tools_handler_impl(tools) }
                }),
            )
            .layer(cors);

        // Create shutdown channel
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        *self.shutdown_tx.write() = Some(shutdown_tx);

        // Start server
        let listener =
            tokio::net::TcpListener::bind(addr)
                .await
                .map_err(|e| McpError::StartFailed {
                    port: port.to_string(),
                    reason: format!("Failed to bind to address: {}", e),
                })?;

        let actual_port = listener.local_addr().map(|a| a.port()).unwrap_or(port);

        self.port.store(actual_port, Ordering::SeqCst);
        self.running.store(true, Ordering::SeqCst);

        info!(
            "MCP Server '{}' started on {}:{} (Streamable HTTP)",
            self.config.name, self.config.host, actual_port
        );
        info!(
            "MCP endpoint: http://{}:{}/mcp",
            self.config.host, actual_port
        );

        // Spawn server task
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = Arc::clone(&running);

        let task_handle = tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    shutdown_rx.recv().await;
                    info!("MCP Server shutting down");
                })
                .await
                .ok();

            running_clone.store(false, Ordering::SeqCst);
        });

        // Store task handle for graceful shutdown
        *self.server_task.write() = Some(task_handle);

        Ok(actual_port)
    }

    /// Stop the server
    pub async fn stop(&self) {
        if !self.running.load(Ordering::SeqCst) {
            return;
        }

        // Cancel all sessions
        self.cancellation_token.cancel();

        // Send shutdown signal - take the sender first to release the lock
        let shutdown_tx = self.shutdown_tx.write().take();
        if let Some(tx) = shutdown_tx {
            let _ = tx.send(()).await;
        }

        // Wait for server task to complete with timeout - take the task first to release the lock
        let server_task = self.server_task.write().take();
        if let Some(task) = server_task {
            match tokio::time::timeout(Duration::from_secs(5), task).await {
                Ok(Ok(())) => {
                    debug!("MCP Server task completed gracefully");
                }
                Ok(Err(e)) => {
                    warn!("MCP Server task panicked: {:?}", e);
                }
                Err(_) => {
                    warn!("MCP Server task did not complete within timeout, aborting");
                }
            }
        }

        self.running.store(false, Ordering::SeqCst);
        self.port.store(0, Ordering::SeqCst);

        info!("MCP Server stopped");
    }

    /// Broadcast an event to all connected clients
    pub async fn broadcast(&self, event: &str, data: Value) {
        // Placeholder for event broadcasting
        debug!("Broadcasting event: {} with data: {:?}", event, data);
    }
}

/// Find a free port
fn find_free_port() -> McpResult<u16> {
    let listener =
        std::net::TcpListener::bind("127.0.0.1:0").map_err(|e| McpError::StartFailed {
            port: "auto".to_string(),
            reason: format!("Failed to find free port: {}", e),
        })?;
    let port = listener.local_addr()?.port();
    Ok(port)
}

/// Health check handler
async fn health_handler() -> axum::Json<Value> {
    axum::Json(serde_json::json!({
        "status": "ok",
        "transport": "streamable-http"
    }))
}

/// Tools listing handler implementation
fn tools_handler_impl(tools: Arc<ToolRegistry>) -> axum::Json<Value> {
    let definitions = tools.definitions();
    let tools_list: Vec<Value> = definitions
        .into_iter()
        .map(|def| {
            serde_json::json!({
                "name": def.name,
                "description": def.description,
                "inputSchema": def.input_schema
            })
        })
        .collect();

    axum::Json(serde_json::json!({
        "ok": true,
        "data": tools_list,
        "count": tools_list.len()
    }))
}
