//! Utility functions and helpers
//!
//! This module provides logging initialization and re-exports the IdGenerator
//! from auroraview-core for backward compatibility.

use tracing_subscriber::{fmt, EnvFilter};

// Re-export IdGenerator from core for backward compatibility
// This may be unused in the library itself but is exported for external users
#[allow(unused_imports)]
pub use auroraview_core::id_generator::IdGenerator;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_init() {
        // Test that logging can be initialized
        init_logging();
        // Call again to ensure it's idempotent
        init_logging();
        // If we get here without panicking, the test passes
    }

    #[test]
    fn test_id_generator_reexport() {
        // Test that the re-exported IdGenerator works
        let gen = IdGenerator::new();
        let id1 = gen.next();
        let id2 = gen.next();
        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
    }
}
