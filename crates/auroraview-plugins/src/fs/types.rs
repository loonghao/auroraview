//! File system plugin types

use serde::{Deserialize, Serialize};

/// Options for reading a file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadFileOptions {
    /// Path to the file
    pub path: String,
    /// Encoding (default: "utf-8")
    #[serde(default)]
    pub encoding: Option<String>,
}

/// Options for writing a file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteFileOptions {
    /// Path to the file
    pub path: String,
    /// Content to write
    pub contents: String,
    /// Append to file instead of overwrite
    #[serde(default)]
    pub append: bool,
}

/// Options for writing binary data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteBinaryOptions {
    /// Path to the file
    pub path: String,
    /// Binary content as base64 or byte array
    pub contents: Vec<u8>,
    /// Append to file instead of overwrite
    #[serde(default)]
    pub append: bool,
}

/// Options for reading a directory
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadDirOptions {
    /// Path to the directory
    pub path: String,
    /// Read recursively
    #[serde(default)]
    pub recursive: bool,
}

/// Options for creating a directory
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDirOptions {
    /// Path to the directory
    pub path: String,
    /// Create parent directories if they don't exist
    #[serde(default = "default_true")]
    pub recursive: bool,
}

fn default_true() -> bool {
    true
}

/// Options for removing a file or directory
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveOptions {
    /// Path to remove
    pub path: String,
    /// Remove recursively (for directories)
    #[serde(default)]
    pub recursive: bool,
}

/// Options for copying
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyOptions {
    /// Source path
    pub from: String,
    /// Destination path
    pub to: String,
}

/// Options for renaming/moving
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameOptions {
    /// Source path
    pub from: String,
    /// Destination path
    pub to: String,
}

/// Options for checking existence
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExistsOptions {
    /// Path to check
    pub path: String,
}

/// Options for getting file stats
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatOptions {
    /// Path to stat
    pub path: String,
}

/// Directory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirEntry {
    /// Entry name
    pub name: String,
    /// Full path
    pub path: String,
    /// Is directory
    pub is_directory: bool,
    /// Is file
    pub is_file: bool,
    /// Is symlink
    pub is_symlink: bool,
}

/// File statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileStat {
    /// Is directory
    pub is_directory: bool,
    /// Is file
    pub is_file: bool,
    /// Is symlink
    pub is_symlink: bool,
    /// File size in bytes
    pub size: u64,
    /// Last modified time (Unix timestamp in ms)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<u64>,
    /// Created time (Unix timestamp in ms)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<u64>,
    /// Last accessed time (Unix timestamp in ms)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessed_at: Option<u64>,
    /// Is readonly
    pub readonly: bool,
}
