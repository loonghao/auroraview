//! Tests for thread safety utilities
//!
//! These tests verify the lock ordering verification system and
//! thread safety configuration.

use auroraview_core::thread_safety::{
    clear_held_locks, held_lock_count, is_verification_enabled, set_verification_enabled,
    LockLevel, LockOrderGuard, ThreadSafetyConfig,
};

mod lock_order_tests {
    use super::*;

    fn setup() {
        clear_held_locks();
        set_verification_enabled(true);
    }

    #[test]
    fn test_lock_level_ordering() {
        assert!(LockLevel::Global < LockLevel::Registry);
        assert!(LockLevel::Registry < LockLevel::Resource);
        assert!(LockLevel::Resource < LockLevel::State);
        assert!(LockLevel::State < LockLevel::Callback);
    }

    #[test]
    fn test_lock_level_values() {
        assert_eq!(LockLevel::Global.as_u8(), 1);
        assert_eq!(LockLevel::Registry.as_u8(), 2);
        assert_eq!(LockLevel::Resource.as_u8(), 3);
        assert_eq!(LockLevel::State.as_u8(), 4);
        assert_eq!(LockLevel::Callback.as_u8(), 5);
    }

    #[test]
    fn test_lock_level_names() {
        assert_eq!(LockLevel::Global.name(), "Global");
        assert_eq!(LockLevel::Registry.name(), "Registry");
        assert_eq!(LockLevel::Resource.name(), "Resource");
        assert_eq!(LockLevel::State.name(), "State");
        assert_eq!(LockLevel::Callback.name(), "Callback");
    }

    #[test]
    fn test_lock_level_display() {
        assert_eq!(format!("{}", LockLevel::Global), "Global(1)");
        assert_eq!(format!("{}", LockLevel::Registry), "Registry(2)");
        assert_eq!(format!("{}", LockLevel::Resource), "Resource(3)");
        assert_eq!(format!("{}", LockLevel::State), "State(4)");
        assert_eq!(format!("{}", LockLevel::Callback), "Callback(5)");
    }

    #[test]
    fn test_valid_lock_order_ascending() {
        setup();

        {
            let _g1 = LockOrderGuard::new(LockLevel::Global, "global_lock");
            assert_eq!(held_lock_count(), 1);

            {
                let _g2 = LockOrderGuard::new(LockLevel::Registry, "registry_lock");
                assert_eq!(held_lock_count(), 2);

                {
                    let _g3 = LockOrderGuard::new(LockLevel::Resource, "resource_lock");
                    assert_eq!(held_lock_count(), 3);

                    {
                        let _g4 = LockOrderGuard::new(LockLevel::State, "state_lock");
                        assert_eq!(held_lock_count(), 4);

                        {
                            let _g5 = LockOrderGuard::new(LockLevel::Callback, "callback_lock");
                            assert_eq!(held_lock_count(), 5);
                        }
                        assert_eq!(held_lock_count(), 4);
                    }
                    assert_eq!(held_lock_count(), 3);
                }
                assert_eq!(held_lock_count(), 2);
            }
            assert_eq!(held_lock_count(), 1);
        }
        assert_eq!(held_lock_count(), 0);
    }

    #[test]
    fn test_skipping_levels_is_valid() {
        setup();

        // It's valid to skip levels (e.g., Global -> Resource)
        {
            let _g1 = LockOrderGuard::new(LockLevel::Global, "global");
            let _g2 = LockOrderGuard::new(LockLevel::Resource, "resource");
            let _g3 = LockOrderGuard::new(LockLevel::Callback, "callback");
            assert_eq!(held_lock_count(), 3);
        }
        assert_eq!(held_lock_count(), 0);
    }

    #[test]
    #[should_panic(expected = "Lock order violation")]
    fn test_invalid_lock_order_descending() {
        setup();

        let _g1 = LockOrderGuard::new(LockLevel::Resource, "resource");
        let _g2 = LockOrderGuard::new(LockLevel::Registry, "registry"); // Should panic!
    }

    #[test]
    #[should_panic(expected = "Lock order violation")]
    fn test_same_level_violation() {
        setup();

        let _g1 = LockOrderGuard::new(LockLevel::Registry, "registry1");
        let _g2 = LockOrderGuard::new(LockLevel::Registry, "registry2"); // Should panic!
    }

    #[test]
    fn test_verification_disabled() {
        setup();
        set_verification_enabled(false);

        // This would normally panic, but verification is disabled
        let _g1 = LockOrderGuard::new(LockLevel::Resource, "resource");
        let _g2 = LockOrderGuard::new(LockLevel::Registry, "registry");

        // Not tracked when disabled
        assert_eq!(held_lock_count(), 0);
    }

    #[test]
    fn test_verification_toggle() {
        setup();

        assert!(is_verification_enabled());
        set_verification_enabled(false);
        assert!(!is_verification_enabled());
        set_verification_enabled(true);
        assert!(is_verification_enabled());
    }

    #[test]
    fn test_unchecked_guard() {
        setup();

        let _g1 = LockOrderGuard::new(LockLevel::Resource, "resource");
        // Unchecked guard doesn't verify or track
        let _g2 = LockOrderGuard::new_unchecked(LockLevel::Registry, "registry");

        // Only the first guard is tracked
        assert_eq!(held_lock_count(), 1);
    }

    #[test]
    fn test_guard_level_accessor() {
        let guard = LockOrderGuard::new_unchecked(LockLevel::State, "test");
        assert_eq!(guard.level(), LockLevel::State);
    }

    #[test]
    fn test_multiple_same_level_with_release() {
        setup();

        // Acquire and release, then acquire same level again - should be OK
        {
            let _g1 = LockOrderGuard::new(LockLevel::Registry, "registry1");
            assert_eq!(held_lock_count(), 1);
        }
        assert_eq!(held_lock_count(), 0);

        // Now we can acquire Registry level again
        {
            let _g2 = LockOrderGuard::new(LockLevel::Registry, "registry2");
            assert_eq!(held_lock_count(), 1);
        }
        assert_eq!(held_lock_count(), 0);
    }

    // --- Additional lock-order tests ---

    #[test]
    fn test_full_chain_global_to_callback() {
        setup();
        {
            let _g1 = LockOrderGuard::new(LockLevel::Global, "g");
            let _g2 = LockOrderGuard::new(LockLevel::Registry, "r");
            let _g3 = LockOrderGuard::new(LockLevel::Resource, "res");
            let _g4 = LockOrderGuard::new(LockLevel::State, "s");
            let _g5 = LockOrderGuard::new(LockLevel::Callback, "c");
            assert_eq!(held_lock_count(), 5);
        }
        assert_eq!(held_lock_count(), 0);
    }

    #[test]
    fn test_state_then_callback_valid() {
        setup();
        {
            let _g1 = LockOrderGuard::new(LockLevel::State, "state");
            let _g2 = LockOrderGuard::new(LockLevel::Callback, "cb");
            assert_eq!(held_lock_count(), 2);
        }
        assert_eq!(held_lock_count(), 0);
    }

    #[test]
    #[should_panic(expected = "Lock order violation")]
    fn test_callback_then_state_violation() {
        setup();
        let _g1 = LockOrderGuard::new(LockLevel::Callback, "cb");
        let _g2 = LockOrderGuard::new(LockLevel::State, "state");
    }

    #[test]
    #[should_panic(expected = "Lock order violation")]
    fn test_resource_then_global_violation() {
        setup();
        let _g1 = LockOrderGuard::new(LockLevel::Resource, "res");
        let _g2 = LockOrderGuard::new(LockLevel::Global, "global");
    }

    #[test]
    fn test_repeated_acquire_release_cycles() {
        setup();
        for _ in 0..10 {
            let _g = LockOrderGuard::new(LockLevel::State, "state");
            assert_eq!(held_lock_count(), 1);
        }
        assert_eq!(held_lock_count(), 0);
    }

    #[test]
    fn test_deep_nesting_all_levels_repeated() {
        setup();
        for iteration in 0..5 {
            {
                let _g1 = LockOrderGuard::new(LockLevel::Global, format!("g{iteration}"));
                let _g2 = LockOrderGuard::new(LockLevel::Registry, format!("r{iteration}"));
                let _g3 = LockOrderGuard::new(LockLevel::Resource, format!("res{iteration}"));
                assert_eq!(held_lock_count(), 3);
            }
            assert_eq!(held_lock_count(), 0);
        }
    }

    #[test]
    fn test_lock_level_clone_and_copy() {
        let lvl = LockLevel::Resource;
        let lvl2 = lvl;
        let lvl3 = lvl;
        assert_eq!(lvl, lvl2);
        assert_eq!(lvl, lvl3);
    }

    #[test]
    fn test_lock_level_debug_format() {
        let dbg = format!("{:?}", LockLevel::State);
        assert!(dbg.contains("State"));
    }

    #[test]
    fn test_lock_level_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(LockLevel::Global);
        set.insert(LockLevel::Registry);
        set.insert(LockLevel::Resource);
        set.insert(LockLevel::State);
        set.insert(LockLevel::Callback);
        assert_eq!(set.len(), 5);
        // Duplicate insertion doesn't grow the set
        set.insert(LockLevel::Global);
        assert_eq!(set.len(), 5);
    }

    #[test]
    fn test_clear_held_locks_mid_scope() {
        setup();
        let _g1 = LockOrderGuard::new(LockLevel::Global, "g");
        let _g2 = LockOrderGuard::new(LockLevel::Registry, "r");
        assert_eq!(held_lock_count(), 2);

        // Force-clear (test utility only)
        clear_held_locks();
        assert_eq!(held_lock_count(), 0);
        // After clear, can re-acquire any level
        let _g3 = LockOrderGuard::new(LockLevel::Global, "g_again");
        assert_eq!(held_lock_count(), 1);
    }
}

mod config_tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ThreadSafetyConfig::default();
        assert_eq!(config.js_eval_timeout_ms, 5000);
        assert_eq!(config.main_thread_timeout_ms, 30000);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_delay_ms, 100);
    }

    #[test]
    fn test_config_builder() {
        let config = ThreadSafetyConfig::new()
            .with_js_eval_timeout(10000)
            .with_main_thread_timeout(60000)
            .with_max_retries(5)
            .with_retry_delay(200)
            .with_lock_order_verification(false);

        assert_eq!(config.js_eval_timeout_ms, 10000);
        assert_eq!(config.main_thread_timeout_ms, 60000);
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.retry_delay_ms, 200);
        assert!(!config.debug_lock_order);
    }

    #[test]
    fn test_config_chaining() {
        let config = ThreadSafetyConfig::new()
            .with_js_eval_timeout(1000)
            .with_js_eval_timeout(2000); // Override

        assert_eq!(config.js_eval_timeout_ms, 2000);
    }

    #[test]
    fn test_config_clone() {
        let original = ThreadSafetyConfig::new()
            .with_js_eval_timeout(9999)
            .with_max_retries(7);
        let cloned = original.clone();
        assert_eq!(cloned.js_eval_timeout_ms, 9999);
        assert_eq!(cloned.max_retries, 7);
    }

    #[test]
    fn test_config_debug_format() {
        let config = ThreadSafetyConfig::default();
        let dbg = format!("{:?}", config);
        assert!(dbg.contains("ThreadSafetyConfig"));
        assert!(dbg.contains("js_eval_timeout_ms"));
    }

    #[test]
    fn test_config_lock_order_verification_toggle() {
        let with_lock = ThreadSafetyConfig::new().with_lock_order_verification(true);
        let without_lock = ThreadSafetyConfig::new().with_lock_order_verification(false);
        assert!(with_lock.debug_lock_order);
        assert!(!without_lock.debug_lock_order);
    }

    #[test]
    fn test_config_zero_values() {
        let config = ThreadSafetyConfig::new()
            .with_js_eval_timeout(0)
            .with_main_thread_timeout(0)
            .with_max_retries(0)
            .with_retry_delay(0);
        assert_eq!(config.js_eval_timeout_ms, 0);
        assert_eq!(config.main_thread_timeout_ms, 0);
        assert_eq!(config.max_retries, 0);
        assert_eq!(config.retry_delay_ms, 0);
    }

    #[test]
    fn test_config_max_values() {
        let config = ThreadSafetyConfig::new()
            .with_js_eval_timeout(u64::MAX)
            .with_main_thread_timeout(u64::MAX)
            .with_max_retries(u32::MAX)
            .with_retry_delay(u64::MAX);
        assert_eq!(config.js_eval_timeout_ms, u64::MAX);
        assert_eq!(config.main_thread_timeout_ms, u64::MAX);
        assert_eq!(config.max_retries, u32::MAX);
        assert_eq!(config.retry_delay_ms, u64::MAX);
    }
}

mod concurrent_arc_mutex_tests {
    use std::sync::{Arc, Mutex, RwLock};
    use std::thread;

    #[test]
    fn test_arc_mutex_concurrent_counter() {
        let counter = Arc::new(Mutex::new(0u64));
        let threads: Vec<_> = (0..16)
            .map(|_| {
                let c = counter.clone();
                thread::spawn(move || {
                    for _ in 0..100 {
                        *c.lock().unwrap() += 1;
                    }
                })
            })
            .collect();
        for t in threads {
            t.join().unwrap();
        }
        assert_eq!(*counter.lock().unwrap(), 1600);
    }

    #[test]
    fn test_arc_rwlock_concurrent_readers() {
        let data = Arc::new(RwLock::new(vec![1u32, 2, 3, 4, 5]));
        let threads: Vec<_> = (0..20)
            .map(|_| {
                let d = data.clone();
                thread::spawn(move || {
                    let r = d.read().unwrap();
                    assert_eq!(r.len(), 5);
                    r.iter().sum::<u32>()
                })
            })
            .collect();
        for t in threads {
            let sum = t.join().unwrap();
            assert_eq!(sum, 15);
        }
    }

    #[test]
    fn test_arc_mutex_nested_collections() {
        // Simulates registry → resource pattern without LockOrderGuard (pure Rust)
        let registry = Arc::new(Mutex::new(Vec::<Arc<Mutex<String>>>::new()));

        // Populate
        for i in 0..5 {
            let item = Arc::new(Mutex::new(format!("resource_{i}")));
            registry.lock().unwrap().push(item);
        }

        // Read all items
        let items: Vec<Arc<Mutex<String>>> = registry.lock().unwrap().clone();
        for item in &items {
            let val = item.lock().unwrap();
            assert!(val.starts_with("resource_"));
        }
    }

    #[test]
    fn test_arc_mutex_concurrent_insert_delete() {
        use std::collections::HashMap;

        let map = Arc::new(Mutex::new(HashMap::<String, u32>::new()));

        let writers: Vec<_> = (0..8)
            .map(|i| {
                let m = map.clone();
                thread::spawn(move || {
                    let mut guard = m.lock().unwrap();
                    guard.insert(format!("key{i}"), i);
                })
            })
            .collect();

        for w in writers {
            w.join().unwrap();
        }

        let final_map = map.lock().unwrap();
        assert_eq!(final_map.len(), 8);
    }

    #[test]
    fn test_arc_mutex_stress_contention() {
        // High-contention: 32 threads all hitting a single Mutex
        let shared = Arc::new(Mutex::new(0u64));
        let threads: Vec<_> = (0..32)
            .map(|_| {
                let s = shared.clone();
                thread::spawn(move || {
                    for _ in 0..50 {
                        let mut g = s.lock().unwrap();
                        *g += 1;
                        drop(g);
                    }
                })
            })
            .collect();
        for t in threads {
            t.join().unwrap();
        }
        assert_eq!(*shared.lock().unwrap(), 1600);
    }

    #[test]
    fn test_arc_rwlock_mixed_read_write() {
        let data = Arc::new(RwLock::new(0u64));

        // 4 writers + 8 readers
        let mut handles = Vec::new();

        for _ in 0..4 {
            let d = data.clone();
            handles.push(thread::spawn(move || {
                for _ in 0..25 {
                    *d.write().unwrap() += 1;
                }
            }));
        }

        for _ in 0..8 {
            let d = data.clone();
            handles.push(thread::spawn(move || {
                // Just read without caring about exact value
                let _ = *d.read().unwrap();
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        // 4 writers × 25 = 100 increments
        assert_eq!(*data.read().unwrap(), 100);
    }

    #[test]
    fn test_lock_order_guards_independent_per_thread() {
        use super::*;

        // Each thread has its own lock stack (thread_local)
        let handles: Vec<_> = (0..8)
            .map(|_| {
                thread::spawn(|| {
                    clear_held_locks();
                    set_verification_enabled(true);

                    let _g1 = LockOrderGuard::new(LockLevel::Global, "g");
                    let _g2 = LockOrderGuard::new(LockLevel::Registry, "r");
                    assert_eq!(held_lock_count(), 2);
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }
    }

    #[test]
    fn test_multiple_threads_no_shared_lock_state() {
        use super::*;

        // Verify thread_local isolation: each thread can violate independently
        // (disabled) without affecting others
        let results = Arc::new(Mutex::new(Vec::new()));
        let handles: Vec<_> = (0..4)
            .map(|i| {
                let r = results.clone();
                thread::spawn(move || {
                    clear_held_locks();
                    set_verification_enabled(false); // disable for this thread

                    // Would normally violate order:
                    let _g1 = LockOrderGuard::new(LockLevel::Callback, "cb");
                    let _g2 = LockOrderGuard::new(LockLevel::Global, "g");

                    r.lock().unwrap().push(i);
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(results.lock().unwrap().len(), 4);
    }
}

