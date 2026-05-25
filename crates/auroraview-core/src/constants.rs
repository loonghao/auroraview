//! Workspace-level shared constants.
//!
//! Centralizes string literals (environment variable names, well-known
//! identifiers, etc.) that may be referenced from multiple crates or
//! entry points (CLI, packed runtime, desktop bindings, future hosts).
//! Keeping them here avoids string drift between code, documentation,
//! and CI scripts.

/// Environment variable used to override the effective
/// `capture_file_drop` setting at runtime in packed apps and any other
/// AuroraView entry point that opts into the same override semantics.
///
/// See RFC 0015 §4.3 and RFC 0017 §3 for the precedence rules:
/// `env > overlay/config > Rust default (false)`.
pub const CAPTURE_FILE_DROP_ENV: &str = "AURORAVIEW_CAPTURE_FILE_DROP";
