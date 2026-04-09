//! Tests for auroraview-pack lib module

use auroraview_pack::VERSION;

#[test]
fn version_has_parseable_semver_parts() {
    assert!(!VERSION.is_empty());

    let parts: Vec<&str> = VERSION.split('.').collect();
    assert!(parts.len() >= 2, "Expected at least major.minor in version");

    let major = parts[0].parse::<u64>().expect("major version should be numeric");
    let minor = parts[1]
        .split('-')
        .next()
        .unwrap_or(parts[1])
        .parse::<u64>()
        .expect("minor version should be numeric");

    let _ = (major, minor);
}

#[test]
fn is_packed_is_stable_in_test_env() {
    let first = auroraview_pack::is_packed();
    let second = auroraview_pack::is_packed();

    assert!(!first);
    assert_eq!(first, second);
}

#[test]
fn read_overlay_returns_none_in_test_env() {
    let overlay = auroraview_pack::read_overlay().expect("read_overlay should succeed in tests");
    assert!(overlay.is_none());
}

