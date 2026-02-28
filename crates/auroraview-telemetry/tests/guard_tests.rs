//! Tests for auroraview-telemetry guard module.

use auroraview_telemetry::Telemetry;

#[test]
fn test_is_enabled_default() {
    // Before init, should be disabled
    assert!(!Telemetry::is_enabled());
}

#[test]
fn test_enable_disable() {
    Telemetry::enable();
    assert!(Telemetry::is_enabled());
    Telemetry::disable();
    assert!(!Telemetry::is_enabled());
}
