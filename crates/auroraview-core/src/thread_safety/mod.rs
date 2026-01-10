//! Thread Safety Utilities for AuroraView
//!
//! This module provides utilities for ensuring thread safety in multi-threaded
//! environments, particularly when integrating with DCC applications.
//!
//! ## Features
//!
//! - **Lock Order Verification** (debug builds only): Detects potential deadlocks
//!   by verifying that locks are acquired in a consistent order.
//!
//! - **Timeout Protection**: Configurable timeouts for all synchronous operations.
//!
//! - **Graceful Shutdown**: Coordinated shutdown across all threads.
//!
//! ## Lock Ordering
//!
//! To prevent deadlocks, locks must be acquired in the following order:
//!
//! | Level | Lock Type | Examples |
//! |-------|-----------|----------|
//! | 1 | Global/Static | `CLICK_THROUGH_DATA` |
//! | 2 | Registry/Collection | `ProcessRegistry`, `ChannelRegistry` |
//! | 3 | Individual Resource | `ManagedProcess`, `IpcChannelHandle` |
//! | 4 | State | `BridgeState`, `ExtensionsState` |
//! | 5 | Callback | `event_callback` |
//!
//! ## Example
//!
//! ```rust,ignore
//! use auroraview_core::thread_safety::{LockLevel, LockOrderGuard};
//!
//! fn example() {
//!     // Acquire registry lock (level 2)
//!     let _guard1 = LockOrderGuard::new(LockLevel::Registry, "processes");
//!     let processes = self.processes.read().unwrap();
//!     
//!     // Acquire resource lock (level 3) - OK because 3 > 2
//!     let _guard2 = LockOrderGuard::new(LockLevel::Resource, "process_123");
//!     let process = processes.get(&pid).unwrap().lock().unwrap();
//! }
//! ```

mod lock_order;

pub use lock_order::{LockLevel, LockOrderGuard};

// Re-export utility functions for testing and debugging
pub use lock_order::{
    clear_held_locks, held_lock_count, is_verification_enabled, set_verification_enabled,
};

/// Configuration for thread-safe operations
#[derive(Debug, Clone)]
pub struct ThreadSafetyConfig {
    /// Default timeout for synchronous JavaScript execution (ms)
    pub js_eval_timeout_ms: u64,

    /// Default timeout for main thread dispatch (ms)
    pub main_thread_timeout_ms: u64,

    /// Maximum retry attempts for failed operations
    pub max_retries: u32,

    /// Delay between retry attempts (ms)
    pub retry_delay_ms: u64,

    /// Enable lock order verification in debug builds
    pub debug_lock_order: bool,
}

impl Default for ThreadSafetyConfig {
    fn default() -> Self {
        Self {
            js_eval_timeout_ms: 5000,
            main_thread_timeout_ms: 30000,
            max_retries: 3,
            retry_delay_ms: 100,
            debug_lock_order: cfg!(debug_assertions),
        }
    }
}

impl ThreadSafetyConfig {
    /// Create a new configuration with custom values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set JavaScript evaluation timeout
    pub fn with_js_eval_timeout(mut self, timeout_ms: u64) -> Self {
        self.js_eval_timeout_ms = timeout_ms;
        self
    }

    /// Set main thread dispatch timeout
    pub fn with_main_thread_timeout(mut self, timeout_ms: u64) -> Self {
        self.main_thread_timeout_ms = timeout_ms;
        self
    }

    /// Set maximum retry attempts
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Set retry delay
    pub fn with_retry_delay(mut self, delay_ms: u64) -> Self {
        self.retry_delay_ms = delay_ms;
        self
    }

    /// Enable or disable lock order verification
    pub fn with_lock_order_verification(mut self, enabled: bool) -> Self {
        self.debug_lock_order = enabled;
        self
    }
}

#[cfg(test)]
mod tests {
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
            .with_retry_delay(200);

        assert_eq!(config.js_eval_timeout_ms, 10000);
        assert_eq!(config.main_thread_timeout_ms, 60000);
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.retry_delay_ms, 200);
    }
}
