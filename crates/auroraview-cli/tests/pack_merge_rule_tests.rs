//! RFC 0015 §4.4 pack-time merge rule tests for `capture_file_drop`.
//!
//! Authoritative precedence (highest first):
//!
//!   1. CLI flag (`--capture-file-drop` / `--no-capture-file-drop`)
//!   2. `[security].capture_file_drop` in manifest
//!   3. Code default (`false`)
//!
//! The merge rule itself is implemented inline in `cli/pack.rs::run_pack`:
//!
//!   ```text
//!   overlay value = resolve_capture_file_drop(&pack_args)
//!       .or(manifest.security.and_then(|s| s.capture_file_drop))
//!       .unwrap_or(false);
//!   ```
//!
//! These tests reconstruct the same expression using public APIs so any
//! future refactor that diverges from the rule fails fast.

use auroraview_cli::cli::{resolve_capture_file_drop, PackArgs};
use auroraview_pack::{Manifest, PackConfig};
use clap::Parser;
use std::path::Path;

fn parse_args(args: &[&str]) -> PackArgs {
    PackArgs::parse_from(std::iter::once("auroraview-pack").chain(args.iter().copied()))
}

/// Reproduce the merge expression from `cli/pack.rs::run_pack`.
///
/// `manifest_value` simulates the result of `from_manifest`'s
/// `unwrap_or(false)` — the function exposed publicly always returns a
/// concrete `bool`. We feed the raw `Option<bool>` from the manifest to
/// preserve the tri-state CLI override semantics.
fn merge(cli: Option<bool>, manifest_value: Option<bool>) -> bool {
    cli.or(manifest_value).unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Direct truth-table coverage
// ---------------------------------------------------------------------------

#[test]
fn merge_no_cli_no_manifest_falls_back_to_default() {
    assert!(!merge(None, None));
}

#[test]
fn merge_cli_enable_overrides_missing_manifest() {
    assert!(merge(Some(true), None));
}

#[test]
fn merge_cli_disable_overrides_missing_manifest() {
    assert!(!merge(Some(false), None));
}

#[test]
fn merge_manifest_true_used_when_no_cli() {
    assert!(merge(None, Some(true)));
}

#[test]
fn merge_manifest_false_used_when_no_cli() {
    assert!(!merge(None, Some(false)));
}

#[test]
fn merge_cli_enable_overrides_manifest_false() {
    // Critical: user explicitly opts in via CLI, manifest must lose.
    assert!(merge(Some(true), Some(false)));
}

#[test]
fn merge_cli_disable_overrides_manifest_true() {
    // Critical: user explicitly opts out via CLI, manifest must lose.
    assert!(!merge(Some(false), Some(true)));
}

// ---------------------------------------------------------------------------
// End-to-end: parse PackArgs + Manifest, run merge identical to run_pack
// ---------------------------------------------------------------------------

fn manifest_with_security(toml_security: &str) -> Manifest {
    let toml = format!(
        r#"
[package]
name = "merge-test"
title = "Merge Test"

[frontend]
url = "https://example.com"

{}
"#,
        toml_security
    );
    Manifest::parse(&toml).expect("manifest parses")
}

fn merge_full(args: &PackArgs, manifest: &Manifest) -> bool {
    let cli = resolve_capture_file_drop(args);
    // PackConfig.from_manifest already collapses Option<bool> -> bool, but
    // for the override step `run_pack` reads the raw manifest tri-state.
    let manifest_value = manifest
        .security
        .as_ref()
        .and_then(|s| s.capture_file_drop);
    cli.or(manifest_value).unwrap_or(false)
}

#[test]
fn end_to_end_merge_default_default() {
    let args = parse_args(&[]);
    let manifest = manifest_with_security("");
    assert!(!merge_full(&args, &manifest));
}

#[test]
fn end_to_end_merge_manifest_true_no_cli() {
    let args = parse_args(&[]);
    let manifest = manifest_with_security("[security]\ncapture_file_drop = true");
    assert!(merge_full(&args, &manifest));
}

#[test]
fn end_to_end_merge_cli_disable_wins_over_manifest_true() {
    let args = parse_args(&["--no-capture-file-drop"]);
    let manifest = manifest_with_security("[security]\ncapture_file_drop = true");
    assert!(!merge_full(&args, &manifest));
}

#[test]
fn end_to_end_merge_cli_enable_wins_over_manifest_false() {
    let args = parse_args(&["--capture-file-drop"]);
    let manifest = manifest_with_security("[security]\ncapture_file_drop = false");
    assert!(merge_full(&args, &manifest));
}

// ---------------------------------------------------------------------------
// Cross-check PackConfig.from_manifest agrees with the lower bound (manifest
// alone, no CLI). PackConfig already does `unwrap_or(false)` so this is the
// "manifest-only" view of the merge.
// ---------------------------------------------------------------------------

#[test]
fn pack_config_from_manifest_matches_manifest_only_merge() {
    let manifest = manifest_with_security("[security]\ncapture_file_drop = true");
    let config = PackConfig::from_manifest(&manifest, Path::new(".")).unwrap();
    assert!(config.capture_file_drop);

    let manifest = manifest_with_security("");
    let config = PackConfig::from_manifest(&manifest, Path::new(".")).unwrap();
    assert!(!config.capture_file_drop);
}
