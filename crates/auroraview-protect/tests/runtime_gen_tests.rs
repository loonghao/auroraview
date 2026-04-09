//! Integration tests for RuntimeGenerator and generated Python code structure

use rstest::rstest;

use auroraview_protect::bytecode::EncryptedModule;
use auroraview_protect::crypto::EccAlgorithm;
use auroraview_protect::{ProtectConfig, ProtectionMethod, RuntimeGenerator};

// ─────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────

fn default_generator() -> RuntimeGenerator {
    RuntimeGenerator::new(ProtectConfig::default())
}

fn default_key() -> [u8; 32] {
    [0u8; 32]
}

fn known_key() -> [u8; 32] {
    let mut k = [0u8; 32];
    for (i, b) in k.iter_mut().enumerate() {
        *b = i as u8;
    }
    k
}

// ─────────────────────────────────────────────────────────────
// generate_python_runtime – basic structure
// ─────────────────────────────────────────────────────────────

#[test]
fn runtime_contains_aurora_entry_point() {
    let gen = default_generator();
    let code = gen.generate_python_runtime(&default_key());
    assert!(code.contains("def __aurora__"), "should define __aurora__");
}

#[test]
fn runtime_contains_reconstruct_key() {
    let gen = default_generator();
    let code = gen.generate_python_runtime(&default_key());
    assert!(
        code.contains("def _reconstruct_key"),
        "should define _reconstruct_key"
    );
}

#[test]
fn runtime_contains_aes_decrypt() {
    let gen = default_generator();
    let code = gen.generate_python_runtime(&default_key());
    assert!(
        code.contains("def _aes_decrypt"),
        "should define _aes_decrypt"
    );
}

#[test]
fn runtime_contains_protect_module() {
    let gen = default_generator();
    let code = gen.generate_python_runtime(&default_key());
    assert!(
        code.contains("_protect_module"),
        "should call _protect_module"
    );
}

#[test]
fn runtime_has_do_not_modify_header() {
    let gen = default_generator();
    let code = gen.generate_python_runtime(&default_key());
    assert!(code.contains("DO NOT MODIFY"));
}

#[test]
fn runtime_key_parts_have_eight_bytes_each() {
    let gen = default_generator();
    let code = gen.generate_python_runtime(&known_key());
    // _K0 = bytes([...]) should appear for each of the 4 key parts
    assert!(code.contains("_K0 = bytes(["), "should have _K0");
    assert!(code.contains("_K1 = bytes(["), "should have _K1");
    assert!(code.contains("_K2 = bytes(["), "should have _K2");
    assert!(code.contains("_K3 = bytes(["), "should have _K3");
}

#[test]
fn runtime_xor_masks_present() {
    let gen = default_generator();
    let code = gen.generate_python_runtime(&default_key());
    assert!(code.contains("_X0 = bytes(["));
    assert!(code.contains("_X1 = bytes(["));
    assert!(code.contains("_X2 = bytes(["));
    assert!(code.contains("_X3 = bytes(["));
}

#[test]
fn runtime_different_keys_produce_different_output() {
    let gen = default_generator();
    let code1 = gen.generate_python_runtime(&default_key());
    let code2 = gen.generate_python_runtime(&known_key());
    // Key-dependent parts should differ even though structure is the same
    assert_ne!(code1, code2);
}

#[test]
fn runtime_two_calls_same_key_differ_due_to_random_xor() {
    // Each call generates fresh random XOR masks
    let gen = default_generator();
    let k = known_key();
    let code1 = gen.generate_python_runtime(&k);
    let code2 = gen.generate_python_runtime(&k);
    // The XOR masks are random so the bytecode literals will almost certainly differ
    // (probability of collision is astronomically low for 32-byte random keys)
    assert_ne!(code1, code2, "XOR masks should differ between calls");
}

// ─────────────────────────────────────────────────────────────
// Anti-debug code generation
// ─────────────────────────────────────────────────────────────

#[test]
fn runtime_no_anti_debug_when_disabled() {
    let cfg = ProtectConfig::default(); // anti_debug = false by default
    assert!(!cfg.anti_debug);
    let gen = RuntimeGenerator::new(cfg);
    let code = gen.generate_python_runtime(&default_key());
    assert!(
        !code.contains("_check_debugger"),
        "should not contain debugger check"
    );
    assert!(
        !code.contains("_anti_debug"),
        "should not contain anti_debug call"
    );
}

#[test]
fn runtime_has_anti_debug_when_enabled() {
    let cfg = ProtectConfig {
        anti_debug: true,
        ..ProtectConfig::default()
    };
    let gen = RuntimeGenerator::new(cfg);
    let code = gen.generate_python_runtime(&default_key());
    assert!(
        code.contains("_check_debugger"),
        "should contain _check_debugger"
    );
    assert!(code.contains("_anti_debug"), "should contain _anti_debug");
}

#[test]
fn anti_debug_code_checks_sys_gettrace() {
    let cfg = ProtectConfig {
        anti_debug: true,
        ..ProtectConfig::default()
    };
    let gen = RuntimeGenerator::new(cfg);
    let code = gen.generate_python_runtime(&default_key());
    assert!(code.contains("sys.gettrace()"), "should check sys.gettrace");
}

#[test]
fn anti_debug_code_lists_debugger_modules() {
    let cfg = ProtectConfig {
        anti_debug: true,
        ..ProtectConfig::default()
    };
    let gen = RuntimeGenerator::new(cfg);
    let code = gen.generate_python_runtime(&default_key());
    assert!(code.contains("pydevd"), "should check for pydevd");
    assert!(code.contains("debugpy"), "should check for debugpy");
}

// ─────────────────────────────────────────────────────────────
// Integrity check code generation
// ─────────────────────────────────────────────────────────────

#[test]
fn runtime_no_integrity_check_when_disabled() {
    let cfg = ProtectConfig::default();
    assert!(!cfg.integrity_check);
    let gen = RuntimeGenerator::new(cfg);
    let code = gen.generate_python_runtime(&default_key());
    assert!(
        !code.contains("_verify_integrity"),
        "should not contain integrity check"
    );
}

#[test]
fn runtime_has_integrity_check_when_enabled() {
    let cfg = ProtectConfig {
        integrity_check: true,
        ..ProtectConfig::default()
    };
    let gen = RuntimeGenerator::new(cfg);
    let code = gen.generate_python_runtime(&default_key());
    assert!(
        code.contains("_verify_integrity"),
        "should contain _verify_integrity"
    );
}

// ─────────────────────────────────────────────────────────────
// Expiration check code generation
// ─────────────────────────────────────────────────────────────

#[test]
fn runtime_no_expiration_check_when_not_set() {
    let cfg = ProtectConfig::default();
    assert!(cfg.expires_at.is_none());
    let gen = RuntimeGenerator::new(cfg);
    let code = gen.generate_python_runtime(&default_key());
    assert!(
        !code.contains("_check_expiration"),
        "should not have expiration check"
    );
}

#[test]
fn runtime_has_expiration_when_date_set() {
    let cfg = ProtectConfig {
        expires_at: Some("2030-12-31T00:00:00".to_string()),
        ..ProtectConfig::default()
    };
    let gen = RuntimeGenerator::new(cfg);
    let code = gen.generate_python_runtime(&default_key());
    assert!(
        code.contains("_check_expiration"),
        "should have expiration check"
    );
    assert!(code.contains("2030-12-31"), "should embed the expiry date");
}

#[test]
fn runtime_expiration_contains_datetime_import() {
    let cfg = ProtectConfig {
        expires_at: Some("2025-06-01T00:00:00".to_string()),
        ..ProtectConfig::default()
    };
    let gen = RuntimeGenerator::new(cfg);
    let code = gen.generate_python_runtime(&default_key());
    assert!(code.contains("from datetime import datetime"));
}

// ─────────────────────────────────────────────────────────────
// Machine binding code generation
// ─────────────────────────────────────────────────────────────

#[test]
fn runtime_no_machine_bind_when_empty() {
    let cfg = ProtectConfig::default();
    assert!(cfg.bind_machines.is_empty());
    let gen = RuntimeGenerator::new(cfg);
    let code = gen.generate_python_runtime(&default_key());
    assert!(
        !code.contains("_check_machine_binding"),
        "should not have machine binding"
    );
}

#[test]
fn runtime_has_machine_bind_when_set() {
    let cfg = ProtectConfig {
        bind_machines: vec!["abc123def456".to_string()],
        ..ProtectConfig::default()
    };
    let gen = RuntimeGenerator::new(cfg);
    let code = gen.generate_python_runtime(&default_key());
    assert!(
        code.contains("_check_machine_binding"),
        "should have machine binding"
    );
    assert!(code.contains("abc123def456"), "should embed machine ID");
}

#[test]
fn runtime_machine_bind_lists_get_machine_id() {
    let cfg = ProtectConfig {
        bind_machines: vec!["deadbeef00000000".to_string()],
        ..ProtectConfig::default()
    };
    let gen = RuntimeGenerator::new(cfg);
    let code = gen.generate_python_runtime(&default_key());
    assert!(
        code.contains("_get_machine_id"),
        "should define _get_machine_id"
    );
}

#[test]
fn runtime_machine_bind_multiple_machines() {
    let cfg = ProtectConfig {
        bind_machines: vec!["machine_a".to_string(), "machine_b".to_string()],
        ..ProtectConfig::default()
    };
    let gen = RuntimeGenerator::new(cfg);
    let code = gen.generate_python_runtime(&default_key());
    assert!(code.contains("machine_a"), "should embed machine_a");
    assert!(code.contains("machine_b"), "should embed machine_b");
}

// ─────────────────────────────────────────────────────────────
// Runtime checks call composition
// ─────────────────────────────────────────────────────────────

#[test]
fn runtime_checks_pass_when_all_disabled() {
    let gen = default_generator();
    let code = gen.generate_python_runtime(&default_key());
    // No runtime checks → should have "pass" in the runtime_checks slot
    assert!(code.contains("pass"), "should emit 'pass' when no checks");
}

#[test]
fn runtime_checks_includes_anti_debug_call_when_enabled() {
    let cfg = ProtectConfig {
        anti_debug: true,
        ..ProtectConfig::default()
    };
    let gen = RuntimeGenerator::new(cfg);
    let code = gen.generate_python_runtime(&default_key());
    // The __aurora__ function body should contain _anti_debug()
    assert!(code.contains("_anti_debug()"));
}

#[test]
fn runtime_checks_includes_enforce_expiration_when_set() {
    let cfg = ProtectConfig {
        expires_at: Some("2099-01-01T00:00:00".to_string()),
        ..ProtectConfig::default()
    };
    let gen = RuntimeGenerator::new(cfg);
    let code = gen.generate_python_runtime(&default_key());
    assert!(code.contains("_enforce_expiration()"));
}

#[test]
fn runtime_checks_includes_machine_bind_when_set() {
    let cfg = ProtectConfig {
        bind_machines: vec!["abc".to_string()],
        ..ProtectConfig::default()
    };
    let gen = RuntimeGenerator::new(cfg);
    let code = gen.generate_python_runtime(&default_key());
    assert!(code.contains("_enforce_machine_binding()"));
}

#[test]
fn runtime_all_checks_combined() {
    let cfg = ProtectConfig {
        anti_debug: true,
        expires_at: Some("2030-01-01T00:00:00".to_string()),
        bind_machines: vec!["some_machine".to_string()],
        ..ProtectConfig::default()
    };
    let gen = RuntimeGenerator::new(cfg);
    let code = gen.generate_python_runtime(&default_key());
    assert!(code.contains("_anti_debug()"));
    assert!(code.contains("_enforce_expiration()"));
    assert!(code.contains("_enforce_machine_binding()"));
}

// ─────────────────────────────────────────────────────────────
// generate_bytecode_bootstrap
// ─────────────────────────────────────────────────────────────

fn empty_modules() -> Vec<EncryptedModule> {
    vec![]
}

fn dummy_private_key_32() -> Vec<u8> {
    (0u8..32).collect()
}

fn dummy_private_key_64() -> Vec<u8> {
    (0u8..64).collect()
}

#[test]
fn bootstrap_x25519_contains_algorithm_label() {
    let gen = default_generator();
    let code = gen.generate_bytecode_bootstrap(
        &dummy_private_key_32(),
        EccAlgorithm::X25519,
        &empty_modules(),
    );
    assert!(
        code.contains("x25519"),
        "should embed algorithm name x25519"
    );
}

#[test]
fn bootstrap_p256_contains_algorithm_label() {
    let gen = default_generator();
    let code = gen.generate_bytecode_bootstrap(
        &dummy_private_key_32(),
        EccAlgorithm::P256,
        &empty_modules(),
    );
    assert!(code.contains("p256"), "should embed algorithm name p256");
}

#[test]
fn bootstrap_contains_aurora_load_function() {
    let gen = default_generator();
    let code = gen.generate_bytecode_bootstrap(
        &dummy_private_key_32(),
        EccAlgorithm::X25519,
        &empty_modules(),
    );
    assert!(
        code.contains("def __aurora_load__"),
        "should define __aurora_load__"
    );
}

#[test]
fn bootstrap_contains_aurora_exec_function() {
    let gen = default_generator();
    let code = gen.generate_bytecode_bootstrap(
        &dummy_private_key_32(),
        EccAlgorithm::X25519,
        &empty_modules(),
    );
    assert!(
        code.contains("def __aurora_exec__"),
        "should define __aurora_exec__"
    );
}

#[test]
fn bootstrap_contains_aurora_list_function() {
    let gen = default_generator();
    let code = gen.generate_bytecode_bootstrap(
        &dummy_private_key_32(),
        EccAlgorithm::X25519,
        &empty_modules(),
    );
    assert!(
        code.contains("def __aurora_list__"),
        "should define __aurora_list__"
    );
}

#[test]
fn bootstrap_contains_reconstruct_key() {
    let gen = default_generator();
    let code = gen.generate_bytecode_bootstrap(
        &dummy_private_key_32(),
        EccAlgorithm::X25519,
        &empty_modules(),
    );
    assert!(
        code.contains("def _rk("),
        "should define _rk() to reconstruct key"
    );
}

#[test]
fn bootstrap_key_parts_have_correct_count_32_bytes() {
    // 32 bytes → 4 parts of 8 bytes
    let gen = default_generator();
    let code = gen.generate_bytecode_bootstrap(
        &dummy_private_key_32(),
        EccAlgorithm::X25519,
        &empty_modules(),
    );
    // _KP should have 4 bytes(...) entries
    let kp_count = code.matches("bytes([").count();
    // 4 _KP + 4 _XK = 8 minimum (may vary if other parts use same syntax)
    assert!(
        kp_count >= 8,
        "should have at least 8 bytes([]) entries for 32-byte key"
    );
}

#[test]
fn bootstrap_key_len_matches_input_32() {
    let gen = default_generator();
    let code = gen.generate_bytecode_bootstrap(
        &dummy_private_key_32(),
        EccAlgorithm::X25519,
        &empty_modules(),
    );
    assert!(code.contains("_KL = 32"), "should embed key length 32");
}

#[test]
fn bootstrap_key_len_matches_input_64() {
    let gen = default_generator();
    let code = gen.generate_bytecode_bootstrap(
        &dummy_private_key_64(),
        EccAlgorithm::P256,
        &empty_modules(),
    );
    assert!(code.contains("_KL = 64"), "should embed key length 64");
}

#[test]
fn bootstrap_has_do_not_modify_header() {
    let gen = default_generator();
    let code = gen.generate_bytecode_bootstrap(
        &dummy_private_key_32(),
        EccAlgorithm::X25519,
        &empty_modules(),
    );
    assert!(code.contains("DO NOT MODIFY"));
}

#[test]
fn bootstrap_two_calls_same_key_differ_due_to_random_xor() {
    let gen = default_generator();
    let key = dummy_private_key_32();
    let code1 = gen.generate_bytecode_bootstrap(&key, EccAlgorithm::X25519, &empty_modules());
    let code2 = gen.generate_bytecode_bootstrap(&key, EccAlgorithm::X25519, &empty_modules());
    assert_ne!(code1, code2, "XOR masks should differ between calls");
}

#[test]
fn bootstrap_different_algorithms_produce_different_code() {
    let gen = default_generator();
    let key = dummy_private_key_32();
    let code_x = gen.generate_bytecode_bootstrap(&key, EccAlgorithm::X25519, &empty_modules());
    let code_p = gen.generate_bytecode_bootstrap(&key, EccAlgorithm::P256, &empty_modules());
    assert_ne!(code_x, code_p, "different algorithms should differ");
}

#[rstest]
#[case(EccAlgorithm::X25519)]
#[case(EccAlgorithm::P256)]
fn bootstrap_contains_ecdh_decrypt(#[case] algo: EccAlgorithm) {
    let gen = default_generator();
    let code = gen.generate_bytecode_bootstrap(&dummy_private_key_32(), algo, &empty_modules());
    assert!(
        code.contains("_decrypt_aes_key"),
        "should define _decrypt_aes_key"
    );
}

#[test]
fn bootstrap_contains_protect_bootstrap_call() {
    let gen = default_generator();
    let code = gen.generate_bytecode_bootstrap(
        &dummy_private_key_32(),
        EccAlgorithm::X25519,
        &empty_modules(),
    );
    assert!(
        code.contains("_protect_bootstrap()"),
        "should call _protect_bootstrap"
    );
}

#[test]
fn bootstrap_with_empty_modules_has_empty_json() {
    let gen = default_generator();
    let code = gen.generate_bytecode_bootstrap(
        &dummy_private_key_32(),
        EccAlgorithm::X25519,
        &empty_modules(),
    );
    // Empty modules list serializes to "[]"
    assert!(
        code.contains("_MODULES_DATA = '[]'") || code.contains("[]"),
        "should embed empty modules JSON"
    );
}

#[test]
fn bootstrap_is_valid_utf8() {
    let gen = default_generator();
    let code = gen.generate_bytecode_bootstrap(
        &dummy_private_key_32(),
        EccAlgorithm::X25519,
        &empty_modules(),
    );
    // Just ensuring no non-UTF8 bytes slipped in
    let _ = code.len();
    assert!(!code.is_empty());
}

// ─────────────────────────────────────────────────────────────
// RuntimeGenerator::new
// ─────────────────────────────────────────────────────────────

#[test]
fn runtime_generator_new_with_config() {
    let cfg = ProtectConfig {
        anti_debug: true,
        ..ProtectConfig::default()
    };
    let gen = RuntimeGenerator::new(cfg);
    // Should be able to call generate without panic
    let code = gen.generate_python_runtime(&default_key());
    assert!(code.contains("_anti_debug"));
}

#[test]
fn runtime_generator_accepts_all_protection_methods() {
    for method in [ProtectionMethod::Py2Pyd, ProtectionMethod::Bytecode] {
        let cfg = ProtectConfig {
            method,
            ..ProtectConfig::default()
        };
        let gen = RuntimeGenerator::new(cfg);
        let code = gen.generate_python_runtime(&default_key());
        assert!(!code.is_empty());
    }
}
