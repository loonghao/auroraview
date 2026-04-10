use crate::{error::Result, types::McpServerConfig};
use mdns_sd::{ServiceDaemon, ServiceInfo};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

/// Broadcasts the AuroraView MCP Server via mDNS so that `dcc-mcp-client` can auto-discover it.
pub struct MdnsBroadcaster {
    daemon: ServiceDaemon,
    service_type: String,
    instance_name: Arc<Mutex<Option<String>>>,
}

impl MdnsBroadcaster {
    const SERVICE_TYPE: &'static str = "_auroraview-mcp._tcp.local.";

    pub fn new() -> Result<Self> {
        let daemon = ServiceDaemon::new()
            .map_err(|e| crate::error::McpError::MdnsBroadcast(e.to_string()))?;
        Ok(Self {
            daemon,
            service_type: Self::SERVICE_TYPE.to_string(),
            instance_name: Arc::new(Mutex::new(None)),
        })
    }

    /// Start broadcasting the MCP server on the network.
    pub async fn start(&self, config: &McpServerConfig) -> Result<()> {
        let host_name = gethostname_str();
        let instance = format!("{}.{}", config.service_name, self.service_type);

        let mut properties = std::collections::HashMap::new();
        properties.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
        properties.insert("protocol".to_string(), "mcp".to_string());
        properties.insert("transport".to_string(), "sse".to_string());
        properties.insert("path".to_string(), "/mcp".to_string());

        let service = ServiceInfo::new(
            &self.service_type,
            &config.service_name,
            &host_name,
            &config.host,
            config.port,
            Some(properties),
        )
        .map_err(|e| crate::error::McpError::MdnsBroadcast(e.to_string()))?;

        self.daemon
            .register(service)
            .map_err(|e| crate::error::McpError::MdnsBroadcast(e.to_string()))?;

        *self.instance_name.lock().await = Some(instance.clone());
        info!(
            "mDNS: broadcasting AuroraView MCP Server as '{}' on port {}",
            instance, config.port
        );
        Ok(())
    }

    /// Stop broadcasting.
    pub async fn stop(&self) {
        let name = self.instance_name.lock().await.take();
        if let Some(n) = name {
            if let Err(e) = self.daemon.unregister(&n) {
                warn!("mDNS: failed to unregister service: {e}");
            } else {
                debug!("mDNS: unregistered service '{n}'");
            }
        }
    }
}

fn gethostname_str() -> String {
    // Simple fallback — gethostname via std is not stable cross-platform without a crate.
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "localhost".to_string())
        + ".local."
}
