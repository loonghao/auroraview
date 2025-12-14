//! Port allocation tests

use auroraview_core::port::PortAllocator;
use rstest::rstest;
use std::net::TcpListener;

#[rstest]
fn test_find_any_port() {
    let port = PortAllocator::find_any_port().unwrap();
    assert!(port > 0);
}

#[rstest]
fn test_is_port_available() {
    // Bind a port
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    // Port should not be available
    assert!(!PortAllocator::is_port_available(port));

    // Drop listener
    drop(listener);

    // Port should be available now
    assert!(PortAllocator::is_port_available(port));
}

#[rstest]
fn test_port_allocator_default() {
    // Test that default allocator can find a port in expected range
    let allocator = PortAllocator::default();
    let port = allocator.find_free_port().unwrap();
    // Default starts at 9001, max 100 attempts, so port should be in [9001, 9101)
    assert!((9001..9101).contains(&port));
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
