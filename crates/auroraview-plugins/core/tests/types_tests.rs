//! Tests for PluginCommand types

use auroraview_plugin_core::PluginCommand;
use rstest::rstest;

#[test]
fn new_command_defaults() {
    let cmd = PluginCommand::new("read_file", "Read a file from disk");

    assert_eq!(cmd.name, "read_file");
    assert_eq!(cmd.description, "Read a file from disk");
    assert!(cmd.required_args.is_empty());
    assert!(cmd.optional_args.is_empty());
}

#[test]
fn new_accepts_string_owned() {
    let name = "write_file".to_string();
    let desc = "Write content to a file".to_string();
    let cmd = PluginCommand::new(name, desc);

    assert_eq!(cmd.name, "write_file");
    assert_eq!(cmd.description, "Write content to a file");
}

#[test]
fn with_required_sets_args() {
    let cmd = PluginCommand::new("copy", "Copy a file")
        .with_required(&["from", "to"]);

    assert_eq!(cmd.required_args, vec!["from", "to"]);
    assert!(cmd.optional_args.is_empty());
}

#[test]
fn with_optional_sets_args() {
    let cmd = PluginCommand::new("read_file", "Read a file")
        .with_optional(&["encoding"]);

    assert!(cmd.required_args.is_empty());
    assert_eq!(cmd.optional_args, vec!["encoding"]);
}

#[test]
fn with_required_and_optional_combined() {
    let cmd = PluginCommand::new("read_file", "Read a file")
        .with_required(&["path"])
        .with_optional(&["encoding", "binary"]);

    assert_eq!(cmd.required_args, vec!["path"]);
    assert_eq!(cmd.optional_args, vec!["encoding", "binary"]);
}

#[test]
fn with_required_empty_slice() {
    let cmd = PluginCommand::new("ping", "Ping").with_required(&[]);
    assert!(cmd.required_args.is_empty());
}

#[test]
fn with_optional_empty_slice() {
    let cmd = PluginCommand::new("ping", "Ping").with_optional(&[]);
    assert!(cmd.optional_args.is_empty());
}

#[test]
fn clone_produces_independent_copy() {
    let cmd = PluginCommand::new("cmd", "desc")
        .with_required(&["a"])
        .with_optional(&["b"]);
    let cloned = cmd.clone();

    assert_eq!(cloned.name, cmd.name);
    assert_eq!(cloned.description, cmd.description);
    assert_eq!(cloned.required_args, cmd.required_args);
    assert_eq!(cloned.optional_args, cmd.optional_args);
}

#[test]
fn debug_contains_name() {
    let cmd = PluginCommand::new("my_cmd", "My command");
    let debug = format!("{:?}", cmd);
    assert!(debug.contains("my_cmd"));
}

#[test]
fn serde_roundtrip() {
    let cmd = PluginCommand::new("stat", "Get file stats")
        .with_required(&["path"])
        .with_optional(&["follow_symlinks"]);

    let json = serde_json::to_string(&cmd).expect("serialize");
    let restored: PluginCommand = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(restored.name, cmd.name);
    assert_eq!(restored.description, cmd.description);
    assert_eq!(restored.required_args, cmd.required_args);
    assert_eq!(restored.optional_args, cmd.optional_args);
}

#[test]
fn serde_json_fields() {
    let cmd = PluginCommand::new("foo", "bar").with_required(&["x"]);
    let json = serde_json::to_string(&cmd).expect("serialize");

    assert!(json.contains("\"name\""));
    assert!(json.contains("\"description\""));
    assert!(json.contains("\"required_args\""));
    assert!(json.contains("\"optional_args\""));
}

#[test]
fn deserialize_from_json() {
    let json = r#"{"name":"remove","description":"Remove file","required_args":["path"],"optional_args":["recursive"]}"#;
    let cmd: PluginCommand = serde_json::from_str(json).expect("deserialize");

    assert_eq!(cmd.name, "remove");
    assert_eq!(cmd.description, "Remove file");
    assert_eq!(cmd.required_args, vec!["path"]);
    assert_eq!(cmd.optional_args, vec!["recursive"]);
}

#[test]
fn with_required_multiple_args() {
    let cmd = PluginCommand::new("multi", "Multiple args")
        .with_required(&["a", "b", "c", "d", "e"]);

    assert_eq!(cmd.required_args.len(), 5);
    assert_eq!(cmd.required_args[0], "a");
    assert_eq!(cmd.required_args[4], "e");
}

#[test]
fn with_optional_overwrites_previous() {
    // Calling with_optional twice replaces (not appends)
    let cmd = PluginCommand::new("cmd", "desc")
        .with_optional(&["a"])
        .with_optional(&["b", "c"]);

    assert_eq!(cmd.optional_args, vec!["b", "c"]);
}

#[test]
fn with_required_overwrites_previous() {
    let cmd = PluginCommand::new("cmd", "desc")
        .with_required(&["x"])
        .with_required(&["y", "z"]);

    assert_eq!(cmd.required_args, vec!["y", "z"]);
}

#[test]
fn empty_name_and_description() {
    let cmd = PluginCommand::new("", "");
    assert_eq!(cmd.name, "");
    assert_eq!(cmd.description, "");
}

#[test]
fn long_name_and_description() {
    let name = "a".repeat(256);
    let desc = "b".repeat(1024);
    let cmd = PluginCommand::new(name.clone(), desc.clone());
    assert_eq!(cmd.name, name);
    assert_eq!(cmd.description, desc);
}

#[rstest]
#[case("read_file", "path")]
#[case("write_file", "path")]
#[case("remove", "path")]
#[case("stat", "path")]
#[case("create_dir", "path")]
fn required_arg_path(#[case] name: &str, #[case] arg: &str) {
    let cmd = PluginCommand::new(name, "desc").with_required(&[arg]);
    assert_eq!(cmd.required_args, vec![arg]);
    assert_eq!(cmd.name, name);
}

#[rstest]
#[case("read_file", &["encoding", "binary"])]
#[case("write_file", &["append", "create_dirs"])]
#[case("read_dir", &["recursive"])]
fn optional_args_set(#[case] name: &str, #[case] opts: &[&str]) {
    let cmd = PluginCommand::new(name, "desc").with_optional(opts);
    assert_eq!(cmd.optional_args.len(), opts.len());
    for (i, o) in opts.iter().enumerate() {
        assert_eq!(cmd.optional_args[i], *o);
    }
}

#[test]
fn is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<PluginCommand>();
}

#[test]
fn serde_no_extra_keys() {
    let cmd = PluginCommand::new("test", "test desc");
    let json = serde_json::to_value(&cmd).expect("to_value");
    let obj = json.as_object().expect("is object");
    // Exactly 4 keys: name, description, required_args, optional_args
    assert_eq!(obj.len(), 4);
}
