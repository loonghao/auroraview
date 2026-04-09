//! Unit tests for clipboard plugin
//!
//! Tests for ClipboardPlugin commands, WriteTextOptions serde, error paths, and concurrent access.

use auroraview_plugins::clipboard::{ClipboardPlugin, WriteTextOptions};
use auroraview_plugins::{PluginHandler, ScopeConfig};
use rstest::rstest;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Plugin identity
// ---------------------------------------------------------------------------

#[test]
fn plugin_name() {
    let plugin = ClipboardPlugin::new();
    assert_eq!(plugin.name(), "clipboard");
}

#[test]
fn plugin_default_name() {
    let plugin = ClipboardPlugin::default();
    assert_eq!(plugin.name(), "clipboard");
}

#[test]
fn plugin_commands_count() {
    let plugin = ClipboardPlugin::new();
    assert_eq!(plugin.commands().len(), 4);
}

#[test]
fn plugin_commands_all_present() {
    let plugin = ClipboardPlugin::new();
    let cmds = plugin.commands();
    assert!(cmds.contains(&"read_text"));
    assert!(cmds.contains(&"write_text"));
    assert!(cmds.contains(&"clear"));
    assert!(cmds.contains(&"has_text"));
}

// ---------------------------------------------------------------------------
// WriteTextOptions serde
// ---------------------------------------------------------------------------

#[test]
fn write_text_options_basic() {
    let json = serde_json::json!({ "text": "hello world" });
    let opts: WriteTextOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.text, "hello world");
}

#[test]
fn write_text_options_empty_string() {
    let json = serde_json::json!({ "text": "" });
    let opts: WriteTextOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.text, "");
}

#[test]
fn write_text_options_unicode() {
    let text = "Unicode: 你好世界 🌍 αβγδ ñoño";
    let json = serde_json::json!({ "text": text });
    let opts: WriteTextOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.text, text);
}

#[test]
fn write_text_options_long_string() {
    let text = "x".repeat(65536);
    let json = serde_json::json!({ "text": text });
    let opts: WriteTextOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.text.len(), 65536);
}

#[test]
fn write_text_options_whitespace_only() {
    let json = serde_json::json!({ "text": "   \t\n   " });
    let opts: WriteTextOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.text, "   \t\n   ");
}

#[test]
fn write_text_options_newlines() {
    let text = "line1\nline2\r\nline3";
    let json = serde_json::json!({ "text": text });
    let opts: WriteTextOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.text, text);
}

#[test]
fn write_text_options_json_special_chars() {
    let text = r#"{"key": "value", "nested": {"a": 1}}"#;
    let json = serde_json::json!({ "text": text });
    let opts: WriteTextOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.text, text);
}

#[test]
fn write_text_options_serde_roundtrip() {
    let original = WriteTextOptions {
        text: "roundtrip test".to_string(),
    };
    let serialized = serde_json::to_value(&original).unwrap();
    let deserialized: WriteTextOptions = serde_json::from_value(serialized).unwrap();
    assert_eq!(deserialized.text, original.text);
}

#[test]
fn write_text_options_clone() {
    let opts = WriteTextOptions {
        text: "clone test".to_string(),
    };
    let cloned = opts.clone();
    assert_eq!(cloned.text, opts.text);
}

#[test]
fn write_text_options_debug() {
    let opts = WriteTextOptions {
        text: "debug".to_string(),
    };
    let debug = format!("{:?}", opts);
    assert!(debug.contains("WriteTextOptions"));
    assert!(debug.contains("debug"));
}

// ---------------------------------------------------------------------------
// Error paths: command_not_found
// ---------------------------------------------------------------------------

#[rstest]
#[case("unknown")]
#[case("READ_TEXT")]
#[case("WRITE_TEXT")]
#[case("")]
#[case("copy")]
#[case("paste")]
fn unknown_command_returns_error(#[case] cmd: &str) {
    let plugin = ClipboardPlugin::new();
    let scope = ScopeConfig::new();
    let result = plugin.handle(cmd, serde_json::json!({}), &scope);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.code().contains("COMMAND_NOT_FOUND") || !err.code().is_empty());
}

// ---------------------------------------------------------------------------
// Error paths: invalid_args for write_text
// ---------------------------------------------------------------------------

#[test]
fn write_text_missing_text_field() {
    let plugin = ClipboardPlugin::new();
    let scope = ScopeConfig::new();
    let result = plugin.handle("write_text", serde_json::json!({ "invalid": "args" }), &scope);
    assert!(result.is_err());
}

#[test]
fn write_text_null_args() {
    let plugin = ClipboardPlugin::new();
    let scope = ScopeConfig::new();
    let result = plugin.handle("write_text", serde_json::json!(null), &scope);
    assert!(result.is_err());
}

#[test]
fn write_text_wrong_type_for_text() {
    let plugin = ClipboardPlugin::new();
    let scope = ScopeConfig::new();
    let result = plugin.handle("write_text", serde_json::json!({ "text": 123 }), &scope);
    assert!(result.is_err());
}

#[test]
fn write_text_array_args() {
    let plugin = ClipboardPlugin::new();
    let scope = ScopeConfig::new();
    let result = plugin.handle("write_text", serde_json::json!(["text", "value"]), &scope);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Multiple plugin instances are independent
// ---------------------------------------------------------------------------

#[test]
fn two_plugin_instances_independent() {
    let p1 = ClipboardPlugin::new();
    let p2 = ClipboardPlugin::new();
    assert_eq!(p1.name(), p2.name());
    assert_eq!(p1.commands().len(), p2.commands().len());
}

// ---------------------------------------------------------------------------
// Concurrent invalid command handling (does not require display)
// ---------------------------------------------------------------------------

#[test]
fn concurrent_unknown_commands() {
    let plugin = Arc::new(ClipboardPlugin::new());
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
fn concurrent_write_text_invalid() {
    let plugin = Arc::new(ClipboardPlugin::new());
    let scope = ScopeConfig::new();
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let p = Arc::clone(&plugin);
            let s = scope.clone();
            std::thread::spawn(move || {
                p.handle("write_text", serde_json::json!({ "bad": "arg" }), &s)
            })
        })
        .collect();

    for h in handles {
        let result = h.join().unwrap();
        assert!(result.is_err());
    }
}


// ---------------------------------------------------------------------------
// R11 Extensions
// ---------------------------------------------------------------------------

#[test]
fn clipboard_plugin_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ClipboardPlugin>();
}

#[test]
fn write_text_options_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<WriteTextOptions>();
}

#[rstest]
#[case("has_text")]
#[case("clear")]
#[case("read_text")]
#[case("write_text")]
fn each_valid_command_is_in_commands_list(#[case] cmd: &str) {
    let plugin = ClipboardPlugin::new();
    assert!(plugin.commands().contains(&cmd), "{} should be in commands", cmd);
}

#[rstest]
#[case(" ")]
#[case("Read_Text")]
#[case("CLEAR")]
#[case("HAS_TEXT")]
#[case("clipboard.read")]
fn case_sensitive_command_lookup_fails(#[case] cmd: &str) {
    let plugin = ClipboardPlugin::new();
    let scope = ScopeConfig::new();
    let result = plugin.handle(cmd, serde_json::json!({}), &scope);
    assert!(result.is_err(), "Command '{}' should fail case-sensitive check", cmd);
}

#[test]
fn plugin_commands_count_is_four() {
    let plugin = ClipboardPlugin::new();
    assert_eq!(plugin.commands().len(), 4);
}

#[test]
fn plugin_name_is_stable_across_calls() {
    let plugin = ClipboardPlugin::new();
    assert_eq!(plugin.name(), plugin.name());
}

#[test]
fn write_text_options_field_is_named_text() {
    let json = serde_json::json!({ "text": "check field name" });
    let opts: WriteTextOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.text, "check field name");
}

#[test]
fn write_text_options_backslash_path() {
    let text = r"C:\Users\test\Documents\file.txt";
    let json = serde_json::json!({ "text": text });
    let opts: WriteTextOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.text, text);
}

#[test]
fn write_text_options_null_bytes() {
    // Null bytes in text (JSON string)
    let text = "before\0after";
    let json = serde_json::json!({ "text": text });
    let opts: WriteTextOptions = serde_json::from_value(json).unwrap();
    assert!(opts.text.contains("before"));
}

#[test]
fn write_text_options_only_spaces() {
    let json = serde_json::json!({ "text": "     " });
    let opts: WriteTextOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.text.len(), 5);
}

#[test]
fn unknown_command_error_code_not_empty() {
    let plugin = ClipboardPlugin::new();
    let scope = ScopeConfig::new();
    let err = plugin.handle("nope", serde_json::json!({}), &scope).unwrap_err();
    assert!(!err.code().is_empty());
}

#[test]
fn plugin_default_equals_new() {
    let p1 = ClipboardPlugin::new();
    let p2 = ClipboardPlugin::default();
    assert_eq!(p1.name(), p2.name());
    assert_eq!(p1.commands(), p2.commands());
}

#[rstest]
#[case(0)]
#[case(1)]
#[case(100)]
#[case(1000)]
#[case(10000)]
fn write_text_options_various_lengths(#[case] len: usize) {
    let text = "a".repeat(len);
    let json = serde_json::json!({ "text": text });
    let opts: WriteTextOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.text.len(), len);
}

#[test]
fn write_text_options_emoji_text() {
    let text = "🦀🦁🦄🌈💎";
    let json = serde_json::json!({ "text": text });
    let opts: WriteTextOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.text, text);
}

#[test]
fn write_text_options_arabic_text() {
    let text = "مرحبا بالعالم";
    let json = serde_json::json!({ "text": text });
    let opts: WriteTextOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.text, text);
}

#[test]
fn unknown_command_error_is_source_diagnostic() {
    let plugin = ClipboardPlugin::new();
    let scope = ScopeConfig::new();
    // Verify that error from unknown command is deterministic
    let err1 = plugin.handle("bad_cmd", serde_json::json!({}), &scope).unwrap_err();
    let err2 = plugin.handle("bad_cmd", serde_json::json!({}), &scope).unwrap_err();
    assert_eq!(err1.code(), err2.code());
}

#[test]
fn plugin_multiple_instances_same_commands() {
    let plugins: Vec<_> = (0..5).map(|_| ClipboardPlugin::new()).collect();
    let first_cmds = plugins[0].commands();
    for p in &plugins[1..] {
        assert_eq!(p.commands(), first_cmds);
    }
}


#[test]
#[ignore = "Requires display server"]
fn clipboard_write_read_roundtrip() {
    let plugin = ClipboardPlugin::new();
    let scope = ScopeConfig::new();

    let write_result = plugin.handle(
        "write_text",
        serde_json::json!({ "text": "Test clipboard content" }),
        &scope,
    );
    assert!(write_result.is_ok());

    let read_result = plugin.handle("read_text", serde_json::json!({}), &scope);
    assert!(read_result.is_ok());
    let data = read_result.unwrap();
    assert_eq!(data["text"], "Test clipboard content");
}

#[test]
#[ignore = "Requires display server"]
fn clipboard_has_text_after_write() {
    let plugin = ClipboardPlugin::new();
    let scope = ScopeConfig::new();

    let _ = plugin.handle("write_text", serde_json::json!({ "text": "test" }), &scope);

    let result = plugin.handle("has_text", serde_json::json!({}), &scope);
    assert!(result.is_ok());
    let data = result.unwrap();
    assert!(data["hasText"].as_bool().unwrap());
}

#[test]
#[ignore = "Requires display server"]
fn clipboard_clear_succeeds() {
    let plugin = ClipboardPlugin::new();
    let scope = ScopeConfig::new();

    let _ = plugin.handle("write_text", serde_json::json!({ "text": "test" }), &scope);

    let result = plugin.handle("clear", serde_json::json!({}), &scope);
    assert!(result.is_ok());
}

#[test]
#[ignore = "Requires display server"]
fn clipboard_write_empty_text() {
    let plugin = ClipboardPlugin::new();
    let scope = ScopeConfig::new();

    let result = plugin.handle(
        "write_text",
        serde_json::json!({ "text": "" }),
        &scope,
    );
    assert!(result.is_ok());
    let data = result.unwrap();
    assert_eq!(data["success"], true);
}

#[test]
#[ignore = "Requires display server"]
fn clipboard_write_unicode() {
    let plugin = ClipboardPlugin::new();
    let scope = ScopeConfig::new();

    let result = plugin.handle(
        "write_text",
        serde_json::json!({ "text": "你好世界 🌍" }),
        &scope,
    );
    assert!(result.is_ok());
}
