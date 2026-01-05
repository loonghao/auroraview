//! Integration tests for MCP Server

use auroraview_mcp::{McpConfig, McpServer, Tool};
use rstest::*;

#[rstest]
fn test_config_default() {
    let config = McpConfig::default();
    assert_eq!(config.host, "127.0.0.1");
    assert_eq!(config.port, 0);
    assert!(config.auto_expose_api);
    assert!(config.expose_dom);
    assert!(config.expose_debug);
}

#[rstest]
fn test_config_builder() {
    let config = McpConfig::new("test-server")
        .with_port(8765)
        .with_auth("secret-token")
        .without_dom()
        .without_debug();

    assert_eq!(config.name, "test-server");
    assert_eq!(config.port, 8765);
    assert!(config.require_auth);
    assert_eq!(config.auth_token, Some("secret-token".to_string()));
    assert!(!config.expose_dom);
    assert!(!config.expose_debug);
}

#[rstest]
fn test_tool_creation() {
    let tool = Tool::new("echo", "Echo back the input")
        .with_param("message", "string", "Message to echo")
        .with_optional_param("prefix", "string", "Optional prefix", None);

    assert_eq!(tool.name, "echo");
    assert_eq!(tool.description, "Echo back the input");
    assert_eq!(tool.params.len(), 2);
    assert!(tool.params[0].required);
    assert!(!tool.params[1].required);
}

#[rstest]
fn test_tool_definition() {
    let tool = Tool::new("test", "Test tool").with_param("input", "string", "Input value");

    let def = tool.to_definition();
    assert_eq!(def.name, "test");
    assert_eq!(def.description, "Test tool");

    let schema = def.input_schema;
    assert_eq!(schema["type"], "object");
    assert!(schema["properties"]["input"].is_object());
    assert_eq!(schema["required"][0], "input");
}

#[rstest]
fn test_tool_with_handler() {
    let tool = Tool::new("add", "Add two numbers")
        .with_param("a", "number", "First number")
        .with_param("b", "number", "Second number")
        .with_handler(|args| {
            let a = args["a"].as_f64().unwrap_or(0.0);
            let b = args["b"].as_f64().unwrap_or(0.0);
            Ok(serde_json::json!({ "result": a + b }))
        });

    assert!(tool.has_handler());

    let result = tool.call(serde_json::json!({ "a": 1, "b": 2 })).unwrap();
    assert_eq!(result["result"], 3.0);
}

#[rstest]
fn test_tool_registry() {
    let registry = auroraview_mcp::ToolRegistry::new();

    let tool1 = Tool::new("tool1", "First tool");
    let tool2 = Tool::new("tool2", "Second tool");

    registry.register(tool1);
    registry.register(tool2);

    assert_eq!(registry.len(), 2);
    assert!(registry.contains("tool1"));
    assert!(registry.contains("tool2"));
    assert!(!registry.contains("tool3"));

    let names = registry.list();
    assert!(names.contains(&"tool1".to_string()));
    assert!(names.contains(&"tool2".to_string()));
}

#[rstest]
fn test_server_creation() {
    let config = McpConfig::new("test");
    let server = McpServer::new(config);

    assert!(!server.is_running());
    assert_eq!(server.port(), 0);
}

#[tokio::test]
async fn test_server_start_stop() {
    let config = McpConfig::new("test-server");
    let server = McpServer::new(config);

    // Register a test tool
    server.register_tool(
        Tool::new("ping", "Ping test").with_handler(|_| Ok(serde_json::json!({ "pong": true }))),
    );

    // Start server
    let port = server.start().await.unwrap();
    assert!(port > 0);
    assert!(server.is_running());
    assert_eq!(server.port(), port);

    // Stop server
    server.stop().await;
    assert!(!server.is_running());
}

#[tokio::test]
async fn test_server_health_endpoint() {
    let config = McpConfig::new("health-test");
    let server = McpServer::new(config);

    let port = server.start().await.unwrap();

    // Test health endpoint
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://127.0.0.1:{}/health", port))
        .send()
        .await
        .unwrap();

    assert!(resp.status().is_success());

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok");
    assert_eq!(body["name"], "health-test");

    server.stop().await;
}

#[tokio::test]
async fn test_server_tools_endpoint() {
    let config = McpConfig::new("tools-test");
    let server = McpServer::new(config);

    server.register_tool(Tool::new("test_tool", "A test tool"));

    let port = server.start().await.unwrap();

    // Test tools endpoint
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://127.0.0.1:{}/tools", port))
        .send()
        .await
        .unwrap();

    assert!(resp.status().is_success());

    let body: serde_json::Value = resp.json().await.unwrap();
    let tools = body["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0]["name"], "test_tool");

    server.stop().await;
}
