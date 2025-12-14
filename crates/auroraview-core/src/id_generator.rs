//! Thread-safe ID generation utilities

use std::sync::atomic::{AtomicU64, Ordering};

/// Thread-safe counter for generating unique IDs
pub struct IdGenerator {
    counter: AtomicU64,
}

impl IdGenerator {
    /// Create a new ID generator
    pub fn new() -> Self {
        Self {
            counter: AtomicU64::new(0),
        }
    }

    /// Create a new ID generator starting from a specific value
    pub fn with_start(start: u64) -> Self {
        Self {
            counter: AtomicU64::new(start),
        }
    }

    /// Generate a new unique ID
    pub fn next(&self) -> u64 {
        self.counter.fetch_add(1, Ordering::SeqCst)
    }

    /// Generate a new unique ID as a string
    pub fn next_string(&self) -> String {
        format!("id_{}", self.next())
    }

    /// Generate a new unique ID with a prefix
    pub fn next_with_prefix(&self, prefix: &str) -> String {
        format!("{}_{}", prefix, self.next())
    }

    /// Get current value without incrementing
    pub fn current(&self) -> u64 {
        self.counter.load(Ordering::SeqCst)
    }
}

impl Default for IdGenerator {
    fn default() -> Self {
        Self::new()
    }
}
