//! Unit tests for ProtectConfig, EncryptionConfig, ProtectionMethod, and ProtectError.

use auroraview_protect::{EncryptionConfig, ProtectConfig, ProtectError, ProtectionMethod};

// ─────────────────────────────────────────────────────────────
// ProtectionMethod
// ─────────────────────────────────────────────────────────────

#[test]
fn protection_method_serialization() {
    let method = ProtectionMethod::Bytecode;
    let json = serde_json::to_string(&method).unwrap();
    assert_eq!(json, r#""bytecode""#);

    // snake_case serialization: Py2Pyd → "py2_pyd"
    let method2 = ProtectionMethod::Py2Pyd;
    let json2 = serde_json::to_string(&method2).unwrap();
    assert_eq!(json2, r#""py2_pyd""#);
}

#[test]
fn protection_method_deserialization() {
    let method: ProtectionMethod = serde_json::from_str(r#""bytecode""#).unwrap();
    assert_eq!(method, ProtectionMethod::Bytecode);

    // snake_case: Py2Pyd deserializes from "py2_pyd"
    let method2: ProtectionMethod = serde_json::from_str(r#""py2_pyd""#).unwrap();
    assert_eq!(method2, ProtectionMethod::Py2Pyd);
}

#[test]
fn protection_method_equality() {
    assert_eq!(ProtectionMethod::Bytecode, ProtectionMethod::Bytecode);
    assert_ne!(ProtectionMethod::Bytecode, ProtectionMethod::Py2Pyd);
}

// ─────────────────────────────────────────────────────────────
// EncryptionConfig
// ─────────────────────────────────────────────────────────────

#[test]
fn encryption_config_default_disabled() {
    let config = EncryptionConfig::default();
    assert!(!config.enabled);
    assert_eq!(config.algorithm, "x25519");
    assert!(config.public_key.is_none());
    assert!(config.private_key.is_none());
    assert!(config.key_output_path.is_none());
}

#[test]
fn encryption_config_enabled() {
    let config = EncryptionConfig::enabled();
    assert!(config.enabled);
    assert_eq!(config.algorithm, "x25519");
}

#[test]
fn encryption_config_with_x25519() {
    let config = EncryptionConfig::enabled().with_x25519();
    assert_eq!(config.algorithm, "x25519");
}

#[test]
fn encryption_config_with_p256() {
    let config = EncryptionConfig::enabled().with_p256();
    assert_eq!(config.algorithm, "p256");
}

#[test]
fn encryption_config_with_keys() {
    let config = EncryptionConfig::enabled()
        .with_keys("pub_hex".to_string(), "priv_hex".to_string());
    assert_eq!(config.public_key, Some("pub_hex".to_string()));
    assert_eq!(config.private_key, Some("priv_hex".to_string()));
}

#[test]
fn encryption_config_serialization_roundtrip() {
    let config = EncryptionConfig::enabled()
        .with_p256()
        .with_keys("pub".to_string(), "priv".to_string());

    let json = serde_json::to_string(&config).unwrap();
    let restored: EncryptionConfig = serde_json::from_str(&json).unwrap();

    assert!(restored.enabled);
    assert_eq!(restored.algorithm, "p256");
    assert_eq!(restored.public_key, Some("pub".to_string()));
}

// ─────────────────────────────────────────────────────────────
// ProtectConfig
// ─────────────────────────────────────────────────────────────

#[test]
fn protect_config_default_method_is_bytecode() {
    let config = ProtectConfig::default();
    assert_eq!(config.method, ProtectionMethod::Bytecode);
}

#[test]
fn protect_config_default_optimization_is_2() {
    let config = ProtectConfig::default();
    assert_eq!(config.optimization, 2);
}

#[test]
fn protect_config_default_exclude_patterns() {
    let config = ProtectConfig::default();
    assert!(!config.exclude.is_empty());
    // Should exclude pycache and test files
    assert!(config.exclude.iter().any(|p| p.contains("__pycache__")));
    assert!(config.exclude.iter().any(|p| p.contains("test_")));
}

#[test]
fn protect_config_builder_python_path() {
    let config = ProtectConfig::new().python_path("/usr/bin/python3");
    assert_eq!(config.python_path, Some("/usr/bin/python3".to_string()));
}

#[test]
fn protect_config_builder_python_version() {
    let config = ProtectConfig::new().python_version("3.11");
    assert_eq!(config.python_version, Some("3.11".to_string()));
}

#[test]
fn protect_config_builder_optimization_clamped_to_3() {
    let config = ProtectConfig::new().optimization(10);
    assert_eq!(config.optimization, 3); // clamped at 3
}

#[test]
fn protect_config_builder_optimization_valid_values() {
    for level in 0..=3u8 {
        let config = ProtectConfig::new().optimization(level);
        assert_eq!(config.optimization, level);
    }
}

#[test]
fn protect_config_builder_keep_temp() {
    let config = ProtectConfig::new().keep_temp(true);
    assert!(config.keep_temp);
}

#[test]
fn protect_config_builder_target_dcc() {
    let config = ProtectConfig::new().target_dcc("maya");
    assert_eq!(config.target_dcc, Some("maya".to_string()));
}

#[test]
fn protect_config_builder_add_exclude() {
    let config = ProtectConfig::new().exclude("**/vendor/**");
    assert!(config.exclude.iter().any(|p| p == "**/vendor/**"));
}

#[test]
fn protect_config_default_no_encryption() {
    let config = ProtectConfig::default();
    assert!(!config.encryption.enabled);
}

#[test]
fn protect_config_anti_debug_defaults_false() {
    let config = ProtectConfig::default();
    assert!(!config.anti_debug);
    assert!(!config.integrity_check);
}

#[test]
fn protect_config_bind_machines_empty() {
    let config = ProtectConfig::default();
    assert!(config.bind_machines.is_empty());
}

#[test]
fn protect_config_serialization_roundtrip() {
    let config = ProtectConfig::new()
        .python_version("3.11")
        .optimization(1)
        .target_dcc("houdini");

    let json = serde_json::to_string(&config).unwrap();
    let restored: ProtectConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.python_version, Some("3.11".to_string()));
    assert_eq!(restored.optimization, 1);
    assert_eq!(restored.target_dcc, Some("houdini".to_string()));
}

// ─────────────────────────────────────────────────────────────
// ProtectError display messages
// ─────────────────────────────────────────────────────────────

#[test]
fn protect_error_messages_contain_context() {
    let err = ProtectError::Compilation("rustc missing".to_string());
    assert!(err.to_string().contains("rustc missing"));

    let err = ProtectError::PythonCompile("syntax error".to_string());
    assert!(err.to_string().contains("syntax error"));

    let err = ProtectError::Encryption("invalid key".to_string());
    assert!(err.to_string().contains("invalid key"));

    let err = ProtectError::Decryption("tag mismatch".to_string());
    assert!(err.to_string().contains("tag mismatch"));

    let err = ProtectError::FileNotFound("module.py".to_string());
    assert!(err.to_string().contains("module.py"));

    let err = ProtectError::Config("missing field".to_string());
    assert!(err.to_string().contains("missing field"));

    let err = ProtectError::InvalidKey("bad hex".to_string());
    assert!(err.to_string().contains("bad hex"));

    let err = ProtectError::KeyGeneration("entropy failed".to_string());
    assert!(err.to_string().contains("entropy failed"));

    let err = ProtectError::Obfuscation("ast error".to_string());
    assert!(err.to_string().contains("ast error"));
}

#[test]
fn protect_error_io_from_conversion() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
    let err: ProtectError = io_err.into();
    assert!(err.to_string().contains("file missing") || err.to_string().contains("IO"));
}

// ─── Additional coverage ──────────────────────────────────────────────────────

#[test]
fn protect_config_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ProtectConfig>();
    assert_send_sync::<EncryptionConfig>();
}

#[test]
fn protection_method_debug_non_empty() {
    let debug_str = format!("{:?}", ProtectionMethod::Bytecode);
    assert!(!debug_str.is_empty());
}

#[test]
fn encryption_config_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<EncryptionConfig>();
}

#[test]
fn protect_config_clone() {
    let config = ProtectConfig::default();
    let cloned = config.clone();
    assert_eq!(cloned.method, config.method);
    assert_eq!(cloned.optimization, config.optimization);
}

#[test]
fn encryption_config_clone() {
    let enc = EncryptionConfig::default();
    let cloned = enc.clone();
    assert_eq!(cloned.enabled, enc.enabled);
}

#[test]
fn protect_config_debug_non_empty() {
    let config = ProtectConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(!debug_str.is_empty());
}

#[test]
fn protection_method_all_variants_roundtrip() {
    for method in [ProtectionMethod::Bytecode, ProtectionMethod::Py2Pyd] {
        let json = serde_json::to_string(&method).unwrap();
        let restored: ProtectionMethod = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, method);
    }
}

#[test]
fn protect_error_debug_non_empty() {
    let err = ProtectError::Compilation("test".to_string());
    let debug_str = format!("{:?}", err);
    assert!(!debug_str.is_empty());
}

#[test]
fn protect_config_optimization_builder_zero() {
    let config = ProtectConfig::default().optimization(0);
    assert_eq!(config.optimization, 0);
}

#[test]
fn protect_config_optimization_builder_one() {
    let config = ProtectConfig::default().optimization(1);
    assert_eq!(config.optimization, 1);
}

#[test]
fn protect_config_keep_temp_true() {
    let config = ProtectConfig::default().keep_temp(true);
    assert!(config.keep_temp);
}

#[test]
fn protect_config_keep_temp_false() {
    let config = ProtectConfig::default().keep_temp(false);
    assert!(!config.keep_temp);
}
