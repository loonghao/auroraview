//! Unit tests for clipboard plugin
//!
//! Tests for ClipboardPlugin commands.

use auroraview_plugins::clipboard::ClipboardPlugin;
use auroraview_plugins::{PluginHandler, ScopeConfig};

#[test]
fn test_clipboard_plugin_commands() {
    let plugin = ClipboardPlugin::new();
    let commands = plugin.commands();
    assert!(commands.contains(&"read_text"));
    assert!(commands.contains(&"write_text"));
    assert!(commands.contains(&"clear"));
    assert!(commands.contains(&"has_text"));
}

#[test]
fn test_clipboard_plugin_name() {
    let plugin = ClipboardPlugin::new();
    assert_eq!(plugin.name(), "clipboard");
}

#[test]
fn test_clipboard_plugin_default() {
    let plugin = ClipboardPlugin::default();
    assert_eq!(plugin.name(), "clipboard");
}

#[test]
fn test_command_not_found() {
    let plugin = ClipboardPlugin::new();
    let scope = ScopeConfig::new();

    let result = plugin.handle("unknown_command", serde_json::json!({}), &scope);
    assert!(result.is_err());
}

#[test]
fn test_write_text_invalid_args() {
    let plugin = ClipboardPlugin::new();
    let scope = ScopeConfig::new();

    let result = plugin.handle(
        "write_text",
        serde_json::json!({ "invalid": "args" }), // Missing required "text"
        &scope,
    );
    assert!(result.is_err());
}

// Note: Clipboard tests require a display server on Linux
// and may not work in headless CI environments
#[test]
#[ignore = "Requires display server"]
fn test_clipboard_write_read() {
    let plugin = ClipboardPlugin::new();
    let scope = ScopeConfig::new();

    // Write text
    let write_result = plugin.handle(
        "write_text",
        serde_json::json!({ "text": "Test clipboard content" }),
        &scope,
    );
    assert!(write_result.is_ok());

    // Read text
    let read_result = plugin.handle("read_text", serde_json::json!({}), &scope);
    assert!(read_result.is_ok());
    let data = read_result.unwrap();
    assert_eq!(data["text"], "Test clipboard content");
}

#[test]
#[ignore = "Requires display server"]
fn test_clipboard_has_text() {
    let plugin = ClipboardPlugin::new();
    let scope = ScopeConfig::new();

    // Write text first
    let _ = plugin.handle("write_text", serde_json::json!({ "text": "test" }), &scope);

    // Check has_text
    let result = plugin.handle("has_text", serde_json::json!({}), &scope);
    assert!(result.is_ok());
    let data = result.unwrap();
    assert!(data["hasText"].as_bool().unwrap());
}

#[test]
#[ignore = "Requires display server"]
fn test_clipboard_clear() {
    let plugin = ClipboardPlugin::new();
    let scope = ScopeConfig::new();

    // Write text first
    let _ = plugin.handle("write_text", serde_json::json!({ "text": "test" }), &scope);

    // Clear
    let result = plugin.handle("clear", serde_json::json!({}), &scope);
    assert!(result.is_ok());
}
