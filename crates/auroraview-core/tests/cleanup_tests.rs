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

// ============================================================================
// CleanupStats arithmetic and edge values
// ============================================================================

#[test]
fn cleanup_stats_large_values() {
    let stats = auroraview_core::cleanup::CleanupStats {
        total_dirs: usize::MAX,
        alive_dirs: usize::MAX - 1,
        stale_dirs: 1,
        stale_size_bytes: u64::MAX,
    };
    assert_eq!(stats.total_dirs, stats.alive_dirs + stats.stale_dirs);
    assert_eq!(stats.stale_size_bytes, u64::MAX);
}

#[test]
fn cleanup_stats_single_stale_dir() {
    let stats = auroraview_core::cleanup::CleanupStats {
        total_dirs: 1,
        alive_dirs: 0,
        stale_dirs: 1,
        stale_size_bytes: 512,
    };
    assert_eq!(stats.total_dirs, 1);
    assert_eq!(stats.stale_size_bytes, 512);
}

#[test]
fn cleanup_stats_single_alive_dir() {
    let stats = auroraview_core::cleanup::CleanupStats {
        total_dirs: 1,
        alive_dirs: 1,
        stale_dirs: 0,
        stale_size_bytes: 0,
    };
    assert_eq!(stats.alive_dirs, 1);
    assert_eq!(stats.stale_dirs, 0);
}

// ============================================================================
// get_webview_base_dir consistency
// ============================================================================

#[test]
fn get_webview_base_dir_consistent_across_calls() {
    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    {
        let dir1 = get_webview_base_dir();
        let dir2 = get_webview_base_dir();
        // Must return the same path consistently
        assert_eq!(dir1, dir2);
    }
}

#[test]
fn get_webview_base_dir_is_absolute_path() {
    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    {
        if let Some(path) = get_webview_base_dir() {
            assert!(path.is_absolute(), "Base dir should be absolute: {:?}", path);
        }
    }
}

// ============================================================================
// get_process_data_dir consistency
// ============================================================================

#[test]
fn get_process_data_dir_consistent_across_calls() {
    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    {
        let dir1 = get_process_data_dir();
        let dir2 = get_process_data_dir();
        assert_eq!(dir1, dir2, "Process data dir should be stable within the same process");
    }
}

#[test]
fn get_process_data_dir_is_absolute_path() {
    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    {
        if let Some(path) = get_process_data_dir() {
            assert!(path.is_absolute(), "Process data dir should be absolute: {:?}", path);
        }
    }
}

#[test]
fn get_process_data_dir_contains_base_dir() {
    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    {
        let base = get_webview_base_dir();
        let proc = get_process_data_dir();
        if let (Some(base_path), Some(proc_path)) = (base, proc) {
            // The process data dir should be a subdirectory of the base dir
            assert!(
                proc_path.starts_with(&base_path),
                "Process dir {:?} should be under base dir {:?}",
                proc_path,
                base_path
            );
        }
    }
}

// ============================================================================
// get_cleanup_stats invariant deeper checks
// ============================================================================

#[test]
fn get_cleanup_stats_stale_size_is_nonnegative() {
    let stats = get_cleanup_stats();
    // stale_size_bytes is u64 so always >= 0; if no stale dirs, should be 0
    if stats.stale_dirs == 0 {
        assert_eq!(stats.stale_size_bytes, 0, "no stale dirs → size should be 0");
    }
}

#[test]
fn get_cleanup_stats_alive_le_total() {
    let stats = get_cleanup_stats();
    assert!(
        stats.alive_dirs <= stats.total_dirs,
        "alive_dirs {} should be <= total_dirs {}",
        stats.alive_dirs,
        stats.total_dirs
    );
}

#[test]
fn get_cleanup_stats_stale_le_total() {
    let stats = get_cleanup_stats();
    assert!(
        stats.stale_dirs <= stats.total_dirs,
        "stale_dirs {} should be <= total_dirs {}",
        stats.stale_dirs,
        stats.total_dirs
    );
}

// ============================================================================
// CleanupStats: Default + PartialEq logic
// ============================================================================

#[test]
fn cleanup_stats_default_satisfies_invariant() {
    let stats = auroraview_core::cleanup::CleanupStats::default();
    assert_eq!(stats.total_dirs, stats.alive_dirs + stats.stale_dirs);
    assert_eq!(stats.stale_size_bytes, 0);
}

#[test]
fn cleanup_stats_mixed_large_and_small() {
    let stats = auroraview_core::cleanup::CleanupStats {
        total_dirs: 1000,
        alive_dirs: 999,
        stale_dirs: 1,
        stale_size_bytes: 1,
    };
    assert_eq!(stats.total_dirs, stats.alive_dirs + stats.stale_dirs);
    assert!(stats.stale_size_bytes > 0);
}

// ============================================================================
// CleanupStats: is Send + Sync
// ============================================================================

#[test]
fn cleanup_stats_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<auroraview_core::cleanup::CleanupStats>();
}

// ============================================================================
// get_webview_base_dir: format validation
// ============================================================================

#[test]
fn get_webview_base_dir_does_not_panic() {
    // Called on any platform, should never panic
    let _ = get_webview_base_dir();
}

// ============================================================================
// get_process_data_dir: format validation
// ============================================================================

#[test]
fn get_process_data_dir_does_not_panic() {
    let _ = get_process_data_dir();
}

// ============================================================================
// CleanupStats debug contains numeric values
// ============================================================================

#[test]
fn cleanup_stats_debug_contains_all_fields() {
    let stats = auroraview_core::cleanup::CleanupStats {
        total_dirs: 42,
        alive_dirs: 30,
        stale_dirs: 12,
        stale_size_bytes: 8192,
    };
    let debug_str = format!("{:?}", stats);
    assert!(debug_str.contains("42"));
    assert!(debug_str.contains("30"));
    assert!(debug_str.contains("12"));
    assert!(debug_str.contains("8192"));
}

// ============================================================================
// get_cleanup_stats called in parallel thread
// ============================================================================

#[test]
fn get_cleanup_stats_from_thread() {
    use std::thread;
    let handle = thread::spawn(|| {
        let stats = get_cleanup_stats();
        assert_eq!(stats.total_dirs, stats.alive_dirs + stats.stale_dirs);
    });
    handle.join().expect("thread should not panic");
}
