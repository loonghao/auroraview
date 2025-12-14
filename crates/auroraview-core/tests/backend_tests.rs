//! Backend module tests

use auroraview_core::backend::{
    BackendConfig, BackendFactory, BackendType, CookieInfo, LoadProgress, NavigationEvent,
    NavigationState, WebViewError, WebViewSettings, WebViewSettingsImpl,
};
use rstest::rstest;
use std::path::PathBuf;

// ============================================================================
// BackendType Tests (from factory.rs)
// ============================================================================

#[test]
fn test_backend_type_default() {
    let default = BackendType::default();
    assert_eq!(default, BackendType::Wry);
}

#[rstest]
#[case("wry", BackendType::Wry)]
#[case("WRY", BackendType::Wry)]
#[case("Wry", BackendType::Wry)]
fn test_backend_type_from_str_wry(#[case] input: &str, #[case] expected: BackendType) {
    let result: BackendType = input.parse().unwrap();
    assert_eq!(result, expected);
}

#[cfg(target_os = "windows")]
#[rstest]
#[case("webview2", BackendType::WebView2)]
#[case("WebView2", BackendType::WebView2)]
#[case("WEBVIEW2", BackendType::WebView2)]
#[case("webview_2", BackendType::WebView2)]
#[case("wv2", BackendType::WebView2)]
fn test_backend_type_from_str_webview2(#[case] input: &str, #[case] expected: BackendType) {
    let result: BackendType = input.parse().unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_backend_type_from_str_invalid() {
    let result: Result<BackendType, _> = "invalid_backend".parse();
    assert!(result.is_err());
    match result {
        Err(WebViewError::UnsupportedBackend(name)) => {
            assert_eq!(name, "invalid_backend");
        }
        _ => panic!("Expected UnsupportedBackend error"),
    }
}

#[test]
fn test_backend_type_display() {
    assert_eq!(BackendType::Wry.to_string(), "wry");

    #[cfg(target_os = "windows")]
    assert_eq!(BackendType::WebView2.to_string(), "webview2");

    #[cfg(target_os = "macos")]
    assert_eq!(BackendType::WKWebView.to_string(), "wkwebview");

    #[cfg(target_os = "linux")]
    assert_eq!(BackendType::WebKitGTK.to_string(), "webkitgtk");
}

#[test]
fn test_backend_config_default() {
    let config = BackendConfig::default();
    assert_eq!(config.backend_type, BackendType::Wry);
    assert_eq!(config.title, "AuroraView");
    assert_eq!(config.width, 800);
    assert_eq!(config.height, 600);
    assert!(config.url.is_none());
    assert!(config.html.is_none());
    assert!(config.parent_handle.is_none());
    assert!(config.asset_root.is_none());
}

#[test]
fn test_backend_config_clone() {
    let config = BackendConfig {
        backend_type: BackendType::Wry,
        title: "Test".to_string(),
        width: 1024,
        height: 768,
        url: Some("https://example.com".to_string()),
        html: None,
        parent_handle: Some(12345),
        asset_root: Some(PathBuf::from("/assets")),
        settings: WebViewSettingsImpl::default(),
    };

    let cloned = config.clone();
    assert_eq!(cloned.title, "Test");
    assert_eq!(cloned.width, 1024);
    assert_eq!(cloned.url, Some("https://example.com".to_string()));
    assert_eq!(cloned.parent_handle, Some(12345));
}

#[test]
fn test_backend_factory_default_backend() {
    let default = BackendFactory::default_backend();
    assert_eq!(default, BackendType::Wry);
}

#[test]
fn test_backend_factory_available_backends() {
    let backends = BackendFactory::available_backends();
    assert!(backends.contains(&BackendType::Wry));

    #[cfg(target_os = "windows")]
    assert!(backends.contains(&BackendType::WebView2));

    #[cfg(target_os = "macos")]
    assert!(backends.contains(&BackendType::WKWebView));

    #[cfg(target_os = "linux")]
    assert!(backends.contains(&BackendType::WebKitGTK));
}

#[test]
fn test_backend_factory_env_variable_name() {
    assert_eq!(BackendFactory::ENV_BACKEND, "AURORAVIEW_BACKEND");
}

#[test]
fn test_backend_factory_create_wry() {
    let config = BackendConfig::default();
    let result = BackendFactory::create(&config);
    assert!(result.is_ok());
}

#[test]
fn test_backend_config_debug() {
    let config = BackendConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("BackendConfig"));
    assert!(debug_str.contains("Wry"));
    assert!(debug_str.contains("AuroraView"));
}

// ============================================================================
// Traits Tests (from traits.rs)
// ============================================================================

#[test]
fn test_navigation_state_variants() {
    let states = [
        NavigationState::Started,
        NavigationState::InProgress,
        NavigationState::Completed,
        NavigationState::Failed,
    ];

    for state in &states {
        let debug_str = format!("{:?}", state);
        assert!(!debug_str.is_empty());
    }
}

#[test]
fn test_navigation_state_eq() {
    assert_eq!(NavigationState::Started, NavigationState::Started);
    assert_ne!(NavigationState::Started, NavigationState::Completed);
}

#[test]
fn test_navigation_state_clone() {
    let state = NavigationState::InProgress;
    let cloned = state;
    assert_eq!(state, cloned);
}

#[test]
fn test_navigation_event_creation() {
    let event = NavigationEvent {
        url: "https://example.com".to_string(),
        state: NavigationState::Started,
        error: None,
    };

    assert_eq!(event.url, "https://example.com");
    assert_eq!(event.state, NavigationState::Started);
    assert!(event.error.is_none());
}

#[test]
fn test_navigation_event_with_error() {
    let event = NavigationEvent {
        url: "https://invalid.example".to_string(),
        state: NavigationState::Failed,
        error: Some("Connection refused".to_string()),
    };

    assert_eq!(event.state, NavigationState::Failed);
    assert_eq!(event.error, Some("Connection refused".to_string()));
}

#[test]
fn test_navigation_event_clone() {
    let event = NavigationEvent {
        url: "https://example.com".to_string(),
        state: NavigationState::Completed,
        error: None,
    };

    let cloned = event.clone();
    assert_eq!(cloned.url, "https://example.com");
    assert_eq!(cloned.state, NavigationState::Completed);
}

#[test]
fn test_load_progress_creation() {
    let progress = LoadProgress {
        percent: 50,
        is_complete: false,
    };

    assert_eq!(progress.percent, 50);
    assert!(!progress.is_complete);
}

#[test]
fn test_load_progress_complete() {
    let progress = LoadProgress {
        percent: 100,
        is_complete: true,
    };

    assert_eq!(progress.percent, 100);
    assert!(progress.is_complete);
}

#[test]
fn test_load_progress_clone() {
    let progress = LoadProgress {
        percent: 75,
        is_complete: false,
    };

    let cloned = progress;
    assert_eq!(cloned.percent, 75);
}

#[test]
fn test_cookie_info_creation() {
    let cookie = CookieInfo {
        domain: "example.com".to_string(),
        name: "session".to_string(),
        value: "abc123".to_string(),
        path: Some("/".to_string()),
        expires: Some(1735689600),
        http_only: true,
        secure: true,
    };

    assert_eq!(cookie.domain, "example.com");
    assert_eq!(cookie.name, "session");
    assert_eq!(cookie.value, "abc123");
    assert_eq!(cookie.path, Some("/".to_string()));
    assert!(cookie.http_only);
    assert!(cookie.secure);
}

#[test]
fn test_cookie_info_minimal() {
    let cookie = CookieInfo {
        domain: "example.com".to_string(),
        name: "test".to_string(),
        value: "value".to_string(),
        path: None,
        expires: None,
        http_only: false,
        secure: false,
    };

    assert!(cookie.path.is_none());
    assert!(cookie.expires.is_none());
    assert!(!cookie.http_only);
    assert!(!cookie.secure);
}

#[test]
fn test_cookie_info_clone() {
    let cookie = CookieInfo {
        domain: "example.com".to_string(),
        name: "token".to_string(),
        value: "xyz".to_string(),
        path: Some("/api".to_string()),
        expires: Some(1735689600),
        http_only: true,
        secure: true,
    };

    let cloned = cookie.clone();
    assert_eq!(cloned.domain, "example.com");
    assert_eq!(cloned.name, "token");
    assert_eq!(cloned.path, Some("/api".to_string()));
}

#[test]
fn test_cookie_info_debug() {
    let cookie = CookieInfo {
        domain: "example.com".to_string(),
        name: "test".to_string(),
        value: "value".to_string(),
        path: None,
        expires: None,
        http_only: false,
        secure: false,
    };

    let debug_str = format!("{:?}", cookie);
    assert!(debug_str.contains("CookieInfo"));
    assert!(debug_str.contains("example.com"));
    assert!(debug_str.contains("test"));
}

// ============================================================================
// Settings Tests (from settings.rs)
// ============================================================================

#[test]
fn test_default_settings() {
    let settings = WebViewSettingsImpl::default();
    assert!(settings.local_storage_enabled());
    assert!(settings.javascript_enabled());
    assert!(!settings.dev_tools_enabled());
    assert!(!settings.allow_file_access());
}

#[test]
fn test_settings_mutation() {
    let mut settings = WebViewSettingsImpl::default();
    settings.set_dev_tools_enabled(true);
    assert!(settings.dev_tools_enabled());

    settings.set_user_agent(Some("CustomAgent/1.0".into()));
    assert_eq!(settings.user_agent(), Some("CustomAgent/1.0".into()));
}

// ============================================================================
// Error Tests (from error.rs)
// ============================================================================

#[test]
fn test_error_display() {
    let err = WebViewError::Navigation("failed to load".into());
    assert_eq!(err.to_string(), "Navigation error: failed to load");
}

#[test]
fn test_error_helpers() {
    let err = WebViewError::init("failed");
    assert!(matches!(err, WebViewError::Initialization(_)));

    let err = WebViewError::javascript("syntax error");
    assert!(matches!(err, WebViewError::JavaScript(_)));
}

// ============================================================================
// WryBackend Tests (from wry_impl.rs)
// ============================================================================

#[test]
fn test_wry_backend_creation() {
    use auroraview_core::backend::WebViewBackend;
    use auroraview_core::backend::WryBackend;

    let backend = WryBackend::new();
    assert!(backend.url().is_none());
    assert!(!backend.is_loading());
    assert!(!backend.is_closed());
}

#[test]
fn test_wry_backend_navigate() {
    use auroraview_core::backend::WebViewBackend;
    use auroraview_core::backend::WryBackend;

    let backend = WryBackend::new();
    backend.navigate("https://example.com").unwrap();
    assert_eq!(backend.url(), Some("https://example.com".to_string()));
    assert!(backend.is_loading());
}

#[test]
fn test_wry_backend_load_progress() {
    use auroraview_core::backend::WebViewBackend;
    use auroraview_core::backend::WryBackend;

    let backend = WryBackend::new();
    backend.set_load_progress(50);
    let progress = backend.load_progress();
    assert_eq!(progress.percent, 50);
}

#[test]
fn test_wry_backend_close() {
    use auroraview_core::backend::WebViewBackend;
    use auroraview_core::backend::WryBackend;

    let backend = WryBackend::new();
    assert!(!backend.is_closed());
    backend.close().unwrap();
    assert!(backend.is_closed());
}

#[test]
fn test_wry_backend_user_agent() {
    use auroraview_core::backend::WebViewBackend;
    use auroraview_core::backend::WryBackend;

    let backend = WryBackend::new();
    let ua = backend.http_user_agent();
    assert!(ua.starts_with("AuroraView/"));
}
