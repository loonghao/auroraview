//! HTTP Discovery Endpoint
//!
//! Provides HTTP REST API for service discovery (for UXP plugins).

use super::Result;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tracing::{debug, error, info};
use warp::Filter;

/// Discovery response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResponse {
    /// Service name
    pub service: String,
    /// Bridge port
    pub port: u16,
    /// Protocol (always "websocket")
    pub protocol: String,
    /// AuroraView version
    pub version: String,
    /// Timestamp
    pub timestamp: u64,
}

/// HTTP discovery server
pub struct HttpDiscovery {
    /// Discovery port (default: 9000)
    discovery_port: u16,
    /// Bridge port to advertise
    bridge_port: u16,
    /// Actual bound port (may differ from discovery_port if 0 was used)
    pub port: u16,
    /// Server shutdown sender
    shutdown_tx: Option<oneshot::Sender<()>>,
    /// Server task handle
    server_handle: Option<JoinHandle<()>>,
}

impl HttpDiscovery {
    /// Create a new HTTP discovery server
    pub fn new(discovery_port: u16, bridge_port: u16) -> Self {
        Self {
            discovery_port,
            bridge_port,
            port: discovery_port,
            shutdown_tx: None,
            server_handle: None,
        }
    }

    /// Start the HTTP discovery server
    pub async fn start(&mut self) -> Result<()> {
        self.start_with_version(env!("CARGO_PKG_VERSION")).await
    }

    /// Start the HTTP discovery server with a specific version string
    pub async fn start_with_version(&mut self, version: &str) -> Result<()> {
        if self.server_handle.is_some() {
            debug!("HTTP discovery server already running");
            return Ok(());
        }

        info!(
            "Starting HTTP discovery server on port {}",
            self.discovery_port
        );

        let bridge_port = self.bridge_port;
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        self.shutdown_tx = Some(shutdown_tx);

        let response = Arc::new(DiscoveryResponse {
            service: "AuroraView Bridge".to_string(),
            port: bridge_port,
            protocol: "websocket".to_string(),
            version: version.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });

        let response_clone = response.clone();
        let discover = warp::path("discover").and(warp::get()).map(move || {
            debug!("Discovery request received");
            warp::reply::json(&*response_clone)
        });

        let cors = warp::cors()
            .allow_any_origin()
            .allow_methods(vec!["GET", "OPTIONS"])
            .allow_headers(vec!["Content-Type"]);

        let routes = discover.with(cors).boxed();
        let addr: SocketAddr = ([127, 0, 0, 1], self.discovery_port).into();

        // Use bind_with_graceful_shutdown which returns (bound_addr, Future)
        let (bound_addr, server_future) =
            warp::serve(routes).bind_with_graceful_shutdown(addr, async move {
                shutdown_rx.await.ok();
            });

        self.port = bound_addr.port();

        info!(
            "HTTP discovery server started at http://{}/discover",
            bound_addr
        );

        let handle = tokio::spawn(server_future);
        self.server_handle = Some(handle);

        Ok(())
    }

    /// Stop the HTTP discovery server
    pub fn stop(&mut self) -> Result<()> {
        info!("Stopping HTTP discovery server");

        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            shutdown_tx.send(()).ok();
        }
        if let Some(handle) = self.server_handle.take() {
            handle.abort();
        }

        info!("HTTP discovery server stopped");
        Ok(())
    }

    /// Check if server is running
    #[allow(dead_code)]
    pub fn is_running(&self) -> bool {
        self.server_handle.is_some()
    }
}

impl Drop for HttpDiscovery {
    fn drop(&mut self) {
        if let Err(e) = self.stop() {
            error!("Failed to stop HTTP discovery server on drop: {}", e);
        }
    }
}
