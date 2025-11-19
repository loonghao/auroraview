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

        // Build full service name
        let full_name = format!("{}.{}", instance_name, SERVICE_TYPE);

        // Convert metadata to TXT records
        let properties: Vec<(&str, &str)> = metadata
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        // Create service info
        let service_info = MdnsServiceInfo::new(
            SERVICE_TYPE,
            instance_name,
            "localhost.local.",
            "", // No specific host
            port,
            &properties[..],
        )
        .map_err(|e| {
            error!("Failed to create service info: {}", e);
            ServiceDiscoveryError::MdnsError(e.to_string())
        })?;

        // Register service
        self.daemon.register(service_info).map_err(|e| {
            error!("Failed to register service: {}", e);
            ServiceDiscoveryError::MdnsError(e.to_string())
        })?;

        // Store service name for later unregistration
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
    ///
    /// # Arguments
    /// * `timeout_secs` - Discovery timeout in seconds
    ///
    /// # Returns
    /// List of discovered services
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

                    // Extract metadata
                    // Note: mdns-sd 0.11 doesn't provide easy iteration over properties
                    // We'll just add basic metadata for now
                    let metadata = HashMap::new();
                    // TODO: Extract TXT record properties when mdns-sd provides better API

                    // Get first address
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
    fn test_register_service() {
        let service = MdnsService::new().unwrap();
        let mut metadata = HashMap::new();
        metadata.insert("version".to_string(), "1.0.0".to_string());
        metadata.insert("app".to_string(), "test".to_string());

        let result = service.register("TestInstance", 9001, metadata);
        assert!(result.is_ok());

        // Verify service name was stored
        let stored_name = service.service_name.lock().clone();
        assert!(stored_name.is_some());
        assert_eq!(stored_name.unwrap(), "TestInstance._auroraview._tcp.local.");
    }

    #[test]
    fn test_register_with_empty_metadata() {
        let service = MdnsService::new().unwrap();
        let metadata = HashMap::new();

        let result = service.register("EmptyMetadata", 9002, metadata);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unregister_without_registration() {
        let service = MdnsService::new().unwrap();

        // Should succeed even if nothing was registered
        let result = service.unregister();
        assert!(result.is_ok());
    }

    #[test]
    fn test_unregister_after_registration() {
        let service = MdnsService::new().unwrap();
        let metadata = HashMap::new();

        service.register("TestUnregister", 9003, metadata).unwrap();

        // Verify registered
        assert!(service.service_name.lock().is_some());

        // Unregister
        let result = service.unregister();
        assert!(result.is_ok());

        // Verify unregistered
        assert!(service.service_name.lock().is_none());
    }

    #[test]
    fn test_multiple_registrations() {
        let service = MdnsService::new().unwrap();
        let metadata = HashMap::new();

        // First registration
        service.register("First", 9004, metadata.clone()).unwrap();
        assert_eq!(
            service.service_name.lock().clone().unwrap(),
            "First._auroraview._tcp.local."
        );

        // Second registration (should replace)
        service.register("Second", 9005, metadata).unwrap();
        assert_eq!(
            service.service_name.lock().clone().unwrap(),
            "Second._auroraview._tcp.local."
        );
    }

    #[test]
    fn test_discover_with_short_timeout() {
        let service = MdnsService::new().unwrap();

        // Discover with 1 second timeout
        let result = service.discover(1);
        assert!(result.is_ok());

        // May or may not find services, but should not error
        let _services = result.unwrap();
        // Services list is valid (no assertion needed on length)
    }

    #[test]
    fn test_discover_returns_service_info() {
        let service = MdnsService::new().unwrap();

        // Register a service first
        let mut metadata = HashMap::new();
        metadata.insert("test".to_string(), "value".to_string());
        service.register("DiscoverTest", 9006, metadata).unwrap();

        // Try to discover (may or may not find our own service)
        let result = service.discover(2);
        assert!(result.is_ok());
    }

    #[test]
    fn test_service_drop_unregisters() {
        let service = MdnsService::new().unwrap();
        let metadata = HashMap::new();

        service.register("DropTest", 9007, metadata).unwrap();
        assert!(service.service_name.lock().is_some());

        // Drop should call unregister
        drop(service);
        // Can't verify after drop, but ensures no panic
    }

    #[test]
    fn test_register_with_special_characters_in_metadata() {
        let service = MdnsService::new().unwrap();
        let mut metadata = HashMap::new();
        metadata.insert(
            "key=with=equals".to_string(),
            "value with spaces".to_string(),
        );
        metadata.insert("unicode".to_string(), "测试🚀".to_string());

        let result = service.register("SpecialChars", 9008, metadata);
        assert!(result.is_ok());
    }

    #[test]
    fn test_register_with_high_port() {
        let service = MdnsService::new().unwrap();
        let metadata = HashMap::new();

        // Test with high port number
        let result = service.register("HighPort", 65535, metadata);
        assert!(result.is_ok());
    }

    #[test]
    fn test_register_with_low_port() {
        let service = MdnsService::new().unwrap();
        let metadata = HashMap::new();

        // Test with low port number (may require privileges)
        let result = service.register("LowPort", 1024, metadata);
        // Should succeed in creating the registration
        assert!(result.is_ok());
    }

    #[test]
    fn test_concurrent_operations() {
        use std::sync::Arc;
        use std::thread;

        let service = Arc::new(MdnsService::new().unwrap());
        let service_clone = Arc::clone(&service);

        let handle = thread::spawn(move || {
            let metadata = HashMap::new();
            service_clone.register("Concurrent", 9009, metadata)
        });

        let result = handle.join().unwrap();
        assert!(result.is_ok());
    }
}
