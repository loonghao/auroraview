//! Utility functions and helpers

use tracing_subscriber::{fmt, EnvFilter};

/// Initialize logging for the library
pub fn init_logging() {
    // Only initialize once
    static INIT: std::sync::Once = std::sync::Once::new();

    INIT.call_once(|| {
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

        fmt()
            .with_env_filter(filter)
            .with_target(false)
            .with_thread_ids(true)
            .with_line_number(true)
            .init();
    });
}

/// Thread-safe counter for generating unique IDs
pub struct IdGenerator {
    #[allow(dead_code)]
    counter: std::sync::atomic::AtomicU64,
}

impl IdGenerator {
    /// Create a new ID generator
    pub fn new() -> Self {
        Self {
            counter: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Generate a new unique ID
    #[allow(dead_code)]
    pub fn next(&self) -> u64 {
        self.counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    /// Generate a new unique ID as a string
    #[allow(dead_code)]
    pub fn next_string(&self) -> String {
        format!("id_{}", self.next())
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
    use rstest::*;

    /// Fixture: Create a new IdGenerator
    #[fixture]
    fn id_generator() -> IdGenerator {
        IdGenerator::new()
    }

    #[rstest]
    fn test_id_generator(id_generator: IdGenerator) {
        let id1 = id_generator.next();
        let id2 = id_generator.next();
        assert_eq!(id2, id1 + 1);
    }

    #[rstest]
    fn test_id_generator_string(id_generator: IdGenerator) {
        let id = id_generator.next_string();
        assert!(id.starts_with("id_"));
    }

    #[rstest]
    fn test_id_generator_default() {
        let gen = IdGenerator::default();
        let id = gen.next();
        assert_eq!(id, 0);
    }

    #[rstest]
    fn test_id_generator_sequential(id_generator: IdGenerator) {
        let ids: Vec<u64> = (0..10).map(|_| id_generator.next()).collect();

        // Verify sequential IDs
        for (i, &id) in ids.iter().enumerate() {
            assert_eq!(id, i as u64);
        }
    }

    #[rstest]
    fn test_id_generator_thread_safe() {
        use std::sync::Arc;
        use std::thread;

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
            let ids = handle.join().unwrap();
            all_ids.extend(ids);
        }

        // Verify all IDs are unique
        all_ids.sort();
        for (i, &id) in all_ids.iter().enumerate() {
            assert_eq!(id, i as u64);
        }
    }

    #[rstest]
    fn test_logging_init() {
        // Test that logging can be initialized
        init_logging();
        // Call again to ensure it's idempotent
        init_logging();
        // If we get here without panicking, the test passes
    }
}
