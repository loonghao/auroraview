//! Regression tests for `PackArgs` parsing of `--capture-file-drop` /
//! `--no-capture-file-drop`. Specifically guards against the clap
//! `SetFalse` defect documented in RFC 0015 §4.2.1.

use auroraview_cli::cli::{resolve_capture_file_drop, PackArgs};
use clap::Parser;

fn parse(args: &[&str]) -> PackArgs {
    PackArgs::parse_from(std::iter::once("auroraview-pack").chain(args.iter().copied()))
}

#[test]
fn pack_args_capture_file_drop_default_is_none() {
    let args = parse(&[]);
    assert_eq!(resolve_capture_file_drop(&args), None);
}

#[test]
fn pack_args_capture_file_drop_explicit_enable() {
    let args = parse(&["--capture-file-drop"]);
    assert_eq!(resolve_capture_file_drop(&args), Some(true));
}

#[test]
fn pack_args_capture_file_drop_explicit_disable() {
    let args = parse(&["--no-capture-file-drop"]);
    assert_eq!(resolve_capture_file_drop(&args), Some(false));
}

/// `clap` `overrides_with` semantic: when both flags appear on the same
/// command line, the LAST occurrence wins. This is the only thing that
/// keeps the `(true, true)` arm of `resolve_capture_file_drop` from
/// tripping its `debug_assert!` in development and falling back to
/// positive-wins in release.
///
/// If a future clap upgrade changes this behavior the `debug_assert!`
/// would start firing in CI. This regression guard makes any such drift
/// fail before the resolver layer is ever exercised.
#[test]
fn pack_args_capture_file_drop_overrides_with_last_wins() {
    let args = parse(&["--capture-file-drop", "--no-capture-file-drop"]);
    assert_eq!(resolve_capture_file_drop(&args), Some(false));

    let args = parse(&["--no-capture-file-drop", "--capture-file-drop"]);
    assert_eq!(resolve_capture_file_drop(&args), Some(true));
}
