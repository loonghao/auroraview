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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_id_generator_sequential() {
        let gen = IdGenerator::new();
        let id1 = gen.next();
        let id2 = gen.next();
        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
    }

    #[test]
    fn test_id_generator_string() {
        let gen = IdGenerator::new();
        let id = gen.next_string();
        assert!(id.starts_with("id_"));
    }

    #[test]
    fn test_id_generator_with_prefix() {
        let gen = IdGenerator::new();
        let id = gen.next_with_prefix("msg");
        assert!(id.starts_with("msg_"));
    }

    #[test]
    fn test_id_generator_thread_safe() {
        let gen = Arc::new(IdGenerator::new());
        let mut handles = vec![];

        for _ in 0..5 {
            let gen_clone = gen.clone();
            let handle = thread::spawn(move || {
                let mut ids = vec![];
                for _ in 0..10 {
                    ids.push(gen_clone.next());
                }
                ids
            });
            handles.push(handle);
        }

        let mut all_ids = vec![];
        for handle in handles {
            all_ids.extend(handle.join().unwrap());
        }

        // Verify all IDs are unique
        all_ids.sort();
        all_ids.dedup();
        assert_eq!(all_ids.len(), 50);
    }

    #[test]
    fn test_id_generator_with_start() {
        let gen = IdGenerator::with_start(100);
        assert_eq!(gen.next(), 100);
        assert_eq!(gen.next(), 101);
    }

    #[test]
    fn test_current_value() {
        let gen = IdGenerator::new();
        assert_eq!(gen.current(), 0);
        gen.next();
        assert_eq!(gen.current(), 1);
    }
}
