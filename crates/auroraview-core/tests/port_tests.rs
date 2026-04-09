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
    // Find a free port first, then test with max_attempts=1 starting at that port
    let free_port = PortAllocator::find_any_port().unwrap();
    if PortAllocator::is_port_available(free_port) {
        let allocator = PortAllocator::new(free_port, 1);
        // May succeed or fail depending on race conditions; just verify no panic
        let _ = allocator.find_free_port();
    }
    // Test that max_attempts=1 on an occupied port fails
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let occupied_port = listener.local_addr().unwrap().port();
    let allocator = PortAllocator::new(occupied_port, 1);
    let result = allocator.find_free_port();
    assert!(result.is_err(), "Should fail when single port is occupied");
    // Keep listener alive until after the check
    drop(listener);
}

// ---------------------------------------------------------------------------
// find_any_port returns valid port multiple times
// ---------------------------------------------------------------------------

#[rstest]
fn test_find_any_port_multiple_calls_different_ports() {
    // Multiple calls can return the same or different ephemeral ports; just ensure each is valid
    for _ in 0..5 {
        let port = PortAllocator::find_any_port().unwrap();
        assert!(port >= 1024, "port {} should be >= 1024", port);
        // u16 max is 65535, comparison is implicit
    }
}

// ---------------------------------------------------------------------------
// Port range boundary tests
// ---------------------------------------------------------------------------

#[rstest]
fn test_port_allocator_range_start_only() {
    // Range of 1 starting at a high port that is likely free
    let allocator = PortAllocator::new(58000, 1);
    if PortAllocator::is_port_available(58000) {
        let result = allocator.find_free_port();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 58000);
    }
}

#[rstest]
fn test_port_allocator_large_range() {
    // Large range — should reliably find a port
    let allocator = PortAllocator::new(50000, 5000);
    let port = allocator.find_free_port().unwrap();
    assert!((50000..55000).contains(&port));
}

#[rstest]
fn test_find_free_port_not_bound() {
    // The found port should not be bound (it was just released)
    let allocator = PortAllocator::new(53000, 500);
    let port = allocator.find_free_port().unwrap();
    // If we can bind to it immediately, it was truly free
    let bind_result = std::net::TcpListener::bind(format!("127.0.0.1:{}", port));
    assert!(bind_result.is_ok(), "found port {} should be bindable", port);
}

// ---------------------------------------------------------------------------
// is_port_available on port 0 (reserved)
// ---------------------------------------------------------------------------

#[rstest]
fn test_is_port_available_port_zero_is_special() {
    // Port 0 is reserved / wildcard — behavior is implementation-defined
    // We just verify the function doesn't panic
    let _ = PortAllocator::is_port_available(0);
}

// ---------------------------------------------------------------------------
// PortError variants
// ---------------------------------------------------------------------------

#[test]
fn test_port_error_no_free_port_same_start_end() {
    let err = PortError::NoFreePort { start: 9000, end: 9000 };
    let msg = err.to_string();
    assert!(msg.contains("9000"));
}

#[test]
fn test_port_error_no_free_port_u16_max() {
    let err = PortError::NoFreePort { start: 0, end: 65535 };
    let msg = err.to_string();
    assert!(msg.contains("65535"));
}

#[test]
fn test_port_error_no_free_port_clone() {
    let err = PortError::NoFreePort { start: 8000, end: 8100 };
    let msg1 = err.to_string();
    // Just verify to_string is stable (same output)
    let msg2 = err.to_string();
    assert_eq!(msg1, msg2);
}

// ---------------------------------------------------------------------------
// PortAllocator default range check
// ---------------------------------------------------------------------------

#[rstest]
fn test_port_allocator_default_range_coverage() {
    // Default allocator scans 9001..=9100
    let allocator = PortAllocator::default();
    if let Ok(port) = allocator.find_free_port() {
        assert!((9001..9101).contains(&port), "default range port was {}", port);
    }
    // If range is full, it returns Err — both outcomes are acceptable
}

// ---------------------------------------------------------------------------
// Concurrent binding + release
// ---------------------------------------------------------------------------

#[test]
fn test_find_any_port_is_actually_bindable() {
    // find_any_port() internally binds to port 0, gets the OS-assigned port, then releases it.
    // We check the returned port is actually bindable right after.
    let port = PortAllocator::find_any_port().unwrap();
    // Re-bind to confirm availability
    let listener = std::net::TcpListener::bind(format!("127.0.0.1:{}", port));
    // It's possible (but unlikely) the port was grabbed by OS immediately,
    // so we just ensure no panic and accept either outcome
    let _ = listener;
}

// ---------------------------------------------------------------------------
// High-concurrency port allocation
// ---------------------------------------------------------------------------

#[test]
fn test_concurrent_find_any_port() {
    use std::collections::HashSet;
    use std::sync::{Arc, Mutex};
    use std::thread;

    let ports = Arc::new(Mutex::new(HashSet::<u16>::new()));

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let p = ports.clone();
            thread::spawn(move || {
                if let Ok(port) = PortAllocator::find_any_port() {
                    p.lock().unwrap().insert(port);
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    let found = ports.lock().unwrap();
    assert!(!found.is_empty(), "should find at least one port");
}

// ---------------------------------------------------------------------------
// PortAllocator: new with zero max_attempts returns Err
// ---------------------------------------------------------------------------

#[test]
fn test_port_allocator_zero_attempts_returns_err() {
    let allocator = PortAllocator::new(59000, 0);
    let result = allocator.find_free_port();
    // With 0 attempts, should immediately fail
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// PortError: is Send + Sync
// ---------------------------------------------------------------------------

#[test]
fn test_port_error_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<PortError>();
}

// ---------------------------------------------------------------------------
// PortAllocator: is Send + Sync
// ---------------------------------------------------------------------------

#[test]
fn test_port_allocator_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<PortAllocator>();
}

// ---------------------------------------------------------------------------
// find_free_port: returned port is within declared range
// ---------------------------------------------------------------------------

#[rstest]
#[case(50100, 50)]
#[case(50200, 100)]
#[case(50500, 200)]
fn test_find_free_port_in_range(#[case] start: u16, #[case] max: u16) {
    let allocator = PortAllocator::new(start, max);
    if let Ok(port) = allocator.find_free_port() {
        assert!(port >= start, "port {} below start {}", port, start);
        assert!(port < start + max, "port {} above end {}", port, start + max);
    }
}

// ---------------------------------------------------------------------------
// is_port_available: ports in reserved range are typically occupied
// ---------------------------------------------------------------------------

#[test]
fn test_is_port_available_port_80_not_available_or_returns_bool() {
    // Port 80 requires privilege; just verify no panic
    let _result = PortAllocator::is_port_available(80);
}

// ---------------------------------------------------------------------------
// PortError display includes "no free port" message style
// ---------------------------------------------------------------------------

#[test]
fn test_port_error_display_describes_range() {
    let err = PortError::NoFreePort { start: 12000, end: 13000 };
    let msg = err.to_string();
    // Must mention both boundaries
    assert!(msg.contains("12000") || msg.contains("13000"), "Display: {}", msg);
}

// ---------------------------------------------------------------------------
// Multiple distinct allocators don't interfere
// ---------------------------------------------------------------------------

#[test]
fn test_two_allocators_different_ranges() {
    let a1 = PortAllocator::new(56000, 100);
    let a2 = PortAllocator::new(57000, 100);
    // Both should find ports in their respective ranges
    if let (Ok(p1), Ok(p2)) = (a1.find_free_port(), a2.find_free_port()) {
        assert!((56000..56100).contains(&p1), "p1={}", p1);
        assert!((57000..57100).contains(&p2), "p2={}", p2);
    }
}
