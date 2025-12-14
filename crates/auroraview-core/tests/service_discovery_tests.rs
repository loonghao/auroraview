//! Tests for service discovery module

use auroraview_core::service_discovery::{
    HttpDiscovery, MdnsService, PortAllocator, ServiceInfo, SERVICE_TYPE,
};
use rstest::rstest;
use std::collections::HashMap;
use std::net::TcpListener;

// ============================================================================
// ServiceInfo tests
// ============================================================================

#[rstest]
fn test_service_info_creation() {
    let service = ServiceInfo::new("test-service".to_string(), "localhost".to_string(), 9001);

    assert_eq!(service.name, "test-service");
    assert_eq!(service.host, "localhost");
    assert_eq!(service.port, 9001);
}

#[rstest]
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

// ============================================================================
// PortAllocator tests
// ============================================================================

#[rstest]
fn test_port_allocator_creation() {
    let allocator = PortAllocator::new(9001, 100);
    assert_eq!(allocator.start_port(), 9001);
    assert_eq!(allocator.max_attempts(), 100);
}

#[rstest]
fn test_default_port_allocator() {
    let allocator = PortAllocator::default();
    assert_eq!(allocator.start_port(), 9001);
    assert_eq!(allocator.max_attempts(), 100);
}

#[rstest]
fn test_find_free_port() {
    let allocator = PortAllocator::new(50000, 100);
    let port = allocator.find_free_port();
    assert!(port.is_ok());

    let port_num = port.unwrap();
    assert!(port_num >= 50000);
    assert!(port_num < 50100);
}

#[rstest]
fn test_is_port_available() {
    // Bind to a port to make it unavailable
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let bound_port = listener.local_addr().unwrap().port();

    // Port should be unavailable while listener is active
    assert!(!PortAllocator::is_port_available(bound_port));

    // Drop listener to free the port
    drop(listener);

    // Port should now be available
    assert!(PortAllocator::is_port_available(bound_port));
}

// ============================================================================
// HttpDiscovery tests
// ============================================================================

#[rstest]
fn test_discovery_response_creation() {
    use auroraview_core::service_discovery::DiscoveryResponse;

    let response = DiscoveryResponse {
        service: "AuroraView Bridge".to_string(),
        port: 9001,
        protocol: "websocket".to_string(),
        version: "1.0.0".to_string(),
        timestamp: 1735689600,
    };

    assert_eq!(response.service, "AuroraView Bridge");
    assert_eq!(response.port, 9001);
    assert_eq!(response.protocol, "websocket");
    assert_eq!(response.version, "1.0.0");
    assert_eq!(response.timestamp, 1735689600);
}

#[rstest]
fn test_discovery_response_clone() {
    use auroraview_core::service_discovery::DiscoveryResponse;

    let response = DiscoveryResponse {
        service: "Test Service".to_string(),
        port: 8080,
        protocol: "websocket".to_string(),
        version: "2.0.0".to_string(),
        timestamp: 1735689600,
    };

    let cloned = response.clone();
    assert_eq!(cloned.service, "Test Service");
    assert_eq!(cloned.port, 8080);
}

#[rstest]
fn test_discovery_response_debug() {
    use auroraview_core::service_discovery::DiscoveryResponse;

    let response = DiscoveryResponse {
        service: "AuroraView".to_string(),
        port: 9000,
        protocol: "websocket".to_string(),
        version: "1.0.0".to_string(),
        timestamp: 1735689600,
    };

    let debug_str = format!("{:?}", response);
    assert!(debug_str.contains("DiscoveryResponse"));
    assert!(debug_str.contains("AuroraView"));
    assert!(debug_str.contains("9000"));
}

#[rstest]
fn test_discovery_response_serialize() {
    use auroraview_core::service_discovery::DiscoveryResponse;

    let response = DiscoveryResponse {
        service: "AuroraView Bridge".to_string(),
        port: 9001,
        protocol: "websocket".to_string(),
        version: "1.0.0".to_string(),
        timestamp: 1735689600,
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("\"service\":\"AuroraView Bridge\""));
    assert!(json.contains("\"port\":9001"));
    assert!(json.contains("\"protocol\":\"websocket\""));
}

#[rstest]
fn test_discovery_response_deserialize() {
    use auroraview_core::service_discovery::DiscoveryResponse;

    let json = r#"{
        "service": "AuroraView Bridge",
        "port": 9001,
        "protocol": "websocket",
        "version": "1.0.0",
        "timestamp": 1735689600
    }"#;

    let response: DiscoveryResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.service, "AuroraView Bridge");
    assert_eq!(response.port, 9001);
}

#[rstest]
fn test_http_discovery_new() {
    let discovery = HttpDiscovery::new(9000, 9001);
    assert!(!discovery.is_running());
}

#[rstest]
fn test_http_discovery_stop_when_not_running() {
    let mut discovery = HttpDiscovery::new(9000, 9001);
    // Should not panic when stopping a non-running server
    let result = discovery.stop();
    assert!(result.is_ok());
}

// ============================================================================
// MdnsService tests
// ============================================================================

#[rstest]
fn test_mdns_service_creation() {
    let result = MdnsService::new();
    assert!(result.is_ok());
}

#[rstest]
fn test_service_type_constant() {
    assert_eq!(SERVICE_TYPE, "_auroraview._tcp.local.");
}

#[rstest]
fn test_unregister_without_registration() {
    let service = MdnsService::new().unwrap();
    let result = service.unregister();
    assert!(result.is_ok());
}

#[rstest]
fn test_service_drop_unregisters() {
    let service = MdnsService::new().unwrap();
    let metadata = HashMap::new();
    service.register("DropTest", 9007, metadata).unwrap();
    drop(service);
}
