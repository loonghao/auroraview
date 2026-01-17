//! WebSocket-based CDP client implementation

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, trace, warn};

use super::client::{CdpClient, TargetInfo};
use crate::error::{InspectorError, Result};

/// WebSocket-based CDP client
pub struct WebSocketCdpClient {
    /// Sender for outgoing messages
    sender: mpsc::Sender<String>,
    /// Pending requests waiting for response
    pending: Arc<DashMap<u64, oneshot::Sender<Value>>>,
    /// Request ID counter
    request_id: AtomicU64,
    /// Connection state
    connected: AtomicBool,
    /// Target info
    target_info: Option<TargetInfo>,
}

impl WebSocketCdpClient {
    /// Connect to CDP endpoint
    ///
    /// # Arguments
    /// * `endpoint` - CDP HTTP endpoint (e.g., "http://localhost:9222")
    pub async fn connect(endpoint: &str) -> Result<Self> {
        // Fetch targets from HTTP endpoint
        let targets_url = format!("{}/json", endpoint.trim_end_matches('/'));
        debug!("Fetching targets from {}", targets_url);

        let response = reqwest::get(&targets_url)
            .await
            .map_err(|e| InspectorError::Connection(e.to_string()))?;

        let targets: Vec<TargetInfo> = response
            .json()
            .await
            .map_err(|e| InspectorError::Connection(e.to_string()))?;

        // Find first page target
        let target = targets
            .into_iter()
            .find(|t| t.target_type == "page")
            .ok_or_else(|| InspectorError::Connection("No page target found".to_string()))?;

        debug!("Connecting to target: {} ({})", target.title, target.url);

        // Connect to WebSocket
        Self::connect_to_target(target).await
    }

    /// Connect directly to a WebSocket URL
    pub async fn connect_ws(ws_url: &str) -> Result<Self> {
        let target = TargetInfo {
            id: String::new(),
            target_type: "page".to_string(),
            title: String::new(),
            url: String::new(),
            web_socket_debugger_url: ws_url.to_string(),
            devtools_frontend_url: String::new(),
            favicon_url: String::new(),
            attached: false,
        };

        Self::connect_to_target(target).await
    }

    /// Connect to a specific target
    async fn connect_to_target(target: TargetInfo) -> Result<Self> {
        let ws_url = &target.web_socket_debugger_url;
        debug!("Connecting to WebSocket: {}", ws_url);

        let (ws_stream, _) = connect_async(ws_url)
            .await
            .map_err(|e| InspectorError::WebSocket(e.to_string()))?;

        let (write, read) = ws_stream.split();

        // Create channels
        let (sender, receiver) = mpsc::channel::<String>(100);
        let pending: Arc<DashMap<u64, oneshot::Sender<Value>>> = Arc::new(DashMap::new());
        let pending_clone = pending.clone();

        // Spawn writer task
        let writer_handle = tokio::spawn(Self::writer_task(receiver, write));

        // Spawn reader task
        let reader_handle = tokio::spawn(Self::reader_task(read, pending_clone));

        // Monitor tasks
        tokio::spawn(async move {
            tokio::select! {
                result = writer_handle => {
                    if let Err(e) = result {
                        error!("Writer task panicked: {:?}", e);
                    }
                }
                result = reader_handle => {
                    if let Err(e) = result {
                        error!("Reader task panicked: {:?}", e);
                    }
                }
            }
        });

        let client = Self {
            sender,
            pending,
            request_id: AtomicU64::new(1),
            connected: AtomicBool::new(true),
            target_info: Some(target),
        };

        // Enable necessary domains
        client.send_simple("Page.enable").await?;
        client.send_simple("Runtime.enable").await?;
        client.send_simple("DOM.enable").await?;

        debug!("CDP client connected and domains enabled");

        Ok(client)
    }

    /// Writer task - sends messages to WebSocket
    async fn writer_task(
        mut receiver: mpsc::Receiver<String>,
        mut write: futures_util::stream::SplitSink<
            WebSocketStream<MaybeTlsStream<TcpStream>>,
            Message,
        >,
    ) {
        while let Some(msg) = receiver.recv().await {
            trace!("Sending: {}", msg);
            if let Err(e) = write.send(Message::Text(msg.into())).await {
                error!("Failed to send message: {}", e);
                break;
            }
        }
        debug!("Writer task exiting");
    }

    /// Reader task - receives messages from WebSocket
    async fn reader_task(
        mut read: futures_util::stream::SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
        pending: Arc<DashMap<u64, oneshot::Sender<Value>>>,
    ) {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    trace!("Received: {}", text);

                    if let Ok(value) = serde_json::from_str::<Value>(&text) {
                        // Check if this is a response (has id)
                        if let Some(id) = value.get("id").and_then(|v| v.as_u64()) {
                            if let Some((_, sender)) = pending.remove(&id) {
                                // Check for error
                                if let Some(error) = value.get("error") {
                                    let error_val = serde_json::json!({
                                        "error": error.clone()
                                    });
                                    let _ = sender.send(error_val);
                                } else {
                                    let result =
                                        value.get("result").cloned().unwrap_or(Value::Null);
                                    let _ = sender.send(result);
                                }
                            }
                        } else if let Some(method) = value.get("method").and_then(|v| v.as_str()) {
                            // This is an event
                            trace!("CDP event: {}", method);
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    debug!("WebSocket closed by server");
                    break;
                }
                Err(e) => {
                    warn!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
        debug!("Reader task exiting");
    }
}

#[async_trait]
impl CdpClient for WebSocketCdpClient {
    async fn send(&self, method: &str, params: Value) -> Result<Value> {
        if !self.connected.load(Ordering::SeqCst) {
            return Err(InspectorError::Connection("Not connected".to_string()));
        }

        let id = self.request_id.fetch_add(1, Ordering::SeqCst);

        let request = serde_json::json!({
            "id": id,
            "method": method,
            "params": params
        });

        let (tx, rx) = oneshot::channel();
        self.pending.insert(id, tx);

        let request_str = serde_json::to_string(&request)?;

        self.sender
            .send(request_str)
            .await
            .map_err(|e| InspectorError::Connection(e.to_string()))?;

        // Wait for response with timeout
        let response = tokio::time::timeout(std::time::Duration::from_secs(30), rx)
            .await
            .map_err(|_| InspectorError::Timeout(format!("Timeout waiting for {}", method)))?
            .map_err(|_| InspectorError::Connection("Response channel closed".to_string()))?;

        // Check for error in response
        if let Some(error) = response.get("error") {
            let message = error["message"].as_str().unwrap_or("Unknown error");
            return Err(InspectorError::Command(message.to_string()));
        }

        Ok(response)
    }

    async fn targets(&self) -> Result<Vec<TargetInfo>> {
        // Note: This would need the HTTP endpoint, not WebSocket
        // For now, return the current target
        if let Some(target) = &self.target_info {
            Ok(vec![target.clone()])
        } else {
            Ok(vec![])
        }
    }

    async fn current_target(&self) -> Result<Option<TargetInfo>> {
        Ok(self.target_info.clone())
    }

    async fn close(&self) -> Result<()> {
        self.connected.store(false, Ordering::SeqCst);
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }
}

impl Drop for WebSocketCdpClient {
    fn drop(&mut self) {
        self.connected.store(false, Ordering::SeqCst);
    }
}
