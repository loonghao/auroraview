//! Tests for PathScope, ShellScope, ScopeConfig, and ScopeError

use auroraview_plugin_core::{PathScope, ScopeConfig, ScopeError, ShellScope};
use rstest::rstest;
use std::path::PathBuf;
use tempfile::TempDir;

// ── ScopeError Display ────────────────────────────────────────────────────────

#[test]
fn scope_error_not_allowed_display() {
    let err = ScopeError::NotAllowed("/tmp/secret".to_string());
    let s = format!("{}", err);
    assert!(s.contains("/tmp/secret"));
}

#[test]
fn scope_error_resolve_failed_display() {
    let err = ScopeError::ResolveFailed("bad path".to_string());
    let s = format!("{}", err);
    assert!(s.contains("bad path"));
}

#[test]
fn scope_error_debug() {
    let err = ScopeError::NotAllowed("x".to_string());
    let s = format!("{:?}", err);
    assert!(!s.is_empty());
}

// ── PathScope::new / default ──────────────────────────────────────────────────

#[test]
fn path_scope_new_blocks_all() {
    let scope = PathScope::new();
    let tmp = TempDir::new().unwrap();
    let result = scope.is_allowed(tmp.path());
    assert!(result.is_err());
}

#[test]
fn path_scope_default_blocks_all() {
    let scope = PathScope::default();
    let tmp = TempDir::new().unwrap();
    assert!(scope.is_allowed(tmp.path()).is_err());
}

// ── PathScope::allow_all ──────────────────────────────────────────────────────

#[test]
fn path_scope_allow_all_permits_existing() {
    let scope = PathScope::allow_all();
    let tmp = TempDir::new().unwrap();
    assert!(scope.is_allowed(tmp.path()).is_ok());
}

// ── PathScope::allow + is_allowed ────────────────────────────────────────────

#[test]
fn path_scope_allow_permits_directory() {
    let tmp = TempDir::new().unwrap();
    let scope = PathScope::new().allow(tmp.path());
    assert!(scope.is_allowed(tmp.path()).is_ok());
}

#[test]
fn path_scope_allow_permits_child_file() {
    let tmp = TempDir::new().unwrap();
    let file = tmp.path().join("data.txt");
    std::fs::write(&file, b"hello").unwrap();
    let scope = PathScope::new().allow(tmp.path());
    assert!(scope.is_allowed(&file).is_ok());
}

#[test]
fn path_scope_blocks_unallowed_path() {
    let allowed = TempDir::new().unwrap();
    let other = TempDir::new().unwrap();
    let scope = PathScope::new().allow(allowed.path());
    assert!(scope.is_allowed(other.path()).is_err());
}

// ── PathScope::deny ───────────────────────────────────────────────────────────

#[test]
fn path_scope_deny_blocks_subpath_in_allow_all() {
    let tmp = TempDir::new().unwrap();
    let secret = tmp.path().join("secret");
    std::fs::create_dir_all(&secret).unwrap();
    let scope = PathScope::allow_all().deny(&secret);
    assert!(scope.is_allowed(&secret).is_err());
}

#[test]
fn path_scope_deny_blocks_explicitly_allowed() {
    let tmp = TempDir::new().unwrap();
    let scope = PathScope::new().allow(tmp.path()).deny(tmp.path());
    assert!(scope.is_allowed(tmp.path()).is_err());
}

// ── PathScope::allow_many / deny_many ────────────────────────────────────────

#[test]
fn path_scope_allow_many() {
    let d1 = TempDir::new().unwrap();
    let d2 = TempDir::new().unwrap();
    let scope = PathScope::new().allow_many(&[d1.path(), d2.path()]);
    assert!(scope.is_allowed(d1.path()).is_ok());
    assert!(scope.is_allowed(d2.path()).is_ok());
}

#[test]
fn path_scope_deny_many() {
    let d1 = TempDir::new().unwrap();
    let d2 = TempDir::new().unwrap();
    let scope = PathScope::allow_all().deny_many(&[d1.path(), d2.path()]);
    assert!(scope.is_allowed(d1.path()).is_err());
    assert!(scope.is_allowed(d2.path()).is_err());
}

// ── PathScope with non-existent paths ────────────────────────────────────────

#[test]
fn path_scope_nonexistent_child_of_allowed_dir() {
    let tmp = TempDir::new().unwrap();
    let scope = PathScope::new().allow(tmp.path());
    let nonexistent = tmp.path().join("does_not_exist.txt");
    // Parent exists and is allowed, so child should be permitted
    assert!(scope.is_allowed(&nonexistent).is_ok());
}

// ── ShellScope::new (blocks all commands) ─────────────────────────────────────

#[test]
fn shell_scope_new_blocks_commands() {
    let scope = ShellScope::new();
    assert!(!scope.is_command_allowed("ls"));
    assert!(!scope.is_command_allowed("echo"));
}

#[test]
fn shell_scope_new_allows_open() {
    let scope = ShellScope::new();
    assert!(scope.allow_open_url);
    assert!(scope.allow_open_file);
}

// ── ShellScope::allow_all ────────────────────────────────────────────────────

#[test]
fn shell_scope_allow_all_permits_any_command() {
    let scope = ShellScope::allow_all();
    assert!(scope.is_command_allowed("rm"));
    assert!(scope.is_command_allowed("arbitrary_cmd_xyz"));
}

// ── ShellScope::allow_command ────────────────────────────────────────────────

#[test]
fn shell_scope_allow_command_permits_only_that_command() {
    let scope = ShellScope::new().allow_command("echo");
    assert!(scope.is_command_allowed("echo"));
    assert!(!scope.is_command_allowed("ls"));
}

#[rstest]
#[case("git")]
#[case("python")]
#[case("cargo")]
fn shell_scope_allow_specific_commands(#[case] cmd: &str) {
    let scope = ShellScope::new().allow_command(cmd);
    assert!(scope.is_command_allowed(cmd));
    assert!(!scope.is_command_allowed("other"));
}

// ── ShellScope::deny_command (takes precedence) ───────────────────────────────

#[test]
fn shell_scope_deny_overrides_allow_all() {
    let scope = ShellScope::allow_all().deny_command("rm");
    assert!(!scope.is_command_allowed("rm"));
    assert!(scope.is_command_allowed("ls"));
}

#[test]
fn shell_scope_deny_overrides_explicit_allow() {
    let scope = ShellScope::new().allow_command("git").deny_command("git");
    assert!(!scope.is_command_allowed("git"));
}

// ── ScopeConfig::new ──────────────────────────────────────────────────────────

#[test]
fn scope_config_new_enables_default_plugins() {
    let config = ScopeConfig::new();
    for plugin in &[
        "fs",
        "clipboard",
        "shell",
        "dialog",
        "process",
        "browser_bridge",
        "extensions",
    ] {
        assert!(
            config.is_plugin_enabled(plugin),
            "{} should be enabled",
            plugin
        );
    }
}

#[test]
fn scope_config_new_unknown_plugin_disabled() {
    let config = ScopeConfig::new();
    assert!(!config.is_plugin_enabled("nonexistent_plugin_xyz"));
}

// ── ScopeConfig::permissive ───────────────────────────────────────────────────

#[test]
fn scope_config_permissive_allows_commands() {
    let config = ScopeConfig::permissive();
    assert!(config.shell.is_command_allowed("any_command"));
}

// ── ScopeConfig enable/disable plugin ────────────────────────────────────────

#[test]
fn scope_config_enable_plugin() {
    let mut config = ScopeConfig::default();
    config.enable_plugin("custom_plugin");
    assert!(config.is_plugin_enabled("custom_plugin"));
}

#[test]
fn scope_config_disable_plugin() {
    let mut config = ScopeConfig::new();
    assert!(config.is_plugin_enabled("fs"));
    config.disable_plugin("fs");
    assert!(!config.is_plugin_enabled("fs"));
}

// ── ScopeConfig builder methods ───────────────────────────────────────────────

#[test]
fn scope_config_with_fs_scope() {
    let tmp = TempDir::new().unwrap();
    let fs_scope = PathScope::new().allow(tmp.path());
    let config = ScopeConfig::new().with_fs_scope(fs_scope);
    assert!(config.fs.is_allowed(tmp.path()).is_ok());
}

#[test]
fn scope_config_with_shell_scope() {
    let shell_scope = ShellScope::new().allow_command("git");
    let config = ScopeConfig::new().with_shell_scope(shell_scope);
    assert!(config.shell.is_command_allowed("git"));
    assert!(!config.shell.is_command_allowed("rm"));
}

// ── ScopeConfig default ───────────────────────────────────────────────────────

#[test]
fn scope_config_default_no_plugins_enabled() {
    let config = ScopeConfig::default();
    assert!(!config.is_plugin_enabled("fs"));
}

// ── PathScope returns canonical path on success ───────────────────────────────

#[test]
fn path_scope_returns_canonical_path() {
    let tmp = TempDir::new().unwrap();
    let scope = PathScope::new().allow(tmp.path());
    let result = scope.is_allowed(tmp.path());
    assert!(result.is_ok());
    let canonical: PathBuf = result.unwrap();
    // Canonical path should be non-empty
    assert!(!canonical.as_os_str().is_empty());
}
