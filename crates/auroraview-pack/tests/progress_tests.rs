//! Integration tests for auroraview_pack::progress:
//! ProgressStyles, PackProgress, ProgressExt, helper functions

use auroraview_pack::progress::{
    progress_bar, spinner, PackProgress, ProgressExt, ProgressStyles,
};
use indicatif::ProgressBar;

// ─── ProgressStyles ──────────────────────────────────────────────────────────

#[test]
fn test_progress_styles_files_no_panic() {
    let _ = ProgressStyles::files();
}

#[test]
fn test_progress_styles_bytes_no_panic() {
    let _ = ProgressStyles::bytes();
}

#[test]
fn test_progress_styles_spinner_no_panic() {
    let _ = ProgressStyles::spinner();
}

#[test]
fn test_progress_styles_download_no_panic() {
    let _ = ProgressStyles::download();
}

#[test]
fn test_progress_styles_compile_no_panic() {
    let _ = ProgressStyles::compile();
}

#[test]
fn test_progress_styles_encrypt_no_panic() {
    let _ = ProgressStyles::encrypt();
}

#[test]
fn test_progress_styles_success_no_panic() {
    let _ = ProgressStyles::success();
}

#[test]
fn test_progress_styles_error_no_panic() {
    let _ = ProgressStyles::error();
}

// ─── PackProgress ─────────────────────────────────────────────────────────────

#[test]
fn test_pack_progress_new() {
    let _progress = PackProgress::new();
}

#[test]
fn test_pack_progress_default() {
    let _progress = PackProgress::default();
}

#[test]
fn test_pack_progress_spinner_finish() {
    let progress = PackProgress::new();
    let pb = progress.spinner("working...");
    pb.finish_with_message("done");
}

#[test]
fn test_pack_progress_files_bar() {
    let progress = PackProgress::new();
    let pb = progress.files(10, "collecting files");
    pb.inc(5);
    assert_eq!(pb.position(), 5);
    pb.finish();
}

#[test]
fn test_pack_progress_bytes_bar() {
    let progress = PackProgress::new();
    let pb = progress.bytes(1024, "reading bytes");
    pb.inc(512);
    assert_eq!(pb.position(), 512);
    pb.finish();
}

#[test]
fn test_pack_progress_compile_bar() {
    let progress = PackProgress::new();
    let pb = progress.compile(20, "compiling");
    pb.inc(10);
    pb.finish();
}

#[test]
fn test_pack_progress_encrypt_bar() {
    let progress = PackProgress::new();
    let pb = progress.encrypt(5, "encrypting");
    pb.finish();
}

#[test]
fn test_pack_progress_download_bar() {
    let progress = PackProgress::new();
    let pb = progress.download(2048, "downloading");
    pb.inc(1024);
    pb.finish();
}

#[test]
fn test_pack_progress_success_message() {
    let progress = PackProgress::new();
    progress.success("build complete");
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
}

#[test]
fn test_pack_progress_warn_message() {
    let progress = PackProgress::new();
    progress.warn("missing file");
}

#[test]
fn test_pack_progress_set_main() {
    let mut progress = PackProgress::new();
    let pb = progress.files(10, "main bar");
    progress.set_main(pb);
}

#[test]
fn test_pack_progress_multi_ref() {
    let progress = PackProgress::new();
    let _multi = progress.multi();
}

// ─── ProgressExt ─────────────────────────────────────────────────────────────

#[test]
fn test_progress_ext_finish_success() {
    let pb = ProgressBar::new(10);
    pb.set_style(ProgressStyles::files());
    pb.inc(10);
    pb.finish_success("all done");
}

#[test]
fn test_progress_ext_finish_error() {
    let pb = ProgressBar::new(10);
    pb.set_style(ProgressStyles::files());
    pb.finish_error("something went wrong");
}

#[test]
fn test_progress_ext_tick_with_message() {
    let pb = ProgressBar::new(10);
    pb.set_style(ProgressStyles::files());
    pb.tick_with_message("step 1");
    pb.tick_with_message("step 2");
    assert_eq!(pb.position(), 2);
    pb.finish();
}

// ─── standalone helpers ───────────────────────────────────────────────────────

#[test]
fn test_standalone_spinner() {
    let pb = spinner("loading");
    pb.finish_with_message("loaded");
}

#[test]
fn test_standalone_progress_bar() {
    let pb = progress_bar(100, "items");
    pb.inc(50);
    assert_eq!(pb.position(), 50);
    pb.finish();
}

#[test]
fn test_standalone_progress_bar_zero_length() {
    let pb = progress_bar(0, "empty");
    pb.finish();
}
