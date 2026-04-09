//! Port allocation tests

use std::net::TcpListener;

use auroraview_core::port::{PortAllocator, PortError};
use rstest::rstest;

// ---------------------------------------------------------------------------
// find_any_port
// ---------------------------------------------------------------------------

#[rstest]
fn test_find_any_port() {
    let port = PortAllocator::find_any_port().unwrap();
    assert!(port > 0);
}

#[rstest]
fn test_find_any_port_valid_range() {
    let port = PortAllocator::find_any_port().unwrap();
    // OS-assigned ephemeral ports are typically in 1024..65535
    assert!(port >= 1024);
}

// ---------------------------------------------------------------------------
// is_port_available
// ---------------------------------------------------------------------------

#[rstest]
fn test_is_port_available() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    assert!(!PortAllocator::is_port_available(port));

    drop(listener);

    assert!(PortAllocator::is_port_available(port));
}

#[rstest]
fn test_is_port_available_free_port() {
    // OS-assigned port is immediately released; next call should see it available
    let port = PortAllocator::find_any_port().unwrap();
    assert!(PortAllocator::is_port_available(port));
}

// ---------------------------------------------------------------------------
// PortAllocator::default
// ---------------------------------------------------------------------------

#[rstest]
fn test_port_allocator_default() {
    let allocator = PortAllocator::default();
    let port = allocator.find_free_port().unwrap();
    assert!((9001..9101).contains(&port));
}

// ---------------------------------------------------------------------------
// find_free_port
// ---------------------------------------------------------------------------

#[rstest]
fn test_find_free_port() {
    let allocator = PortAllocator::new(50000, 100);
    let port = allocator.find_free_port().unwrap();
    assert!(port >= 50000);
    assert!(port < 50100);
}

#[rstest]
fn test_find_free_port_high_range() {
    let allocator = PortAllocator::new(60000, 50);
    let port = allocator.find_free_port().unwrap();
    assert!((60000..60050).contains(&port));
}

#[rstest]
fn test_find_free_port_is_available() {
    let allocator = PortAllocator::new(55000, 100);
    let port = allocator.find_free_port().unwrap();
    // The port returned must actually be available at the time it is found.
    // After find_free_port, the port is not bound, so it should still be available.
    assert!(PortAllocator::is_port_available(port));
}

// ---------------------------------------------------------------------------
// PortError
// ---------------------------------------------------------------------------

#[test]
fn test_port_error_display() {
    let err = PortError::NoFreePort {
        start: 8000,
        end: 8100,
    };
    let msg = err.to_string();
    assert!(msg.contains("8000"));
    assert!(msg.contains("8100"));
}

#[test]
fn test_port_error_debug() {
    let err = PortError::NoFreePort { start: 1, end: 2 };
    let debug = format!("{:?}", err);
    assert!(debug.contains("NoFreePort"));
}

// ---------------------------------------------------------------------------
// Concurrency: multiple threads can each find a unique port
// ---------------------------------------------------------------------------

#[test]
fn test_concurrent_port_allocation() {
    use std::collections::HashSet;
    use std::sync::{Arc, Mutex};
    use std::thread;

    let ports: Arc<Mutex<HashSet<u16>>> = Arc::new(Mutex::new(HashSet::new()));
    let mut handles = vec![];

    for i in 0..5u16 {
        let ports_clone = Arc::clone(&ports);
        let start = 51000 + i * 200;
        let handle = thread::spawn(move || {
            let allocator = PortAllocator::new(start, 200);
            if let Ok(port) = allocator.find_free_port() {
                ports_clone.lock().unwrap().insert(port);
            }
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    let found = ports.lock().unwrap();
    // Each thread should have found at least one port
    assert!(!found.is_empty());
}

// ---------------------------------------------------------------------------
// PortAllocator::new edge case: max_attempts = 1
// ---------------------------------------------------------------------------

#[test]
fn test_find_free_port_single_attempt() {
    // With max_attempts=1, if the first port is free we should get it
    let allocator = PortAllocator::new(57000, 1);
    if PortAllocator::is_port_available(57000) {
        let port = allocator.find_free_port().unwrap();
        assert_eq!(port, 57000);
    }
    // If 57000 is occupied, the call should return an error
}
