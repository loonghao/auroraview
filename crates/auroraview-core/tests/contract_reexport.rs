// Verifies `auroraview_core::contract` is the same crate as `auroraview_contract`.
#[test]
fn core_reexports_the_contract_crate() {
    use auroraview_core::contract::{BackendFamily, EmbedMode, Features, ThreadModel};
    assert_eq!(BackendFamily::Native.name(), "native");
    assert_eq!(EmbedMode::OutOfProcess.name(), "out_of_process");
    assert!(!ThreadModel::StaApartment.blocking_dispatch_is_safe());
    assert!(Features::ALL.contains(Features::CDP));
}
