//! Integration tests for mDNS service discovery
//!
//! These tests verify the complete mDNS service registration and discovery workflow.

use auroraview_core::service_discovery::mdns_service::MdnsService;
use rstest::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

/// Fixture: Create a new MdnsService instance
#[fixture]
fn mdns_service() -> MdnsService {
    MdnsService::new().expect("Failed to create MdnsService")
}

/// Fixture: Create metadata for testing
#[fixture]
fn test_metadata() -> HashMap<String, String> {
    let mut metadata = HashMap::new();
    metadata.insert("version".to_string(), "1.0.0".to_string());
    metadata.insert("app".to_string(), "test".to_string());
    metadata
}

#[rstest]
fn test_register_service(mdns_service: MdnsService, test_metadata: HashMap<String, String>) {
    let result = mdns_service.register("TestInstance", 9001, test_metadata);
    assert!(result.is_ok(), "Service registration should succeed");
}

#[rstest]
fn test_register_with_empty_metadata(mdns_service: MdnsService) {
    let metadata = HashMap::new();
    let result = mdns_service.register("EmptyMetadata", 9002, metadata);
    assert!(result.is_ok());
}

#[rstest]
fn test_unregister_after_registration(mdns_service: MdnsService) {
    let metadata = HashMap::new();
    mdns_service
        .register("TestUnregister", 9003, metadata)
        .expect("Registration should succeed");

    // Unregister should succeed
    let result = mdns_service.unregister();
    assert!(
        result.is_ok(),
        "Unregister should succeed after registration"
    );
}

#[rstest]
fn test_multiple_registrations(mdns_service: MdnsService) {
    let metadata = HashMap::new();

    // First registration
    let result1 = mdns_service.register("First", 9004, metadata.clone());
    assert!(result1.is_ok(), "First registration should succeed");

    // Second registration (should replace the first)
    let result2 = mdns_service.register("Second", 9005, metadata);
    assert!(result2.is_ok(), "Second registration should succeed");
}

#[rstest]
fn test_discover_with_short_timeout(mdns_service: MdnsService) {
    // Discover with 1 second timeout
    let result = mdns_service.discover(1);
    assert!(result.is_ok());

    // May or may not find services, but should not error
    let _services = result.unwrap();
}

#[rstest]
fn test_discover_returns_service_info(mdns_service: MdnsService) {
    // Register a service first
    let mut metadata = HashMap::new();
    metadata.insert("test".to_string(), "value".to_string());
    mdns_service
        .register("DiscoverTest", 9006, metadata)
        .unwrap();

    // Try to discover (may or may not find our own service)
    let result = mdns_service.discover(2);
    assert!(result.is_ok());
}

#[rstest]
fn test_register_with_special_characters_in_metadata(mdns_service: MdnsService) {
    let mut metadata = HashMap::new();
    // mDNS TXT records support spaces in values and UTF-8
    metadata.insert("description".to_string(), "value with spaces".to_string());
    metadata.insert("unicode".to_string(), "测试🚀".to_string());

    let result = mdns_service.register("SpecialChars", 9008, metadata);
    assert!(result.is_ok());
}

#[rstest]
#[case(65535)] // High port
#[case(1024)] // Low port
fn test_register_with_port_numbers(mdns_service: MdnsService, #[case] port: u16) {
    let metadata = HashMap::new();
    let result = mdns_service.register("PortTest", port, metadata);
    assert!(result.is_ok());
}

#[rstest]
fn test_concurrent_operations() {
    let service: Arc<MdnsService> = Arc::new(MdnsService::new().unwrap());
    let service_clone = Arc::clone(&service);

    let handle = thread::spawn(move || {
        let metadata = HashMap::new();
        service_clone.register("Concurrent", 9009, metadata)
    });

    let result = handle.join().unwrap();
    assert!(result.is_ok());
}
