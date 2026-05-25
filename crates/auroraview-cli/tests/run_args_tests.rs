//! Regression tests for `RunArgs` parsing of `--capture-file-drop` /
//! `--no-capture-file-drop`. These mirror the `PackArgs` suite so the
//! `auroraview run` and `auroraview pack` CLI entry points share the
//! same `Optional[bool]` semantics.

use auroraview_cli::cli::{resolve_run_capture_file_drop, RunArgs};
use clap::Parser;

fn parse(args: &[&str]) -> RunArgs {
    // `RunArgs` requires `--url` or `--html` for `run_webview`, but the
    // `Parser::parse_from` step does not enforce that — it only fails on
    // unrecognized flags. We supply a stub URL so the resulting struct is
    // representative of a real invocation.
    RunArgs::parse_from(
        std::iter::once("auroraview-run")
            .chain(["--url", "https://example.com"].iter().copied())
            .chain(args.iter().copied()),
    )
}

#[test]
fn run_args_capture_file_drop_default_is_none() {
    let args = parse(&[]);
    assert_eq!(resolve_run_capture_file_drop(&args), None);
}

#[test]
fn run_args_capture_file_drop_explicit_enable() {
    let args = parse(&["--capture-file-drop"]);
    assert_eq!(resolve_run_capture_file_drop(&args), Some(true));
}

#[test]
fn run_args_capture_file_drop_explicit_disable() {
    let args = parse(&["--no-capture-file-drop"]);
    assert_eq!(resolve_run_capture_file_drop(&args), Some(false));
}

/// Same `clap` `overrides_with` regression guard as `pack_args_tests.rs`:
/// when both flags appear on the same command line, the LAST one wins.
/// This is what keeps the `(true, true)` arm in
/// `resolve_run_capture_file_drop` from tripping its `debug_assert!`.
#[test]
fn run_args_capture_file_drop_overrides_with_last_wins() {
    let args = parse(&["--capture-file-drop", "--no-capture-file-drop"]);
    assert_eq!(resolve_run_capture_file_drop(&args), Some(false));

    let args = parse(&["--no-capture-file-drop", "--capture-file-drop"]);
    assert_eq!(resolve_run_capture_file_drop(&args), Some(true));
}
