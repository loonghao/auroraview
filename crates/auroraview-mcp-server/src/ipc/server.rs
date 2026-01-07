//! IPC Server for main AuroraView process.
//!
//! The IPC Server runs in the main AuroraView process and handles
//! tool call requests from the Sidecar process.

use super::{IpcError, IpcResult};
use crate::protocol::{
    AuthHelloParams, ErrorCode, Request, Response, RpcError, ToolCallParams, ToolDefinition,
};
use ipckit::local_socket::{LocalSocketListener, LocalSocketStream};
use parking_lot::RwLock;
use serde_json::Value;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Tool handler function type.
pub type ToolHandler = Box<dyn Fn(Value) -> Result<Value, String> + Send + Sync>;

/// IPC Server for main process.
///
/// Accepts connections from Sidecar and handles tool calls.
pub struct IpcServer {
    channel_name: String,
    expected_token: String,
    listener: RwLock<Option<LocalSocketListener>>,
    running: Arc<AtomicBool>,
    tool_handlers: Arc<RwLock<HashMap<String, ToolHandler>>>,
    tool_definitions: Arc<RwLock<Vec<ToolDefinition>>>,
}

impl IpcServer {
    /// Create a new IPC server.
    pub fn new(channel_name: impl Into<String>, expected_token: impl Into<String>) -> Self {
        Self::with_running_flag(
            channel_name,
            expected_token,
            Arc::new(AtomicBool::new(false)),
        )
    }

    /// Create a new IPC server with a shared running flag.
    ///
    /// This allows external code to signal shutdown by setting the flag to false.
    pub fn with_running_flag(
        channel_name: impl Into<String>,
        expected_token: impl Into<String>,
        running: Arc<AtomicBool>,
    ) -> Self {
        Self {
            channel_name: channel_name.into(),
            expected_token: expected_token.into(),
            listener: RwLock::new(None),
            running,
            tool_handlers: Arc::new(RwLock::new(HashMap::new())),
            tool_definitions: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a tool handler.
    pub fn register_tool<F>(&self, definition: ToolDefinition, handler: F)
    where
        F: Fn(Value) -> Result<Value, String> + Send + Sync + 'static,
    {
        let name = definition.name.clone();
        self.tool_definitions.write().push(definition);
        self.tool_handlers.write().insert(name, Box::new(handler));
    }

    /// Start the IPC server (blocking).
    ///
    /// This method can be called on `&self` (Arc-compatible).
    pub fn start(&self) -> IpcResult<()> {
        tracing::info!("[IpcServer] Starting on channel: {}", self.channel_name);

        let listener = LocalSocketListener::bind(&self.channel_name).map_err(|e| {
            tracing::error!("[IpcServer] Bind failed: {}", e);
            IpcError::ConnectionFailed(e.to_string())
        })?;

        *self.listener.write() = Some(listener);
        self.running.store(true, Ordering::SeqCst);

        tracing::info!("[IpcServer] Listening for connections...");
        self.accept_loop()
    }

    /// Signal the server to stop.
    ///
    /// This sets the running flag to false. The server will exit
    /// after the current accept() call returns (may need a dummy connection).
    pub fn stop(&self) {
        tracing::info!("[IpcServer] Stop requested");
        self.running.store(false, Ordering::SeqCst);
    }

    /// Check if the server is running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Get a clone of the running flag for external shutdown signaling.
    pub fn running_flag(&self) -> Arc<AtomicBool> {
        self.running.clone()
    }

    fn accept_loop(&self) -> IpcResult<()> {
        let listener_guard = self.listener.read();
        let listener = listener_guard.as_ref().unwrap();

        while self.running.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok(stream) => {
                    // Check if we should stop (dummy connection to unblock)
                    if !self.running.load(Ordering::SeqCst) {
                        tracing::debug!("[IpcServer] Shutdown signal received, exiting accept loop");
                        break;
                    }
                    tracing::info!("[IpcServer] Client connected");
                    if let Err(e) = self.handle_connection(stream) {
                        tracing::error!("[IpcServer] Connection error: {}", e);
                    }
                }
                Err(e) => {
                    if self.running.load(Ordering::SeqCst) {
                        tracing::error!("[IpcServer] Accept error: {}", e);
                    }
                    break;
                }
            }
        }

        tracing::info!("[IpcServer] Accept loop exited");
        Ok(())
    }

    fn handle_connection(&self, mut stream: LocalSocketStream) -> IpcResult<()> {
        let mut authenticated = false;
        let mut line = String::new();

        loop {
            line.clear();

            // Read a line from the stream
            let mut reader = BufReader::new(&mut stream);
            match reader.read_line(&mut line) {
                Ok(0) => {
                    tracing::info!("[IpcServer] Client disconnected");
                    break;
                }
                Ok(_) => {
                    let response = self.handle_request(&line, &mut authenticated);
                    let json = serde_json::to_string(&response)?;
                    writeln!(stream, "{}", json)?;
                    stream.flush()?;
                }
                Err(e) => {
                    tracing::error!("[IpcServer] Read error: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    fn handle_request(&self, line: &str, authenticated: &mut bool) -> Response {
        let request: Request = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(e) => {
                return Response::error_with_code(
                    None,
                    ErrorCode::ParseError,
                    format!("Failed to parse request: {}", e),
                );
            }
        };

        let id = request.id.clone();

        // Check authentication for non-auth methods
        if request.method != "auth.hello" && !*authenticated {
            return Response::error_with_code(
                id,
                ErrorCode::AuthenticationFailed,
                "Not authenticated",
            );
        }

        match request.method.as_str() {
            "auth.hello" => self.handle_auth_hello(id, request.params, authenticated),
            "tool.list" => self.handle_tool_list(id),
            "tool.call" => self.handle_tool_call(id, request.params),
            "lifecycle.shutdown" => {
                self.running.store(false, Ordering::SeqCst);
                Response::success(id, serde_json::json!({"status": "shutting_down"}))
            }
            _ => Response::error_with_code(
                id,
                ErrorCode::MethodNotFound,
                format!("Unknown method: {}", request.method),
            ),
        }
    }

    fn handle_auth_hello(
        &self,
        id: Option<crate::protocol::RequestId>,
        params: Option<Value>,
        authenticated: &mut bool,
    ) -> Response {
        let params: AuthHelloParams = match params {
            Some(p) => match serde_json::from_value(p) {
                Ok(p) => p,
                Err(e) => {
                    return Response::error_with_code(
                        id,
                        ErrorCode::InvalidParams,
                        format!("Invalid auth params: {}", e),
                    );
                }
            },
            None => {
                return Response::error_with_code(
                    id,
                    ErrorCode::InvalidParams,
                    "Missing auth params",
                );
            }
        };

        if params.token != self.expected_token {
            tracing::warn!("[IpcServer] Authentication failed: invalid token");
            return Response::error_with_code(id, ErrorCode::AuthenticationFailed, "Invalid token");
        }

        *authenticated = true;
        tracing::info!("[IpcServer] Client authenticated");
        Response::success(id, serde_json::json!({"status": "authenticated"}))
    }

    fn handle_tool_list(&self, id: Option<crate::protocol::RequestId>) -> Response {
        let tools = self.tool_definitions.read().clone();
        Response::success(
            id,
            serde_json::to_value(tools).unwrap_or(Value::Array(vec![])),
        )
    }

    fn handle_tool_call(
        &self,
        id: Option<crate::protocol::RequestId>,
        params: Option<Value>,
    ) -> Response {
        let params: ToolCallParams = match params {
            Some(p) => match serde_json::from_value(p) {
                Ok(p) => p,
                Err(e) => {
                    return Response::error_with_code(
                        id,
                        ErrorCode::InvalidParams,
                        format!("Invalid tool call params: {}", e),
                    );
                }
            },
            None => {
                return Response::error_with_code(
                    id,
                    ErrorCode::InvalidParams,
                    "Missing tool call params",
                );
            }
        };

        let handlers = self.tool_handlers.read();
        let handler = match handlers.get(&params.name) {
            Some(h) => h,
            None => {
                return Response::error(id, RpcError::tool_not_found(&params.name));
            }
        };

        match handler(params.arguments) {
            Ok(result) => Response::success(id, result),
            Err(e) => Response::error(id, RpcError::execution_error(e)),
        }
    }

    /// Get the channel name.
    pub fn channel_name(&self) -> &str {
        &self.channel_name
    }
}
