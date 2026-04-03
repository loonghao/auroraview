//! Integration tests for bytecode protection

use auroraview_protect::{
    compile_to_bytecode,
    crypto::{decrypt_hybrid, EccAlgorithm, EccKeyPair},
    encrypt_bytecode, BytecodeFile,
};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test the complete bytecode compilation and encryption flow
#[test]
fn test_bytecode_compile_and_encrypt() {
    let temp_dir = TempDir::new().unwrap();
    let py_file = temp_dir.path().join("test_module.py");

    fs::write(
        &py_file,
        r#"
def hello(name):
    return f"Hello, {name}!"

def add(a, b):
    return a + b

class Calculator:
    def multiply(self, x, y):
        return x * y

if __name__ == "__main__":
    print(hello("World"))
"#,
    )
    .unwrap();

    let bytecode = compile_to_bytecode(&py_file, None, 0);

    if bytecode.is_err() {
        eprintln!("Skipping test: Python not available");
        return;
    }

    let bytecode = bytecode.unwrap();
    assert!(!bytecode.is_empty(), "Bytecode should not be empty");

    let bytecode_file = BytecodeFile {
        source: py_file.clone(),
        relative_path: PathBuf::from("test_module.py"),
        module_name: "test_module".to_string(),
        bytecode: bytecode.clone(),
        source_size: fs::metadata(&py_file).unwrap().len(),
    };

    // Test X25519 encryption
    let key_pair = EccKeyPair::generate(EccAlgorithm::X25519);
    let encryption_result = encrypt_bytecode(
        std::slice::from_ref(&bytecode_file),
        EccAlgorithm::X25519,
        Some(key_pair.clone()),
    )
    .unwrap();

    assert_eq!(encryption_result.modules.len(), 1);
    assert_eq!(encryption_result.modules[0].name, "test_module");

    let decrypted =
        decrypt_hybrid(&encryption_result.modules[0].package, &key_pair.private_key).unwrap();
    assert_eq!(
        decrypted, bytecode,
        "Decrypted bytecode should match original"
    );

    // Test P-256 encryption
    let p256_key_pair = EccKeyPair::generate(EccAlgorithm::P256);
    let p256_result = encrypt_bytecode(
        &[bytecode_file],
        EccAlgorithm::P256,
        Some(p256_key_pair.clone()),
    )
    .unwrap();

    let p256_decrypted =
        decrypt_hybrid(&p256_result.modules[0].package, &p256_key_pair.private_key).unwrap();
    assert_eq!(
        p256_decrypted, bytecode,
        "P-256 decrypted bytecode should match original"
    );
}

/// Test that wrong key fails decryption
#[test]
fn test_wrong_key_fails() {
    let temp_dir = TempDir::new().unwrap();
    let py_file = temp_dir.path().join("secret.py");

    fs::write(&py_file, "secret_value = 42").unwrap();

    let bytecode = compile_to_bytecode(&py_file, None, 0);
    if bytecode.is_err() {
        eprintln!("Skipping test: Python not available");
        return;
    }

    let bytecode = bytecode.unwrap();
    let bytecode_file = BytecodeFile {
        source: py_file.clone(),
        relative_path: PathBuf::from("secret.py"),
        module_name: "secret".to_string(),
        bytecode,
        source_size: fs::metadata(&py_file).unwrap().len(),
    };

    let key_pair1 = EccKeyPair::generate(EccAlgorithm::X25519);
    let encryption_result =
        encrypt_bytecode(&[bytecode_file], EccAlgorithm::X25519, Some(key_pair1)).unwrap();

    let key_pair2 = EccKeyPair::generate(EccAlgorithm::X25519);
    let result = decrypt_hybrid(
        &encryption_result.modules[0].package,
        &key_pair2.private_key,
    );

    assert!(result.is_err(), "Decryption with wrong key should fail");
}

/// Test key pair serialization
#[test]
fn test_key_pair_serialization() {
    let key_pair = EccKeyPair::generate(EccAlgorithm::X25519);

    let json = serde_json::to_string(&key_pair).unwrap();

    let deserialized: EccKeyPair = serde_json::from_str(&json).unwrap();

    assert_eq!(key_pair.algorithm, deserialized.algorithm);
    assert_eq!(key_pair.public_key, deserialized.public_key);
    assert_eq!(key_pair.private_key, deserialized.private_key);
}

// ─── Additional ECC key pair tests ────────────────────────────────────────────

#[test]
fn test_key_pair_generate_x25519_has_non_empty_keys() {
    let kp = EccKeyPair::generate(EccAlgorithm::X25519);
    assert!(!kp.public_key.is_empty());
    assert!(!kp.private_key.is_empty());
}

#[test]
fn test_key_pair_generate_p256_has_non_empty_keys() {
    let kp = EccKeyPair::generate(EccAlgorithm::P256);
    assert!(!kp.public_key.is_empty());
    assert!(!kp.private_key.is_empty());
}

#[test]
fn test_key_pair_generate_x25519_distinct_keys() {
    let kp1 = EccKeyPair::generate(EccAlgorithm::X25519);
    let kp2 = EccKeyPair::generate(EccAlgorithm::X25519);
    // Two random key pairs should be distinct
    assert_ne!(kp1.public_key, kp2.public_key);
    assert_ne!(kp1.private_key, kp2.private_key);
}

#[test]
fn test_key_pair_generate_p256_distinct_keys() {
    let kp1 = EccKeyPair::generate(EccAlgorithm::P256);
    let kp2 = EccKeyPair::generate(EccAlgorithm::P256);
    assert_ne!(kp1.public_key, kp2.public_key);
    assert_ne!(kp1.private_key, kp2.private_key);
}

#[test]
fn test_key_pair_p256_serialization_roundtrip() {
    let kp = EccKeyPair::generate(EccAlgorithm::P256);
    let json = serde_json::to_string(&kp).unwrap();
    let restored: EccKeyPair = serde_json::from_str(&json).unwrap();
    assert_eq!(kp.public_key, restored.public_key);
    assert_eq!(kp.private_key, restored.private_key);
    assert_eq!(kp.algorithm, restored.algorithm);
}

#[test]
fn test_key_pair_x25519_key_hex_valid() {
    let kp = EccKeyPair::generate(EccAlgorithm::X25519);
    // Keys should be valid hex
    assert!(kp.public_key.chars().all(|c| c.is_ascii_hexdigit()));
    assert!(kp.private_key.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_key_pair_p256_key_hex_valid() {
    let kp = EccKeyPair::generate(EccAlgorithm::P256);
    assert!(kp.public_key.chars().all(|c| c.is_ascii_hexdigit()));
    assert!(kp.private_key.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_key_pair_x25519_public_key_bytes() {
    let kp = EccKeyPair::generate(EccAlgorithm::X25519);
    let bytes = kp.public_key_bytes().unwrap();
    // X25519 public key is 32 bytes
    assert_eq!(bytes.len(), 32);
}

#[test]
fn test_key_pair_x25519_private_key_bytes() {
    let kp = EccKeyPair::generate(EccAlgorithm::X25519);
    let bytes = kp.private_key_bytes().unwrap();
    // X25519 private key is 32 bytes
    assert_eq!(bytes.len(), 32);
}

#[test]
fn test_ecc_algorithm_default() {
    let algo = EccAlgorithm::default();
    assert_eq!(algo, EccAlgorithm::X25519);
}

#[test]
fn test_ecc_algorithm_from_str_x25519() {
    use std::str::FromStr;
    assert_eq!(EccAlgorithm::from_str("x25519").unwrap(), EccAlgorithm::X25519);
    assert_eq!(EccAlgorithm::from_str("X25519").unwrap(), EccAlgorithm::X25519);
}

#[test]
fn test_ecc_algorithm_from_str_p256() {
    use std::str::FromStr;
    assert_eq!(EccAlgorithm::from_str("p256").unwrap(), EccAlgorithm::P256);
    assert_eq!(EccAlgorithm::from_str("P-256").unwrap(), EccAlgorithm::P256);
    assert_eq!(EccAlgorithm::from_str("secp256r1").unwrap(), EccAlgorithm::P256);
}

#[test]
fn test_ecc_algorithm_from_str_unknown_returns_error() {
    use std::str::FromStr;
    let result = EccAlgorithm::from_str("rsa2048");
    assert!(result.is_err());
}

#[test]
fn test_bytecode_file_clone() {
    let temp_dir = TempDir::new().unwrap();
    let py_file = temp_dir.path().join("mod.py");
    fs::write(&py_file, "x = 1").unwrap();

    let bf = BytecodeFile {
        source: py_file.clone(),
        relative_path: PathBuf::from("mod.py"),
        module_name: "mod".to_string(),
        bytecode: vec![1, 2, 3],
        source_size: 5,
    };

    let cloned = bf.clone();
    assert_eq!(bf.module_name, cloned.module_name);
    assert_eq!(bf.bytecode, cloned.bytecode);
    assert_eq!(bf.source_size, cloned.source_size);
}

#[test]
fn test_encrypt_bytecode_empty_slice_returns_empty() {
    let result = encrypt_bytecode(&[], EccAlgorithm::X25519, None).unwrap();
    assert_eq!(result.modules.len(), 0);
}

#[test]
fn test_encrypt_bytecode_auto_generates_key_when_none() {
    let temp_dir = TempDir::new().unwrap();
    let py_file = temp_dir.path().join("auto.py");
    fs::write(&py_file, "value = 99").unwrap();

    let bf = BytecodeFile {
        source: py_file.clone(),
        relative_path: PathBuf::from("auto.py"),
        module_name: "auto".to_string(),
        bytecode: vec![10, 20, 30, 40],
        source_size: fs::metadata(&py_file).unwrap().len(),
    };

    // When key_pair is None, implementation auto-generates
    let result = encrypt_bytecode(&[bf], EccAlgorithm::X25519, None);
    // Should not error out
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.modules.len(), 1);
    assert_eq!(result.modules[0].name, "auto");
}

#[test]
fn test_wrong_p256_key_fails() {
    let bf = BytecodeFile {
        source: PathBuf::from("test.py"),
        relative_path: PathBuf::from("test.py"),
        module_name: "test".to_string(),
        bytecode: vec![1, 2, 3, 4, 5],
        source_size: 5,
    };

    let key1 = EccKeyPair::generate(EccAlgorithm::P256);
    let result = encrypt_bytecode(&[bf], EccAlgorithm::P256, Some(key1)).unwrap();

    let key2 = EccKeyPair::generate(EccAlgorithm::P256);
    let decrypt_result = decrypt_hybrid(&result.modules[0].package, &key2.private_key);
    assert!(decrypt_result.is_err(), "Wrong P256 key should fail decryption");
}
