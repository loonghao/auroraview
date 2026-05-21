//! Regression tests for `AURORAVIEW_CAPTURE_FILE_DROP` env-var override
//! (RFC 0015 §4.3).
//!
//! Behaviour matrix:
//!
//! | Env var value     | Result                                  | Logging   |
//! |-------------------|------------------------------------------|-----------|
//! | unset             | overlay value (passthrough)              | none      |
//! | recognized true   | `true`                                   | info!     |
//! | recognized false  | `false`                                  | info!     |
//! | invalid literal   | overlay value (fallback)                 | warn!     |
//!
//! Recognized literals (case-insensitive, trimmed):
//!   - true:  `1` / `true` / `on` / `yes` / `enabled`
//!   - false: `0` / `false` / `off` / `no` / `disabled`
//!
//! Tests serialize on a global mutex because `std::env::set_var` is
//! process-global and racing tests would corrupt each other's reads.

use auroraview_cli::packed::resolve_packed_capture_file_drop;
use std::sync::Mutex;

const ENV_VAR: &str = "AURORAVIEW_CAPTURE_FILE_DROP";

// All env-var manipulation tests share one mutex to avoid the race
// described in https://doc.rust-lang.org/std/env/fn.set_var.html .
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Helper that sets the env var, runs `f`, then restores prior state.
fn with_env<F: FnOnce() -> R, R>(value: Option<&str>, f: F) -> R {
    let _guard = ENV_LOCK.lock().unwrap();
    let previous = std::env::var(ENV_VAR).ok();

    match value {
        Some(v) => std::env::set_var(ENV_VAR, v),
        None => std::env::remove_var(ENV_VAR),
    }

    let result = f();

    // Restore previous state so unrelated tests are not affected.
    match previous {
        Some(v) => std::env::set_var(ENV_VAR, v),
        None => std::env::remove_var(ENV_VAR),
    }

    result
}

#[test]
fn env_var_unset_returns_overlay_value() {
    with_env(None, || {
        assert!(!resolve_packed_capture_file_drop(false));
        assert!(resolve_packed_capture_file_drop(true));
    });
}

#[test]
fn env_var_truthy_literals_force_enable() {
    for literal in ["1", "true", "on", "yes", "enabled", "TRUE", "On", "Yes"] {
        with_env(Some(literal), || {
            assert!(
                resolve_packed_capture_file_drop(false),
                "literal {:?} should force-enable even when overlay=false",
                literal
            );
            assert!(
                resolve_packed_capture_file_drop(true),
                "literal {:?} should keep enabled when overlay=true",
                literal
            );
        });
    }
}

#[test]
fn env_var_falsy_literals_force_disable() {
    for literal in ["0", "false", "off", "no", "disabled", "FALSE", "Off", "No"] {
        with_env(Some(literal), || {
            assert!(
                !resolve_packed_capture_file_drop(true),
                "literal {:?} should force-disable even when overlay=true",
                literal
            );
            assert!(
                !resolve_packed_capture_file_drop(false),
                "literal {:?} should keep disabled when overlay=false",
                literal
            );
        });
    }
}

#[test]
fn env_var_invalid_literal_falls_back_to_overlay() {
    for literal in ["hello", "", "  ", "maybe", "2"] {
        with_env(Some(literal), || {
            assert!(
                !resolve_packed_capture_file_drop(false),
                "literal {:?} (invalid) should fall back to overlay=false",
                literal
            );
            assert!(
                resolve_packed_capture_file_drop(true),
                "literal {:?} (invalid) should fall back to overlay=true",
                literal
            );
        });
    }
}

#[test]
fn env_var_trims_whitespace() {
    with_env(Some("  true  "), || {
        assert!(resolve_packed_capture_file_drop(false));
    });
    with_env(Some("\tfalse\n"), || {
        assert!(!resolve_packed_capture_file_drop(true));
    });
}
