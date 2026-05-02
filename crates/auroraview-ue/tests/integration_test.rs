//! Integration tests for `auroraview-ue`.
//!
//! These tests verify the UE integration types and utilities.

use auroraview_ue::{
    GameThreadId, SlateWidgetHandle, UeEmbedMode, UeGameThreadExecutor, UeIntegration,
    UeWebViewConfig,
};

// ---------------------------------------------------------------------------
// GameThreadId tests
// ---------------------------------------------------------------------------

#[test]
fn game_thread_id_current() {
    let id = GameThreadId::current();
    assert!(id.is_current());
}

#[test]
fn game_thread_id_is_current() {
    let id = GameThreadId::current();
    // We're on the main thread, so is_current() should return true
    assert!(id.is_current());
}

// ---------------------------------------------------------------------------
// UeGameThreadExecutor tests
// ---------------------------------------------------------------------------

#[test]
fn executor_creation() {
    let (_executor, _rx) = UeGameThreadExecutor::new();
    // If we reach here, creation succeeded
}

#[test]
fn executor_is_game_thread() {
    let (executor, _rx) = UeGameThreadExecutor::new();
    // We're on the main thread, which is the "GameThread" in test
    assert!(executor.is_game_thread());
}

#[test]
fn executor_execute_on_game_thread() {
    let (executor, _rx) = UeGameThreadExecutor::new();
    let result = std::sync::Arc::new(std::sync::Mutex::new(None));
    let result_clone = result.clone();

    executor.execute(move || {
        let mut r = result_clone.lock().unwrap();
        *r = Some(42);
    });

    // Since we're on GameThread, it should execute immediately
    let r = result.lock().unwrap();
    assert_eq!(*r, Some(42));
}

// ---------------------------------------------------------------------------
// SlateWidgetHandle tests
// ---------------------------------------------------------------------------

#[test]
fn slate_widget_handle_null() {
    let handle = SlateWidgetHandle::null();
    assert!(handle.is_null());
}

#[test]
fn slate_widget_handle_non_null() {
    let handle = SlateWidgetHandle(123);
    assert!(!handle.is_null());
}

#[test]
fn slate_widget_handle_partial_eq() {
    let h1 = SlateWidgetHandle(123);
    let h2 = SlateWidgetHandle(123);
    let h3 = SlateWidgetHandle(456);

    assert_eq!(h1, h2);
    assert_ne!(h1, h3);
}

// ---------------------------------------------------------------------------
// UeEmbedMode tests
// ---------------------------------------------------------------------------

#[test]
fn embed_mode_variants() {
    let modes = vec![
        UeEmbedMode::SlateWidget,
        UeEmbedMode::NativeChildWindow,
        UeEmbedMode::FloatingWindow,
    ];

    // Just verify they can be constructed
    for mode in &modes {
        let _ = *mode; // Copy
    }
}

// ---------------------------------------------------------------------------
// UeWebViewConfig tests
// ---------------------------------------------------------------------------

#[test]
fn webview_config_default() {
    let cfg = UeWebViewConfig::default();
    assert_eq!(cfg.initial_size, (800, 600));
    assert_eq!(cfg.embed_mode, UeEmbedMode::SlateWidget);
    assert!(!cfg.dev_tools);
    assert!(cfg.init_script.is_none());
}

#[test]
fn webview_config_custom() {
    let cfg = UeWebViewConfig {
        initial_size: (1024, 768),
        embed_mode: UeEmbedMode::NativeChildWindow,
        dev_tools: true,
        init_script: Some("console.log('hello')".to_string()),
    };

    assert_eq!(cfg.initial_size, (1024, 768));
    assert_eq!(cfg.embed_mode, UeEmbedMode::NativeChildWindow);
    assert!(cfg.dev_tools);
    assert_eq!(cfg.init_script, Some("console.log('hello')".to_string()));
}

// ---------------------------------------------------------------------------
// UeIntegration tests
// ---------------------------------------------------------------------------

#[test]
fn integration_creation_default() {
    let _integration = UeIntegration::new(UeWebViewConfig::default());
    // If we reach here, creation succeeded
}

#[test]
fn integration_creation_custom() {
    let cfg = UeWebViewConfig {
        initial_size: (1024, 768),
        embed_mode: UeEmbedMode::FloatingWindow,
        dev_tools: true,
        init_script: None,
    };
    let _integration = UeIntegration::new(cfg);
}

#[test]
fn integration_executor() {
    let integration = UeIntegration::new(UeWebViewConfig::default());
    let _executor = integration.executor();
    // If we reach here, executor() succeeded
}

#[test]
fn integration_process_tasks_empty() {
    let integration = UeIntegration::new(UeWebViewConfig::default());
    // Should not panic even with no tasks
    integration.process_tasks();
}

#[test]
fn integration_set_parent_widget() {
    let mut integration = UeIntegration::new(UeWebViewConfig::default());
    let handle = SlateWidgetHandle(123);
    integration.set_parent_widget(handle);
    // If we reach here, set_parent_widget() succeeded
}

#[test]
fn integration_create_webview_on_game_thread() {
    let integration = UeIntegration::new(UeWebViewConfig::default());
    // We're on the "GameThread" in test, so this should work
    let result = integration.create_webview("https://example.com");
    assert!(result.is_ok());
    // Currently returns SlateWidgetHandle::null() (placeholder)
    let handle = result.unwrap();
    assert!(handle.is_null()); // Placeholder implementation
}

// ---------------------------------------------------------------------------
// UeError tests
// ---------------------------------------------------------------------------

#[test]
fn ue_error_display() {
    let err1 = auroraview_ue::UeError::NotOnGameThread;
    assert_eq!(err1.to_string(), "operation must be on GameThread");

    let err2 = auroraview_ue::UeError::InvalidHandle;
    assert_eq!(err2.to_string(), "invalid Slate widget handle");

    let err3 = auroraview_ue::UeError::WebViewCreationFailed("test".into());
    assert!(err3.to_string().contains("test"));

    let err4 = auroraview_ue::UeError::ObjectCollected;
    assert_eq!(err4.to_string(), "UE object was garbage collected");
}
