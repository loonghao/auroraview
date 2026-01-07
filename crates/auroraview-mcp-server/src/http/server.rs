//! HTTP Server for MCP Sidecar.
//!
//! Implements the Streamable HTTP transport using rmcp.

use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::Arc;
use std::time::Duration;

use axum::http::Method;
use axum::Router;
use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager, tower::{StreamableHttpServerConfig, StreamableHttpService},
};
use serde_json::Value;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tower_http::cors::{Any as CorsAny, CorsLayer};

use crate::ipc::IpcClient;

use super::McpSidecarService;

/// HTTP Server configuration.
#[derive(Debug, Clone)]
pub struct HttpServerConfig {
    /// Host to bind to
    pub host: String,
    /// Port to bind to (0 for auto)
    pub port: u16,
    /// Server name
    pub name: String,
    /// Server version
    pub version: String,
    /// SSE keep-alive interval
    pub heartbeat_interval: u64,
}

impl Default for HttpServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 0,
            name: "auroraview-mcp-sidecar".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            heartbeat_interval: 30,
        }
    }
}

/// HTTP Server for MCP Sidecar.
pub struct HttpServer {
    config: HttpServerConfig,
    ipc_client: Arc<IpcClient>,
    port: AtomicU16,
    running: AtomicBool,
    shutdown_tx: parking_lot::RwLock<Option<mpsc::Sender<()>>>,
    cancellation_token: CancellationToken,
    server_task: parking_lot::RwLock<Option<tokio::task::JoinHandle<()>>>,
}

impl HttpServer {
    /// Create a new HTTP server.
    pub fn new(config: HttpServerConfig, ipc_client: Arc<IpcClient>) -> Self {
        Self {
            config,
            ipc_client,
            port: AtomicU16::new(0),
            running: AtomicBool::new(false),
            shutdown_tx: parking_lot::RwLock::new(None),
            cancellation_token: CancellationToken::new(),
            server_task: parking_lot::RwLock::new(None),
        }
    }

    /// Start the HTTP server with Streamable HTTP transport.
    pub async fn start(&self) -> Result<u16, String> {
        if self.running.load(Ordering::SeqCst) {
            let port = self.port.load(Ordering::SeqCst);
            tracing::warn!("[HttpServer] Already running on port {}", port);
            return Ok(port);
        }

        // Find available port
        let port = if self.config.port == 0 {
            Self::find_free_port()?
        } else {
            self.config.port
        };

        let addr: SocketAddr = format!("{}:{}", self.config.host, port)
            .parse()
            .map_err(|e| format!("Invalid address: {}", e))?;

        // Create service factory
        let name = self.config.name.clone();
        let version = self.config.version.clone();
        let ipc_client = Arc::clone(&self.ipc_client);

        let service_factory = move || {
            Ok(McpSidecarService::new(
                name.clone(),
                version.clone(),
                Arc::clone(&ipc_client),
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
        let app = Router::new()
            .nest_service("/mcp", mcp_service)
            .route("/health", axum::routing::get(health_handler))
            .layer(cors);

        // Create shutdown channel
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        *self.shutdown_tx.write() = Some(shutdown_tx);

        // Start server
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| format!("Failed to bind: {}", e))?;

        let actual_port = listener.local_addr().map(|a| a.port()).unwrap_or(port);

        self.port.store(actual_port, Ordering::SeqCst);
        self.running.store(true, Ordering::SeqCst);

        tracing::info!(
            "[HttpServer] Started on {}:{} (Streamable HTTP)",
            self.config.host, actual_port
        );
        tracing::info!("[HttpServer] MCP endpoint: http://{}:{}/mcp", self.config.host, actual_port);

        // Spawn server task
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = Arc::clone(&running);

        let task_handle = tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    shutdown_rx.recv().await;
                    tracing::info!("[HttpServer] Shutting down...");
                })
                .await
                .ok();
            running_clone.store(false, Ordering::SeqCst);
        });

        *self.server_task.write() = Some(task_handle);

        Ok(actual_port)
    }

    /// Stop the HTTP server.
    pub async fn stop(&self) {
        if !self.running.load(Ordering::SeqCst) {
            return;
        }

        // Cancel all sessions
        self.cancellation_token.cancel();

        // Send shutdown signal
        let shutdown_tx = self.shutdown_tx.write().take();
        if let Some(tx) = shutdown_tx {
            let _ = tx.send(()).await;
        }

        // Wait for server task to complete with timeout
        let server_task = self.server_task.write().take();
        if let Some(task) = server_task {
            match tokio::time::timeout(Duration::from_secs(5), task).await {
                Ok(Ok(())) => {
                    tracing::debug!("[HttpServer] Task completed gracefully");
                }
                Ok(Err(e)) => {
                    tracing::warn!("[HttpServer] Task panicked: {:?}", e);
                }
                Err(_) => {
                    tracing::warn!("[HttpServer] Task did not complete within timeout");
                }
            }
        }

        self.running.store(false, Ordering::SeqCst);
        self.port.store(0, Ordering::SeqCst);

        tracing::info!("[HttpServer] Stopped");
    }

    /// Check if server is running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Get the current port.
    pub fn port(&self) -> u16 {
        self.port.load(Ordering::SeqCst)
    }

    /// Find a free port.
    fn find_free_port() -> Result<u16, String> {
        let listener = std::net::TcpListener::bind("127.0.0.1:0")
            .map_err(|e| format!("Failed to find free port: {}", e))?;
        let port = listener.local_addr().map_err(|e| e.to_string())?.port();
        Ok(port)
    }
}

/// Health check handler.
async fn health_handler() -> axum::Json<Value> {
    axum::Json(serde_json::json!({
        "status": "ok",
        "transport": "streamable-http",
        "server": "auroraview-mcp-sidecar"
    }))
}

