//! Integration tests for window utilities
//!
//! These tests verify the complete window management functionality on Windows.

#[cfg(target_os = "windows")]
use _core::window_utils::{find_windows_by_title, get_all_windows, get_foreground_window};
#[cfg(target_os = "windows")]
use rstest::rstest;

#[rstest]
#[cfg(target_os = "windows")]
fn test_get_foreground_window() {
    let result = get_foreground_window();
    assert!(result.is_ok(), "get_foreground_window should succeed");
    // May or may not have a foreground window depending on test environment
}

#[rstest]
#[cfg(target_os = "windows")]
fn test_get_all_windows() {
    let result = get_all_windows();
    assert!(result.is_ok(), "get_all_windows should succeed");

    let windows = result.unwrap();
    // Should have at least some windows in a normal environment
    assert!(!windows.is_empty(), "Should find at least one window");
}

#[rstest]
#[cfg(target_os = "windows")]
fn test_find_windows_by_title() {
    let result = find_windows_by_title("test");
    assert!(result.is_ok(), "find_windows_by_title should succeed");
    // May or may not find windows depending on test environment
}

#[rstest]
#[cfg(target_os = "windows")]
fn test_find_windows_by_empty_title() {
    let result = find_windows_by_title("");
    assert!(
        result.is_ok(),
        "find_windows_by_title with empty string should succeed"
    );
}
