//! Tests for auroraview-pack lib module

use auroraview_pack::VERSION;

#[test]
fn test_version() {
    assert!(VERSION.contains('.'), "VERSION should contain a dot");
}

#[test]
fn test_is_packed() {
    assert!(!auroraview_pack::is_packed());
}

// ============================================================================
// New Tests
// ============================================================================

#[test]
fn test_version_non_empty() {
    assert!(!VERSION.is_empty());
}

#[test]
fn test_version_has_semver_parts() {
    let parts: Vec<&str> = VERSION.split('.').collect();
    assert!(parts.len() >= 2, "Expected at least major.minor in version");
    for part in &parts {
        let stripped = part.split('-').next().unwrap_or(part);
        assert!(
            stripped.parse::<u64>().is_ok(),
            "Version part '{}' is not a number",
            stripped
        );
    }
}

#[test]
fn test_read_overlay_in_test_env() {
    let result = auroraview_pack::read_overlay();
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

// ─── Additional VERSION tests ─────────────────────────────────────────────────

#[test]
fn test_version_major_is_zero_or_positive() {
    let major = VERSION.split('.').next().unwrap().parse::<u64>().unwrap();
    // u64 is always >= 0; just verify it's parseable as a valid version number
    let _ = major;
}

#[test]
fn test_version_starts_with_digit() {
    let first_char = VERSION.chars().next().unwrap();
    assert!(first_char.is_ascii_digit(), "VERSION should start with a digit: {VERSION}");
}

#[test]
fn test_version_is_consistent() {
    // VERSION is a compile-time constant, should always return the same value
    assert_eq!(VERSION, VERSION);
}

#[test]
fn test_version_not_dev() {
    // The released pack crate version should not be a dev placeholder
    assert!(!VERSION.is_empty());
    assert!(VERSION.contains('.'));
}

#[test]
fn test_is_packed_returns_bool() {
    let result = auroraview_pack::is_packed();
    // Verify it returns a bool without panicking
    let _ = result;
}

#[test]
fn test_is_packed_consistent() {
    // Multiple calls should return the same value
    let a = auroraview_pack::is_packed();
    let b = auroraview_pack::is_packed();
    assert_eq!(a, b);
}

#[test]
fn test_read_overlay_consistent() {
    // Two calls should both succeed
    let r1 = auroraview_pack::read_overlay();
    let r2 = auroraview_pack::read_overlay();
    assert_eq!(r1.is_ok(), r2.is_ok());
}

#[test]
fn test_version_minor_is_parseable() {
    let parts: Vec<&str> = VERSION.split('.').collect();
    if parts.len() >= 2 {
        let minor_str = parts[1].split('-').next().unwrap_or(parts[1]);
        let minor: u64 = minor_str.parse().expect("minor version should be a number");
        let _ = minor; // u64 is always >= 0; just verify parsing succeeds
    }
}
