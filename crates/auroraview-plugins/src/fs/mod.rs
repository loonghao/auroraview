//! File System Plugin
//!
//! Provides native file system operations accessible from JavaScript.
//!
//! ## Commands
//!
//! - `read_file` - Read file contents (text or binary)
//! - `write_file` - Write content to a file
//! - `read_dir` - List directory contents
//! - `create_dir` - Create a directory
//! - `remove` - Remove a file or directory
//! - `copy` - Copy a file or directory
//! - `rename` - Rename/move a file or directory
//! - `exists` - Check if a path exists
//! - `stat` - Get file/directory metadata
//!
//! ## Security
//!
//! All paths are validated against the configured scope.
//! Operations outside the allowed scope will be rejected.

mod operations;
mod types;

pub use operations::*;
pub use types::*;

use crate::{PluginError, PluginHandler, PluginResult, ScopeConfig};
use serde_json::Value;

/// File system plugin
pub struct FsPlugin {
    /// Plugin name
    name: String,
}

impl FsPlugin {
    /// Create a new file system plugin
    pub fn new() -> Self {
        Self {
            name: "fs".to_string(),
        }
    }
}

impl Default for FsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginHandler for FsPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn handle(&self, command: &str, args: Value, scope: &ScopeConfig) -> PluginResult<Value> {
        match command {
            "read_file" => {
                let opts: ReadFileOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;
                let result = read_file(&opts.path, opts.encoding.as_deref(), &scope.fs)?;
                Ok(serde_json::to_value(result).unwrap())
            }
            "read_file_binary" => {
                let opts: ReadFileOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;
                let result = read_file_binary(&opts.path, &scope.fs)?;
                Ok(serde_json::to_value(result).unwrap())
            }
            "write_file" => {
                let opts: WriteFileOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;
                write_file(&opts.path, &opts.contents, opts.append, &scope.fs)?;
                Ok(serde_json::json!({"success": true}))
            }
            "write_file_binary" => {
                let opts: WriteBinaryOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;
                write_file_binary(&opts.path, &opts.contents, opts.append, &scope.fs)?;
                Ok(serde_json::json!({"success": true}))
            }
            "read_dir" => {
                let opts: ReadDirOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;
                let result = read_dir(&opts.path, opts.recursive, &scope.fs)?;
                Ok(serde_json::to_value(result).unwrap())
            }
            "create_dir" => {
                let opts: CreateDirOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;
                create_dir(&opts.path, opts.recursive, &scope.fs)?;
                Ok(serde_json::json!({"success": true}))
            }
            "remove" => {
                let opts: RemoveOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;
                remove(&opts.path, opts.recursive, &scope.fs)?;
                Ok(serde_json::json!({"success": true}))
            }
            "copy" => {
                let opts: CopyOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;
                copy(&opts.from, &opts.to, &scope.fs)?;
                Ok(serde_json::json!({"success": true}))
            }
            "rename" => {
                let opts: RenameOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;
                rename(&opts.from, &opts.to, &scope.fs)?;
                Ok(serde_json::json!({"success": true}))
            }
            "exists" => {
                let opts: ExistsOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;
                let result = exists(&opts.path, &scope.fs)?;
                Ok(serde_json::json!({"exists": result}))
            }
            "stat" => {
                let opts: StatOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;
                let result = stat(&opts.path, &scope.fs)?;
                Ok(serde_json::to_value(result).unwrap())
            }
            _ => Err(PluginError::command_not_found(command)),
        }
    }

    fn commands(&self) -> Vec<&str> {
        vec![
            "read_file",
            "read_file_binary",
            "write_file",
            "write_file_binary",
            "read_dir",
            "create_dir",
            "remove",
            "copy",
            "rename",
            "exists",
            "stat",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PathScope, PluginRequest, PluginRouter};
    use tempfile::tempdir;

    #[test]
    fn test_fs_plugin_commands() {
        let plugin = FsPlugin::new();
        let commands = plugin.commands();
        assert!(commands.contains(&"read_file"));
        assert!(commands.contains(&"write_file"));
        assert!(commands.contains(&"exists"));
        assert!(commands.contains(&"stat"));
    }

    #[test]
    fn test_write_and_read_file() {
        let temp = tempdir().unwrap();
        let scope = ScopeConfig::new().with_fs_scope(PathScope::new().allow(temp.path()));
        let router = PluginRouter::with_scope(scope);

        let file_path = temp.path().join("test.txt");
        let file_path_str = file_path.to_string_lossy().to_string();

        // Write file
        let write_req = PluginRequest::new(
            "fs",
            "write_file",
            serde_json::json!({
                "path": file_path_str,
                "contents": "Hello, AuroraView!"
            }),
        );
        let write_resp = router.handle(write_req);
        assert!(write_resp.success, "Write failed: {:?}", write_resp.error);

        // Read file
        let read_req = PluginRequest::new(
            "fs",
            "read_file",
            serde_json::json!({ "path": file_path_str }),
        );
        let read_resp = router.handle(read_req);
        assert!(read_resp.success, "Read failed: {:?}", read_resp.error);
        assert_eq!(read_resp.data.unwrap(), "Hello, AuroraView!");
    }

    #[test]
    fn test_exists_command() {
        let temp = tempdir().unwrap();
        let scope = ScopeConfig::new().with_fs_scope(PathScope::new().allow(temp.path()));
        let router = PluginRouter::with_scope(scope);

        // Create a file
        let file_path = temp.path().join("exists_test.txt");
        std::fs::write(&file_path, "test").unwrap();

        // Check exists
        let req = PluginRequest::new(
            "fs",
            "exists",
            serde_json::json!({ "path": file_path.to_string_lossy() }),
        );
        let resp = router.handle(req);
        assert!(resp.success);
        let data = resp.data.unwrap();
        assert_eq!(data["exists"], true);
    }

    #[test]
    fn test_scope_violation() {
        let temp = tempdir().unwrap();
        let scope = ScopeConfig::new().with_fs_scope(PathScope::new().allow(temp.path()));
        let router = PluginRouter::with_scope(scope);

        // Try to read outside scope (should fail)
        let req = PluginRequest::new(
            "fs",
            "read_file",
            serde_json::json!({ "path": "C:\\Windows\\System32\\config.sys" }),
        );
        let resp = router.handle(req);
        assert!(!resp.success);
        assert_eq!(resp.code, Some("SCOPE_VIOLATION".to_string()));
    }
}
