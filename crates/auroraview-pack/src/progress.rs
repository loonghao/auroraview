//! Progress bar utilities for CLI operations
//!
//! Provides beautiful progress indicators for long-running tasks using indicatif.

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::time::Duration;

/// Style presets for different types of progress indicators
pub struct ProgressStyles;

impl ProgressStyles {
    /// Style for file processing operations (shows count and speed)
    pub fn files() -> ProgressStyle {
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({per_sec}) {msg}",
        )
        .unwrap()
        .progress_chars("█▓▒░  ")
    }

    /// Style for byte-based operations (shows size and speed)
    pub fn bytes() -> ProgressStyle {
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}) {msg}",
        )
        .unwrap()
        .progress_chars("█▓▒░  ")
    }

    /// Style for indeterminate operations (spinner only)
    pub fn spinner() -> ProgressStyle {
        ProgressStyle::with_template("{spinner:.green} {msg} [{elapsed_precise}]")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
    }

    /// Style for download operations
    pub fn download() -> ProgressStyle {
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}) ETA: {eta} {msg}",
        )
        .unwrap()
        .progress_chars("█▓▒░  ")
    }

    /// Style for compilation operations
    pub fn compile() -> ProgressStyle {
        ProgressStyle::with_template(
            "{spinner:.yellow} [{elapsed_precise}] [{bar:40.yellow/white}] {pos}/{len} {msg}",
        )
        .unwrap()
        .progress_chars("▰▰▱")
    }

    /// Style for encryption operations
    pub fn encrypt() -> ProgressStyle {
        ProgressStyle::with_template(
            "{spinner:.magenta} [{elapsed_precise}] [{bar:40.magenta/white}] {pos}/{len} {msg}",
        )
        .unwrap()
        .progress_chars("🔒🔐🔓")
    }

    /// Style for success message
    pub fn success() -> ProgressStyle {
        ProgressStyle::with_template("{prefix:.green} {msg}").unwrap()
    }

    /// Style for error message
    pub fn error() -> ProgressStyle {
        ProgressStyle::with_template("{prefix:.red} {msg}").unwrap()
    }
}

/// Progress tracker for pack operations
pub struct PackProgress {
    multi: MultiProgress,
    main_bar: Option<ProgressBar>,
}

impl PackProgress {
    /// Create a new pack progress tracker
    pub fn new() -> Self {
        Self {
            multi: MultiProgress::new(),
            main_bar: None,
        }
    }

    /// Create a spinner for an indeterminate operation
    pub fn spinner(&self, msg: &str) -> ProgressBar {
        let pb = self.multi.add(ProgressBar::new_spinner());
        pb.set_style(ProgressStyles::spinner());
        pb.set_message(msg.to_string());
        pb.enable_steady_tick(Duration::from_millis(80));
        pb
    }

    /// Create a progress bar for file operations
    pub fn files(&self, total: u64, msg: &str) -> ProgressBar {
        let pb = self.multi.add(ProgressBar::new(total));
        pb.set_style(ProgressStyles::files());
        pb.set_message(msg.to_string());
        pb
    }

    /// Create a progress bar for byte operations
    pub fn bytes(&self, total: u64, msg: &str) -> ProgressBar {
        let pb = self.multi.add(ProgressBar::new(total));
        pb.set_style(ProgressStyles::bytes());
        pb.set_message(msg.to_string());
        pb
    }

    /// Create a progress bar for compilation
    pub fn compile(&self, total: u64, msg: &str) -> ProgressBar {
        let pb = self.multi.add(ProgressBar::new(total));
        pb.set_style(ProgressStyles::compile());
        pb.set_message(msg.to_string());
        pb
    }

    /// Create a progress bar for encryption
    pub fn encrypt(&self, total: u64, msg: &str) -> ProgressBar {
        let pb = self.multi.add(ProgressBar::new(total));
        pb.set_style(ProgressStyles::encrypt());
        pb.set_message(msg.to_string());
        pb
    }

    /// Create a progress bar for download
    pub fn download(&self, total: u64, msg: &str) -> ProgressBar {
        let pb = self.multi.add(ProgressBar::new(total));
        pb.set_style(ProgressStyles::download());
        pb.set_message(msg.to_string());
        pb
    }

    /// Set the main progress bar
    pub fn set_main(&mut self, pb: ProgressBar) {
        self.main_bar = Some(pb);
    }

    /// Get the multi-progress instance
    pub fn multi(&self) -> &MultiProgress {
        &self.multi
    }

    /// Print a success message
    pub fn success(&self, msg: &str) {
        let pb = self.multi.add(ProgressBar::new(0));
        pb.set_style(ProgressStyles::success());
        pb.set_prefix("✓");
        pb.finish_with_message(msg.to_string());
    }

    /// Print an error message
    pub fn error(&self, msg: &str) {
        let pb = self.multi.add(ProgressBar::new(0));
        pb.set_style(ProgressStyles::error());
        pb.set_prefix("✗");
        pb.finish_with_message(msg.to_string());
    }

    /// Print an info message
    pub fn info(&self, msg: &str) {
        self.multi.println(format!("  ℹ {}", msg)).ok();
    }

    /// Print a warning message
    pub fn warn(&self, msg: &str) {
        self.multi.println(format!("  ⚠ {}", msg)).ok();
    }
}

impl Default for PackProgress {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper trait for progress bar operations
pub trait ProgressExt {
    /// Finish with a success message
    fn finish_success(&self, msg: &str);

    /// Finish with an error message
    fn finish_error(&self, msg: &str);

    /// Update message and increment
    fn tick_with_message(&self, msg: &str);
}

impl ProgressExt for ProgressBar {
    fn finish_success(&self, msg: &str) {
        self.set_style(ProgressStyle::with_template("{prefix:.green} {msg}").unwrap());
        self.set_prefix("✓");
        self.finish_with_message(msg.to_string());
    }

    fn finish_error(&self, msg: &str) {
        self.set_style(ProgressStyle::with_template("{prefix:.red} {msg}").unwrap());
        self.set_prefix("✗");
        self.finish_with_message(msg.to_string());
    }

    fn tick_with_message(&self, msg: &str) {
        self.set_message(msg.to_string());
        self.inc(1);
    }
}

/// Create a simple spinner for quick operations
pub fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyles::spinner());
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

/// Create a simple progress bar for file operations
pub fn progress_bar(total: u64, msg: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(ProgressStyles::files());
    pb.set_message(msg.to_string());
    pb
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_styles() {
        // Just verify styles can be created without panicking
        let _ = ProgressStyles::files();
        let _ = ProgressStyles::bytes();
        let _ = ProgressStyles::spinner();
        let _ = ProgressStyles::download();
        let _ = ProgressStyles::compile();
        let _ = ProgressStyles::encrypt();
    }

    #[test]
    fn test_pack_progress() {
        let progress = PackProgress::new();
        let pb = progress.spinner("Testing...");
        pb.finish_with_message("Done");
    }
}
