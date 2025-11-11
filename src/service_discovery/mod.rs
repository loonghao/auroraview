//! Service Discovery Module
//!
//! Provides automatic service discovery for AuroraView Bridge:
//! - Dynamic port allocation (avoid conflicts)
//! - mDNS/Zeroconf service registration and discovery
//! - HTTP discovery endpoint for UXP plugins

pub mod http_discovery;
pub mod mdns_service;
pub mod port_allocator;

#[cfg(feature = "python-bindings")]
pub mod python_bindings;

pub use http_discovery::HttpDiscovery;
pub use mdns_service::MdnsService;
pub use port_allocator::PortAllocator;

use thiserror::Error;

/// Service discovery errors
#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum ServiceDiscoveryError {
    #[error("No free port found in range {start}-{end}")]
    NoFreePort { start: u16, end: u16 },

    #[error("Port {0} is already in use")]
    PortInUse(u16),

    #[error("mDNS service error: {0}")]
    MdnsError(String),

    #[error("HTTP discovery error: {0}")]
    HttpError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Service not found: {0}")]
    ServiceNotFound(String),
}

pub type Result<T> = std::result::Result<T, ServiceDiscoveryError>;

/// Service information
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    /// Service name
    pub name: String,

    /// Host address
    pub host: String,

    /// Port number
    pub port: u16,

    /// Service metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl ServiceInfo {
    pub fn new(name: String, host: String, port: u16) -> Self {
        Self {
            name,
            host,
            port,
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_info_creation() {
        let service = ServiceInfo::new("test-service".to_string(), "localhost".to_string(), 9001);

        assert_eq!(service.name, "test-service");
        assert_eq!(service.host, "localhost");
        assert_eq!(service.port, 9001);
    }

    #[test]
    fn test_service_info_with_metadata() {
        let service = ServiceInfo::new("test-service".to_string(), "localhost".to_string(), 9001)
            .with_metadata("version".to_string(), "1.0.0".to_string())
            .with_metadata("protocol".to_string(), "websocket".to_string());

        assert_eq!(service.metadata.get("version"), Some(&"1.0.0".to_string()));
        assert_eq!(
            service.metadata.get("protocol"),
            Some(&"websocket".to_string())
        );
    }
}
