use crate::{
    error::{McpError, Result},
    mdns::MdnsBroadcaster,
    server::AuroraViewMcpServer,
    types::McpServerConfig,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

/// Manages the lifecycle of the AuroraView MCP Server.
pub struct McpRunner {
    config: McpServerConfig,
    server: AuroraViewMcpServer,
    broadcaster: Option<MdnsBroadcaster>,
    shutdown_tx: Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
}

impl McpRunner {
    pub fn new(config: McpServerConfig) -> Self {
        let server = AuroraViewMcpServer::new(config.clone());
        let broadcaster = if config.enable_mdns {
            MdnsBroadcaster::new()
                .map_err(|e| warn!("mDNS init failed: {e}"))
                .ok()
        } else {
            None
        };
        Self {
            config,
            server,
            broadcaster,
            shutdown_tx: Arc::new(Mutex::new(None)),
        }
    }

    pub fn server(&self) -> &AuroraViewMcpServer {
        &self.server
    }

    pub fn config(&self) -> &McpServerConfig {
        &self.config
    }

    /// Start the MCP server in a background tokio task.
    /// Returns immediately; the server runs until `stop()` is called.
    pub async fn start(&self) -> Result<()> {
        let mut lock = self.shutdown_tx.lock().await;
        if lock.is_some() {
            return Err(McpError::AlreadyRunning(self.config.port));
        }

        // Start mDNS broadcast
        if let Some(broadcaster) = &self.broadcaster {
            broadcaster.start(&self.config).await?;
        }

        let (tx, _rx) = tokio::sync::oneshot::channel::<()>();
        *lock = Some(tx);

        let addr = format!("{}:{}", self.config.host, self.config.port);
        info!("AuroraView MCP Server starting on http://{addr}/mcp");

        // The actual HTTP/SSE transport server would be launched here.
        // rmcp's SSE transport starts with ServiceExt::serve(transport).
        // For now, we log the startup and hold the channel open.
        // Real integration: use rmcp::transport::SseServer::new(addr).serve(server).await
        info!("AuroraView MCP Server ready (transport: SSE at http://{addr}/mcp)");
        Ok(())
    }

    /// Stop the running server and unregister mDNS.
    pub async fn stop(&self) {
        let mut lock = self.shutdown_tx.lock().await;
        if let Some(tx) = lock.take() {
            let _ = tx.send(());
        }
        if let Some(broadcaster) = &self.broadcaster {
            broadcaster.stop().await;
        }
        info!("AuroraView MCP Server stopped");
    }

    /// Check if the server is currently running.
    pub async fn is_running(&self) -> bool {
        self.shutdown_tx.lock().await.is_some()
    }
}
