//! Tests for auroraview-telemetry guard module.

use auroraview_telemetry::Telemetry;

#[test]
fn test_is_enabled_default() {
    // Before init, telemetry should not be enabled
    assert!(!Telemetry::is_enabled());
}

#[test]
fn test_enable_disable() {
    Telemetry::disable();
    assert!(!Telemetry::is_enabled());
    Telemetry::enable();
    assert!(Telemetry::is_enabled());
    // Reset
    Telemetry::disable();
}
