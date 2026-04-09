//! Tests for fs plugin operations and types

use auroraview_plugin_core::PluginHandler;
use auroraview_plugin_core::{PathScope, ScopeConfig};
use auroraview_plugin_fs::{
    copy, create_dir, exists, read_dir, read_file, read_file_binary, remove, rename, stat,
    write_file, write_file_binary, DirEntry, FileStat, FsPlugin, ReadFileOptions, WriteFileOptions,
};
use rstest::rstest;
use serde_json::json;
use std::fs as std_fs;
use tempfile::TempDir;

fn make_scope(dir: &TempDir) -> PathScope {
    PathScope::new().allow(dir.path())
}

fn make_scope_config(dir: &TempDir) -> ScopeConfig {
    ScopeConfig::new().with_fs_scope(make_scope(dir))
}

// ─── read_file ───────────────────────────────────────────────────────────────

#[test]
fn read_file_text_ok() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("hello.txt");
    std_fs::write(&path, "hello world").unwrap();
    let scope = make_scope(&dir);

    let content = read_file(path.to_str().unwrap(), None, &scope).unwrap();
    assert_eq!(content, "hello world");
}

#[test]
fn read_file_utf8_encoding_ok() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("f.txt");
    std_fs::write(&path, "hello").unwrap();
    let scope = make_scope(&dir);

    let content = read_file(path.to_str().unwrap(), Some("utf-8"), &scope).unwrap();
    assert_eq!(content, "hello");
}

#[test]
fn read_file_not_found_err() {
    let dir = TempDir::new().unwrap();
    let scope = make_scope(&dir);
    let missing = dir
        .path()
        .join("missing.txt")
        .to_string_lossy()
        .into_owned();

    let result = read_file(&missing, None, &scope);
    assert!(result.is_err());
}

#[test]
fn read_file_scope_violation_err() {
    let dir = TempDir::new().unwrap();
    let other = TempDir::new().unwrap();
    let path = other.path().join("secret.txt");
    std_fs::write(&path, "secret").unwrap();
    let scope = make_scope(&dir);

    let result = read_file(path.to_str().unwrap(), None, &scope);
    assert!(result.is_err());
}

// ─── read_file_binary ────────────────────────────────────────────────────────

#[test]
fn read_file_binary_returns_base64() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("data.bin");
    std_fs::write(&path, b"\x00\x01\x02\x03").unwrap();
    let scope = make_scope(&dir);

    let b64 = read_file_binary(path.to_str().unwrap(), &scope).unwrap();
    // base64 of [0,1,2,3] == "AAECAw=="
    assert_eq!(b64, "AAECAw==");
}

#[test]
fn read_file_binary_not_found_err() {
    let dir = TempDir::new().unwrap();
    let scope = make_scope(&dir);
    let missing = dir.path().join("nope.bin").to_string_lossy().into_owned();

    let result = read_file_binary(&missing, &scope);
    assert!(result.is_err());
}

// ─── write_file / write_file_binary ─────────────────────────────────────────

#[test]
fn write_file_creates_new_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("out.txt");
    let scope = make_scope(&dir);

    write_file(path.to_str().unwrap(), "written", false, &scope).unwrap();
    assert_eq!(std_fs::read_to_string(&path).unwrap(), "written");
}

#[test]
fn write_file_overwrite() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("ow.txt");
    std_fs::write(&path, "old").unwrap();
    let scope = make_scope(&dir);

    write_file(path.to_str().unwrap(), "new", false, &scope).unwrap();
    assert_eq!(std_fs::read_to_string(&path).unwrap(), "new");
}

#[test]
fn write_file_append() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("app.txt");
    std_fs::write(&path, "line1\n").unwrap();
    let scope = make_scope(&dir);

    write_file(path.to_str().unwrap(), "line2\n", true, &scope).unwrap();
    let content = std_fs::read_to_string(&path).unwrap();
    assert_eq!(content, "line1\nline2\n");
}

#[test]
fn write_file_binary_creates_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("bin.bin");
    let scope = make_scope(&dir);

    write_file_binary(path.to_str().unwrap(), &[10u8, 20, 30], false, &scope).unwrap();
    let bytes = std_fs::read(&path).unwrap();
    assert_eq!(bytes, vec![10u8, 20, 30]);
}

#[test]
fn write_file_binary_append() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("binapp.bin");
    std_fs::write(&path, b"\x01\x02").unwrap();
    let scope = make_scope(&dir);

    write_file_binary(path.to_str().unwrap(), &[3u8, 4], true, &scope).unwrap();
    let bytes = std_fs::read(&path).unwrap();
    assert_eq!(bytes, vec![1u8, 2, 3, 4]);
}

// ─── read_dir ────────────────────────────────────────────────────────────────

#[test]
fn read_dir_lists_files() {
    let dir = TempDir::new().unwrap();
    std_fs::write(dir.path().join("a.txt"), "a").unwrap();
    std_fs::write(dir.path().join("b.txt"), "b").unwrap();
    let scope = make_scope(&dir);

    let entries = read_dir(dir.path().to_str().unwrap(), false, &scope).unwrap();
    assert_eq!(entries.len(), 2);
}

#[test]
fn read_dir_recursive() {
    let dir = TempDir::new().unwrap();
    std_fs::create_dir(dir.path().join("sub")).unwrap();
    std_fs::write(dir.path().join("root.txt"), "root").unwrap();
    std_fs::write(dir.path().join("sub/child.txt"), "child").unwrap();
    let scope = make_scope(&dir);

    let entries = read_dir(dir.path().to_str().unwrap(), true, &scope).unwrap();
    assert!(entries.len() >= 3); // root.txt, sub/, sub/child.txt
}

#[test]
fn read_dir_not_found_err() {
    let dir = TempDir::new().unwrap();
    let scope = make_scope(&dir);
    let missing = dir
        .path()
        .join("nonexistent")
        .to_string_lossy()
        .into_owned();

    let result = read_dir(&missing, false, &scope);
    assert!(result.is_err());
}

// ─── create_dir ──────────────────────────────────────────────────────────────

#[test]
fn create_dir_single() {
    let dir = TempDir::new().unwrap();
    let new_dir = dir.path().join("newdir");
    let scope = make_scope(&dir);

    create_dir(new_dir.to_str().unwrap(), false, &scope).unwrap();
    assert!(new_dir.exists());
}

#[test]
fn create_dir_recursive() {
    let dir = TempDir::new().unwrap();
    let nested = dir.path().join("a/b/c");
    let scope = make_scope(&dir);

    create_dir(nested.to_str().unwrap(), true, &scope).unwrap();
    assert!(nested.exists());
}

// ─── remove ──────────────────────────────────────────────────────────────────

#[test]
fn remove_file_ok() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("del.txt");
    std_fs::write(&path, "data").unwrap();
    let scope = make_scope(&dir);

    remove(path.to_str().unwrap(), false, &scope).unwrap();
    assert!(!path.exists());
}

#[test]
fn remove_dir_recursive_ok() {
    let dir = TempDir::new().unwrap();
    let subdir = dir.path().join("subdir");
    std_fs::create_dir_all(subdir.join("inner")).unwrap();
    std_fs::write(subdir.join("f.txt"), "x").unwrap();
    let scope = make_scope(&dir);

    remove(subdir.to_str().unwrap(), true, &scope).unwrap();
    assert!(!subdir.exists());
}

#[test]
fn remove_not_found_err() {
    let dir = TempDir::new().unwrap();
    let scope = make_scope(&dir);
    let missing = dir.path().join("ghost.txt").to_string_lossy().into_owned();

    let result = remove(&missing, false, &scope);
    assert!(result.is_err());
}

// ─── copy / rename ───────────────────────────────────────────────────────────

#[test]
fn copy_file_ok() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("src.txt");
    let dst = dir.path().join("dst.txt");
    std_fs::write(&src, "copy me").unwrap();
    let scope = make_scope(&dir);

    copy(src.to_str().unwrap(), dst.to_str().unwrap(), &scope).unwrap();
    assert!(src.exists()); // source still exists
    assert_eq!(std_fs::read_to_string(&dst).unwrap(), "copy me");
}

#[test]
fn copy_not_found_err() {
    let dir = TempDir::new().unwrap();
    let scope = make_scope(&dir);
    let missing = dir.path().join("nope.txt").to_string_lossy().into_owned();
    let dst = dir.path().join("dst.txt").to_string_lossy().into_owned();

    let result = copy(&missing, &dst, &scope);
    assert!(result.is_err());
}

#[test]
fn rename_file_ok() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("old.txt");
    let dst = dir.path().join("new.txt");
    std_fs::write(&src, "data").unwrap();
    let scope = make_scope(&dir);

    rename(src.to_str().unwrap(), dst.to_str().unwrap(), &scope).unwrap();
    assert!(!src.exists());
    assert_eq!(std_fs::read_to_string(&dst).unwrap(), "data");
}

// ─── exists ──────────────────────────────────────────────────────────────────

#[test]
fn exists_true_for_existing_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("exist.txt");
    std_fs::write(&path, "yes").unwrap();
    let scope = make_scope(&dir);

    assert!(exists(path.to_str().unwrap(), &scope).unwrap());
}

#[test]
fn exists_false_for_missing_file() {
    let dir = TempDir::new().unwrap();
    let scope = make_scope(&dir);
    let missing = dir.path().join("no.txt").to_string_lossy().into_owned();

    assert!(!exists(&missing, &scope).unwrap());
}

#[test]
fn exists_false_outside_scope() {
    let dir = TempDir::new().unwrap();
    let other = TempDir::new().unwrap();
    let scope = make_scope(&dir);
    let path = other.path().join("x.txt").to_string_lossy().into_owned();

    // exists returns false (not error) for out-of-scope paths
    assert!(!exists(&path, &scope).unwrap());
}

// ─── stat ─────────────────────────────────────────────────────────────────────

#[test]
fn stat_file_ok() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("s.txt");
    std_fs::write(&path, "hello").unwrap();
    let scope = make_scope(&dir);

    let st = stat(path.to_str().unwrap(), &scope).unwrap();
    assert!(st.is_file);
    assert!(!st.is_directory);
    assert_eq!(st.size, 5);
}

#[test]
fn stat_dir_ok() {
    let dir = TempDir::new().unwrap();
    let scope = make_scope(&dir);

    let st = stat(dir.path().to_str().unwrap(), &scope).unwrap();
    assert!(st.is_directory);
    assert!(!st.is_file);
}

#[test]
fn stat_not_found_err() {
    let dir = TempDir::new().unwrap();
    let scope = make_scope(&dir);
    let missing = dir.path().join("ghost.txt").to_string_lossy().into_owned();

    let result = stat(&missing, &scope);
    assert!(result.is_err());
}

// ─── FsPlugin handler ────────────────────────────────────────────────────────

#[test]
fn fs_plugin_name() {
    let plugin = FsPlugin::new();
    assert_eq!(plugin.name(), "fs");
}

#[test]
fn fs_plugin_commands_list() {
    let plugin = FsPlugin::new();
    let cmds = plugin.commands();
    assert!(cmds.contains(&"read_file"));
    assert!(cmds.contains(&"write_file"));
    assert!(cmds.contains(&"read_dir"));
    assert!(cmds.contains(&"remove"));
    assert!(cmds.contains(&"exists"));
    assert!(cmds.contains(&"stat"));
}

#[test]
fn fs_plugin_default_is_new() {
    let plugin = FsPlugin::default();
    assert_eq!(plugin.name(), "fs");
}

#[test]
fn fs_plugin_handle_read_file_ok() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.txt");
    std_fs::write(&path, "plugin read").unwrap();

    let plugin = FsPlugin::new();
    let scope = make_scope_config(&dir);
    let args = json!({"path": path.to_str().unwrap()});

    let result = plugin.handle("read_file", args, &scope).unwrap();
    assert_eq!(result, json!("plugin read"));
}

#[test]
fn fs_plugin_handle_write_file_ok() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("written.txt");
    let plugin = FsPlugin::new();
    let scope = make_scope_config(&dir);
    let args = json!({"path": path.to_str().unwrap(), "contents": "plugin write"});

    let result = plugin.handle("write_file", args, &scope).unwrap();
    assert_eq!(result["success"], json!(true));
    assert_eq!(std_fs::read_to_string(&path).unwrap(), "plugin write");
}

#[test]
fn fs_plugin_handle_unknown_command_err() {
    let dir = TempDir::new().unwrap();
    let plugin = FsPlugin::new();
    let scope = make_scope_config(&dir);

    let result = plugin.handle("nonexistent_cmd", json!({}), &scope);
    assert!(result.is_err());
}

#[test]
fn fs_plugin_handle_invalid_args_err() {
    let dir = TempDir::new().unwrap();
    let plugin = FsPlugin::new();
    let scope = make_scope_config(&dir);

    // read_file requires "path" key
    let result = plugin.handle("read_file", json!({"not_path": "x"}), &scope);
    assert!(result.is_err());
}

#[test]
fn fs_plugin_handle_exists_ok() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("exists_test.txt");
    std_fs::write(&path, "data").unwrap();
    let plugin = FsPlugin::new();
    let scope = make_scope_config(&dir);
    let args = json!({"path": path.to_str().unwrap()});

    let result = plugin.handle("exists", args, &scope).unwrap();
    assert_eq!(result["exists"], json!(true));
}

// ─── Types: serde roundtrips ──────────────────────────────────────────────────

#[test]
fn read_file_options_serde() {
    let opts = ReadFileOptions {
        path: "/tmp/x.txt".to_string(),
        encoding: Some("utf-8".to_string()),
    };
    let json = serde_json::to_string(&opts).unwrap();
    let back: ReadFileOptions = serde_json::from_str(&json).unwrap();
    assert_eq!(back.path, opts.path);
    assert_eq!(back.encoding, opts.encoding);
}

#[test]
fn write_file_options_serde() {
    let opts = WriteFileOptions {
        path: "/tmp/out.txt".to_string(),
        contents: "data".to_string(),
        append: true,
    };
    let json = serde_json::to_string(&opts).unwrap();
    let back: WriteFileOptions = serde_json::from_str(&json).unwrap();
    assert_eq!(back.path, opts.path);
    assert!(back.append);
}

#[test]
fn dir_entry_serde() {
    let entry = DirEntry {
        name: "file.txt".to_string(),
        path: "/tmp/file.txt".to_string(),
        is_directory: false,
        is_file: true,
        is_symlink: false,
    };
    let json = serde_json::to_string(&entry).unwrap();
    let back: DirEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, "file.txt");
    assert!(back.is_file);
    assert!(!back.is_directory);
}

#[test]
fn file_stat_serde() {
    let stat = FileStat {
        is_directory: false,
        is_file: true,
        is_symlink: false,
        size: 42,
        modified_at: Some(1000),
        created_at: None,
        accessed_at: None,
        readonly: false,
    };
    let json = serde_json::to_value(&stat).unwrap();
    // skip_serializing_if = None → created_at/accessed_at should be absent
    assert!(json.get("createdAt").is_none());
    assert!(json.get("accessedAt").is_none());
    assert_eq!(json["size"], 42);
}

#[rstest]
#[case("read_file")]
#[case("write_file")]
#[case("read_dir")]
#[case("create_dir")]
#[case("remove")]
#[case("copy")]
#[case("rename")]
#[case("exists")]
#[case("stat")]
#[case("read_file_binary")]
#[case("write_file_binary")]
fn plugin_commands_contains(#[case] cmd: &str) {
    let plugin = FsPlugin::new();
    assert!(plugin.commands().contains(&cmd), "missing command: {cmd}");
}
