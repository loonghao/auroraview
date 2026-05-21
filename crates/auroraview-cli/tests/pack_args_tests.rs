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
