//! Unit tests for file system types
//!
//! Tests for fs plugin option types, DirEntry, FileStat, serde roundtrips, edge cases.

use auroraview_plugins::fs::{
    CopyOptions, CreateDirOptions, DirEntry, ExistsOptions, FileStat, ReadDirOptions,
    ReadFileOptions, RemoveOptions, RenameOptions, StatOptions, WriteBinaryOptions,
    WriteFileOptions,
};
use rstest::rstest;

// ---------------------------------------------------------------------------
// ReadFileOptions
// ---------------------------------------------------------------------------

#[test]
fn read_file_options_with_encoding() {
    let json = r#"{"path": "/test/file.txt", "encoding": "utf-8"}"#;
    let opts: ReadFileOptions = serde_json::from_str(json).unwrap();
    assert_eq!(opts.path, "/test/file.txt");
    assert_eq!(opts.encoding, Some("utf-8".to_string()));
}

#[test]
fn read_file_options_without_encoding() {
    let json = r#"{"path": "/test/file.txt"}"#;
    let opts: ReadFileOptions = serde_json::from_str(json).unwrap();
    assert_eq!(opts.path, "/test/file.txt");
    assert!(opts.encoding.is_none());
}

#[rstest]
#[case("/path/to/file.txt", "utf-8")]
#[case("/path/to/binary.bin", "base64")]
#[case("/root/doc.md", "utf-16")]
fn read_file_options_encodings(#[case] path: &str, #[case] encoding: &str) {
    let json = serde_json::json!({ "path": path, "encoding": encoding });
    let opts: ReadFileOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.path, path);
    assert_eq!(opts.encoding.as_deref(), Some(encoding));
}

#[test]
fn read_file_options_unicode_path() {
    let path = "/path/to/文件/test.txt";
    let json = serde_json::json!({ "path": path });
    let opts: ReadFileOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.path, path);
}

#[test]
fn read_file_options_clone() {
    let opts = ReadFileOptions {
        path: "/a/b.txt".to_string(),
        encoding: Some("utf-8".to_string()),
    };
    let cloned = opts.clone();
    assert_eq!(cloned.path, opts.path);
    assert_eq!(cloned.encoding, opts.encoding);
}

#[test]
fn read_file_options_debug() {
    let opts = ReadFileOptions {
        path: "/test.txt".to_string(),
        encoding: Some("utf-8".to_string()),
    };
    let debug = format!("{:?}", opts);
    assert!(debug.contains("ReadFileOptions"));
    assert!(debug.contains("/test.txt"));
}

#[test]
fn read_file_options_serde_roundtrip() {
    let original = ReadFileOptions {
        path: "/roundtrip/file.txt".to_string(),
        encoding: Some("utf-8".to_string()),
    };
    let serialized = serde_json::to_value(&original).unwrap();
    let deserialized: ReadFileOptions = serde_json::from_value(serialized).unwrap();
    assert_eq!(deserialized.path, original.path);
    assert_eq!(deserialized.encoding, original.encoding);
}

// ---------------------------------------------------------------------------
// WriteFileOptions
// ---------------------------------------------------------------------------

#[test]
fn write_file_options_full() {
    let json = r#"{"path": "/test/file.txt", "contents": "Hello World", "append": true}"#;
    let opts: WriteFileOptions = serde_json::from_str(json).unwrap();
    assert_eq!(opts.path, "/test/file.txt");
    assert_eq!(opts.contents, "Hello World");
    assert!(opts.append);
}

#[test]
fn write_file_options_default_append() {
    let json = r#"{"path": "/test/file.txt", "contents": "Hello"}"#;
    let opts: WriteFileOptions = serde_json::from_str(json).unwrap();
    assert!(!opts.append);
}

#[test]
fn write_file_options_empty_contents() {
    let json = serde_json::json!({ "path": "/empty.txt", "contents": "" });
    let opts: WriteFileOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.contents, "");
}

#[test]
fn write_file_options_unicode_contents() {
    let json = serde_json::json!({ "path": "/unicode.txt", "contents": "你好 🌍 αβγ" });
    let opts: WriteFileOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.contents, "你好 🌍 αβγ");
}

#[test]
fn write_file_options_clone() {
    let opts = WriteFileOptions {
        path: "/path.txt".to_string(),
        contents: "data".to_string(),
        append: true,
    };
    let cloned = opts.clone();
    assert_eq!(cloned.path, opts.path);
    assert_eq!(cloned.contents, opts.contents);
    assert_eq!(cloned.append, opts.append);
}

#[test]
fn write_file_options_serde_roundtrip() {
    let original = WriteFileOptions {
        path: "/rt/file.txt".to_string(),
        contents: "roundtrip".to_string(),
        append: false,
    };
    let serialized = serde_json::to_value(&original).unwrap();
    let deserialized: WriteFileOptions = serde_json::from_value(serialized).unwrap();
    assert_eq!(deserialized.path, original.path);
    assert_eq!(deserialized.contents, original.contents);
    assert_eq!(deserialized.append, original.append);
}

// ---------------------------------------------------------------------------
// WriteBinaryOptions
// ---------------------------------------------------------------------------

#[test]
fn write_binary_options_basic() {
    let json = r#"{"path": "/test/file.bin", "contents": [1, 2, 3, 4], "append": false}"#;
    let opts: WriteBinaryOptions = serde_json::from_str(json).unwrap();
    assert_eq!(opts.path, "/test/file.bin");
    assert_eq!(opts.contents, vec![1u8, 2, 3, 4]);
    assert!(!opts.append);
}

#[test]
fn write_binary_options_empty_contents() {
    let json = serde_json::json!({ "path": "/empty.bin", "contents": [] });
    let opts: WriteBinaryOptions = serde_json::from_value(json).unwrap();
    assert!(opts.contents.is_empty());
}

#[test]
fn write_binary_options_all_byte_values() {
    let contents: Vec<u8> = (0..=255).collect();
    let json = serde_json::json!({ "path": "/all_bytes.bin", "contents": contents });
    let opts: WriteBinaryOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.contents.len(), 256);
    assert_eq!(opts.contents[0], 0);
    assert_eq!(opts.contents[255], 255);
}

#[test]
fn write_binary_options_serde_roundtrip() {
    let original = WriteBinaryOptions {
        path: "/rt.bin".to_string(),
        contents: vec![0xDE, 0xAD, 0xBE, 0xEF],
        append: true,
    };
    let serialized = serde_json::to_value(&original).unwrap();
    let deserialized: WriteBinaryOptions = serde_json::from_value(serialized).unwrap();
    assert_eq!(deserialized.path, original.path);
    assert_eq!(deserialized.contents, original.contents);
    assert_eq!(deserialized.append, original.append);
}

// ---------------------------------------------------------------------------
// ReadDirOptions
// ---------------------------------------------------------------------------

#[test]
fn read_dir_options_recursive() {
    let json = r#"{"path": "/test/dir", "recursive": true}"#;
    let opts: ReadDirOptions = serde_json::from_str(json).unwrap();
    assert_eq!(opts.path, "/test/dir");
    assert!(opts.recursive);
}

#[test]
fn read_dir_options_default_recursive() {
    let json = r#"{"path": "/test/dir"}"#;
    let opts: ReadDirOptions = serde_json::from_str(json).unwrap();
    assert!(!opts.recursive);
}

#[test]
fn read_dir_options_clone() {
    let opts = ReadDirOptions {
        path: "/dir".to_string(),
        recursive: true,
    };
    let cloned = opts.clone();
    assert_eq!(cloned.path, opts.path);
    assert_eq!(cloned.recursive, opts.recursive);
}

// ---------------------------------------------------------------------------
// CreateDirOptions
// ---------------------------------------------------------------------------

#[test]
fn create_dir_options_explicit_not_recursive() {
    let json = r#"{"path": "/test/new_dir", "recursive": false}"#;
    let opts: CreateDirOptions = serde_json::from_str(json).unwrap();
    assert_eq!(opts.path, "/test/new_dir");
    assert!(!opts.recursive);
}

#[test]
fn create_dir_options_default_recursive() {
    let json = r#"{"path": "/test/new_dir"}"#;
    let opts: CreateDirOptions = serde_json::from_str(json).unwrap();
    assert!(opts.recursive);
}

#[test]
fn create_dir_options_serde_roundtrip() {
    let original = CreateDirOptions {
        path: "/rt/dir".to_string(),
        recursive: true,
    };
    let serialized = serde_json::to_value(&original).unwrap();
    let deserialized: CreateDirOptions = serde_json::from_value(serialized).unwrap();
    assert_eq!(deserialized.path, original.path);
    assert_eq!(deserialized.recursive, original.recursive);
}

// ---------------------------------------------------------------------------
// RemoveOptions
// ---------------------------------------------------------------------------

#[test]
fn remove_options_basic() {
    let json = r#"{"path": "/test/file.txt", "recursive": true}"#;
    let opts: RemoveOptions = serde_json::from_str(json).unwrap();
    assert_eq!(opts.path, "/test/file.txt");
    assert!(opts.recursive);
}

#[test]
fn remove_options_clone() {
    let opts = RemoveOptions {
        path: "/rm/path".to_string(),
        recursive: false,
    };
    let cloned = opts.clone();
    assert_eq!(cloned.path, opts.path);
    assert_eq!(cloned.recursive, opts.recursive);
}

// ---------------------------------------------------------------------------
// CopyOptions
// ---------------------------------------------------------------------------

#[test]
fn copy_options_basic() {
    let json = r#"{"from": "/source/file.txt", "to": "/dest/file.txt"}"#;
    let opts: CopyOptions = serde_json::from_str(json).unwrap();
    assert_eq!(opts.from, "/source/file.txt");
    assert_eq!(opts.to, "/dest/file.txt");
}

#[test]
fn copy_options_clone() {
    let opts = CopyOptions {
        from: "/a".to_string(),
        to: "/b".to_string(),
    };
    let cloned = opts.clone();
    assert_eq!(cloned.from, "/a");
    assert_eq!(cloned.to, "/b");
}

#[test]
fn copy_options_serde_roundtrip() {
    let original = CopyOptions {
        from: "/src/a.txt".to_string(),
        to: "/dst/b.txt".to_string(),
    };
    let serialized = serde_json::to_value(&original).unwrap();
    let deserialized: CopyOptions = serde_json::from_value(serialized).unwrap();
    assert_eq!(deserialized.from, original.from);
    assert_eq!(deserialized.to, original.to);
}

#[test]
fn copy_options_unicode_paths() {
    let json = serde_json::json!({
        "from": "/源路径/文件.txt",
        "to": "/目标路径/文件.txt"
    });
    let opts: CopyOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.from, "/源路径/文件.txt");
    assert_eq!(opts.to, "/目标路径/文件.txt");
}

// ---------------------------------------------------------------------------
// RenameOptions
// ---------------------------------------------------------------------------

#[test]
fn rename_options_basic() {
    let json = r#"{"from": "/old/path.txt", "to": "/new/path.txt"}"#;
    let opts: RenameOptions = serde_json::from_str(json).unwrap();
    assert_eq!(opts.from, "/old/path.txt");
    assert_eq!(opts.to, "/new/path.txt");
}

#[test]
fn rename_options_clone() {
    let opts = RenameOptions {
        from: "/old.txt".to_string(),
        to: "/new.txt".to_string(),
    };
    let cloned = opts.clone();
    assert_eq!(cloned.from, opts.from);
    assert_eq!(cloned.to, opts.to);
}

// ---------------------------------------------------------------------------
// ExistsOptions / StatOptions
// ---------------------------------------------------------------------------

#[test]
fn exists_options_basic() {
    let json = r#"{"path": "/test/file.txt"}"#;
    let opts: ExistsOptions = serde_json::from_str(json).unwrap();
    assert_eq!(opts.path, "/test/file.txt");
}

#[test]
fn stat_options_basic() {
    let json = r#"{"path": "/test/file.txt"}"#;
    let opts: StatOptions = serde_json::from_str(json).unwrap();
    assert_eq!(opts.path, "/test/file.txt");
}

#[rstest]
#[case("/absolute/path/file.txt")]
#[case("relative/path.txt")]
#[case("")]
#[case("/path with spaces/file.txt")]
#[case("/path/日本語/file.txt")]
fn path_options_various_paths(#[case] path: &str) {
    let json = serde_json::json!({ "path": path });
    let opts: ExistsOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.path, path);
}

// ---------------------------------------------------------------------------
// DirEntry
// ---------------------------------------------------------------------------

#[test]
fn dir_entry_file() {
    let entry = DirEntry {
        name: "test.txt".to_string(),
        path: "/path/to/test.txt".to_string(),
        is_directory: false,
        is_file: true,
        is_symlink: false,
    };
    assert_eq!(entry.name, "test.txt");
    assert!(!entry.is_directory);
    assert!(entry.is_file);
    assert!(!entry.is_symlink);
}

#[test]
fn dir_entry_directory() {
    let entry = DirEntry {
        name: "subdir".to_string(),
        path: "/path/to/subdir".to_string(),
        is_directory: true,
        is_file: false,
        is_symlink: false,
    };
    assert!(entry.is_directory);
    assert!(!entry.is_file);
    assert!(!entry.is_symlink);
}

#[test]
fn dir_entry_symlink() {
    let entry = DirEntry {
        name: "link.txt".to_string(),
        path: "/path/to/link.txt".to_string(),
        is_directory: false,
        is_file: false,
        is_symlink: true,
    };
    assert!(entry.is_symlink);
    assert!(!entry.is_file);
}

#[test]
fn dir_entry_serialize() {
    let entry = DirEntry {
        name: "folder".to_string(),
        path: "/path/to/folder".to_string(),
        is_directory: true,
        is_file: false,
        is_symlink: false,
    };
    let json = serde_json::to_string(&entry).unwrap();
    assert!(json.contains("\"name\":\"folder\""));
    assert!(json.contains("\"isDirectory\":true"));
    assert!(json.contains("\"isFile\":false"));
}

#[test]
fn dir_entry_serde_roundtrip() {
    let original = DirEntry {
        name: "rt_file.txt".to_string(),
        path: "/rt/rt_file.txt".to_string(),
        is_directory: false,
        is_file: true,
        is_symlink: false,
    };
    let serialized = serde_json::to_value(&original).unwrap();
    let deserialized: DirEntry = serde_json::from_value(serialized).unwrap();
    assert_eq!(deserialized.name, original.name);
    assert_eq!(deserialized.path, original.path);
    assert_eq!(deserialized.is_file, original.is_file);
    assert_eq!(deserialized.is_directory, original.is_directory);
}

#[test]
fn dir_entry_unicode_name() {
    let entry = DirEntry {
        name: "文件夹".to_string(),
        path: "/path/文件夹".to_string(),
        is_directory: true,
        is_file: false,
        is_symlink: false,
    };
    assert_eq!(entry.name, "文件夹");
}

#[test]
fn dir_entry_clone() {
    let entry = DirEntry {
        name: "clone.txt".to_string(),
        path: "/clone.txt".to_string(),
        is_directory: false,
        is_file: true,
        is_symlink: false,
    };
    let cloned = entry.clone();
    assert_eq!(cloned.name, entry.name);
    assert_eq!(cloned.path, entry.path);
    assert_eq!(cloned.is_file, entry.is_file);
}

#[test]
fn dir_entry_debug() {
    let entry = DirEntry {
        name: "debug.txt".to_string(),
        path: "/debug.txt".to_string(),
        is_directory: false,
        is_file: true,
        is_symlink: false,
    };
    let debug = format!("{:?}", entry);
    assert!(debug.contains("DirEntry") || debug.contains("debug.txt"));
}

// ---------------------------------------------------------------------------
// FileStat
// ---------------------------------------------------------------------------

#[test]
fn file_stat_basic() {
    let stat = FileStat {
        is_directory: false,
        is_file: true,
        is_symlink: false,
        size: 1024,
        modified_at: Some(1735689600000),
        created_at: Some(1735689500000),
        accessed_at: Some(1735689600000),
        readonly: false,
    };
    assert!(stat.is_file);
    assert!(!stat.is_directory);
    assert_eq!(stat.size, 1024);
    assert!(!stat.readonly);
}

#[test]
fn file_stat_serialize_skips_none() {
    let stat = FileStat {
        is_directory: false,
        is_file: true,
        is_symlink: false,
        size: 512,
        modified_at: None,
        created_at: None,
        accessed_at: None,
        readonly: true,
    };
    let json = serde_json::to_string(&stat).unwrap();
    assert!(!json.contains("modifiedAt"));
    assert!(!json.contains("createdAt"));
    assert!(!json.contains("accessedAt"));
    assert!(json.contains("\"readonly\":true"));
}

#[test]
fn file_stat_clone() {
    let stat = FileStat {
        is_directory: true,
        is_file: false,
        is_symlink: false,
        size: 4096,
        modified_at: Some(1735689600000),
        created_at: None,
        accessed_at: None,
        readonly: false,
    };
    let cloned = stat.clone();
    assert_eq!(cloned.size, 4096);
    assert!(cloned.is_directory);
}

#[test]
fn file_stat_zero_size() {
    let stat = FileStat {
        is_directory: false,
        is_file: true,
        is_symlink: false,
        size: 0,
        modified_at: None,
        created_at: None,
        accessed_at: None,
        readonly: false,
    };
    assert_eq!(stat.size, 0);
}

#[test]
fn file_stat_large_size() {
    let stat = FileStat {
        is_directory: false,
        is_file: true,
        is_symlink: false,
        size: u64::MAX,
        modified_at: None,
        created_at: None,
        accessed_at: None,
        readonly: false,
    };
    assert_eq!(stat.size, u64::MAX);
}

#[test]
fn file_stat_serde_roundtrip() {
    let original = FileStat {
        is_directory: false,
        is_file: true,
        is_symlink: false,
        size: 2048,
        modified_at: Some(1735000000000),
        created_at: Some(1734000000000),
        accessed_at: Some(1735500000000),
        readonly: false,
    };
    let serialized = serde_json::to_value(&original).unwrap();
    let deserialized: FileStat = serde_json::from_value(serialized).unwrap();
    assert_eq!(deserialized.size, original.size);
    assert_eq!(deserialized.is_file, original.is_file);
    assert_eq!(deserialized.modified_at, original.modified_at);
    assert_eq!(deserialized.created_at, original.created_at);
}

#[test]
fn file_stat_directory_type() {
    let stat = FileStat {
        is_directory: true,
        is_file: false,
        is_symlink: false,
        size: 0,
        modified_at: None,
        created_at: None,
        accessed_at: None,
        readonly: false,
    };
    assert!(stat.is_directory);
    assert!(!stat.is_file);
    assert!(!stat.is_symlink);
}
