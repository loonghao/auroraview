//! Unit tests for dialog plugin
//!
//! Tests for DialogPlugin commands and options.

use auroraview_plugins::dialog::{
    DialogPlugin, FileDialogOptions, FileFilter, MessageDialogOptions,
};
use auroraview_plugins::{PluginHandler, ScopeConfig};

#[test]
fn test_dialog_plugin_commands() {
    let plugin = DialogPlugin::new();
    let commands = plugin.commands();
    assert!(commands.contains(&"open_file"));
    assert!(commands.contains(&"open_files"));
    assert!(commands.contains(&"open_folder"));
    assert!(commands.contains(&"open_folders"));
    assert!(commands.contains(&"save_file"));
    assert!(commands.contains(&"message"));
    assert!(commands.contains(&"confirm"));
}

#[test]
fn test_dialog_plugin_name() {
    let plugin = DialogPlugin::new();
    assert_eq!(plugin.name(), "dialog");
}

#[test]
fn test_dialog_plugin_default() {
    let plugin = DialogPlugin::default();
    assert_eq!(plugin.name(), "dialog");
}

#[test]
fn test_command_not_found() {
    let plugin = DialogPlugin::new();
    let scope = ScopeConfig::new();

    let result = plugin.handle("unknown_command", serde_json::json!({}), &scope);
    assert!(result.is_err());
}

#[test]
fn test_file_filter_deserialization() {
    let json = serde_json::json!({
        "name": "Images",
        "extensions": ["png", "jpg", "gif"]
    });
    let filter: FileFilter = serde_json::from_value(json).unwrap();
    assert_eq!(filter.name, "Images");
    assert_eq!(filter.extensions, vec!["png", "jpg", "gif"]);
}

#[test]
fn test_file_dialog_options_deserialization() {
    let json = serde_json::json!({
        "title": "Select a file",
        "defaultPath": "/home/user",
        "filters": [
            { "name": "Text", "extensions": ["txt"] },
            { "name": "All", "extensions": ["*"] }
        ],
        "defaultName": "document.txt"
    });
    let opts: FileDialogOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.title, Some("Select a file".to_string()));
    assert_eq!(opts.default_path, Some("/home/user".to_string()));
    assert_eq!(opts.filters.len(), 2);
    assert_eq!(opts.default_name, Some("document.txt".to_string()));
}

#[test]
fn test_file_dialog_options_defaults() {
    let json = serde_json::json!({});
    let opts: FileDialogOptions = serde_json::from_value(json).unwrap();
    assert!(opts.title.is_none());
    assert!(opts.default_path.is_none());
    assert!(opts.filters.is_empty());
    assert!(opts.default_name.is_none());
}

#[test]
fn test_message_dialog_options_deserialization() {
    let json = serde_json::json!({
        "title": "Warning",
        "message": "Are you sure?",
        "level": "warning",
        "buttons": "yes_no"
    });
    let opts: MessageDialogOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.title, Some("Warning".to_string()));
    assert_eq!(opts.message, "Are you sure?");
    assert_eq!(opts.level, Some("warning".to_string()));
    assert_eq!(opts.buttons, Some("yes_no".to_string()));
}

#[test]
fn test_message_dialog_options_defaults() {
    let json = serde_json::json!({
        "message": "Hello"
    });
    let opts: MessageDialogOptions = serde_json::from_value(json).unwrap();
    assert!(opts.title.is_none());
    assert_eq!(opts.message, "Hello");
    assert!(opts.level.is_none());
    assert!(opts.buttons.is_none());
}

#[test]
fn test_message_invalid_args() {
    let plugin = DialogPlugin::new();
    let scope = ScopeConfig::new();

    let result = plugin.handle(
        "message",
        serde_json::json!({ "invalid": "args" }), // Missing required "message"
        &scope,
    );
    assert!(result.is_err());
}

#[test]
fn test_confirm_invalid_args() {
    let plugin = DialogPlugin::new();
    let scope = ScopeConfig::new();

    let result = plugin.handle(
        "confirm",
        serde_json::json!({ "invalid": "args" }), // Missing required "message"
        &scope,
    );
    assert!(result.is_err());
}

// Note: Dialog tests require user interaction and can't be automated
// These tests just verify the plugin structure and option parsing
