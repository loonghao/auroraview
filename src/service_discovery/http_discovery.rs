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

    /// Server shutdown sender
    shutdown_tx: Option<oneshot::Sender<()>>,

    /// Server task handle
    server_handle: Option<JoinHandle<()>>,
}

impl HttpDiscovery {
    /// Create a new HTTP discovery server
    ///
    /// # Arguments
    /// * `discovery_port` - Port for HTTP server (default: 9000)
    /// * `bridge_port` - Bridge WebSocket port to advertise
    pub fn new(discovery_port: u16, bridge_port: u16) -> Self {
        Self {
            discovery_port,
            bridge_port,
            shutdown_tx: None,
            server_handle: None,
        }
    }

    /// Start the HTTP discovery server
    pub async fn start(&mut self) -> Result<()> {
        if self.server_handle.is_some() {
            debug!("HTTP discovery server already running");
            return Ok(());
        }

        info!(
            "Starting HTTP discovery server on port {}",
            self.discovery_port
        );

        let bridge_port = self.bridge_port;
        let discovery_port = self.discovery_port;

        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        self.shutdown_tx = Some(shutdown_tx);

        // Build discovery response
        let response = Arc::new(DiscoveryResponse {
            service: "AuroraView Bridge".to_string(),
            port: bridge_port,
            protocol: "websocket".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });

        // Create routes
        let response_clone = response.clone();
        let discover = warp::path("discover").and(warp::get()).map(move || {
            debug!("Discovery request received");
            warp::reply::json(&*response_clone)
        });

        // Add CORS for UXP plugins
        let cors = warp::cors()
            .allow_any_origin()
            .allow_methods(vec!["GET", "OPTIONS"])
            .allow_headers(vec!["Content-Type"]);

        let routes = discover.with(cors);

        // Start server
        let addr: SocketAddr = ([127, 0, 0, 1], discovery_port).into();

        let (addr_tx, addr_rx) = oneshot::channel();

        let server = warp::serve(routes).bind_with_graceful_shutdown(addr, async {
            shutdown_rx.await.ok();
        });

        // Spawn server task
        let handle = tokio::spawn(async move {
            let (actual_addr, server_fut) = server;
            addr_tx.send(actual_addr).ok();
            server_fut.await;
        });

        self.server_handle = Some(handle);

        // Wait for server to start
        if let Ok(actual_addr) = addr_rx.await {
            info!(
                "✅ HTTP discovery server started at http://{}/discover",
                actual_addr
            );
        }

        Ok(())
    }

    /// Stop the HTTP discovery server
    pub fn stop(&mut self) -> Result<()> {
        info!("Stopping HTTP discovery server");

        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            shutdown_tx.send(()).ok();
        }

        if let Some(handle) = self.server_handle.take() {
            // Abort the server task
            handle.abort();
        }

        info!("✅ HTTP discovery server stopped");
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

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::client::HttpConnector;
    use hyper::{body::to_bytes, Client};

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_http_discovery_start_stop_and_request() {
        let mut server = HttpDiscovery::new(9002, 9101);
        assert!(!server.is_running());
        server.start().await.expect("server should start");
        assert!(server.is_running());

        // Query the discovery endpoint
        let client: Client<HttpConnector, hyper::Body> = Client::new();
        let uri: hyper::Uri = "http://127.0.0.1:9002/discover".parse().unwrap();
        let resp = client.get(uri).await.expect("GET /discover should succeed");
        assert!(resp.status().is_success());
        let body = to_bytes(resp.into_body()).await.unwrap();
        let json: DiscoveryResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(json.service, "AuroraView Bridge");
        assert_eq!(json.port, 9101);
        assert_eq!(json.protocol, "websocket");

        // Stop server
        server.stop().expect("server should stop");
        assert!(!server.is_running());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_http_discovery_double_start_and_double_stop_are_safe() {
        let mut server = HttpDiscovery::new(0, 9102); // 0 → OS-assign port
                                                      // First start
        server.start().await.expect("first start ok");
        assert!(server.is_running());
        // Second start should be a no-op and Ok
        server.start().await.expect("second start ok");
        assert!(server.is_running());
        // First stop
        server.stop().expect("first stop ok");
        assert!(!server.is_running());
        // Second stop should not panic
        server.stop().expect("second stop ok");
        assert!(!server.is_running());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_http_discovery_unknown_path_returns_404() {
        // Use a port outside the default allocator scan (9001..=9100) to avoid flakiness
        let mut server = HttpDiscovery::new(9301, 9101);
        server.start().await.expect("server start");
        let client: Client<HttpConnector, hyper::Body> = Client::new();
        let uri: hyper::Uri = "http://127.0.0.1:9301/unknown".parse().unwrap();
        let resp = client.get(uri).await.expect("GET should succeed");
        assert_eq!(resp.status(), hyper::StatusCode::NOT_FOUND);
        server.stop().expect("server stop");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_http_discovery_stop_without_start_is_ok() {
        let mut server = HttpDiscovery::new(0, 9101);
        // Calling stop on a non-started server should be a no-op and Ok
        server.stop().expect("stop ok");
        assert!(!server.is_running());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_http_discovery_drop_stops_server() {
        let mut server = HttpDiscovery::new(0, 9101);
        server.start().await.expect("start ok");
        assert!(server.is_running());
        drop(server); // should invoke Drop::drop -> stop()
                      // Allow a short moment for graceful shutdown
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
}
