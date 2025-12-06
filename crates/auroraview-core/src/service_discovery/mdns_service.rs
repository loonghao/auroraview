//! mDNS Service Registration and Discovery
//!
//! Provides Zeroconf/Bonjour service discovery for AuroraView Bridge.

use super::{Result, ServiceDiscoveryError, ServiceInfo};
use mdns_sd::{ServiceDaemon, ServiceInfo as MdnsServiceInfo};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Service type for AuroraView Bridge
pub const SERVICE_TYPE: &str = "_auroraview._tcp.local.";

/// mDNS service for registration and discovery
pub struct MdnsService {
    /// mDNS daemon
    daemon: Arc<ServiceDaemon>,

    /// Registered service name
    service_name: Arc<Mutex<Option<String>>>,
}

impl MdnsService {
    /// Create a new mDNS service
    pub fn new() -> Result<Self> {
        info!("Initializing mDNS service");

        let daemon = ServiceDaemon::new().map_err(|e| {
            error!("Failed to create mDNS daemon: {}", e);
            ServiceDiscoveryError::MdnsError(e.to_string())
        })?;

        info!("✅ mDNS service initialized");

        Ok(Self {
            daemon: Arc::new(daemon),
            service_name: Arc::new(Mutex::new(None)),
        })
    }

    /// Register a service with mDNS
    ///
    /// # Arguments
    /// * `instance_name` - Unique instance name (e.g., "My App")
    /// * `port` - Service port
    /// * `metadata` - Additional service metadata
    pub fn register(
        &self,
        instance_name: &str,
        port: u16,
        metadata: HashMap<String, String>,
    ) -> Result<()> {
        info!(
            "Registering mDNS service: {} on port {}",
            instance_name, port
        );

        let full_name = format!("{}.{}", instance_name, SERVICE_TYPE);
        let properties: Vec<(&str, &str)> = metadata
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        let service_info = MdnsServiceInfo::new(
            SERVICE_TYPE,
            instance_name,
            "localhost.local.",
            "",
            port,
            &properties[..],
        )
        .map_err(|e| {
            error!("Failed to create service info: {}", e);
            ServiceDiscoveryError::MdnsError(e.to_string())
        })?;

        self.daemon.register(service_info).map_err(|e| {
            error!("Failed to register service: {}", e);
            ServiceDiscoveryError::MdnsError(e.to_string())
        })?;

        *self.service_name.lock() = Some(full_name.clone());
        info!("✅ mDNS service registered: {}", full_name);
        Ok(())
    }

    /// Unregister the service
    pub fn unregister(&self) -> Result<()> {
        let service_name = self.service_name.lock().clone();

        if let Some(name) = service_name {
            info!("Unregistering mDNS service: {}", name);
            self.daemon.unregister(&name).map_err(|e| {
                error!("Failed to unregister service: {}", e);
                ServiceDiscoveryError::MdnsError(e.to_string())
            })?;
            *self.service_name.lock() = None;
            info!("✅ mDNS service unregistered");
        } else {
            debug!("No service to unregister");
        }
        Ok(())
    }

    /// Discover services of the given type
    pub fn discover(&self, timeout_secs: u64) -> Result<Vec<ServiceInfo>> {
        info!(
            "Discovering {} services (timeout: {}s)",
            SERVICE_TYPE, timeout_secs
        );

        let receiver = self.daemon.browse(SERVICE_TYPE).map_err(|e| {
            error!("Failed to start browse: {}", e);
            ServiceDiscoveryError::MdnsError(e.to_string())
        })?;

        let mut services = Vec::new();
        let start = std::time::Instant::now();

        while start.elapsed().as_secs() < timeout_secs {
            if let Ok(event) = receiver.recv_timeout(std::time::Duration::from_secs(1)) {
                use mdns_sd::ServiceEvent;
                if let ServiceEvent::ServiceResolved(info) = event {
                    debug!("Discovered service: {}", info.get_fullname());
                    let metadata = HashMap::new();
                    let host = info
                        .get_addresses()
                        .iter()
                        .next()
                        .map(|addr| addr.to_string())
                        .unwrap_or_else(|| "localhost".to_string());

                    services.push(ServiceInfo {
                        name: info.get_fullname().to_string(),
                        host,
                        port: info.get_port(),
                        metadata,
                    });
                }
            }
        }
        info!("✅ Discovered {} services", services.len());
        Ok(services)
    }
}

impl Drop for MdnsService {
    fn drop(&mut self) {
        if let Err(e) = self.unregister() {
            warn!("Failed to unregister service on drop: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mdns_service_creation() {
        let result = MdnsService::new();
        assert!(result.is_ok());
        let service = result.unwrap();
        assert!(service.service_name.lock().is_none());
    }

    #[test]
    fn test_service_type_constant() {
        assert_eq!(SERVICE_TYPE, "_auroraview._tcp.local.");
    }

    #[test]
    fn test_unregister_without_registration() {
        let service = MdnsService::new().unwrap();
        let result = service.unregister();
        assert!(result.is_ok());
    }

    #[test]
    fn test_service_drop_unregisters() {
        let service = MdnsService::new().unwrap();
        let metadata = HashMap::new();
        service.register("DropTest", 9007, metadata).unwrap();
        assert!(service.service_name.lock().is_some());
        drop(service);
    }
}
