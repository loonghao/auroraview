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
//!
//! # Rust 2024 Edition Migration
//!
//! In Rust 2024 edition, `std::env::set_var` and `std::env::remove_var`
//! become `unsafe fn` (stabilized via rust-lang/rust#27970). The `unsafe`
//! blocks below are forward-compatible: they compile on both 2021 and
//! 2024 editions. The `Mutex` serialization satisfies the safety
//! requirement that no other thread concurrently reads the env.
//! When migrating to edition 2024, simply remove the
//! `#[allow(unused_unsafe)]` attribute.

// In Rust 2021, `set_var`/`remove_var` are safe, so the `unsafe` blocks
// below produce an "unnecessary unsafe" warning. Suppress it — when
// migrating to edition 2024 the `unsafe` becomes required.
#![allow(unused_unsafe)]

use auroraview_cli::packed::resolve_packed_capture_file_drop;
use std::sync::Mutex;

const ENV_VAR: &str = "AURORAVIEW_CAPTURE_FILE_DROP";

// All env-var manipulation tests share one mutex to avoid the race
// described in https://doc.rust-lang.org/std/env/fn.set_var.html .
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Helper that sets the env var, runs `f`, then restores prior state.
///
/// # Safety (edition 2024)
///
/// `set_var` / `remove_var` are `unsafe` in edition 2024 because
/// concurrent reads from other threads are UB. We hold `ENV_LOCK` for
/// the entire scope, and no other code in this test binary reads
/// `AURORAVIEW_CAPTURE_FILE_DROP` outside this lock.
fn with_env<F: FnOnce() -> R, R>(value: Option<&str>, f: F) -> R {
    let _guard = ENV_LOCK.lock().unwrap();
    let previous = std::env::var(ENV_VAR).ok();

    match value {
        // SAFETY: Mutex held; no concurrent readers of this env var.
        Some(v) => unsafe { std::env::set_var(ENV_VAR, v) },
        None => unsafe { std::env::remove_var(ENV_VAR) },
    }

    let result = f();

    // Restore previous state so unrelated tests are not affected.
    match previous {
        // SAFETY: Mutex still held; no concurrent readers.
        Some(v) => unsafe { std::env::set_var(ENV_VAR, v) },
        None => unsafe { std::env::remove_var(ENV_VAR) },
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
