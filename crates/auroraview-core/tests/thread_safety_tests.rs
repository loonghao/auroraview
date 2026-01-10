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
}
