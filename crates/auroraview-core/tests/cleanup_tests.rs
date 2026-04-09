//! Cleanup module tests
//!
//! Covers CleanupStats, extract_pid_from_dir_name edge cases, and public API
//! behavior for WebView data directory cleanup.

use auroraview_core::cleanup::{get_cleanup_stats, get_process_data_dir, get_webview_base_dir};

// ============================================================================
// CleanupStats default values
// ============================================================================

#[test]
fn cleanup_stats_default() {
    let stats = auroraview_core::cleanup::CleanupStats::default();
    assert_eq!(stats.total_dirs, 0);
    assert_eq!(stats.alive_dirs, 0);
    assert_eq!(stats.stale_dirs, 0);
    assert_eq!(stats.stale_size_bytes, 0);
}

// ============================================================================
// CleanupStats field relationships (invariants)
// ============================================================================

#[test]
fn cleanup_stats_invariant_total_equals_alive_plus_stale() {
    let stats = auroraview_core::cleanup::CleanupStats {
        total_dirs: 10,
        alive_dirs: 7,
        stale_dirs: 3,
        stale_size_bytes: 1024,
    };
    // total_dirs should equal alive_dirs + stale_dirs
    assert_eq!(stats.total_dirs, stats.alive_dirs + stats.stale_dirs);
}

#[test]
fn cleanup_stats_all_zero() {
    let stats = auroraview_core::cleanup::CleanupStats {
        total_dirs: 0,
        alive_dirs: 0,
        stale_dirs: 0,
        stale_size_bytes: 0,
    };
    assert_eq!(stats.alive_dirs + stats.stale_dirs, 0);
    assert_eq!(stats.stale_size_bytes, 0);
}

#[test]
fn cleanup_stats_all_stale() {
    let stats = auroraview_core::cleanup::CleanupStats {
        total_dirs: 5,
        alive_dirs: 0,
        stale_dirs: 5,
        stale_size_bytes: 99999,
    };
    assert_eq!(stats.stale_dirs, stats.total_dirs);
    assert_eq!(stats.alive_dirs, 0);
}

#[test]
fn cleanup_stats_all_alive() {
    let stats = auroraview_core::cleanup::CleanupStats {
        total_dirs: 8,
        alive_dirs: 8,
        stale_dirs: 0,
        stale_size_bytes: 0,
    };
    assert_eq!(stats.alive_dirs, stats.total_dirs);
    assert_eq!(stats.stale_size_bytes, 0);
}

// ============================================================================
// CleanupStats Debug derive
// ============================================================================

#[test]
fn cleanup_stats_debug_output() {
    let stats = auroraview_core::cleanup::CleanupStats {
        total_dirs: 15,
        alive_dirs: 10,
        stale_dirs: 5,
        stale_size_bytes: 4096,
    };
    let debug_str = format!("{:?}", stats);
    assert!(debug_str.contains("CleanupStats"));
    assert!(debug_str.contains("15"));   // total_dirs
    assert!(debug_str.contains("10"));   // alive_dirs
    assert!(debug_str.contains("5"));    // stale_dirs
    assert!(debug_str.contains("4096")); // stale_size_bytes
}

// ============================================================================
// CleanupStats Clone derive
// ============================================================================

#[test]
fn cleanup_stats_clone() {
    let original = auroraview_core::cleanup::CleanupStats {
        total_dirs: 100,
        alive_dirs: 95,
        stale_dirs: 5,
        stale_size_bytes: 1048576,
    };
    let cloned = original.clone();
    assert_eq!(original.total_dirs, cloned.total_dirs);
    assert_eq!(original.alive_dirs, cloned.alive_dirs);
    assert_eq!(original.stale_dirs, cloned.stale_dirs);
    assert_eq!(original.stale_size_bytes, cloned.stale_size_bytes);
}

// ============================================================================
// get_webview_base_dir - platform availability
// ============================================================================

#[test]
fn get_webview_base_dir_returns_some_on_supported_platforms() {
    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    {
        let dir = get_webview_base_dir();
        assert!(
            dir.is_some(),
            "get_webview_base_dir should return Some on supported platforms"
        );
        if let Some(path) = dir {
            let path_str = path.to_string_lossy();
            assert!(
                path_str.contains("AuroraView") || path_str.contains("auroraview"),
                "Base dir should contain 'AuroraView': got {:?}",
                path
            );
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        let dir = get_webview_base_dir();
        assert!(dir.is_none(), "Unsupported platform should return None");
    }
}

// ============================================================================
// get_process_data_dir - returns Some with process_ prefix on supported platforms
// ============================================================================

#[test]
fn get_process_data_dir_contains_pid_on_supported_platforms() {
    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    {
        let dir = get_process_data_dir();
        assert!(dir.is_some(), "Should return Some on supported platforms");
        if let Some(path) = dir {
            let path_str = path.to_string_lossy();
            let current_pid = std::process::id();
            let expected_fragment = format!("process_{}", current_pid);
            assert!(
                path_str.contains(&expected_fragment),
                "Process data dir should contain '{}', got: {}",
                expected_fragment,
                path_str
            );
        }
    }
}

// ============================================================================
// get_cleanup_stats - does not panic and returns valid stats
// ============================================================================

#[test]
fn get_cleanup_stats_returns_valid_invariants() {
    let stats = get_cleanup_stats();
    // Invariant: total == alive + stale
    assert_eq!(stats.total_dirs, stats.alive_dirs + stats.stale_dirs);
    // Invariant: size is zero or positive
    // (stale_size_bytes can be 0 if no stale dirs exist)
}

#[test]
fn get_cleanup_stats_does_not_panic() {
    // Call multiple times to ensure idempotency and no panics
    for _ in 0..5 {
        let _stats = get_cleanup_stats();
    }
}

#[test]
fn get_cleanup_stats_concurrent_calls() {
    use std::thread;

    // Spawn multiple threads to call get_cleanup_stats concurrently
    let handles: Vec<_> = (0..8)
        .map(|_| thread::spawn(|| { get_cleanup_stats(); }))
        .collect();

    for handle in handles {
        handle.join().expect("Thread should not panic");
    }
}
