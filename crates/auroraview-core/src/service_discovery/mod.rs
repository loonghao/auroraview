//! Service Discovery Core Module
//!
//! Provides core service discovery utilities:
//! - Dynamic port allocation (avoid conflicts)
//! - Service information structures
//! - mDNS service registration and discovery
//! - HTTP discovery endpoint

pub mod http_discovery;
pub mod mdns_service;
pub mod port_allocator;

pub use http_discovery::{DiscoveryResponse, HttpDiscovery};
pub use mdns_service::{MdnsService, SERVICE_TYPE};
pub use port_allocator::PortAllocator;

use std::collections::HashMap;
use thiserror::Error;

/// Service discovery errors
#[derive(Error, Debug)]
pub enum ServiceDiscoveryError {
    #[error("No free port found in range {start}-{end}")]
    NoFreePort { start: u16, end: u16 },

    #[error("Port {0} is already in use")]
    PortInUse(u16),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("mDNS error: {0}")]
    MdnsError(String),

    #[error("HTTP error: {0}")]
    HttpError(String),
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
    pub metadata: HashMap<String, String>,
}

impl ServiceInfo {
    pub fn new(name: String, host: String, port: u16) -> Self {
        Self {
            name,
            host,
            port,
            metadata: HashMap::new(),
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
