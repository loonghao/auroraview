//! Tests for auroraview-pack lib module

use auroraview_pack::VERSION;

#[test]
fn test_version() {
    assert!(VERSION.contains('.'), "VERSION should contain a dot");
}

#[test]
fn test_is_packed() {
    // In test environment, should not be packed
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
    // Semver has at least major.minor
    assert!(parts.len() >= 2, "Expected at least major.minor in version");
    // Each part should parse as a number
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
    // In test environment current_exe is the test runner, not a packed app
    let result = auroraview_pack::read_overlay();
    // Should not error out even if the exe has no overlay
    assert!(result.is_ok());
    // And should return None (no overlay in test runner)
    assert!(result.unwrap().is_none());
}
