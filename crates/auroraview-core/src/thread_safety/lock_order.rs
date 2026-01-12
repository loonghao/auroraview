//! Lock Order Verification for Deadlock Prevention
//!
//! This module provides debug-mode verification that locks are acquired
//! in a consistent order to prevent deadlocks.
//!
//! ## How It Works
//!
//! Each thread maintains a stack of currently held locks. When a new lock
//! is acquired, we verify that its level is higher than all currently held
//! locks. If not, a panic is raised in debug builds.
//!
//! ## Lock Levels
//!
//! Locks are categorized into levels (lower numbers must be acquired first):
//!
//! 1. **Global**: Static/global locks (e.g., `CLICK_THROUGH_DATA`)
//! 2. **Registry**: Collection locks (e.g., `ProcessRegistry`)
//! 3. **Resource**: Individual resource locks (e.g., `ManagedProcess`)
//! 4. **State**: Component state locks (e.g., `BridgeState`)
//! 5. **Callback**: Callback locks (always last)
//!
//! ## Usage
//!
//! ```rust,ignore
//! use auroraview_core::thread_safety::{LockLevel, LockOrderGuard};
//!
//! fn safe_operation() {
//!     // Create guard before acquiring lock
//!     let _guard = LockOrderGuard::new(LockLevel::Registry, "processes");
//!     let lock = self.processes.read().unwrap();
//!     
//!     // Guard automatically released when dropped
//! }
//! ```

use std::cell::RefCell;

/// Lock levels for ordering verification
///
/// Lower levels must be acquired before higher levels to prevent deadlocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum LockLevel {
    /// Global/static locks (level 1)
    /// Examples: CLICK_THROUGH_DATA, global configuration
    Global = 1,

    /// Registry/collection locks (level 2)
    /// Examples: ProcessRegistry, ChannelRegistry
    Registry = 2,

    /// Individual resource locks (level 3)
    /// Examples: ManagedProcess, IpcChannelHandle
    Resource = 3,

    /// Component state locks (level 4)
    /// Examples: BridgeState, ExtensionsState
    State = 4,

    /// Callback locks (level 5)
    /// Examples: event_callback
    /// Always acquire last to prevent callback-under-lock deadlocks
    Callback = 5,
}

impl LockLevel {
    /// Get the numeric level value
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    /// Get a human-readable name for the level
    pub fn name(self) -> &'static str {
        match self {
            LockLevel::Global => "Global",
            LockLevel::Registry => "Registry",
            LockLevel::Resource => "Resource",
            LockLevel::State => "State",
            LockLevel::Callback => "Callback",
        }
    }
}

impl std::fmt::Display for LockLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.name(), self.as_u8())
    }
}

/// Information about a held lock
#[derive(Debug, Clone)]
struct HeldLock {
    level: LockLevel,
    name: String,
}

thread_local! {
    /// Stack of currently held locks for this thread
    static HELD_LOCKS: RefCell<Vec<HeldLock>> = const { RefCell::new(Vec::new()) };

    /// Whether lock order verification is enabled for this thread
    static VERIFICATION_ENABLED: RefCell<bool> = const { RefCell::new(cfg!(debug_assertions)) };
}

/// Enable or disable lock order verification for the current thread
///
/// This is useful for tests or when you need to temporarily disable verification.
pub fn set_verification_enabled(enabled: bool) {
    VERIFICATION_ENABLED.with(|v| *v.borrow_mut() = enabled);
}

/// Check if lock order verification is enabled for the current thread
pub fn is_verification_enabled() -> bool {
    VERIFICATION_ENABLED.with(|v| *v.borrow())
}

/// RAII guard for lock order verification
///
/// Create this guard before acquiring a lock. The guard will:
/// 1. Verify that the lock level is valid (higher than currently held locks)
/// 2. Track the lock as held
/// 3. Automatically release tracking when dropped
///
/// # Panics
///
/// In debug builds, panics if a lock order violation is detected.
///
/// # Example
///
/// ```rust,ignore
/// let _guard = LockOrderGuard::new(LockLevel::Registry, "my_registry");
/// let lock = my_registry.read().unwrap();
/// // Guard dropped here, releasing tracking
/// ```
pub struct LockOrderGuard {
    level: LockLevel,
    #[allow(dead_code)]
    name: String,
    /// Whether this guard was tracked (pushed to the lock stack)
    tracked: bool,
}

impl LockOrderGuard {
    /// Create a new lock order guard
    ///
    /// # Arguments
    ///
    /// * `level` - The level of the lock being acquired
    /// * `name` - A descriptive name for the lock (for error messages)
    ///
    /// # Panics
    ///
    /// Panics in debug builds if acquiring this lock would violate lock ordering.
    pub fn new(level: LockLevel, name: impl Into<String>) -> Self {
        let name = name.into();
        let tracked = is_verification_enabled();

        if tracked {
            verify_lock_order(level, &name);
            push_lock(level, name.clone());
        }

        Self {
            level,
            name,
            tracked,
        }
    }

    /// Create a guard without verification (for special cases)
    ///
    /// Use this only when you're certain the lock order is correct
    /// but verification would cause false positives.
    pub fn new_unchecked(level: LockLevel, name: impl Into<String>) -> Self {
        Self {
            level,
            name: name.into(),
            tracked: false, // Never track unchecked guards
        }
    }

    /// Get the lock level
    pub fn level(&self) -> LockLevel {
        self.level
    }
}

impl Drop for LockOrderGuard {
    fn drop(&mut self) {
        if self.tracked {
            pop_lock(self.level);
        }
    }
}

/// Verify that acquiring a lock at the given level is valid
fn verify_lock_order(level: LockLevel, name: &str) {
    HELD_LOCKS.with(|locks| {
        let locks = locks.borrow();

        if let Some(last) = locks.last() {
            if level <= last.level {
                let held_str: String = locks
                    .iter()
                    .map(|l| format!("{}({})", l.name, l.level))
                    .collect::<Vec<_>>()
                    .join(" -> ");

                panic!(
                    "Lock order violation detected!\n\
                     Attempting to acquire: {}({})\n\
                     Currently held (in order): {}\n\
                     \n\
                     Lock levels must be acquired in ascending order:\n\
                     1. Global -> 2. Registry -> 3. Resource -> 4. State -> 5. Callback\n\
                     \n\
                     To fix this, either:\n\
                     1. Reorder lock acquisitions to follow the hierarchy\n\
                     2. Release higher-level locks before acquiring lower-level ones\n\
                     3. Use try_lock() if the operation can be skipped",
                    name, level, held_str
                );
            }
        }
    });
}

/// Push a lock onto the held locks stack
fn push_lock(level: LockLevel, name: String) {
    HELD_LOCKS.with(|locks| {
        locks.borrow_mut().push(HeldLock { level, name });
    });
}

/// Pop a lock from the held locks stack
fn pop_lock(expected_level: LockLevel) {
    HELD_LOCKS.with(|locks| {
        let mut locks = locks.borrow_mut();
        if let Some(lock) = locks.pop() {
            debug_assert_eq!(
                lock.level, expected_level,
                "Lock release order mismatch: expected {:?}, got {:?}",
                expected_level, lock.level
            );
        }
    });
}

/// Get the number of currently held locks for the current thread
///
/// Useful for debugging and testing.
pub fn held_lock_count() -> usize {
    HELD_LOCKS.with(|locks| locks.borrow().len())
}

/// Clear all held locks for the current thread
///
/// This is useful for test cleanup. Should not be used in production code.
pub fn clear_held_locks() {
    HELD_LOCKS.with(|locks| locks.borrow_mut().clear());
}

#[cfg(test)]
mod tests {
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
    fn test_valid_lock_order() {
        setup();

        {
            let _g1 = LockOrderGuard::new(LockLevel::Registry, "registry");
            assert_eq!(held_lock_count(), 1);

            {
                let _g2 = LockOrderGuard::new(LockLevel::Resource, "resource");
                assert_eq!(held_lock_count(), 2);

                {
                    let _g3 = LockOrderGuard::new(LockLevel::State, "state");
                    assert_eq!(held_lock_count(), 3);
                }
                assert_eq!(held_lock_count(), 2);
            }
            assert_eq!(held_lock_count(), 1);
        }
        assert_eq!(held_lock_count(), 0);
    }

    #[test]
    #[should_panic(expected = "Lock order violation")]
    fn test_invalid_lock_order() {
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

        assert_eq!(held_lock_count(), 0); // Not tracked when disabled
    }

    #[test]
    fn test_unchecked_guard() {
        setup();

        let _g1 = LockOrderGuard::new(LockLevel::Resource, "resource");
        // Unchecked guard doesn't verify or track
        let _g2 = LockOrderGuard::new_unchecked(LockLevel::Registry, "registry");
    }

    #[test]
    fn test_lock_level_display() {
        assert_eq!(format!("{}", LockLevel::Global), "Global(1)");
        assert_eq!(format!("{}", LockLevel::Registry), "Registry(2)");
        assert_eq!(format!("{}", LockLevel::Resource), "Resource(3)");
        assert_eq!(format!("{}", LockLevel::State), "State(4)");
        assert_eq!(format!("{}", LockLevel::Callback), "Callback(5)");
    }
}
