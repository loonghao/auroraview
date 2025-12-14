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
