//! Python bindings for service discovery
//!
//! Exposes service discovery functionality to Python via PyO3.

use super::{HttpDiscovery, MdnsService, PortAllocator, ServiceInfo as RustServiceInfo};
use parking_lot::Mutex;
use pyo3::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info};

/// Python-exposed service information
#[pyclass(name = "ServiceInfo")]
#[derive(Clone)]
pub struct PyServiceInfo {
    #[pyo3(get)]
    pub name: String,

    #[pyo3(get)]
    pub host: String,

    #[pyo3(get)]
    pub port: u16,

    #[pyo3(get)]
    pub metadata: HashMap<String, String>,
}

#[pymethods]
impl PyServiceInfo {
    fn __repr__(&self) -> String {
        format!(
            "ServiceInfo(name='{}', host='{}', port={})",
            self.name, self.host, self.port
        )
    }
}

impl From<RustServiceInfo> for PyServiceInfo {
    fn from(info: RustServiceInfo) -> Self {
        Self {
            name: info.name,
            host: info.host,
            port: info.port,
            metadata: info.metadata,
        }
    }
}

/// Python-exposed service discovery
#[pyclass(name = "ServiceDiscovery")]
pub struct PyServiceDiscovery {
    /// Port allocator
    port_allocator: PortAllocator,

    /// mDNS service (optional)
    mdns_service: Option<Arc<Mutex<MdnsService>>>,

    /// HTTP discovery server (optional)
    http_discovery: Option<Arc<Mutex<HttpDiscovery>>>,

    /// Allocated bridge port
    #[pyo3(get)]
    bridge_port: u16,

    /// Discovery port
    #[pyo3(get)]
    discovery_port: u16,

    /// Whether mDNS is enabled
    #[pyo3(get)]
    mdns_enabled: bool,
}

#[pymethods]
impl PyServiceDiscovery {
    /// Create a new service discovery instance
    ///
    /// Args:
    ///     bridge_port: Bridge WebSocket port (0 = auto-allocate)
    ///     discovery_port: HTTP discovery port (default: 9000)
    ///     enable_mdns: Enable mDNS service discovery
    ///     service_name: Service instance name (default: "AuroraView")
    #[new]
    #[pyo3(signature = (bridge_port=0, discovery_port=9000, enable_mdns=true, _service_name="AuroraView"))]
    fn new(
        bridge_port: u16,
        discovery_port: u16,
        enable_mdns: bool,
        _service_name: &str,
    ) -> PyResult<Self> {
        info!(
            "Creating ServiceDiscovery (bridge_port={}, discovery_port={}, mdns={})",
            bridge_port, discovery_port, enable_mdns
        );

        // Allocate port if needed
        let port_allocator = PortAllocator::default();
        let actual_bridge_port = if bridge_port == 0 {
            port_allocator
                .find_free_port()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?
        } else {
            bridge_port
        };

        info!("Bridge port: {}", actual_bridge_port);

        // Create mDNS service if enabled
        let mdns_service = if enable_mdns {
            match MdnsService::new() {
                Ok(service) => {
                    info!("mDNS service created");
                    Some(Arc::new(Mutex::new(service)))
                }
                Err(e) => {
                    error!("Failed to create mDNS service: {}", e);
                    None
                }
            }
        } else {
            None
        };

        let mdns_enabled = mdns_service.is_some();

        // Create HTTP discovery server
        let http_discovery = HttpDiscovery::new(discovery_port, actual_bridge_port);

        Ok(Self {
            port_allocator,
            mdns_service,
            http_discovery: Some(Arc::new(Mutex::new(http_discovery))),
            bridge_port: actual_bridge_port,
            discovery_port,
            mdns_enabled,
        })
    }

    /// Start service discovery (HTTP + mDNS)
    ///
    /// Args:
    ///     metadata: Optional metadata dict for mDNS
    fn start(
        &self,
        _py: Python,
        metadata: Option<&Bound<'_, pyo3::types::PyDict>>,
    ) -> PyResult<()> {
        info!("Starting service discovery");

        // Start HTTP discovery in background thread
        if let Some(http) = &self.http_discovery {
            let http_clone = http.clone();
            std::thread::spawn(move || {
                let rt = match tokio::runtime::Runtime::new() {
                    Ok(rt) => rt,
                    Err(e) => {
                        error!("Failed to create Tokio runtime for HTTP discovery: {}", e);
                        return;
                    }
                };

                #[allow(clippy::await_holding_lock)]
                rt.block_on(async {
                    let result = http_clone.lock().start().await;
                    if let Err(e) = result {
                        error!("Failed to start HTTP discovery: {}", e);
                    } else {
                        info!("✅ HTTP discovery started on background thread");
                    }
                });
            });
        }

        // Start mDNS if enabled
        if let Some(mdns) = &self.mdns_service {
            let mut props = HashMap::new();
            props.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
            props.insert("protocol".to_string(), "websocket".to_string());

            // Add custom metadata
            if let Some(meta) = metadata {
                for item in meta.items() {
                    if let (Ok(k), Ok(v)) = (
                        item.get_item(0).and_then(|k| k.extract::<String>()),
                        item.get_item(1).and_then(|v| v.extract::<String>()),
                    ) {
                        props.insert(k, v);
                    }
                }
            }

            mdns.lock()
                .register("AuroraView", self.bridge_port, props)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        }

        info!("✅ Service discovery started");
        Ok(())
    }

    /// Stop service discovery
    fn stop(&self) -> PyResult<()> {
        info!("Stopping service discovery");

        // Stop HTTP discovery
        if let Some(http) = &self.http_discovery {
            http.lock()
                .stop()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        }

        // Stop mDNS
        if let Some(mdns) = &self.mdns_service {
            mdns.lock()
                .unregister()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        }

        info!("✅ Service discovery stopped");
        Ok(())
    }

    /// Discover services via mDNS
    ///
    /// Args:
    ///     timeout_secs: Discovery timeout in seconds (default: 5)
    ///
    /// Returns:
    ///     List of discovered services
    #[pyo3(signature = (timeout_secs=5))]
    fn discover_services(&self, timeout_secs: u64) -> PyResult<Vec<PyServiceInfo>> {
        if let Some(mdns) = &self.mdns_service {
            let services = mdns
                .lock()
                .discover(timeout_secs)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            Ok(services.into_iter().map(PyServiceInfo::from).collect())
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "mDNS is not enabled",
            ))
        }
    }

    /// Find a free port
    ///
    /// Returns:
    ///     Available port number
    fn find_free_port(&self) -> PyResult<u16> {
        self.port_allocator
            .find_free_port()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Check if a port is available
    ///
    /// Args:
    ///     port: Port number to check
    ///
    /// Returns:
    ///     True if port is available
    #[staticmethod]
    fn is_port_available(port: u16) -> bool {
        PortAllocator::is_port_available(port)
    }

    fn __repr__(&self) -> String {
        format!(
            "ServiceDiscovery(bridge_port={}, discovery_port={}, mdns={})",
            self.bridge_port, self.discovery_port, self.mdns_enabled
        )
    }
}

/// Register service discovery module with Python
pub fn register_service_discovery(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyServiceDiscovery>()?;
    m.add_class::<PyServiceInfo>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::time::Duration;

    #[test]
    fn test_new_no_mdns_sets_fields_and_ports() {
        // bridge_port = 0 auto-allocates; discovery_port = 0 (ephemeral when started)
        let sd = PyServiceDiscovery::new(0, 0, false, "AuroraView").expect("new() should succeed");
        assert!(
            !sd.mdns_enabled,
            "mdns should be disabled when enable_mdns=false"
        );
        assert!(sd.bridge_port > 0, "bridge_port should be a valid port");
        assert_eq!(
            sd.discovery_port, 0,
            "discovery_port should be kept as provided (0)"
        );
    }

    #[test]
    fn test_find_free_port_and_is_port_available() {
        let sd = PyServiceDiscovery::new(0, 0, false, "AuroraView").expect("new() should succeed");
        let port = sd.find_free_port().expect("should find a free port");
        assert!(port > 0);
        assert!(PyServiceDiscovery::is_port_available(port));

        // Occupy the port and verify availability toggles
        let listener = TcpListener::bind(("127.0.0.1", port)).expect("should bind to free port");
        assert!(!PyServiceDiscovery::is_port_available(port));
        drop(listener);
    }

    #[test]
    fn test_register_service_discovery_module() {
        pyo3::Python::with_gil(|py| {
            let m = pyo3::types::PyModule::new(py, "svc").unwrap();
            super::register_service_discovery(&m).expect("register should succeed");
            assert!(m.getattr("ServiceDiscovery").is_ok());
            assert!(m.getattr("ServiceInfo").is_ok());
        });
    }

    #[test]
    fn test_start_and_stop_http_only() {
        // Do not enable mDNS; discovery_port = 0 so OS assigns a free port
        let sd = PyServiceDiscovery::new(0, 0, false, "AuroraView").expect("new() should succeed");
        Python::with_gil(|py| {
            sd.start(py, None)
                .expect("start should succeed without mDNS");
        });
        // Give the background runtime a brief moment
        std::thread::sleep(Duration::from_millis(50));
        sd.stop().expect("stop should succeed");
    }
}

#[test]
fn test_new_with_mdns_enabled_does_not_panic() {
    // On environments without mDNS backend, this should still succeed and set mdns_enabled accordingly
    let sd = PyServiceDiscovery::new(0, 0, true, "AuroraView")
        .expect("new() with mdns should not panic");
    // mdns_enabled may be true or false depending on platform support; just assert fields are sane
    assert!(sd.bridge_port > 0);
}

#[test]
fn test_discover_services_without_mdns_errors() {
    let sd = PyServiceDiscovery::new(0, 0, false, "AuroraView").expect("new() should succeed");
    match sd.discover_services(1) {
        Err(e) => {
            let msg = e.to_string();
            assert!(msg.contains("mDNS is not enabled"));
        }
        Ok(v) => panic!("expected error, got {} entries", v.len()),
    }
}
