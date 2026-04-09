//! Comprehensive tests for all error types in auroraview-core

use std::io;

use auroraview_core::backend::{WebViewError, WebViewResult};
use auroraview_core::bom::{BomError, BomResult};
use auroraview_core::port::PortError;
use auroraview_core::service_discovery::ServiceDiscoveryError;
use rstest::rstest;

// ============================================================================
// WebViewError — variant construction, Display messages, Clone
// ============================================================================

#[rstest]
fn webview_initialization_display() {
    let e = WebViewError::Initialization("backend init failed".into());
    assert_eq!(e.to_string(), "Initialization error: backend init failed");
}

#[rstest]
fn webview_navigation_display() {
    let e = WebViewError::Navigation("bad url".into());
    assert_eq!(e.to_string(), "Navigation error: bad url");
}

#[rstest]
fn webview_javascript_display() {
    let e = WebViewError::JavaScript("syntax error".into());
    assert_eq!(e.to_string(), "JavaScript error: syntax error");
}

#[rstest]
fn webview_cookie_display() {
    let e = WebViewError::Cookie("cookie rejected".into());
    assert_eq!(e.to_string(), "Cookie error: cookie rejected");
}

#[rstest]
fn webview_settings_display() {
    let e = WebViewError::Settings("invalid key".into());
    assert_eq!(e.to_string(), "Settings error: invalid key");
}

#[rstest]
fn webview_unsupported_platform_display() {
    let e = WebViewError::UnsupportedPlatform("FreeBSD".into());
    assert_eq!(e.to_string(), "Unsupported platform: FreeBSD");
}

#[rstest]
fn webview_unsupported_backend_display() {
    let e = WebViewError::UnsupportedBackend("cef".into());
    assert_eq!(e.to_string(), "Unsupported backend: cef");
}

#[rstest]
fn webview_closed_display() {
    let e = WebViewError::Closed;
    assert_eq!(e.to_string(), "WebView is closed");
}

#[rstest]
fn webview_not_found_display() {
    let e = WebViewError::NotFound("resource.js".into());
    assert_eq!(e.to_string(), "Not found: resource.js");
}

#[rstest]
fn webview_permission_denied_display() {
    let e = WebViewError::PermissionDenied("read".into());
    assert_eq!(e.to_string(), "Permission denied: read");
}

#[rstest]
fn webview_network_display() {
    let e = WebViewError::Network("connection refused".into());
    assert_eq!(e.to_string(), "Network error: connection refused");
}

#[rstest]
fn webview_timeout_display() {
    let e = WebViewError::Timeout("30s".into());
    assert_eq!(e.to_string(), "Timeout: 30s");
}

#[rstest]
fn webview_invalid_argument_display() {
    let e = WebViewError::InvalidArgument("null url".into());
    assert_eq!(e.to_string(), "Invalid argument: null url");
}

#[rstest]
fn webview_internal_display() {
    let e = WebViewError::Internal("unexpected state".into());
    assert_eq!(e.to_string(), "Internal error: unexpected state");
}

#[rstest]
fn webview_unsupported_display() {
    let e = WebViewError::Unsupported("clipboard".into());
    assert_eq!(e.to_string(), "Unsupported operation: clipboard");
}

#[rstest]
fn webview_icon_display() {
    let e = WebViewError::Icon("missing icon.png".into());
    assert_eq!(e.to_string(), "Icon error: missing icon.png");
}

// -- Constructor helpers --

#[rstest]
fn webview_init_helper() {
    let e = WebViewError::init("msg");
    assert!(matches!(e, WebViewError::Initialization(_)));
    assert_eq!(e.to_string(), "Initialization error: msg");
}

#[rstest]
fn webview_navigation_helper() {
    let e = WebViewError::navigation("bad url");
    assert!(matches!(e, WebViewError::Navigation(_)));
}

#[rstest]
fn webview_javascript_helper() {
    let e = WebViewError::javascript("err");
    assert!(matches!(e, WebViewError::JavaScript(_)));
}

#[rstest]
fn webview_invalid_arg_helper() {
    let e = WebViewError::invalid_arg("x");
    assert!(matches!(e, WebViewError::InvalidArgument(_)));
}

#[rstest]
fn webview_internal_helper() {
    let e = WebViewError::internal("x");
    assert!(matches!(e, WebViewError::Internal(_)));
}

#[rstest]
fn webview_icon_helper() {
    let e = WebViewError::icon("x.ico");
    assert!(matches!(e, WebViewError::Icon(_)));
}

// -- Clone --

#[rstest]
fn webview_error_clone() {
    let e = WebViewError::Navigation("fail".into());
    let e2 = e.clone();
    assert_eq!(e.to_string(), e2.to_string());
}

#[rstest]
fn webview_closed_clone() {
    let e = WebViewError::Closed;
    let e2 = e.clone();
    assert_eq!(e.to_string(), e2.to_string());
}

// -- std::error::Error --

#[rstest]
fn webview_error_implements_std_error() {
    let e: &dyn std::error::Error = &WebViewError::Closed;
    assert!(e.source().is_none());
}

// -- WebViewResult --

#[rstest]
fn webview_result_ok() {
    let val: i32 = 42;
    let r: WebViewResult<i32> = Ok(val);
    assert!(r.is_ok());
}

#[rstest]
fn webview_result_err() {
    let r: WebViewResult<i32> = Err(WebViewError::Closed);
    assert!(r.is_err());
}

// ============================================================================
// BomError — variant construction, Display messages, Clone
// ============================================================================

#[rstest]
fn bom_webview_unavailable_display() {
    let e = BomError::WebViewUnavailable;
    assert_eq!(e.to_string(), "WebView not available or locked");
}

#[rstest]
fn bom_window_unavailable_display() {
    let e = BomError::WindowUnavailable;
    assert_eq!(e.to_string(), "Window not available");
}

#[rstest]
fn bom_js_execution_failed_display() {
    let e = BomError::JsExecutionFailed("null ref".into());
    assert_eq!(e.to_string(), "JavaScript execution failed: null ref");
}

#[rstest]
fn bom_platform_not_supported_display() {
    let e = BomError::PlatformNotSupported;
    assert_eq!(e.to_string(), "Platform not supported for this operation");
}

#[rstest]
fn bom_operation_failed_display() {
    let e = BomError::OperationFailed("resize failed".into());
    assert_eq!(e.to_string(), "Operation failed: resize failed");
}

#[rstest]
fn bom_error_clone() {
    let e = BomError::OperationFailed("x".into());
    let e2 = e.clone();
    assert_eq!(e.to_string(), e2.to_string());
}

#[rstest]
fn bom_error_unit_clone() {
    let e = BomError::PlatformNotSupported;
    let e2 = e.clone();
    assert_eq!(e.to_string(), e2.to_string());
}

#[rstest]
fn bom_error_implements_std_error() {
    let e: &dyn std::error::Error = &BomError::WebViewUnavailable;
    assert!(e.source().is_none());
}

#[rstest]
fn bom_result_ok() {
    let val: u32 = 1;
    let r: BomResult<u32> = Ok(val);
    assert!(r.is_ok());
}

#[rstest]
fn bom_result_err() {
    let r: BomResult<u32> = Err(BomError::WindowUnavailable);
    assert!(r.is_err());
}

// ============================================================================
// PortError — Display, Clone, std::error::Error
// ============================================================================

#[rstest]
fn port_error_display() {
    let e = PortError::NoFreePort {
        start: 9001,
        end: 9100,
    };
    assert_eq!(e.to_string(), "No free port found in range 9001-9100");
}

#[rstest]
fn port_error_zero_range_display() {
    let e = PortError::NoFreePort { start: 0, end: 0 };
    assert_eq!(e.to_string(), "No free port found in range 0-0");
}

#[rstest]
fn port_error_clone() {
    let e = PortError::NoFreePort {
        start: 8000,
        end: 8099,
    };
    let e2 = e.clone();
    assert_eq!(e.to_string(), e2.to_string());
}

#[rstest]
fn port_error_implements_std_error() {
    let e: &dyn std::error::Error = &PortError::NoFreePort {
        start: 1024,
        end: 2048,
    };
    assert!(e.source().is_none());
}

// ============================================================================
// ServiceDiscoveryError — Display, From<io::Error>, std::error::Error
// ============================================================================

#[rstest]
fn service_discovery_no_free_port_display() {
    let e = ServiceDiscoveryError::NoFreePort {
        start: 9000,
        end: 9999,
    };
    assert_eq!(e.to_string(), "No free port found in range 9000-9999");
}

#[rstest]
fn service_discovery_port_in_use_display() {
    let e = ServiceDiscoveryError::PortInUse(8080);
    assert_eq!(e.to_string(), "Port 8080 is already in use");
}

#[rstest]
fn service_discovery_mdns_error_display() {
    let e = ServiceDiscoveryError::MdnsError("resolve failed".into());
    assert_eq!(e.to_string(), "mDNS error: resolve failed");
}

#[rstest]
fn service_discovery_http_error_display() {
    let e = ServiceDiscoveryError::HttpError("500".into());
    assert_eq!(e.to_string(), "HTTP error: 500");
}

#[rstest]
fn service_discovery_io_error_from() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file missing");
    let e = ServiceDiscoveryError::from(io_err);
    assert!(matches!(e, ServiceDiscoveryError::IoError(_)));
    assert!(e.to_string().contains("file missing"));
}

#[rstest]
fn service_discovery_io_error_display() {
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
    let e: ServiceDiscoveryError = io_err.into();
    assert!(e.to_string().contains("access denied"));
}

#[rstest]
fn service_discovery_implements_std_error() {
    let e: &dyn std::error::Error = &ServiceDiscoveryError::PortInUse(80);
    // io_error variant chains to the io error source
    assert!(e.source().is_none());
}

#[rstest]
fn service_discovery_io_error_source() {
    let io_err = io::Error::new(io::ErrorKind::BrokenPipe, "pipe broken");
    let e = ServiceDiscoveryError::IoError(io_err);
    let e_ref: &dyn std::error::Error = &e;
    // IoError wraps io::Error which IS the source
    assert!(e_ref.source().is_some());
}

// ============================================================================
// Error types are Send + Sync (compile-time assertion)
// ============================================================================

fn assert_send_sync<T: Send + Sync>() {}

#[rstest]
fn error_types_are_send_sync() {
    assert_send_sync::<WebViewError>();
    assert_send_sync::<BomError>();
    assert_send_sync::<PortError>();
    // ServiceDiscoveryError wraps io::Error which is Send + Sync
    assert_send_sync::<ServiceDiscoveryError>();
}

// ============================================================================
// Error variant matching (pattern completeness check)
// ============================================================================

#[rstest]
fn webview_error_all_variants_match() {
    let variants: Vec<WebViewError> = vec![
        WebViewError::Initialization("a".into()),
        WebViewError::Navigation("b".into()),
        WebViewError::JavaScript("c".into()),
        WebViewError::Cookie("d".into()),
        WebViewError::Settings("e".into()),
        WebViewError::UnsupportedPlatform("f".into()),
        WebViewError::UnsupportedBackend("g".into()),
        WebViewError::Closed,
        WebViewError::NotFound("h".into()),
        WebViewError::PermissionDenied("i".into()),
        WebViewError::Network("j".into()),
        WebViewError::Timeout("k".into()),
        WebViewError::InvalidArgument("l".into()),
        WebViewError::Internal("m".into()),
        WebViewError::Unsupported("n".into()),
        WebViewError::Icon("o".into()),
    ];
    assert_eq!(variants.len(), 16);
    for v in &variants {
        assert!(!v.to_string().is_empty());
    }
}

#[rstest]
fn bom_error_all_variants_match() {
    let variants: Vec<BomError> = vec![
        BomError::WebViewUnavailable,
        BomError::WindowUnavailable,
        BomError::JsExecutionFailed("x".into()),
        BomError::PlatformNotSupported,
        BomError::OperationFailed("y".into()),
    ];
    assert_eq!(variants.len(), 5);
    for v in &variants {
        assert!(!v.to_string().is_empty());
    }
}
