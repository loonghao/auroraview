//! Unit tests for scope module
//!
//! Tests for PathScope and ShellScope security systems.

use std::sync::Arc;

use auroraview_plugins::{PathScope, ScopeConfig};
use rstest::*;
use tempfile::tempdir;

// ===========================================================================
// PathScope – original tests
// ===========================================================================

#[test]
fn scope_allow_all() {
    let scope = PathScope::allow_all();
    let temp = tempdir().unwrap();
    let result = scope.is_allowed(temp.path());
    assert!(result.is_ok());
}

#[test]
fn scope_deny() {
    let temp = tempdir().unwrap();
    let scope = PathScope::allow_all().deny(temp.path());
    let result = scope.is_allowed(temp.path());
    assert!(result.is_err());
}

#[test]
fn scope_allow_specific() {
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
fn scope_block_by_default() {
    let temp = tempdir().unwrap();
    let scope = PathScope::new();
    let result = scope.is_allowed(temp.path());
    assert!(result.is_err());
}

#[test]
fn scope_config_default() {
    let config = ScopeConfig::new();
    assert!(config.is_plugin_enabled("fs"));
    assert!(config.is_plugin_enabled("clipboard"));
    assert!(config.is_plugin_enabled("shell"));
    assert!(config.is_plugin_enabled("dialog"));
    assert!(config.is_plugin_enabled("process"));
}

#[test]
fn scope_config_permissive() {
    let config = ScopeConfig::permissive();
    assert!(config.fs.allow_all);
    assert!(config.shell.allow_all);
}

#[test]
fn scope_config_enable_disable_plugin() {
    let mut config = ScopeConfig::new();
    assert!(config.is_plugin_enabled("fs"));

    config.disable_plugin("fs");
    assert!(!config.is_plugin_enabled("fs"));

    config.enable_plugin("fs");
    assert!(config.is_plugin_enabled("fs"));
}

#[test]
fn scope_allow_many() {
    let temp1 = tempdir().unwrap();
    let temp2 = tempdir().unwrap();

    let scope = PathScope::new().allow_many(&[temp1.path(), temp2.path()]);

    assert!(scope.is_allowed(temp1.path()).is_ok());
    assert!(scope.is_allowed(temp2.path()).is_ok());
}

#[test]
fn scope_deny_many() {
    let temp1 = tempdir().unwrap();
    let temp2 = tempdir().unwrap();

    let scope = PathScope::allow_all().deny_many(&[temp1.path(), temp2.path()]);

    assert!(scope.is_allowed(temp1.path()).is_err());
    assert!(scope.is_allowed(temp2.path()).is_err());
}

// ===========================================================================
// PathScope – new edge cases
// ===========================================================================

// Deny takes precedence over allow (same path)
#[test]
fn scope_deny_overrides_allow_same_path() {
    let temp = tempdir().unwrap();
    let scope = PathScope::new().allow(temp.path()).deny(temp.path());
    // deny wins
    assert!(scope.is_allowed(temp.path()).is_err());
}

// allow_all + deny subdir
#[test]
fn scope_allow_all_deny_subdir_blocks_subdir() {
    let temp = tempdir().unwrap();
    let subdir = temp.path().join("secret");
    std::fs::create_dir(&subdir).unwrap();

    let scope = PathScope::allow_all().deny(&subdir);
    assert!(scope.is_allowed(temp.path()).is_ok());
    assert!(scope.is_allowed(&subdir).is_err());
}

// deep subdirectory under allowed root
#[test]
fn scope_deep_subdir_is_allowed_under_allowed_root() {
    let temp = tempdir().unwrap();
    let deep = temp.path().join("a").join("b").join("c");
    std::fs::create_dir_all(&deep).unwrap();

    let scope = PathScope::new().allow(temp.path());
    assert!(scope.is_allowed(&deep).is_ok());
}

// deep subdir under denied path is blocked
#[test]
fn scope_deep_subdir_is_blocked_when_root_is_denied() {
    let temp = tempdir().unwrap();
    let subdir = temp.path().join("blocked");
    let deep = subdir.join("deep").join("file.txt");
    std::fs::create_dir_all(subdir.join("deep")).unwrap();
    std::fs::write(&deep, b"content").unwrap();

    let scope = PathScope::allow_all().deny(&subdir);
    assert!(scope.is_allowed(&deep).is_err());
}

// non-existent file under allowed directory is allowed
#[test]
fn scope_nonexistent_file_under_allowed_dir_ok() {
    let temp = tempdir().unwrap();
    let scope = PathScope::new().allow(temp.path());
    let nonexist = temp.path().join("does_not_exist.txt");
    // parent exists, so canonicalization of parent + filename is used
    assert!(scope.is_allowed(&nonexist).is_ok());
}

// non-existent path outside allowed dir is blocked
#[test]
fn scope_nonexistent_file_outside_allowed_dir_blocked() {
    let allowed = tempdir().unwrap();
    let other = tempdir().unwrap();
    let scope = PathScope::new().allow(allowed.path());
    let outside = other.path().join("secret.txt");
    assert!(scope.is_allowed(&outside).is_err());
}

// empty scope (no allowed, no denied) blocks everything
#[test]
fn scope_empty_blocks_all_paths() {
    let scope = PathScope::new();
    let temp = tempdir().unwrap();
    assert!(scope.is_allowed(temp.path()).is_err());

    let file = temp.path().join("f.txt");
    std::fs::write(&file, b"x").unwrap();
    assert!(scope.is_allowed(&file).is_err());
}

// multiple dirs: allow only one, other is blocked
#[test]
fn scope_only_one_of_two_dirs_allowed() {
    let allowed = tempdir().unwrap();
    let blocked = tempdir().unwrap();

    let scope = PathScope::new().allow(allowed.path());
    assert!(scope.is_allowed(allowed.path()).is_ok());
    assert!(scope.is_allowed(blocked.path()).is_err());
}

// allow one path, deny its parent — deny wins
#[test]
fn scope_allow_child_deny_parent_child_is_blocked() {
    let temp = tempdir().unwrap();
    let child = temp.path().join("child");
    std::fs::create_dir(&child).unwrap();

    // allow child, but deny parent — deny should win because child starts_with parent
    let scope = PathScope::new().allow(&child).deny(temp.path());
    assert!(scope.is_allowed(&child).is_err());
}

// allowed paths fields are preserved
#[test]
fn scope_allowed_field_contains_added_paths() {
    let temp = tempdir().unwrap();
    let scope = PathScope::new().allow(temp.path());
    assert_eq!(scope.allowed.len(), 1);
    assert_eq!(scope.denied.len(), 0);
}

// allow_all field is true
#[test]
fn scope_allow_all_flag_is_set() {
    let scope = PathScope::allow_all();
    assert!(scope.allow_all);
}

// deny list field check
#[test]
fn scope_deny_field_contains_added_paths() {
    let temp = tempdir().unwrap();
    let scope = PathScope::allow_all().deny(temp.path());
    assert_eq!(scope.denied.len(), 1);
}

// serde roundtrip for PathScope
#[test]
fn scope_serde_roundtrip() {
    let temp = tempdir().unwrap();
    let scope = PathScope::new().allow(temp.path());
    let json = serde_json::to_string(&scope).unwrap();
    let restored: PathScope = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.allowed.len(), 1);
    assert!(!restored.allow_all);
}

// serde roundtrip allow_all=true
#[test]
fn scope_allow_all_serde_roundtrip() {
    let scope = PathScope::allow_all();
    let json = serde_json::to_string(&scope).unwrap();
    let restored: PathScope = serde_json::from_str(&json).unwrap();
    assert!(restored.allow_all);
}

// ScopeConfig: unknown plugin not in default set → not enabled
#[test]
fn scope_config_unknown_plugin_not_enabled() {
    let config = ScopeConfig::new();
    // plugins not in the default enabled set are not enabled
    assert!(!config.is_plugin_enabled("unknown_plugin"));
}

// ScopeConfig: disable multiple plugins
#[test]
fn scope_config_disable_multiple_plugins() {
    let mut config = ScopeConfig::new();
    config.disable_plugin("fs");
    config.disable_plugin("clipboard");

    assert!(!config.is_plugin_enabled("fs"));
    assert!(!config.is_plugin_enabled("clipboard"));
    assert!(config.is_plugin_enabled("shell"));
}

// ScopeConfig: re-enable after disable
#[test]
fn scope_config_re_enable_after_disable() {
    let mut config = ScopeConfig::new();
    config.disable_plugin("shell");
    assert!(!config.is_plugin_enabled("shell"));

    config.enable_plugin("shell");
    assert!(config.is_plugin_enabled("shell"));
}

// ScopeConfig: default not permissive
#[test]
fn scope_config_default_not_allow_all() {
    let config = ScopeConfig::new();
    assert!(!config.fs.allow_all);
    assert!(!config.shell.allow_all);
}

// Concurrent PathScope reads (Arc-wrapped)
#[test]
fn scope_concurrent_reads_no_panic() {
    let temp = tempdir().unwrap();
    let subdir = temp.path().join("shared");
    std::fs::create_dir(&subdir).unwrap();

    let scope = Arc::new(PathScope::new().allow(temp.path()));

    let handles: Vec<_> = (0..8)
        .map(|_| {
            let scope = Arc::clone(&scope);
            let path = subdir.clone();
            std::thread::spawn(move || {
                for _ in 0..100 {
                    let _ = scope.is_allowed(&path);
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }
}

// rstest: allow_all allows various path types
#[rstest]
#[case("subdir")]
#[case("subdir/nested")]
#[case("file.txt")]
fn scope_allow_all_permits_any_subpath(#[case] rel: &str) {
    let temp = tempdir().unwrap();
    let full = temp.path().join(rel);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    if rel.ends_with(".txt") {
        std::fs::write(&full, b"x").ok();
    } else {
        std::fs::create_dir_all(&full).ok();
    }

    let scope = PathScope::allow_all();
    assert!(
        scope.is_allowed(&full).is_ok(),
        "Expected {rel} to be allowed"
    );
}

// rstest: specific paths blocked by empty scope
#[rstest]
#[case("blocked.txt")]
#[case("a/b/c.dat")]
fn scope_empty_blocks_specific_paths(#[case] rel: &str) {
    let temp = tempdir().unwrap();
    let full = temp.path().join(rel);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    std::fs::write(&full, b"data").ok();

    let scope = PathScope::new();
    assert!(
        scope.is_allowed(&full).is_err(),
        "Expected {rel} to be blocked"
    );
}

// ===========================================================================
// Shell scope tests
// ===========================================================================

mod shell_scope {
    use auroraview_plugins::ShellScope;
    use rstest::*;

    #[test]
    fn shell_scope_new() {
        let scope = ShellScope::new();
        assert!(!scope.allow_all);
        assert!(scope.allow_open_url);
        assert!(scope.allow_open_file);
    }

    #[test]
    fn shell_scope_allow_all() {
        let scope = ShellScope::allow_all();
        assert!(scope.allow_all);
        assert!(scope.is_command_allowed("any_command"));
    }

    #[test]
    fn shell_scope_allow_command() {
        let scope = ShellScope::new().allow_command("git").allow_command("npm");

        assert!(scope.is_command_allowed("git"));
        assert!(scope.is_command_allowed("npm"));
        assert!(!scope.is_command_allowed("rm"));
    }

    #[test]
    fn shell_scope_deny_command() {
        let scope = ShellScope::allow_all().deny_command("rm");

        assert!(scope.is_command_allowed("git"));
        assert!(!scope.is_command_allowed("rm"));
    }

    #[test]
    fn shell_scope_deny_takes_precedence() {
        let scope = ShellScope::new().allow_command("rm").deny_command("rm");

        assert!(!scope.is_command_allowed("rm"));
    }

    // New: empty scope blocks all commands
    #[test]
    fn shell_scope_empty_blocks_all() {
        let scope = ShellScope::new();
        assert!(!scope.is_command_allowed("ls"));
        assert!(!scope.is_command_allowed("git"));
    }

    // New: allow_all + deny specific
    #[test]
    fn shell_scope_allow_all_deny_specific_blocked() {
        let scope = ShellScope::allow_all()
            .deny_command("rm")
            .deny_command("sudo");
        assert!(!scope.is_command_allowed("rm"));
        assert!(!scope.is_command_allowed("sudo"));
        assert!(scope.is_command_allowed("git"));
    }

    // New: case-sensitive command matching
    #[test]
    fn shell_scope_case_sensitive_commands() {
        let scope = ShellScope::new().allow_command("Git");
        assert!(scope.is_command_allowed("Git"));
        // "git" != "Git" — case sensitive
        assert!(!scope.is_command_allowed("git"));
    }

    // New: multiple allow + deny interactions
    #[test]
    fn shell_scope_multiple_allowed_commands() {
        let scope = ShellScope::new()
            .allow_command("cargo")
            .allow_command("rustfmt")
            .allow_command("clippy");

        assert!(scope.is_command_allowed("cargo"));
        assert!(scope.is_command_allowed("rustfmt"));
        assert!(scope.is_command_allowed("clippy"));
        assert!(!scope.is_command_allowed("npm"));
    }

    // New: rstest parametric command allow
    #[rstest]
    #[case("git", true)]
    #[case("npm", true)]
    #[case("rm", false)]
    #[case("sudo", false)]
    fn shell_scope_parametric_allow(#[case] cmd: &str, #[case] expected: bool) {
        let scope = ShellScope::new().allow_command("git").allow_command("npm");
        assert_eq!(scope.is_command_allowed(cmd), expected);
    }

    // New: serde roundtrip for ShellScope
    #[test]
    fn shell_scope_serde_roundtrip() {
        let scope = ShellScope::new().allow_command("git").deny_command("rm");
        let json = serde_json::to_string(&scope).unwrap();
        let restored: ShellScope = serde_json::from_str(&json).unwrap();
        assert!(!restored.allow_all);
        assert!(restored.is_command_allowed("git"));
        assert!(!restored.is_command_allowed("rm"));
    }
}
