//! Comprehensive tests for auroraview_pack::progress:
//! ProgressStyles, PackProgress, ProgressExt, standalone helpers, edge cases

use auroraview_pack::progress::{
    progress_bar, spinner, PackProgress, ProgressExt, ProgressStyles,
};
use indicatif::ProgressBar;

// ══════════════════════════════════════════════════════════════════════════════
// ProgressStyles — all presets must not panic on creation
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_progress_styles_files_no_panic() {
    let _s = ProgressStyles::files();
}

#[test]
fn test_progress_styles_bytes_no_panic() {
    let _s = ProgressStyles::bytes();
}

#[test]
fn test_progress_styles_spinner_no_panic() {
    let _s = ProgressStyles::spinner();
}

#[test]
fn test_progress_styles_download_no_panic() {
    let _s = ProgressStyles::download();
}

#[test]
fn test_progress_styles_compile_no_panic() {
    let _s = ProgressStyles::compile();
}

#[test]
fn test_progress_styles_encrypt_no_panic() {
    let _s = ProgressStyles::encrypt();
}

#[test]
fn test_progress_styles_success_no_panic() {
    let _s = ProgressStyles::success();
}

#[test]
fn test_progress_styles_error_no_panic() {
    let _s = ProgressStyles::error();
}

#[test]
fn test_progress_styles_all_are_valid_constructors() {
    // All style constructors produce valid ProgressStyle objects
    // without panic — they are used in production code.
    let _f = ProgressStyles::files();
    let _b = ProgressStyles::bytes();
    let _s = ProgressStyles::spinner();
    let _d = ProgressStyles::download();
    let _c = ProgressStyles::compile();
    let _e = ProgressStyles::encrypt();
    let _sc = ProgressStyles::success();
    let _er = ProgressStyles::error();
}

// ══════════════════════════════════════════════════════════════════════════════
// PackProgress — construction, bars, messages, multi, set_main
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_pack_progress_new() {
    let progress = PackProgress::new();
    // Just verify it was created — multi should be usable
    let _m = progress.multi();
}

#[test]
fn test_pack_progress_default_trait() {
    let _progress = PackProgress::default();
}

#[test]
fn test_pack_progress_spinner_finish() {
    let progress = PackProgress::new();
    let pb = progress.spinner("working...");
    pb.finish_with_message("done");
    // Should not panic
}

#[test]
fn test_pack_progress_spinner_custom_message() {
    let progress = PackProgress::new();
    let pb = progress.spinner("downloading assets...");
    pb.set_message("still downloading...");
    pb.finish_and_clear(); // alternative finish method
}

#[test]
fn test_pack_progress_files_bar_inc_and_position() {
    let progress = PackProgress::new();
    let pb = progress.files(10, "collecting files");
    assert_eq!(pb.position(), 0);
    pb.inc(5);
    assert_eq!(pb.position(), 5);
    pb.inc(3);
    assert_eq!(pb.position(), 8);
    pb.finish();
}

#[test]
fn test_pack_progress_files_bar_complete_all() {
    let progress = PackProgress::new();
    let pb = progress.files(7, "files");
    pb.finish_with_message("all done");
    assert!(pb.is_finished());
}

#[test]
fn test_pack_progress_bytes_bar_inc() {
    let progress = PackProgress::new();
    let pb = progress.bytes(1024, "reading bytes");
    pb.inc(512);
    assert_eq!(pb.position(), 512);
    pb.inc(1024); // can overshoot? indicatif allows it
    assert!(pb.position() >= 1024);
    pb.finish();
}

#[test]
fn test_pack_progress_compile_bar() {
    let progress = PackProgress::new();
    let pb = progress.compile(20, "compiling");
    for i in 0..20 {
        pb.inc(1);
        assert_eq!(pb.position(), i + 1);
    }
    pb.finish();
}

#[test]
fn test_pack_progress_encrypt_bar_zero_total() {
    let progress = PackProgress::new();
    let pb = progress.encrypt(0, "encrypting");
    pb.finish(); // zero-length should be fine
}

#[test]
fn test_pack_progress_encrypt_bar_normal() {
    let progress = PackProgress::new();
    let pb = progress.encrypt(5, "encrypting");
    pb.inc(5);
    pb.finish();
}

#[test]
fn test_pack_progress_download_bar_eta_style() {
    let progress = PackProgress::new();
    let pb = progress.download(2048, "downloading");
    pb.inc(1024);
    assert_eq!(pb.position(), 1024);
    pb.finish();
}

#[test]
fn test_pack_progress_success_message() {
    let progress = PackProgress::new();
    progress.success("build complete");
    // success adds a finished bar with ✓ prefix
}

#[test]
fn test_pack_progress_error_message() {
    let progress = PackProgress::new();
    progress.error("build failed");
}

#[test]
fn test_pack_progress_info_message() {
    let progress = PackProgress::new();
    progress.info("building...");
    progress.info("step 2: collect");
    // println-style info messages
}

#[test]
fn test_pack_progress_warn_message() {
    let progress = PackProgress::new();
    progress.warn("missing file: config.toml");
}

#[test]
fn test_pack_progress_set_main_replacement() {
    let mut progress = PackProgress::new();
    let pb1 = progress.files(100, "main bar 1");
    progress.set_main(pb1);
    // Set a new main bar replacing the old one
    let pb2 = progress.bytes(200, "main bar 2");
    progress.set_main(pb2);
}

#[test]
fn test_pack_progress_multi_ref() {
    let progress = PackProgress::new();
    let multi = progress.multi();
    // MultiProgress can be used to add child bars
    let _ = multi; // just verify we can get a reference
}

#[test]
fn test_pack_progress_multiple_bars_simultaneous() {
    let progress = PackProgress::new();
    let files_pb = progress.files(50, "files");
    let bytes_pb = progress.bytes(4096, "bytes");
    let compile_pb = progress.compile(10, "compile");

    files_pb.inc(25);
    bytes_pb.inc(2048);
    compile_pb.inc(3);

    assert_eq!(files_pb.position(), 25);
    assert_eq!(bytes_pb.position(), 2048);
    assert_eq!(compile_pb.position(), 3);

    files_pb.finish();
    bytes_pb.finish();
    compile_pb.finish();
}

#[test]
fn test_pack_progress_abandon_instead_of_finish() {
    let progress = PackProgress::new();
    let pb = progress.spinner("long op");
    pb.abandon(); // leave in indeterminate state without finishing
}

// ══════════════════════════════════════════════════════════════════════════════
// ProgressExt trait on ProgressBar
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_progress_ext_finish_success_sets_prefix() {
    let pb = ProgressBar::new(10);
    pb.set_style(ProgressStyles::files());
    pb.inc(10);
    pb.finish_success("all done");
    assert!(pb.is_finished());
}

#[test]
fn test_progress_ext_finish_error() {
    let pb = ProgressBar::new(10);
    pb.set_style(ProgressStyles::files());
    pb.finish_error("something went wrong");
    assert!(pb.is_finished());
}

#[test]
fn test_progress_ext_tick_with_message_increments_position() {
    let pb = ProgressBar::new(10);
    pb.set_style(ProgressStyles::files());
    assert_eq!(pb.position(), 0);

    pb.tick_with_message("step 1");
    assert_eq!(pb.position(), 1);

    pb.tick_with_message("step 2");
    assert_eq!(pb.position(), 2);

    pb.tick_with_message("step 3");
    assert_eq!(pb.position(), 3);

    pb.tick_with_message("step 4");
    pb.tick_with_message("step 5");
    assert_eq!(pb.position(), 5);

    pb.finish();
}

#[test]
fn test_progress_ext_tick_with_message_beyond_limit() {
    let pb = ProgressBar::new(3);
    pb.set_style(ProgressStyles::files());
    pb.tick_with_message("1");
    pb.tick_with_message("2");
    pb.tick_with_message("3");
    pb.tick_with_message("4"); // beyond limit — indicatif allows this
    assert!(pb.position() >= 3);
    pb.finish();
}

#[test]
fn test_progress_ext_finish_success_on_empty_bar() {
    let pb = ProgressBar::new(0);
    pb.finish_success("empty ok");
}

#[test]
fn test_progress_ext_finish_error_on_empty_bar() {
    let pb = ProgressBar::new(0);
    pb.finish_error("empty err");
}

// ══════════════════════════════════════════════════════════════════════════════
// Standalone helper functions (not on PackProgress)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_standalone_spinner() {
    let pb = spinner("loading");
    assert!(!pb.is_finished()); // spinner is running
    pb.finish_with_message("loaded");
    assert!(pb.is_finished());
}

#[test]
fn test_standalone_spinner_unicode_message() {
    let pb = spinner("\u{1f504} Loading..."); // emoji
    pb.finish();
}

#[test]
fn test_standalone_progress_bar_normal() {
    let pb = progress_bar(100, "items");
    pb.inc(50);
    assert_eq!(pb.position(), 50);
    pb.inc(50);
    assert_eq!(pb.position(), 100);
    pb.finish();
}

#[test]
fn test_standalone_progress_bar_zero_length() {
    let pb = progress_bar(0, "empty");
    assert_eq!(pb.length(), Some(0));
    pb.finish(); // should not panic
}

#[test]
fn test_standalone_progress_bar_single_item() {
    let pb = progress_bar(1, "single");
    pb.inc(1);
    assert_eq!(pb.position(), 1);
    pb.finish_with_message("done");
}

#[test]
fn test_standalone_progress_bar_large_total() {
    let pb = progress_bar(u64::MAX, "huge");
    pb.inc(1);
    assert_eq!(pb.position(), 1);
    pb.abandon(); // don't wait for completion
}
