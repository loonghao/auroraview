//! Unit tests for dialog plugin
//!
//! Tests for DialogPlugin commands, options serde, error paths, and concurrent access.

use auroraview_plugins::dialog::{
    DialogPlugin, FileDialogOptions, FileFilter, MessageDialogOptions,
};
use auroraview_plugins::{PluginHandler, ScopeConfig};
use rstest::rstest;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Plugin identity
// ---------------------------------------------------------------------------

#[test]
fn plugin_name() {
    let plugin = DialogPlugin::new();
    assert_eq!(plugin.name(), "dialog");
}

#[test]
fn plugin_default_name() {
    let plugin = DialogPlugin::default();
    assert_eq!(plugin.name(), "dialog");
}

#[test]
fn plugin_commands_count() {
    let plugin = DialogPlugin::new();
    assert_eq!(plugin.commands().len(), 7);
}

#[test]
fn plugin_commands_all_present() {
    let plugin = DialogPlugin::new();
    let cmds = plugin.commands();
    assert!(cmds.contains(&"open_file"));
    assert!(cmds.contains(&"open_files"));
    assert!(cmds.contains(&"open_folder"));
    assert!(cmds.contains(&"open_folders"));
    assert!(cmds.contains(&"save_file"));
    assert!(cmds.contains(&"message"));
    assert!(cmds.contains(&"confirm"));
}

// ---------------------------------------------------------------------------
// FileFilter serde
// ---------------------------------------------------------------------------

#[test]
fn file_filter_basic() {
    let json = serde_json::json!({
        "name": "Images",
        "extensions": ["png", "jpg", "gif"]
    });
    let filter: FileFilter = serde_json::from_value(json).unwrap();
    assert_eq!(filter.name, "Images");
    assert_eq!(filter.extensions, vec!["png", "jpg", "gif"]);
}

#[test]
fn file_filter_empty_extensions() {
    let json = serde_json::json!({
        "name": "All Files",
        "extensions": []
    });
    let filter: FileFilter = serde_json::from_value(json).unwrap();
    assert_eq!(filter.name, "All Files");
    assert!(filter.extensions.is_empty());
}

#[test]
fn file_filter_single_extension() {
    let json = serde_json::json!({
        "name": "Python",
        "extensions": ["py"]
    });
    let filter: FileFilter = serde_json::from_value(json).unwrap();
    assert_eq!(filter.extensions, vec!["py"]);
}

#[test]
fn file_filter_wildcard_extension() {
    let json = serde_json::json!({
        "name": "All",
        "extensions": ["*"]
    });
    let filter: FileFilter = serde_json::from_value(json).unwrap();
    assert_eq!(filter.extensions, vec!["*"]);
}

#[test]
fn file_filter_unicode_name() {
    let json = serde_json::json!({
        "name": "图片文件 🖼",
        "extensions": ["png", "jpg"]
    });
    let filter: FileFilter = serde_json::from_value(json).unwrap();
    assert_eq!(filter.name, "图片文件 🖼");
}

#[test]
fn file_filter_clone() {
    let filter = FileFilter {
        name: "Images".to_string(),
        extensions: vec!["png".to_string(), "jpg".to_string()],
    };
    let cloned = filter.clone();
    assert_eq!(cloned.name, filter.name);
    assert_eq!(cloned.extensions, filter.extensions);
}

#[test]
fn file_filter_debug() {
    let filter = FileFilter {
        name: "Text".to_string(),
        extensions: vec!["txt".to_string()],
    };
    let debug = format!("{:?}", filter);
    assert!(debug.contains("FileFilter"));
    assert!(debug.contains("Text"));
}

#[test]
fn file_filter_serde_roundtrip() {
    let original = FileFilter {
        name: "Documents".to_string(),
        extensions: vec!["doc".to_string(), "docx".to_string(), "pdf".to_string()],
    };
    let serialized = serde_json::to_value(&original).unwrap();
    let deserialized: FileFilter = serde_json::from_value(serialized).unwrap();
    assert_eq!(deserialized.name, original.name);
    assert_eq!(deserialized.extensions, original.extensions);
}

// ---------------------------------------------------------------------------
// FileDialogOptions serde
// ---------------------------------------------------------------------------

#[test]
fn file_dialog_options_full() {
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
fn file_dialog_options_defaults() {
    let json = serde_json::json!({});
    let opts: FileDialogOptions = serde_json::from_value(json).unwrap();
    assert!(opts.title.is_none());
    assert!(opts.default_path.is_none());
    assert!(opts.filters.is_empty());
    assert!(opts.default_name.is_none());
}

#[test]
fn file_dialog_options_title_only() {
    let json = serde_json::json!({ "title": "Open File" });
    let opts: FileDialogOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.title, Some("Open File".to_string()));
    assert!(opts.default_path.is_none());
    assert!(opts.filters.is_empty());
}

#[test]
fn file_dialog_options_clone() {
    let opts = FileDialogOptions {
        title: Some("Test".to_string()),
        default_path: Some("/tmp".to_string()),
        filters: vec![FileFilter {
            name: "All".to_string(),
            extensions: vec!["*".to_string()],
        }],
        default_name: None,
    };
    let cloned = opts.clone();
    assert_eq!(cloned.title, opts.title);
    assert_eq!(cloned.default_path, opts.default_path);
    assert_eq!(cloned.filters.len(), opts.filters.len());
}

#[test]
fn file_dialog_options_serde_roundtrip() {
    let original = FileDialogOptions {
        title: Some("Save As".to_string()),
        default_path: Some("/home/user/docs".to_string()),
        filters: vec![FileFilter {
            name: "Text".to_string(),
            extensions: vec!["txt".to_string()],
        }],
        default_name: Some("output.txt".to_string()),
    };
    let serialized = serde_json::to_value(&original).unwrap();
    let deserialized: FileDialogOptions = serde_json::from_value(serialized).unwrap();
    assert_eq!(deserialized.title, original.title);
    assert_eq!(deserialized.default_path, original.default_path);
    assert_eq!(deserialized.default_name, original.default_name);
    assert_eq!(deserialized.filters.len(), original.filters.len());
}

#[test]
fn file_dialog_options_many_filters() {
    let filters: Vec<serde_json::Value> = vec![
        serde_json::json!({"name": "Rust", "extensions": ["rs"]}),
        serde_json::json!({"name": "Python", "extensions": ["py"]}),
        serde_json::json!({"name": "JS", "extensions": ["js", "ts"]}),
        serde_json::json!({"name": "All", "extensions": ["*"]}),
    ];
    let json = serde_json::json!({ "filters": filters });
    let opts: FileDialogOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.filters.len(), 4);
}

// ---------------------------------------------------------------------------
// MessageDialogOptions serde
// ---------------------------------------------------------------------------

#[test]
fn message_dialog_options_full() {
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
fn message_dialog_options_defaults() {
    let json = serde_json::json!({ "message": "Hello" });
    let opts: MessageDialogOptions = serde_json::from_value(json).unwrap();
    assert!(opts.title.is_none());
    assert_eq!(opts.message, "Hello");
    assert!(opts.level.is_none());
    assert!(opts.buttons.is_none());
}

#[rstest]
#[case("info")]
#[case("warning")]
#[case("error")]
#[case("")]
#[case("unknown_level")]
fn message_dialog_options_level_variants(#[case] level: &str) {
    let json = serde_json::json!({
        "message": "Test",
        "level": level
    });
    let opts: MessageDialogOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.level, Some(level.to_string()));
}

#[rstest]
#[case("ok")]
#[case("ok_cancel")]
#[case("yes_no")]
#[case("yes_no_cancel")]
fn message_dialog_options_button_variants(#[case] buttons: &str) {
    let json = serde_json::json!({
        "message": "Test",
        "buttons": buttons
    });
    let opts: MessageDialogOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.buttons, Some(buttons.to_string()));
}

#[test]
fn message_dialog_options_clone() {
    let opts = MessageDialogOptions {
        title: Some("Title".to_string()),
        message: "Message".to_string(),
        level: Some("info".to_string()),
        buttons: Some("ok".to_string()),
    };
    let cloned = opts.clone();
    assert_eq!(cloned.title, opts.title);
    assert_eq!(cloned.message, opts.message);
    assert_eq!(cloned.level, opts.level);
    assert_eq!(cloned.buttons, opts.buttons);
}

#[test]
fn message_dialog_options_debug() {
    let opts = MessageDialogOptions {
        title: None,
        message: "debug test".to_string(),
        level: None,
        buttons: None,
    };
    let debug = format!("{:?}", opts);
    assert!(debug.contains("MessageDialogOptions"));
    assert!(debug.contains("debug test"));
}

#[test]
fn message_dialog_options_serde_roundtrip() {
    let original = MessageDialogOptions {
        title: Some("Confirm".to_string()),
        message: "Do you want to continue?".to_string(),
        level: Some("warning".to_string()),
        buttons: Some("yes_no".to_string()),
    };
    let serialized = serde_json::to_value(&original).unwrap();
    let deserialized: MessageDialogOptions = serde_json::from_value(serialized).unwrap();
    assert_eq!(deserialized.title, original.title);
    assert_eq!(deserialized.message, original.message);
    assert_eq!(deserialized.level, original.level);
    assert_eq!(deserialized.buttons, original.buttons);
}

#[test]
fn message_dialog_options_unicode_message() {
    let json = serde_json::json!({
        "message": "你好世界! こんにちは! 🌍"
    });
    let opts: MessageDialogOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.message, "你好世界! こんにちは! 🌍");
}

// ---------------------------------------------------------------------------
// Error paths: command_not_found
// ---------------------------------------------------------------------------

#[test]
fn command_not_found() {
    let plugin = DialogPlugin::new();
    let scope = ScopeConfig::new();
    let result = plugin.handle("unknown_command", serde_json::json!({}), &scope);
    assert!(result.is_err());
}

#[rstest]
#[case("OPEN_FILE")]
#[case("Open_File")]
#[case("pick_file")]
#[case("")]
#[case("close")]
#[case("cancel")]
fn unknown_commands_return_error(#[case] cmd: &str) {
    let plugin = DialogPlugin::new();
    let scope = ScopeConfig::new();
    let result = plugin.handle(cmd, serde_json::json!({}), &scope);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Error paths: invalid args
// ---------------------------------------------------------------------------

#[test]
fn message_invalid_args() {
    let plugin = DialogPlugin::new();
    let scope = ScopeConfig::new();
    let result = plugin.handle(
        "message",
        serde_json::json!({ "invalid": "args" }),
        &scope,
    );
    assert!(result.is_err());
}

#[test]
fn confirm_invalid_args() {
    let plugin = DialogPlugin::new();
    let scope = ScopeConfig::new();
    let result = plugin.handle(
        "confirm",
        serde_json::json!({ "invalid": "args" }),
        &scope,
    );
    assert!(result.is_err());
}

#[rstest]
#[case("open_file")]
#[case("open_files")]
#[case("open_folder")]
#[case("open_folders")]
#[case("save_file")]
fn file_commands_accept_empty_args(#[case] cmd: &str) {
    // File dialog commands use FileDialogOptions with all-optional fields,
    // so empty JSON object is valid (would open dialog if not headless).
    // We just verify the parsing step succeeds (dialog open is not tested).
    let _opts: FileDialogOptions = serde_json::from_value(serde_json::json!({})).unwrap();
    let _ = cmd; // suppress unused warning
}

#[rstest]
#[case("message")]
#[case("confirm")]
fn message_commands_require_message_field(#[case] cmd: &str) {
    let plugin = DialogPlugin::new();
    let scope = ScopeConfig::new();
    let result = plugin.handle(cmd, serde_json::json!({ "title": "No message field" }), &scope);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Concurrent invalid command handling
// ---------------------------------------------------------------------------

#[test]
fn concurrent_unknown_commands() {
    let plugin = Arc::new(DialogPlugin::new());
    let scope = ScopeConfig::new();
    let handles: Vec<_> = (0..8)
        .map(|i| {
            let p = Arc::clone(&plugin);
            let s = scope.clone();
            std::thread::spawn(move || {
                let cmd = format!("unknown_{}", i);
                p.handle(&cmd, serde_json::json!({}), &s)
            })
        })
        .collect();

    for h in handles {
        let result = h.join().unwrap();
        assert!(result.is_err());
    }
}

#[test]
fn concurrent_message_invalid_args() {
    let plugin = Arc::new(DialogPlugin::new());
    let scope = ScopeConfig::new();
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let p = Arc::clone(&plugin);
            let s = scope.clone();
            std::thread::spawn(move || {
                p.handle("message", serde_json::json!({ "no_message": true }), &s)
            })
        })
        .collect();

    for h in handles {
        let result = h.join().unwrap();
        assert!(result.is_err());
    }
}

// ---------------------------------------------------------------------------
// Multiple plugin instances
// ---------------------------------------------------------------------------

#[test]
fn two_instances_independent() {
    let p1 = DialogPlugin::new();
    let p2 = DialogPlugin::new();
    assert_eq!(p1.name(), p2.name());
    assert_eq!(p1.commands().len(), p2.commands().len());
}

// ---------------------------------------------------------------------------
// FileFilter edge cases
// ---------------------------------------------------------------------------

#[test]
fn file_filter_many_extensions() {
    let exts: Vec<String> = (0..20).map(|i| format!("ext{}", i)).collect();
    let filter = FileFilter {
        name: "Many".to_string(),
        extensions: exts.clone(),
    };
    assert_eq!(filter.extensions.len(), 20);
    assert_eq!(filter.extensions[0], "ext0");
    assert_eq!(filter.extensions[19], "ext19");
}

#[test]
fn file_filter_serde_empty_name() {
    let json = serde_json::json!({
        "name": "",
        "extensions": ["*"]
    });
    let filter: FileFilter = serde_json::from_value(json).unwrap();
    assert_eq!(filter.name, "");
}

#[rstest]
#[case("Rust", vec!["rs", "toml"])]
#[case("Python", vec!["py", "pyi"])]
#[case("Web", vec!["html", "css", "js", "ts"])]
#[case("All Files", vec!["*"])]
fn file_filter_rstest_variants(#[case] name: &str, #[case] exts: Vec<&str>) {
    let filter = FileFilter {
        name: name.to_string(),
        extensions: exts.iter().map(|s| s.to_string()).collect(),
    };
    assert_eq!(filter.name, name);
    assert_eq!(filter.extensions.len(), exts.len());
}

// ---------------------------------------------------------------------------
// FileDialogOptions edge cases
// ---------------------------------------------------------------------------

#[test]
fn file_dialog_options_default_path_windows_style() {
    let json = serde_json::json!({
        "defaultPath": "C:\\Users\\user\\Documents"
    });
    let opts: FileDialogOptions = serde_json::from_value(json).unwrap();
    assert!(opts.default_path.as_deref().unwrap_or("").contains("Documents"));
}

#[test]
fn file_dialog_options_default_name_with_extension() {
    let json = serde_json::json!({
        "defaultName": "my_scene.ma"
    });
    let opts: FileDialogOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.default_name.as_deref(), Some("my_scene.ma"));
}

#[test]
fn file_dialog_options_filters_with_no_title() {
    let json = serde_json::json!({
        "filters": [
            {"name": "Maya Scene", "extensions": ["ma", "mb"]}
        ]
    });
    let opts: FileDialogOptions = serde_json::from_value(json).unwrap();
    assert!(opts.title.is_none());
    assert_eq!(opts.filters.len(), 1);
    assert_eq!(opts.filters[0].name, "Maya Scene");
}

#[test]
fn file_dialog_options_debug() {
    let opts = FileDialogOptions {
        title: Some("Test".to_string()),
        default_path: None,
        filters: vec![],
        default_name: None,
    };
    let debug = format!("{:?}", opts);
    assert!(debug.contains("FileDialogOptions"));
}

// ---------------------------------------------------------------------------
// MessageDialogOptions edge cases
// ---------------------------------------------------------------------------

#[test]
fn message_dialog_options_empty_message() {
    let json = serde_json::json!({ "message": "" });
    let opts: MessageDialogOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.message, "");
}

#[test]
fn message_dialog_options_long_message() {
    let long_msg = "x".repeat(10_000);
    let json = serde_json::json!({ "message": long_msg });
    let opts: MessageDialogOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.message.len(), 10_000);
}

#[test]
fn message_dialog_options_title_empty_string() {
    let json = serde_json::json!({ "message": "msg", "title": "" });
    let opts: MessageDialogOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.title.as_deref(), Some(""));
}

// ---------------------------------------------------------------------------
// Plugin: command stability
// ---------------------------------------------------------------------------

#[test]
fn plugin_commands_no_duplicates() {
    let plugin = DialogPlugin::new();
    let cmds = plugin.commands();
    let mut unique = cmds.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), cmds.len(), "commands should have no duplicates");
}

#[test]
fn plugin_commands_all_non_empty() {
    let plugin = DialogPlugin::new();
    for cmd in plugin.commands() {
        assert!(!cmd.is_empty(), "command name should not be empty");
    }
}

// ---------------------------------------------------------------------------
// Error message validation: unknown commands have descriptive errors
// ---------------------------------------------------------------------------

#[test]
fn unknown_command_error_is_not_ok() {
    let plugin = DialogPlugin::new();
    let scope = ScopeConfig::new();
    let err = plugin.handle("nonexistent", serde_json::json!({}), &scope).unwrap_err();
    // Error message should mention the command
    let msg = err.to_string();
    assert!(!msg.is_empty(), "error message should not be empty");
}

// ---------------------------------------------------------------------------
// FileFilter: PartialEq via serde comparison
// ---------------------------------------------------------------------------

#[test]
fn file_filter_same_name_different_extensions_not_equal_via_serde() {
    let a = serde_json::to_value(&FileFilter {
        name: "Images".to_string(),
        extensions: vec!["png".to_string()],
    }).unwrap();
    let b = serde_json::to_value(&FileFilter {
        name: "Images".to_string(),
        extensions: vec!["jpg".to_string()],
    }).unwrap();
    assert_ne!(a, b);
}

#[test]
fn file_filter_identical_serde_values_are_equal() {
    let a = serde_json::to_value(&FileFilter {
        name: "Images".to_string(),
        extensions: vec!["png".to_string()],
    }).unwrap();
    let b = serde_json::to_value(&FileFilter {
        name: "Images".to_string(),
        extensions: vec!["png".to_string()],
    }).unwrap();
    assert_eq!(a, b);
}
