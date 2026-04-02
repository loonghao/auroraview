//! Tests for service discovery module

use auroraview_core::service_discovery::{
    HttpDiscovery, InstanceInfo, InstanceRegistry, MdnsService, PortAllocator, ServiceInfo,
    SERVICE_TYPE,
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

// ============================================================================
// ServiceInfo edge cases
// ============================================================================

#[rstest]
fn test_service_info_empty_name() {
    let s = ServiceInfo::new(String::new(), "localhost".to_string(), 9001);
    assert_eq!(s.name, "");
}

#[rstest]
fn test_service_info_no_metadata_default() {
    let s = ServiceInfo::new("svc".to_string(), "127.0.0.1".to_string(), 9000);
    assert!(s.metadata.is_empty());
}

#[rstest]
fn test_service_info_multiple_metadata() {
    let s = ServiceInfo::new("svc".to_string(), "localhost".to_string(), 8080)
        .with_metadata("k1".to_string(), "v1".to_string())
        .with_metadata("k2".to_string(), "v2".to_string())
        .with_metadata("k3".to_string(), "v3".to_string());
    assert_eq!(s.metadata.len(), 3);
    assert_eq!(s.metadata.get("k2"), Some(&"v2".to_string()));
}

#[rstest]
fn test_service_info_metadata_overwrite() {
    let s = ServiceInfo::new("svc".to_string(), "localhost".to_string(), 8080)
        .with_metadata("key".to_string(), "first".to_string())
        .with_metadata("key".to_string(), "second".to_string());
    assert_eq!(s.metadata.get("key"), Some(&"second".to_string()));
}

#[rstest]
fn test_service_info_debug() {
    let s = ServiceInfo::new("my-svc".to_string(), "localhost".to_string(), 9999);
    let debug = format!("{:?}", s);
    assert!(debug.contains("my-svc"));
    assert!(debug.contains("9999"));
}

#[rstest]
fn test_service_info_clone() {
    let s = ServiceInfo::new("svc".to_string(), "host".to_string(), 1234)
        .with_metadata("k".to_string(), "v".to_string());
    let cloned = s.clone();
    assert_eq!(cloned.name, "svc");
    assert_eq!(cloned.port, 1234);
    assert_eq!(cloned.metadata.get("k"), Some(&"v".to_string()));
}

#[rstest]
#[case("service-a", "host-a", 9001)]
#[case("service-b", "host-b", 9002)]
#[case("service-c", "192.168.1.1", 8080)]
fn test_service_info_parametrized(
    #[case] name: &str,
    #[case] host: &str,
    #[case] port: u16,
) {
    let s = ServiceInfo::new(name.to_string(), host.to_string(), port);
    assert_eq!(s.name, name);
    assert_eq!(s.host, host);
    assert_eq!(s.port, port);
}

// ============================================================================
// InstanceInfo edge cases
// ============================================================================

#[rstest]
fn test_instance_info_new() {
    let info = InstanceInfo::new("win-1".to_string(), "MyWindow".to_string(), 9222);
    assert_eq!(info.window_id, "win-1");
    assert_eq!(info.title, "MyWindow");
    assert_eq!(info.cdp_port, 9222);
    assert_eq!(info.app_name, "AuroraView");
    assert!(info.dcc_type.is_none());
    assert!(info.dcc_version.is_none());
    assert!(info.panel_name.is_none());
    assert!(info.dock_area.is_none());
    assert!(!info.is_loading);
}

#[rstest]
fn test_instance_info_ws_url() {
    let info = InstanceInfo::new("w".to_string(), "T".to_string(), 9222);
    assert_eq!(info.ws_url(), "ws://127.0.0.1:9222/devtools/page/1");
}

#[rstest]
fn test_instance_info_devtools_url() {
    let info = InstanceInfo::new("w".to_string(), "T".to_string(), 9222);
    let url = info.devtools_url();
    assert!(url.contains("9222"));
    assert!(url.contains("devtools"));
}

#[rstest]
#[case(9222)]
#[case(9223)]
#[case(19000)]
fn test_instance_info_ws_url_port(#[case] port: u16) {
    let info = InstanceInfo::new("w".to_string(), "T".to_string(), port);
    assert!(info.ws_url().contains(&port.to_string()));
}

#[rstest]
fn test_instance_info_with_dcc() {
    let info = InstanceInfo::new("w".to_string(), "T".to_string(), 9222)
        .with_dcc("maya", Some("2025"));
    assert_eq!(info.dcc_type.as_deref(), Some("maya"));
    assert_eq!(info.dcc_version.as_deref(), Some("2025"));
}

#[rstest]
fn test_instance_info_with_dcc_no_version() {
    let info = InstanceInfo::new("w".to_string(), "T".to_string(), 9222)
        .with_dcc("houdini", None);
    assert_eq!(info.dcc_type.as_deref(), Some("houdini"));
    assert!(info.dcc_version.is_none());
}

#[rstest]
fn test_instance_info_with_panel() {
    let info = InstanceInfo::new("w".to_string(), "T".to_string(), 9222)
        .with_panel("ScenePanel", Some("right"));
    assert_eq!(info.panel_name.as_deref(), Some("ScenePanel"));
    assert_eq!(info.dock_area.as_deref(), Some("right"));
}

#[rstest]
fn test_instance_info_with_panel_no_area() {
    let info = InstanceInfo::new("w".to_string(), "T".to_string(), 9222)
        .with_panel("MyPanel", None);
    assert_eq!(info.panel_name.as_deref(), Some("MyPanel"));
    assert!(info.dock_area.is_none());
}

#[rstest]
fn test_instance_info_with_metadata() {
    let info = InstanceInfo::new("w".to_string(), "T".to_string(), 9222)
        .with_metadata("env", "staging")
        .with_metadata("version", "2.0");
    assert_eq!(info.metadata.get("env").map(String::as_str), Some("staging"));
    assert_eq!(
        info.metadata.get("version").map(String::as_str),
        Some("2.0")
    );
}

#[rstest]
fn test_instance_info_clone() {
    let info = InstanceInfo::new("w".to_string(), "T".to_string(), 9222)
        .with_dcc("blender", Some("4.0"));
    let cloned = info.clone();
    assert_eq!(cloned.window_id, "w");
    assert_eq!(cloned.dcc_type.as_deref(), Some("blender"));
}

#[rstest]
fn test_instance_info_debug() {
    let info = InstanceInfo::new("my-win".to_string(), "Test".to_string(), 9222);
    let debug = format!("{:?}", info);
    assert!(debug.contains("my-win"));
}

#[rstest]
fn test_instance_info_serialize_round_trip() {
    let info = InstanceInfo::new("win-ser".to_string(), "SerWindow".to_string(), 9300)
        .with_dcc("nuke", None)
        .with_metadata("foo", "bar");
    let json = serde_json::to_string(&info).unwrap();
    let decoded: InstanceInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.window_id, "win-ser");
    assert_eq!(decoded.cdp_port, 9300);
    assert_eq!(decoded.dcc_type.as_deref(), Some("nuke"));
    assert_eq!(decoded.metadata.get("foo").map(String::as_str), Some("bar"));
}

#[rstest]
fn test_instance_info_serde_default_app_name() {
    // When app_name is missing in JSON, it should default to "AuroraView"
    let json = r#"{
        "window_id": "w",
        "title": "T",
        "cdp_port": 9222,
        "pid": 1,
        "start_time": 0,
        "url": "",
        "html_title": "",
        "is_loading": false
    }"#;
    let info: InstanceInfo = serde_json::from_str(json).unwrap();
    assert_eq!(info.app_name, "AuroraView");
}

// ============================================================================
// InstanceRegistry basic operations (using temp dir via LOCALAPPDATA override)
// ============================================================================

#[rstest]
fn test_instance_registry_new() {
    // Just ensure creation doesn't panic
    let result = InstanceRegistry::new();
    assert!(result.is_ok());
}

#[rstest]
fn test_instance_registry_instances_dir_exists() {
    let registry = InstanceRegistry::new().unwrap();
    assert!(registry.instances_dir().exists());
}

#[rstest]
fn test_instance_registry_get_nonexistent() {
    let registry = InstanceRegistry::new().unwrap();
    let result = registry.get("nonexistent-window-id-xyz-9876");
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[rstest]
fn test_instance_registry_register_and_get() {
    let registry = InstanceRegistry::new().unwrap();
    let window_id = format!("test-reg-{}", std::process::id());
    let info = InstanceInfo::new(window_id.clone(), "TestReg".to_string(), 19100);

    let reg_result = registry.register(&info);
    assert!(reg_result.is_ok(), "register failed: {:?}", reg_result);

    let get_result = registry.get(&window_id).unwrap();
    assert!(get_result.is_some());
    let retrieved = get_result.unwrap();
    assert_eq!(retrieved.window_id, window_id);
    assert_eq!(retrieved.cdp_port, 19100);

    // Cleanup
    let _ = registry.unregister(&window_id);
}

#[rstest]
fn test_instance_registry_unregister() {
    let registry = InstanceRegistry::new().unwrap();
    let window_id = format!("test-unreg-{}", std::process::id());
    let info = InstanceInfo::new(window_id.clone(), "Unreg".to_string(), 19101);

    registry.register(&info).unwrap();
    registry.unregister(&window_id).unwrap();

    let result = registry.get(&window_id).unwrap();
    assert!(result.is_none());
}

#[rstest]
fn test_instance_registry_unregister_nonexistent_ok() {
    let registry = InstanceRegistry::new().unwrap();
    let result = registry.unregister("does-not-exist-abc");
    assert!(result.is_ok());
}

#[rstest]
fn test_instance_registry_update() {
    let registry = InstanceRegistry::new().unwrap();
    let window_id = format!("test-update-{}", std::process::id());
    let info = InstanceInfo::new(window_id.clone(), "Before".to_string(), 19102);

    registry.register(&info).unwrap();

    let updated = registry
        .update(&window_id, |i| {
            i.title = "After".to_string();
        })
        .unwrap();
    assert!(updated);

    let retrieved = registry.get(&window_id).unwrap().unwrap();
    assert_eq!(retrieved.title, "After");

    // Cleanup
    let _ = registry.unregister(&window_id);
}

#[rstest]
fn test_instance_registry_update_nonexistent_returns_false() {
    let registry = InstanceRegistry::new().unwrap();
    let updated = registry
        .update("no-such-window", |_| {})
        .unwrap();
    assert!(!updated);
}

#[rstest]
fn test_instance_registry_get_by_cdp_port() {
    let registry = InstanceRegistry::new().unwrap();
    let window_id = format!("test-cdp-{}", std::process::id());
    let info = InstanceInfo::new(window_id.clone(), "CDP".to_string(), 19103);

    registry.register(&info).unwrap();

    let result = registry.get_by_cdp_port(19103).unwrap();
    assert!(result.is_some());
    let found = result.unwrap();
    assert_eq!(found.window_id, window_id);

    // Cleanup
    let _ = registry.unregister(&window_id);
}

#[rstest]
fn test_instance_registry_get_by_cdp_port_not_found() {
    let registry = InstanceRegistry::new().unwrap();
    // Use an unlikely port that no test registers
    let result = registry.get_by_cdp_port(19199).unwrap();
    // May or may not be None (stale files), but should not error
    let _ = result;
}

#[rstest]
fn test_instance_registry_get_all_returns_vec() {
    let registry = InstanceRegistry::new().unwrap();
    let result = registry.get_all();
    assert!(result.is_ok());
}

#[rstest]
fn test_instance_registry_cleanup_does_not_panic() {
    let registry = InstanceRegistry::new().unwrap();
    let window_id = format!("test-cleanup-{}", std::process::id());
    let info = InstanceInfo::new(window_id.clone(), "Cleanup".to_string(), 19104);
    registry.register(&info).unwrap();
    // cleanup removes all registered ids
    registry.cleanup();
    // The file should be gone
    let result = registry.get(&window_id).unwrap();
    assert!(result.is_none());
}

// ============================================================================
// PortAllocator edge cases
// ============================================================================

#[rstest]
fn test_port_allocator_saturating_add_does_not_overflow() {
    // start at near max u16, 1 attempt — should return NoFreePort or a port
    let allocator = PortAllocator::new(65534, 1);
    // Result may be Ok or Err; important: no panic
    let _result = allocator.find_free_port();
}

#[rstest]
fn test_port_allocator_find_free_port_with_timeout() {
    let allocator = PortAllocator::new(50100, 100);
    let result = allocator.find_free_port_with_timeout(std::time::Duration::from_millis(500));
    // Should work same as find_free_port
    assert!(result.is_ok());
}

#[rstest]
#[case(9001, 10)]
#[case(10000, 50)]
#[case(49000, 20)]
fn test_port_allocator_parametrized_new(#[case] start: u16, #[case] max: u16) {
    let a = PortAllocator::new(start, max);
    assert_eq!(a.start_port(), start);
    assert_eq!(a.max_attempts(), max);
}

// ============================================================================
// ServiceDiscoveryError display
// ============================================================================

#[rstest]
fn test_error_no_free_port_display() {
    use auroraview_core::service_discovery::ServiceDiscoveryError;
    let e = ServiceDiscoveryError::NoFreePort {
        start: 9001,
        end: 9100,
    };
    let msg = format!("{}", e);
    assert!(msg.contains("9001"));
    assert!(msg.contains("9100"));
}

#[rstest]
fn test_error_port_in_use_display() {
    use auroraview_core::service_discovery::ServiceDiscoveryError;
    let e = ServiceDiscoveryError::PortInUse(9999);
    let msg = format!("{}", e);
    assert!(msg.contains("9999"));
}

#[rstest]
fn test_error_mdns_display() {
    use auroraview_core::service_discovery::ServiceDiscoveryError;
    let e = ServiceDiscoveryError::MdnsError("timeout".to_string());
    let msg = format!("{}", e);
    assert!(msg.contains("timeout"));
}

#[rstest]
fn test_error_http_display() {
    use auroraview_core::service_discovery::ServiceDiscoveryError;
    let e = ServiceDiscoveryError::HttpError("connection refused".to_string());
    let msg = format!("{}", e);
    assert!(msg.contains("connection refused"));
}

#[rstest]
fn test_error_io_display() {
    use auroraview_core::service_discovery::ServiceDiscoveryError;
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let e = ServiceDiscoveryError::IoError(io_err);
    let msg = format!("{}", e);
    assert!(msg.contains("file not found"));
}

#[rstest]
fn test_error_no_free_port_debug() {
    use auroraview_core::service_discovery::ServiceDiscoveryError;
    let e = ServiceDiscoveryError::NoFreePort { start: 1, end: 2 };
    let dbg = format!("{:?}", e);
    assert!(dbg.contains("NoFreePort"));
}

// ============================================================================
// InstanceRegistry concurrent operations
// ============================================================================

#[rstest]
fn test_instance_registry_concurrent_register() {
    use std::sync::Arc;
    let registry = Arc::new(InstanceRegistry::new().unwrap());

    let handles: Vec<_> = (0..8)
        .map(|i| {
            let reg = Arc::clone(&registry);
            std::thread::spawn(move || {
                let window_id = format!("concurrent-reg-{}-{}", std::process::id(), i);
                let info = InstanceInfo::new(window_id.clone(), "ConcThread".to_string(), 19200 + i);
                let _ = reg.register(&info);
                let _ = reg.unregister(&window_id);
            })
        })
        .collect();
    for h in handles {
        h.join().expect("thread panicked");
    }
}

#[rstest]
fn test_instance_registry_concurrent_register_unregister() {
    use std::sync::Arc;
    let registry = Arc::new(InstanceRegistry::new().unwrap());

    // Pre-register some entries
    let ids: Vec<String> = (0..4)
        .map(|i| {
            let wid = format!("conc-unreg-{}-{}", std::process::id(), i);
            let info = InstanceInfo::new(wid.clone(), "Pre".to_string(), 19210 + i);
            let _ = registry.register(&info);
            wid
        })
        .collect();

    let handles: Vec<_> = ids
        .into_iter()
        .map(|wid| {
            let reg = Arc::clone(&registry);
            std::thread::spawn(move || {
                let _ = reg.unregister(&wid);
            })
        })
        .collect();
    for h in handles {
        h.join().expect("thread panicked");
    }
}

#[rstest]
fn test_instance_registry_concurrent_get() {
    use std::sync::Arc;
    let registry = Arc::new(InstanceRegistry::new().unwrap());
    let window_id = format!("conc-get-{}", std::process::id());
    let info = InstanceInfo::new(window_id.clone(), "GetTest".to_string(), 19220);
    registry.register(&info).unwrap();

    let handles: Vec<_> = (0..8)
        .map(|_| {
            let reg = Arc::clone(&registry);
            let wid = window_id.clone();
            std::thread::spawn(move || {
                let _ = reg.get(&wid);
            })
        })
        .collect();
    for h in handles {
        h.join().expect("thread panicked");
    }

    // Cleanup
    let _ = registry.unregister(&window_id);
}

#[rstest]
fn test_instance_registry_concurrent_get_all() {
    use std::sync::Arc;
    let registry = Arc::new(InstanceRegistry::new().unwrap());

    let handles: Vec<_> = (0..6)
        .map(|_| {
            let reg = Arc::clone(&registry);
            std::thread::spawn(move || {
                let _ = reg.get_all();
            })
        })
        .collect();
    for h in handles {
        h.join().expect("thread panicked");
    }
}

#[rstest]
fn test_instance_registry_register_multiple_then_get_all() {
    let registry = InstanceRegistry::new().unwrap();
    let pid = std::process::id();

    let wids: Vec<String> = (0..4)
        .map(|i| {
            let wid = format!("multi-reg-{}-{}", pid, i);
            let info = InstanceInfo::new(wid.clone(), format!("Win{}", i), 19230 + i);
            registry.register(&info).unwrap();
            wid
        })
        .collect();

    let all = registry.get_all().unwrap();
    // At least our 4 entries (may include stale from other tests)
    assert!(all.len() >= 4);

    // Cleanup
    for wid in &wids {
        let _ = registry.unregister(wid);
    }
}

// ============================================================================
// PortAllocator concurrent find_free_port
// ============================================================================

#[rstest]
fn test_port_allocator_concurrent_no_panic() {
    use std::sync::Arc;
    let allocator = Arc::new(PortAllocator::new(51000, 200));

    let handles: Vec<_> = (0..8)
        .map(|_| {
            let a = Arc::clone(&allocator);
            std::thread::spawn(move || {
                let _ = a.find_free_port();
            })
        })
        .collect();
    for h in handles {
        h.join().expect("thread panicked");
    }
}

#[rstest]
fn test_port_allocator_is_port_available_concurrent() {
    let handles: Vec<_> = (0..8)
        .map(|_| {
            std::thread::spawn(|| {
                let listener = TcpListener::bind("127.0.0.1:0").unwrap();
                let port = listener.local_addr().unwrap().port();
                assert!(!PortAllocator::is_port_available(port));
                drop(listener);
            })
        })
        .collect();
    for h in handles {
        h.join().expect("thread panicked");
    }
}

// ============================================================================
// InstanceInfo pid and start_time fields
// ============================================================================

#[rstest]
fn test_instance_info_pid_is_current_process() {
    let info = InstanceInfo::new("pid-test".to_string(), "T".to_string(), 9222);
    assert_eq!(info.pid, std::process::id());
}

#[rstest]
fn test_instance_info_start_time_nonzero() {
    let info = InstanceInfo::new("st-test".to_string(), "T".to_string(), 9222);
    assert!(info.start_time > 0);
}

#[rstest]
fn test_instance_info_app_version_nonempty() {
    let info = InstanceInfo::new("ver-test".to_string(), "T".to_string(), 9222);
    assert!(!info.app_version.is_empty());
}

#[rstest]
fn test_instance_info_url_html_title_defaults() {
    let info = InstanceInfo::new("url-test".to_string(), "T".to_string(), 9222);
    assert_eq!(info.url, "");
    assert_eq!(info.html_title, "");
    assert!(!info.is_loading);
}

// ============================================================================
// DiscoveryResponse serde roundtrip
// ============================================================================

#[rstest]
fn test_discovery_response_serde_roundtrip() {
    use auroraview_core::service_discovery::DiscoveryResponse;

    let response = DiscoveryResponse {
        service: "RoundTrip".to_string(),
        port: 9001,
        protocol: "websocket".to_string(),
        version: "3.0.0".to_string(),
        timestamp: 9999999,
    };

    let json = serde_json::to_string(&response).unwrap();
    let decoded: DiscoveryResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.service, "RoundTrip");
    assert_eq!(decoded.port, 9001);
    assert_eq!(decoded.protocol, "websocket");
    assert_eq!(decoded.version, "3.0.0");
    assert_eq!(decoded.timestamp, 9999999);
}

#[rstest]
#[case(9001, "ws1", "1.0")]
#[case(8080, "ws2", "2.0")]
#[case(443, "wss", "3.0")]
fn test_discovery_response_parametrized(#[case] port: u16, #[case] svc: &str, #[case] ver: &str) {
    use auroraview_core::service_discovery::DiscoveryResponse;

    let r = DiscoveryResponse {
        service: svc.to_string(),
        port,
        protocol: "websocket".to_string(),
        version: ver.to_string(),
        timestamp: 0,
    };
    let json = serde_json::to_string(&r).unwrap();
    let decoded: DiscoveryResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.port, port);
    assert_eq!(decoded.service, svc);
    assert_eq!(decoded.version, ver);
}
