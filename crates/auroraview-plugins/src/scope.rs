//! Path Scope Security System
//!
//! Provides scope-based security for file system operations.
//! Similar to Tauri's scope system, paths must be explicitly allowed.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Scope configuration error
#[derive(Debug, Error)]
pub enum ScopeError {
    /// Path is not allowed
    #[error("Path '{0}' is not in allowed scope")]
    NotAllowed(String),

    /// Path canonicalization failed
    #[error("Failed to resolve path: {0}")]
    ResolveFailed(String),
}

/// Path scope for access control
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PathScope {
    /// Allowed paths (can be files or directories)
    #[serde(default)]
    pub allowed: Vec<PathBuf>,

    /// Denied paths (take precedence over allowed)
    #[serde(default)]
    pub denied: Vec<PathBuf>,

    /// Allow all paths (dangerous, use with caution)
    #[serde(default)]
    pub allow_all: bool,
}

impl PathScope {
    /// Create a new empty scope (blocks all)
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a scope that allows all paths
    pub fn allow_all() -> Self {
        Self {
            allow_all: true,
            ..Default::default()
        }
    }

    /// Add an allowed path
    pub fn allow(mut self, path: impl AsRef<Path>) -> Self {
        self.allowed.push(path.as_ref().to_path_buf());
        self
    }

    /// Add multiple allowed paths
    pub fn allow_many(mut self, paths: &[impl AsRef<Path>]) -> Self {
        for path in paths {
            self.allowed.push(path.as_ref().to_path_buf());
        }
        self
    }

    /// Add a denied path
    pub fn deny(mut self, path: impl AsRef<Path>) -> Self {
        self.denied.push(path.as_ref().to_path_buf());
        self
    }

    /// Add multiple denied paths
    pub fn deny_many(mut self, paths: &[impl AsRef<Path>]) -> Self {
        for path in paths {
            self.denied.push(path.as_ref().to_path_buf());
        }
        self
    }

    /// Check if a path is allowed
    pub fn is_allowed(&self, path: impl AsRef<Path>) -> Result<PathBuf, ScopeError> {
        let path = path.as_ref();

        // Canonicalize the path if it exists, otherwise use cleaned path
        let canonical = if path.exists() {
            dunce::canonicalize(path).map_err(|e| ScopeError::ResolveFailed(e.to_string()))?
        } else {
            // For non-existent paths, try to canonicalize parent
            if let Some(parent) = path.parent() {
                if parent.exists() {
                    let canonical_parent = dunce::canonicalize(parent)
                        .map_err(|e| ScopeError::ResolveFailed(e.to_string()))?;
                    if let Some(file_name) = path.file_name() {
                        canonical_parent.join(file_name)
                    } else {
                        return Err(ScopeError::ResolveFailed("Invalid path".to_string()));
                    }
                } else {
                    path.to_path_buf()
                }
            } else {
                path.to_path_buf()
            }
        };

        // Allow all mode
        if self.allow_all {
            // Still check denied list
            if self.is_path_denied(&canonical) {
                return Err(ScopeError::NotAllowed(path.display().to_string()));
            }
            return Ok(canonical);
        }

        // Check denied list first (takes precedence)
        if self.is_path_denied(&canonical) {
            return Err(ScopeError::NotAllowed(path.display().to_string()));
        }

        // Check allowed list
        if self.is_path_in_list(&canonical, &self.allowed) {
            return Ok(canonical);
        }

        Err(ScopeError::NotAllowed(path.display().to_string()))
    }

    fn is_path_denied(&self, path: &Path) -> bool {
        self.is_path_in_list(path, &self.denied)
    }

    fn is_path_in_list(&self, path: &Path, list: &[PathBuf]) -> bool {
        for allowed_path in list {
            // Canonicalize allowed path if possible
            let canonical_allowed = if allowed_path.exists() {
                dunce::canonicalize(allowed_path).unwrap_or_else(|_| allowed_path.clone())
            } else {
                allowed_path.clone()
            };

            // Check if path starts with allowed path (directory)
            if path.starts_with(&canonical_allowed) {
                return true;
            }

            // Check exact match
            if path == canonical_allowed {
                return true;
            }
        }
        false
    }
}

/// Global scope configuration for all plugins
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScopeConfig {
    /// File system scope
    #[serde(default)]
    pub fs: PathScope,

    /// Shell command scope (allowed commands/programs)
    #[serde(default)]
    pub shell: ShellScope,

    /// Enable/disable individual plugins
    #[serde(default)]
    pub enabled_plugins: HashSet<String>,
}

impl ScopeConfig {
    /// Create a new scope configuration with default plugins enabled
    pub fn new() -> Self {
        let mut config = Self::default();
        // Enable default plugins
        config.enabled_plugins.insert("fs".to_string());
        config.enabled_plugins.insert("clipboard".to_string());
        config.enabled_plugins.insert("shell".to_string());
        config.enabled_plugins.insert("dialog".to_string());
        config.enabled_plugins.insert("process".to_string());
        config
    }

    /// Create a permissive configuration (allow all)
    pub fn permissive() -> Self {
        let mut config = Self::new();
        config.fs = PathScope::allow_all();
        config.shell = ShellScope::allow_all();
        config
    }

    /// Check if a plugin is enabled
    pub fn is_plugin_enabled(&self, name: &str) -> bool {
        self.enabled_plugins.contains(name)
    }

    /// Enable a plugin
    pub fn enable_plugin(&mut self, name: impl Into<String>) {
        self.enabled_plugins.insert(name.into());
    }

    /// Disable a plugin
    pub fn disable_plugin(&mut self, name: &str) {
        self.enabled_plugins.remove(name);
    }

    /// Set the file system scope
    pub fn with_fs_scope(mut self, scope: PathScope) -> Self {
        self.fs = scope;
        self
    }

    /// Set the shell scope
    pub fn with_shell_scope(mut self, scope: ShellScope) -> Self {
        self.shell = scope;
        self
    }
}

/// Shell command scope for controlling which commands can be executed
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShellScope {
    /// Allowed commands/programs (by name or path)
    #[serde(default)]
    pub allowed_commands: Vec<String>,

    /// Denied commands (take precedence over allowed)
    #[serde(default)]
    pub denied_commands: Vec<String>,

    /// Allow all commands (dangerous!)
    #[serde(default)]
    pub allow_all: bool,

    /// Allow opening URLs with default browser
    #[serde(default = "default_true")]
    pub allow_open_url: bool,

    /// Allow opening files with default application
    #[serde(default = "default_true")]
    pub allow_open_file: bool,
}

fn default_true() -> bool {
    true
}

impl ShellScope {
    /// Create a new empty scope (blocks all commands)
    pub fn new() -> Self {
        Self {
            allow_open_url: true,
            allow_open_file: true,
            ..Default::default()
        }
    }

    /// Create a scope that allows all commands
    pub fn allow_all() -> Self {
        Self {
            allow_all: true,
            allow_open_url: true,
            allow_open_file: true,
            ..Default::default()
        }
    }

    /// Add an allowed command
    pub fn allow_command(mut self, cmd: impl Into<String>) -> Self {
        self.allowed_commands.push(cmd.into());
        self
    }

    /// Add a denied command
    pub fn deny_command(mut self, cmd: impl Into<String>) -> Self {
        self.denied_commands.push(cmd.into());
        self
    }

    /// Check if a command is allowed
    pub fn is_command_allowed(&self, cmd: &str) -> bool {
        // Check denied list first
        if self.denied_commands.iter().any(|c| c == cmd) {
            return false;
        }

        // Allow all mode
        if self.allow_all {
            return true;
        }

        // Check allowed list
        self.allowed_commands.iter().any(|c| c == cmd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_scope_allow_all() {
        let scope = PathScope::allow_all();
        let temp = tempdir().unwrap();
        let result = scope.is_allowed(temp.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_scope_deny() {
        let temp = tempdir().unwrap();
        let scope = PathScope::allow_all().deny(temp.path());
        let result = scope.is_allowed(temp.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_scope_allow_specific() {
        let temp = tempdir().unwrap();
        let scope = PathScope::new().allow(temp.path());

        // Allowed path
        let result = scope.is_allowed(temp.path());
        assert!(result.is_ok());

        // Subdirectory should also be allowed
        let subdir = temp.path().join("subdir");
        std::fs::create_dir(&subdir).unwrap();
        let result = scope.is_allowed(&subdir);
        assert!(result.is_ok());
    }

    #[test]
    fn test_scope_block_by_default() {
        let temp = tempdir().unwrap();
        let scope = PathScope::new();
        let result = scope.is_allowed(temp.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_scope_config_default() {
        let config = ScopeConfig::new();
        assert!(config.is_plugin_enabled("fs"));
        assert!(config.is_plugin_enabled("clipboard"));
        assert!(config.is_plugin_enabled("shell"));
        assert!(config.is_plugin_enabled("dialog"));
        assert!(config.is_plugin_enabled("process"));
    }

    #[test]
    fn test_shell_scope() {
        let scope = ShellScope::new().allow_command("git").allow_command("npm");

        assert!(scope.is_command_allowed("git"));
        assert!(scope.is_command_allowed("npm"));
        assert!(!scope.is_command_allowed("rm"));
    }

    #[test]
    fn test_shell_scope_deny() {
        let scope = ShellScope::allow_all().deny_command("rm");

        assert!(scope.is_command_allowed("git"));
        assert!(!scope.is_command_allowed("rm"));
    }
}
