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

// ---------------------------------------------------------------------------
// UeBlueprintNode tests
// ---------------------------------------------------------------------------

#[test]
fn blueprint_node_creation() {
    let node = auroraview_ue::UeBlueprintNode::new("node_1", "Print String");
    assert_eq!(node.id, "node_1");
    assert_eq!(node.title, "Print String");
    assert!(node.inputs.is_empty());
    assert!(node.outputs.is_empty());
    assert!(node.connections.is_empty());
}

#[test]
fn blueprint_node_add_pins() {
    let mut node = auroraview_ue::UeBlueprintNode::new("node_2", "Add Numbers");
    node.add_input("A", "float");
    node.add_input("B", "float");
    node.add_output("Result", "float");
    
    assert_eq!(node.inputs.len(), 2);
    assert_eq!(node.outputs.len(), 1);
    assert_eq!(node.inputs[0], ("A".to_string(), "float".to_string()));
    assert_eq!(node.outputs[0], ("Result".to_string(), "float".to_string()));
}

#[test]
fn blueprint_node_connect() {
    let mut node1 = auroraview_ue::UeBlueprintNode::new("node_1", "Output");

    node1.connect_to("node_2");
    assert_eq!(node1.connections.len(), 1);
    assert_eq!(node1.connections[0], "node_2");
}

#[test]
fn blueprint_node_to_json() {
    let mut node = auroraview_ue::UeBlueprintNode::new("node_1", "Test Node");
    node.add_input("Input1", "string");
    node.add_output("Output1", "bool");
    node.connect_to("node_2");
    
    let json = node.to_json();
    assert_eq!(json["id"], "node_1");
    assert_eq!(json["title"], "Test Node");
    assert_eq!(json["inputs"][0][0], "Input1");
    assert_eq!(json["outputs"][0][0], "Output1");
    assert_eq!(json["connections"][0], "node_2");
}

#[test]
fn blueprint_error_display() {
    let err1 = auroraview_ue::UeBlueprintError::NodeNotFound("node_99".into());
    assert!(err1.to_string().contains("node_99"));
    
    let err2 = auroraview_ue::UeBlueprintError::InvalidPinType("void".into());
    assert!(err2.to_string().contains("void"));
    
    let err3 = auroraview_ue::UeBlueprintError::CompilationFailed("syntax error".into());
    assert!(err3.to_string().contains("syntax error"));
}

#[test]
fn blueprint_node_remove_input() {
    let mut node = auroraview_ue::UeBlueprintNode::new("node_1", "Test Node");
    node.add_input("Input1", "string");
    node.add_input("Input2", "float");
    assert_eq!(node.inputs.len(), 2);
    
    node.remove_input("Input1");
    assert_eq!(node.inputs.len(), 1);
    assert_eq!(node.inputs[0], ("Input2".to_string(), "float".to_string()));
}

#[test]
fn blueprint_node_remove_output() {
    let mut node = auroraview_ue::UeBlueprintNode::new("node_2", "Test Node");
    node.add_output("Output1", "string");
    node.add_output("Output2", "bool");
    assert_eq!(node.outputs.len(), 2);
    
    node.remove_output("Output1");
    assert_eq!(node.outputs.len(), 1);
    assert_eq!(node.outputs[0], ("Output2".to_string(), "bool".to_string()));
}

#[test]
fn blueprint_node_remove_connection() {
    let mut node1 = auroraview_ue::UeBlueprintNode::new("node_1", "Source");
    node1.connect_to("node_2");
    assert_eq!(node1.connections.len(), 1);

    node1.remove_connection("node_2");
    assert!(node1.connections.is_empty());
}

#[test]
fn blueprint_node_clear() {
    let mut node = auroraview_ue::UeBlueprintNode::new("node_3", "Clear Test");
    node.add_input("A", "int");
    node.add_input("B", "float");
    node.add_output("C", "string");
    node.connect_to("node_x");
    node.connect_to("node_y");
    
    assert!(!node.inputs.is_empty());
    assert!(!node.outputs.is_empty());
    assert!(!node.connections.is_empty());
    
    node.clear_inputs();
    node.clear_outputs();
    node.clear_connections();
    
    assert!(node.inputs.is_empty());
    assert!(node.outputs.is_empty());
    assert!(node.connections.is_empty());
}
