//! Unit tests for file system types
//!
//! Tests for fs plugin option types and data structures.

use auroraview_plugins::fs::{
    CopyOptions, CreateDirOptions, DirEntry, ExistsOptions, FileStat, ReadDirOptions,
    ReadFileOptions, RemoveOptions, RenameOptions, StatOptions, WriteBinaryOptions,
    WriteFileOptions,
};

#[test]
fn test_read_file_options_deserialize() {
    let json = r#"{"path": "/test/file.txt", "encoding": "utf-8"}"#;
    let opts: ReadFileOptions = serde_json::from_str(json).unwrap();
    assert_eq!(opts.path, "/test/file.txt");
    assert_eq!(opts.encoding, Some("utf-8".to_string()));
}

#[test]
fn test_read_file_options_without_encoding() {
    let json = r#"{"path": "/test/file.txt"}"#;
    let opts: ReadFileOptions = serde_json::from_str(json).unwrap();
    assert_eq!(opts.path, "/test/file.txt");
    assert!(opts.encoding.is_none());
}

#[test]
fn test_write_file_options_deserialize() {
    let json = r#"{"path": "/test/file.txt", "contents": "Hello World", "append": true}"#;
    let opts: WriteFileOptions = serde_json::from_str(json).unwrap();
    assert_eq!(opts.path, "/test/file.txt");
    assert_eq!(opts.contents, "Hello World");
    assert!(opts.append);
}

#[test]
fn test_write_file_options_default_append() {
    let json = r#"{"path": "/test/file.txt", "contents": "Hello"}"#;
    let opts: WriteFileOptions = serde_json::from_str(json).unwrap();
    assert!(!opts.append);
}

#[test]
fn test_write_binary_options_deserialize() {
    let json = r#"{"path": "/test/file.bin", "contents": [1, 2, 3, 4], "append": false}"#;
    let opts: WriteBinaryOptions = serde_json::from_str(json).unwrap();
    assert_eq!(opts.path, "/test/file.bin");
    assert_eq!(opts.contents, vec![1, 2, 3, 4]);
    assert!(!opts.append);
}

#[test]
fn test_read_dir_options_deserialize() {
    let json = r#"{"path": "/test/dir", "recursive": true}"#;
    let opts: ReadDirOptions = serde_json::from_str(json).unwrap();
    assert_eq!(opts.path, "/test/dir");
    assert!(opts.recursive);
}

#[test]
fn test_read_dir_options_default_recursive() {
    let json = r#"{"path": "/test/dir"}"#;
    let opts: ReadDirOptions = serde_json::from_str(json).unwrap();
    assert!(!opts.recursive);
}

#[test]
fn test_create_dir_options_deserialize() {
    let json = r#"{"path": "/test/new_dir", "recursive": false}"#;
    let opts: CreateDirOptions = serde_json::from_str(json).unwrap();
    assert_eq!(opts.path, "/test/new_dir");
    assert!(!opts.recursive);
}

#[test]
fn test_create_dir_options_default_recursive() {
    let json = r#"{"path": "/test/new_dir"}"#;
    let opts: CreateDirOptions = serde_json::from_str(json).unwrap();
    // Default is true for recursive
    assert!(opts.recursive);
}

#[test]
fn test_remove_options_deserialize() {
    let json = r#"{"path": "/test/file.txt", "recursive": true}"#;
    let opts: RemoveOptions = serde_json::from_str(json).unwrap();
    assert_eq!(opts.path, "/test/file.txt");
    assert!(opts.recursive);
}

#[test]
fn test_copy_options_deserialize() {
    let json = r#"{"from": "/source/file.txt", "to": "/dest/file.txt"}"#;
    let opts: CopyOptions = serde_json::from_str(json).unwrap();
    assert_eq!(opts.from, "/source/file.txt");
    assert_eq!(opts.to, "/dest/file.txt");
}

#[test]
fn test_rename_options_deserialize() {
    let json = r#"{"from": "/old/path.txt", "to": "/new/path.txt"}"#;
    let opts: RenameOptions = serde_json::from_str(json).unwrap();
    assert_eq!(opts.from, "/old/path.txt");
    assert_eq!(opts.to, "/new/path.txt");
}

#[test]
fn test_exists_options_deserialize() {
    let json = r#"{"path": "/test/file.txt"}"#;
    let opts: ExistsOptions = serde_json::from_str(json).unwrap();
    assert_eq!(opts.path, "/test/file.txt");
}

#[test]
fn test_stat_options_deserialize() {
    let json = r#"{"path": "/test/file.txt"}"#;
    let opts: StatOptions = serde_json::from_str(json).unwrap();
    assert_eq!(opts.path, "/test/file.txt");
}

#[test]
fn test_dir_entry_creation() {
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
fn test_dir_entry_serialize() {
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
fn test_file_stat_creation() {
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
fn test_file_stat_serialize_skips_none() {
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
fn test_file_stat_clone() {
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
fn test_options_clone() {
    let opts = CopyOptions {
        from: "/a".to_string(),
        to: "/b".to_string(),
    };
    let cloned = opts.clone();
    assert_eq!(cloned.from, "/a");
    assert_eq!(cloned.to, "/b");
}

#[test]
fn test_options_debug() {
    let opts = ReadFileOptions {
        path: "/test.txt".to_string(),
        encoding: Some("utf-8".to_string()),
    };
    let debug = format!("{:?}", opts);
    assert!(debug.contains("ReadFileOptions"));
    assert!(debug.contains("/test.txt"));
}
