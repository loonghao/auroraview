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
    ///
    /// # Arguments
    /// * `discovery_port` - Port for HTTP server (default: 9000)
    /// * `bridge_port` - Bridge WebSocket port to advertise
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

        let routes = discover.with(cors).boxed();

        // Start server
        let addr: SocketAddr = ([127, 0, 0, 1], discovery_port).into();

        // Use a channel to communicate the bound address
        let (addr_tx, mut addr_rx) = tokio::sync::mpsc::channel(1);

        // Spawn server task
        let handle = tokio::spawn(async move {
            // Bind to the address - bind() returns a future that resolves to a Server
            let server = warp::serve(routes).bind(addr).await;

            // Send the address immediately after binding
            let _ = addr_tx.send(addr).await;

            // Run the server with graceful shutdown
            server
                .graceful(async {
                    shutdown_rx.await.ok();
                })
                .run()
                .await;
        });

        self.server_handle = Some(handle);

        // Wait for the address to be received (with timeout)
        match tokio::time::timeout(std::time::Duration::from_secs(5), addr_rx.recv()).await {
            Ok(Some(bound_addr)) => {
                self.port = bound_addr.port();
                info!(
                    "✅ HTTP discovery server started at http://{}/discover",
                    bound_addr
                );
                Ok(())
            }
            Ok(None) => {
                Err(std::io::Error::other("Server channel closed before sending address").into())
            }
            Err(_) => Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Failed to start HTTP discovery server: timeout waiting for bind",
            )
            .into()),
        }
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_http_discovery_start_stop_and_request() {
        let mut server = HttpDiscovery::new(0, 9101); // Use port 0 for OS-assigned port
        assert!(!server.is_running());
        server.start().await.expect("server should start");
        assert!(server.is_running());

        // Get the actual bound port from the server
        let bound_port = server.port;
        assert!(bound_port > 0, "Port should be set after start");

        // Query the discovery endpoint using reqwest
        let client = reqwest::Client::new();
        let resp = client
            .get(format!("http://127.0.0.1:{}/discover", bound_port))
            .send()
            .await
            .expect("GET /discover should succeed");
        assert!(resp.status().is_success());
        let json: DiscoveryResponse = resp.json().await.unwrap();
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
        // Use port 0 for OS-assigned port to avoid flakiness
        let mut server = HttpDiscovery::new(0, 9101);
        server.start().await.expect("server start");

        let bound_port = server.port;
        assert!(bound_port > 0, "Port should be set after start");
        let client = reqwest::Client::new();
        let resp = client
            .get(format!("http://127.0.0.1:{}/unknown", bound_port))
            .send()
            .await
            .expect("GET should succeed");
        assert_eq!(resp.status(), reqwest::StatusCode::NOT_FOUND);
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
