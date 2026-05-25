//! Direct unit-style tests for `cli::resolve_flag_pair`.
//!
//! `pack_args_tests.rs` and `run_args_tests.rs` already exercise this
//! helper transitively through clap. The tests here pin down the
//! function-level contract so a future refactor of `PackArgs` /
//! `RunArgs` cannot silently change the resolver semantics.
//!
//! Note: the `(true, true)` arm is documented as "should be unreachable
//! with clap `overrides_with`". The resolver guards it with a
//! `debug_assert!` (loud failure during development) and falls back to
//! `Some(true)` (positive wins, last-wins semantics) in release builds
//! so a misconfigured CLI never crashes a user shell. The tests below
//! pin down that exact safety-net behaviour, with separate variants for
//! debug vs release profiles.

use auroraview_cli::cli::resolve_flag_pair;

#[test]
fn flag_pair_both_absent_returns_none() {
    assert_eq!(resolve_flag_pair(false, false), None);
}

#[test]
fn flag_pair_positive_only_returns_some_true() {
    assert_eq!(resolve_flag_pair(true, false), Some(true));
}

#[test]
fn flag_pair_negative_only_returns_some_false() {
    assert_eq!(resolve_flag_pair(false, true), Some(false));
}

/// In debug builds the `(true, true)` arm trips a `debug_assert!` to
/// surface the misconfiguration during development.
#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "clap overrides_with should make (true, true) impossible")]
fn flag_pair_both_set_panics_in_debug() {
    let _ = resolve_flag_pair(true, true);
}

/// In release builds the `(true, true)` arm falls back to positive-wins
/// (last-wins semantics) so a misconfigured CLI never crashes the user shell.
#[cfg(not(debug_assertions))]
#[test]
fn flag_pair_both_set_falls_back_to_positive_in_release() {
    assert_eq!(resolve_flag_pair(true, true), Some(true));
}
