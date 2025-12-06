//! Integration tests for port allocation
//!
//! These tests verify the PortAllocator functionality.

use auroraview_core::service_discovery::port_allocator::PortAllocator;
use rstest::*;
use std::net::TcpListener;
use std::time::Duration;

/// Fixture: Create a default port allocator
#[fixture]
fn allocator() -> PortAllocator {
    PortAllocator::default()
}

/// Fixture: Create a port allocator with custom range
#[fixture]
fn custom_allocator() -> PortAllocator {
    PortAllocator::new(50000, 100)
}

#[rstest]
fn test_default_allocator_values(allocator: PortAllocator) {
    // Default values should be 9001 start port and 100 max attempts
    let port = allocator.find_free_port();
    assert!(port.is_ok());
    let port_num = port.unwrap();
    assert!(port_num >= 9001);
    assert!(port_num < 9101);
}

#[rstest]
fn test_custom_allocator_range(custom_allocator: PortAllocator) {
    let port = custom_allocator.find_free_port();
    assert!(port.is_ok());
    let port_num = port.unwrap();
    assert!(port_num >= 50000);
    assert!(port_num < 50100);
}

#[rstest]
#[case(50000, 10)]
#[case(60000, 50)]
#[case(40000, 20)]
fn test_allocator_with_different_ranges(#[case] start: u16, #[case] max_attempts: u16) {
    let allocator = PortAllocator::new(start, max_attempts);
    let port = allocator.find_free_port();
    assert!(port.is_ok());
    let port_num = port.unwrap();
    assert!(port_num >= start);
    assert!(port_num < start + max_attempts);
}

#[rstest]
fn test_is_port_available_with_bound_port() {
    // Bind to a random port
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let bound_port = listener.local_addr().unwrap().port();

    // Port should be unavailable while listener is active
    assert!(!PortAllocator::is_port_available(bound_port));

    // Drop listener to free the port
    drop(listener);

    // Port should now be available
    assert!(PortAllocator::is_port_available(bound_port));
}

#[rstest]
fn test_is_port_available_with_free_port() {
    // Find a free port first
    let allocator = PortAllocator::new(55000, 10);
    let port = allocator.find_free_port().unwrap();

    // Port should be available
    assert!(PortAllocator::is_port_available(port));
}

#[rstest]
fn test_find_free_port_skips_occupied_ports() {
    // Bind to the first port in range
    let start_port = 56000;
    let _listener1 = TcpListener::bind(format!("127.0.0.1:{}", start_port)).unwrap();
    let _listener2 = TcpListener::bind(format!("127.0.0.1:{}", start_port + 1)).unwrap();

    // Allocator should skip occupied ports and find the next free one
    let allocator = PortAllocator::new(start_port, 10);
    let port = allocator.find_free_port();
    assert!(port.is_ok());
    let port_num = port.unwrap();
    assert!(port_num >= start_port + 2);
}

#[rstest]
fn test_find_free_port_with_timeout() {
    let allocator = PortAllocator::new(57000, 50);
    let port = allocator.find_free_port_with_timeout(Duration::from_millis(100));
    assert!(port.is_ok());
}

#[rstest]
fn test_port_allocator_exhaustion() {
    // Create allocator with very limited range
    let allocator = PortAllocator::new(65530, 5);

    // Bind to all ports in range
    let mut listeners = Vec::new();
    for offset in 0..5 {
        let port = 65530 + offset;
        if let Ok(listener) = TcpListener::bind(format!("127.0.0.1:{}", port)) {
            listeners.push(listener);
        }
    }

    // Should fail to find a free port
    let result = allocator.find_free_port();
    // May succeed if some ports in range are still free, or fail if all occupied
    match result {
        Ok(port) => {
            // If it succeeds, port should be in range
            assert!((65530..65535).contains(&port));
        }
        Err(_) => {
            // Expected if all ports are occupied
        }
    }
}

#[rstest]
fn test_port_allocator_at_boundary() {
    // Test at the upper boundary of port range
    let allocator = PortAllocator::new(65535, 1);
    let result = allocator.find_free_port();

    match result {
        Ok(port) => assert_eq!(port, 65535),
        Err(_) => {
            // Expected if port 65535 is in use
        }
    }
}

#[rstest]
fn test_multiple_allocators_find_different_ports() {
    let allocator1 = PortAllocator::new(58000, 100);
    let allocator2 = PortAllocator::new(58000, 100);

    let port1 = allocator1.find_free_port().unwrap();
    let _listener = TcpListener::bind(format!("127.0.0.1:{}", port1)).unwrap();

    let port2 = allocator2.find_free_port().unwrap();

    // Second allocator should find a different port
    assert_ne!(port1, port2);
}

#[rstest]
fn test_port_allocator_concurrent_access() {
    use std::sync::Arc;
    use std::thread;

    let allocator = Arc::new(PortAllocator::new(59000, 100));
    let mut handles = vec![];

    // Spawn multiple threads trying to allocate ports
    for _ in 0..5 {
        let allocator_clone = allocator.clone();
        let handle = thread::spawn(move || allocator_clone.find_free_port());
        handles.push(handle);
    }

    // Collect results
    let mut ports = vec![];
    for handle in handles {
        if let Ok(Ok(port)) = handle.join() {
            ports.push(port);
        }
    }

    // All threads should have found ports
    assert_eq!(ports.len(), 5);

    // All ports should be in the expected range
    for port in &ports {
        assert!(*port >= 59000 && *port < 59100);
    }
}

#[rstest]
fn test_port_zero_handling() {
    // Port 0 should be skipped as it's invalid
    let allocator = PortAllocator::new(0, 10);
    let port = allocator.find_free_port();

    if let Ok(port_num) = port {
        // Should skip port 0 and find a valid port
        assert!(port_num > 0);
    }
}
