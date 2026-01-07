//! IPC Client for Sidecar process.
//!
//! The IPC Client runs in the Sidecar process and connects to the main
//! AuroraView process's IPC Server to forward tool calls.

use super::{IpcError, IpcResult};
use crate::protocol::{AuthHelloParams, Request, Response, ToolCallParams, ToolDefinition};
use ipckit::local_socket::LocalSocketStream;
use parking_lot::Mutex;
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};

/// IPC Client for Sidecar process.
///
/// Connects to the main process's IPC Server and forwards tool calls.
pub struct IpcClient {
    channel_name: String,
    token: String,
    stream: Mutex<Option<LocalSocketStream>>,
    connected: AtomicBool,
    next_id: AtomicI64,
}

impl IpcClient {
    /// Create a new IPC client.
    pub fn new(channel_name: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            channel_name: channel_name.into(),
            token: token.into(),
            stream: Mutex::new(None),
            connected: AtomicBool::new(false),
            next_id: AtomicI64::new(1),
        }
    }

    /// Connect to the IPC server.
    pub fn connect(&self) -> IpcResult<()> {
        tracing::info!("[IpcClient] Connecting to channel: {}", self.channel_name);

        let stream = LocalSocketStream::connect(&self.channel_name).map_err(|e| {
            tracing::error!("[IpcClient] Connection failed: {}", e);
            IpcError::ConnectionFailed(e.to_string())
        })?;

        *self.stream.lock() = Some(stream);
        self.connected.store(true, Ordering::SeqCst);

        tracing::info!("[IpcClient] Connected, performing auth handshake...");
        self.auth_handshake()?;

        tracing::info!("[IpcClient] Authentication successful");
        Ok(())
    }

    /// Perform authentication handshake.
    fn auth_handshake(&self) -> IpcResult<()> {
        let params = AuthHelloParams {
            token: self.token.clone(),
        };

        let response = self.call("auth.hello", serde_json::to_value(params)?)?;

        if let Some(error) = response.error {
            return Err(IpcError::AuthenticationFailed(error.message));
        }

        Ok(())
    }

    /// Send a JSON-RPC request and wait for response.
    pub fn call(&self, method: &str, params: Value) -> IpcResult<Response> {
        if !self.connected.load(Ordering::SeqCst) {
            return Err(IpcError::ConnectionFailed("Not connected".to_string()));
        }

        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let request = Request::with_params(id, method, params);

        self.send_request(&request)?;
        self.receive_response()
    }

    /// Send a JSON-RPC notification (no response expected).
    pub fn notify(&self, method: &str, params: Value) -> IpcResult<()> {
        if !self.connected.load(Ordering::SeqCst) {
            return Err(IpcError::ConnectionFailed("Not connected".to_string()));
        }

        let request = Request::notification_with_params(method, params);
        self.send_request(&request)
    }

    fn send_request(&self, request: &Request) -> IpcResult<()> {
        let mut guard = self.stream.lock();
        let stream = guard
            .as_mut()
            .ok_or_else(|| IpcError::ConnectionFailed("Stream not available".to_string()))?;

        let json = serde_json::to_string(request)?;
        tracing::debug!("[IpcClient] Sending: {}", json);

        writeln!(stream, "{}", json).map_err(|e| IpcError::SendFailed(e.to_string()))?;
        stream
            .flush()
            .map_err(|e| IpcError::SendFailed(e.to_string()))?;

        Ok(())
    }

    fn receive_response(&self) -> IpcResult<Response> {
        let mut guard = self.stream.lock();
        let stream = guard
            .as_mut()
            .ok_or_else(|| IpcError::ConnectionFailed("Stream not available".to_string()))?;

        let mut line = String::new();
        let mut reader = BufReader::new(&mut *stream);

        reader
            .read_line(&mut line)
            .map_err(|e| IpcError::ReceiveFailed(e.to_string()))?;

        if line.is_empty() {
            return Err(IpcError::ConnectionClosed);
        }

        tracing::debug!("[IpcClient] Received: {}", line.trim());

        let response: Response = serde_json::from_str(&line)?;
        Ok(response)
    }

    /// Get the list of available tools from the main process.
    pub fn get_tool_list(&self) -> IpcResult<Vec<ToolDefinition>> {
        let response = self.call("tool.list", Value::Null)?;

        if let Some(error) = response.error {
            return Err(IpcError::ProtocolError(error.message));
        }

        let tools: Vec<ToolDefinition> = response
            .result
            .map(|v| serde_json::from_value(v).unwrap_or_default())
            .unwrap_or_default();

        Ok(tools)
    }

    /// Call a tool on the main process.
    pub fn call_tool(
        &self,
        name: &str,
        arguments: Value,
        timeout_ms: Option<u64>,
    ) -> IpcResult<Value> {
        let params = ToolCallParams {
            name: name.to_string(),
            arguments,
            timeout_ms,
            trace_id: None,
        };

        let response = self.call("tool.call", serde_json::to_value(params)?)?;

        if let Some(error) = response.error {
            return Err(IpcError::ProtocolError(error.message));
        }

        Ok(response.result.unwrap_or(Value::Null))
    }

    /// Check if the IPC connection is still alive.
    ///
    /// Sends a ping request to the main process and waits for response.
    /// Returns true if the connection is healthy, false otherwise.
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }

    /// Perform a health check on the IPC connection.
    ///
    /// Sends a lifecycle.ping request to the main process.
    /// If this fails, the connection is considered dead.
    pub fn health_check(&self) -> bool {
        if !self.connected.load(Ordering::SeqCst) {
            return false;
        }

        // Try to send a ping - if it fails, connection is dead
        match self.call("lifecycle.ping", Value::Null) {
            Ok(_) => true,
            Err(e) => {
                tracing::warn!("[IpcClient] Health check failed: {}", e);
                self.connected.store(false, Ordering::SeqCst);
                false
            }
        }
    }

    /// Mark the connection as disconnected.
    pub fn mark_disconnected(&self) {
        self.connected.store(false, Ordering::SeqCst);
    }
}
