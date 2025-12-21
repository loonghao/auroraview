//! Integration tests for bytecode protection

use aurora_protect::{
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
    // Create a temporary directory with a Python file
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

    // Compile to bytecode
    let bytecode = compile_to_bytecode(&py_file, None, 0);

    // Skip if Python is not available
    if bytecode.is_err() {
        eprintln!("Skipping test: Python not available");
        return;
    }

    let bytecode = bytecode.unwrap();
    assert!(!bytecode.is_empty(), "Bytecode should not be empty");

    // Create a BytecodeFile for encryption
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

    // Decrypt and verify
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

    // Encrypt with one key
    let key_pair1 = EccKeyPair::generate(EccAlgorithm::X25519);
    let encryption_result =
        encrypt_bytecode(&[bytecode_file], EccAlgorithm::X25519, Some(key_pair1)).unwrap();

    // Try to decrypt with different key
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

    // Serialize to JSON
    let json = serde_json::to_string(&key_pair).unwrap();

    // Deserialize
    let deserialized: EccKeyPair = serde_json::from_str(&json).unwrap();

    assert_eq!(key_pair.algorithm, deserialized.algorithm);
    assert_eq!(key_pair.public_key, deserialized.public_key);
    assert_eq!(key_pair.private_key, deserialized.private_key);
}
