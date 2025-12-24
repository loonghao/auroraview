//! Unit tests for scope module
//!
//! Tests for PathScope and ShellScope security systems.

use auroraview_plugins::{PathScope, ScopeConfig};
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
fn test_scope_config_permissive() {
    let config = ScopeConfig::permissive();
    assert!(config.fs.allow_all);
    assert!(config.shell.allow_all);
}

#[test]
fn test_scope_config_enable_disable_plugin() {
    let mut config = ScopeConfig::new();
    assert!(config.is_plugin_enabled("fs"));

    config.disable_plugin("fs");
    assert!(!config.is_plugin_enabled("fs"));

    config.enable_plugin("fs");
    assert!(config.is_plugin_enabled("fs"));
}

#[test]
fn test_scope_allow_many() {
    let temp1 = tempdir().unwrap();
    let temp2 = tempdir().unwrap();

    let scope = PathScope::new().allow_many(&[temp1.path(), temp2.path()]);

    assert!(scope.is_allowed(temp1.path()).is_ok());
    assert!(scope.is_allowed(temp2.path()).is_ok());
}

#[test]
fn test_scope_deny_many() {
    let temp1 = tempdir().unwrap();
    let temp2 = tempdir().unwrap();

    let scope = PathScope::allow_all().deny_many(&[temp1.path(), temp2.path()]);

    assert!(scope.is_allowed(temp1.path()).is_err());
    assert!(scope.is_allowed(temp2.path()).is_err());
}

// Shell scope tests
mod shell_scope {
    use auroraview_plugins::ShellScope;

    #[test]
    fn test_shell_scope_new() {
        let scope = ShellScope::new();
        assert!(!scope.allow_all);
        assert!(scope.allow_open_url);
        assert!(scope.allow_open_file);
    }

    #[test]
    fn test_shell_scope_allow_all() {
        let scope = ShellScope::allow_all();
        assert!(scope.allow_all);
        assert!(scope.is_command_allowed("any_command"));
    }

    #[test]
    fn test_shell_scope_allow_command() {
        let scope = ShellScope::new().allow_command("git").allow_command("npm");

        assert!(scope.is_command_allowed("git"));
        assert!(scope.is_command_allowed("npm"));
        assert!(!scope.is_command_allowed("rm"));
    }

    #[test]
    fn test_shell_scope_deny_command() {
        let scope = ShellScope::allow_all().deny_command("rm");

        assert!(scope.is_command_allowed("git"));
        assert!(!scope.is_command_allowed("rm"));
    }

    #[test]
    fn test_shell_scope_deny_takes_precedence() {
        let scope = ShellScope::new().allow_command("rm").deny_command("rm");

        assert!(!scope.is_command_allowed("rm"));
    }
}
