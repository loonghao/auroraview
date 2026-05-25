//! Workspace-level shared constants.
//!
//! Centralizes string literals (environment variable names, well-known
//! identifiers, etc.) that may be referenced from multiple crates or
//! entry points (CLI, packed runtime, desktop bindings, future hosts).
//! Keeping them here avoids string drift between code, documentation,
//! and CI scripts.
//!
//! Also hosts small environment-variable parsing helpers (e.g.
//! [`parse_truthy`]) that are referenced alongside these constants — co-locating
//! them avoids a single-function `env` module while keeping the same
//! call sites consistent.

/// Environment variable used to override the effective
/// `capture_file_drop` setting at runtime in packed apps and any other
/// AuroraView entry point that opts into the same override semantics.
///
/// See RFC 0015 §4.3 and RFC 0017 §3 for the precedence rules:
/// `env > overlay/config > Rust default (false)`.
pub const CAPTURE_FILE_DROP_ENV: &str = "AURORAVIEW_CAPTURE_FILE_DROP";

/// Parse a truthy/falsy literal (case-insensitive, trimmed).
///
/// Recognises the union of common boolean spellings used by the runtimes
/// AuroraView interoperates with:
///
/// - truthy: `1`, `true`, `on`, `yes`, `enabled`
/// - falsy:  `0`, `false`, `off`, `no`, `disabled`
///
/// Anything else returns `None`. Callers are expected to log / warn on
/// `None` so misspellings surface to operators instead of silently
/// falling back to a default.
pub fn parse_truthy(s: &str) -> Option<bool> {
    let s = s.trim();
    const TRUE_LITERALS: &[&str] = &["1", "true", "on", "yes", "enabled"];
    const FALSE_LITERALS: &[&str] = &["0", "false", "off", "no", "disabled"];

    if TRUE_LITERALS.iter().any(|v| s.eq_ignore_ascii_case(v)) {
        Some(true)
    } else if FALSE_LITERALS.iter().any(|v| s.eq_ignore_ascii_case(v)) {
        Some(false)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truthy_literals() {
        for s in ["1", "true", "TRUE", "True", "on", "ON", "yes", "Enabled"] {
            assert_eq!(parse_truthy(s), Some(true), "expected truthy: {s}");
        }
    }

    #[test]
    fn falsy_literals() {
        for s in ["0", "false", "FALSE", "off", "OFF", "no", "Disabled"] {
            assert_eq!(parse_truthy(s), Some(false), "expected falsy: {s}");
        }
    }

    #[test]
    fn unknown_returns_none() {
        for s in ["", "  ", "maybe", "2", "tru", "of"] {
            assert_eq!(parse_truthy(s), None, "expected None: {s:?}");
        }
    }

    #[test]
    fn whitespace_is_trimmed() {
        assert_eq!(parse_truthy("  true  "), Some(true));
        assert_eq!(parse_truthy("\tfalse\n"), Some(false));
    }
}
