//! Unit tests for the hybrid encryption module (ECC + AES-256-GCM / ChaCha20-Poly1305).
//!
//! Covers EccKeyPair generation, encrypt_hybrid / decrypt_hybrid round-trips,
//! integrity verification, and error paths.

use auroraview_protect::crypto::{
    decrypt_hybrid, encrypt_hybrid, EccAlgorithm, EccKeyPair, AES_KEY_SIZE, P256_PUBLIC_KEY_SIZE,
    X25519_PUBLIC_KEY_SIZE,
};

// ─────────────────────────────────────────────────────────────
// EccAlgorithm
// ─────────────────────────────────────────────────────────────

#[test]
fn ecc_algorithm_default_is_x25519() {
    let algo = EccAlgorithm::default();
    assert_eq!(algo, EccAlgorithm::X25519);
}

#[test]
fn ecc_algorithm_parse_x25519() {
    let algo: EccAlgorithm = "x25519".parse().unwrap();
    assert_eq!(algo, EccAlgorithm::X25519);
}

#[test]
fn ecc_algorithm_parse_p256_variants() {
    for s in &["p256", "p-256", "secp256r1", "P256", "P-256"] {
        let algo: EccAlgorithm = s.parse().unwrap();
        assert_eq!(algo, EccAlgorithm::P256, "failed for: {s}");
    }
}

#[test]
fn ecc_algorithm_parse_unknown_returns_err() {
    let result: Result<EccAlgorithm, _> = "ed25519".parse();
    assert!(result.is_err());
}

#[test]
fn ecc_algorithm_serialization_roundtrip() {
    let algo = EccAlgorithm::P256;
    let json = serde_json::to_string(&algo).unwrap();
    let restored: EccAlgorithm = serde_json::from_str(&json).unwrap();
    assert_eq!(restored, EccAlgorithm::P256);
}

// ─────────────────────────────────────────────────────────────
// EccKeyPair generation
// ─────────────────────────────────────────────────────────────

#[test]
fn key_pair_x25519_has_correct_sizes() {
    let kp = EccKeyPair::generate(EccAlgorithm::X25519);
    assert_eq!(kp.algorithm, EccAlgorithm::X25519);
    // Hex string is 2× byte length
    assert_eq!(kp.public_key.len(), X25519_PUBLIC_KEY_SIZE * 2);
    assert_eq!(kp.private_key.len(), X25519_PUBLIC_KEY_SIZE * 2);
}

#[test]
fn key_pair_p256_has_correct_sizes() {
    let kp = EccKeyPair::generate(EccAlgorithm::P256);
    assert_eq!(kp.algorithm, EccAlgorithm::P256);
    // Compressed P-256 public key: 33 bytes → 66 hex chars
    assert_eq!(kp.public_key.len(), P256_PUBLIC_KEY_SIZE * 2);
    // Private key: 32 bytes → 64 hex chars
    assert_eq!(kp.private_key.len(), AES_KEY_SIZE * 2);
}

#[test]
fn key_pair_generate_x25519_and_p256_are_different_algorithms() {
    let x = EccKeyPair::generate(EccAlgorithm::X25519);
    let p = EccKeyPair::generate(EccAlgorithm::P256);
    assert_ne!(x.algorithm, p.algorithm);
}

#[test]
fn key_pair_x25519_each_generation_is_unique() {
    let kp1 = EccKeyPair::generate(EccAlgorithm::X25519);
    let kp2 = EccKeyPair::generate(EccAlgorithm::X25519);
    assert_ne!(kp1.public_key, kp2.public_key);
    assert_ne!(kp1.private_key, kp2.private_key);
}

#[test]
fn key_pair_p256_each_generation_is_unique() {
    let kp1 = EccKeyPair::generate(EccAlgorithm::P256);
    let kp2 = EccKeyPair::generate(EccAlgorithm::P256);
    assert_ne!(kp1.public_key, kp2.public_key);
}

#[test]
fn key_pair_public_key_bytes_roundtrip() {
    let kp = EccKeyPair::generate(EccAlgorithm::X25519);
    let bytes = kp.public_key_bytes().unwrap();
    assert_eq!(bytes.len(), X25519_PUBLIC_KEY_SIZE);
}

#[test]
fn key_pair_private_key_bytes_roundtrip() {
    let kp = EccKeyPair::generate(EccAlgorithm::X25519);
    let bytes = kp.private_key_bytes().unwrap();
    assert_eq!(bytes.len(), X25519_PUBLIC_KEY_SIZE);
}

#[test]
fn key_pair_serialization_roundtrip() {
    let kp = EccKeyPair::generate(EccAlgorithm::X25519);
    let json = serde_json::to_string(&kp).unwrap();
    let restored: EccKeyPair = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.public_key, kp.public_key);
    assert_eq!(restored.private_key, kp.private_key);
    assert_eq!(restored.algorithm, kp.algorithm);
}

// ─────────────────────────────────────────────────────────────
// X25519 encrypt / decrypt round-trips
// ─────────────────────────────────────────────────────────────

#[test]
fn x25519_roundtrip_small_payload() {
    let kp = EccKeyPair::generate(EccAlgorithm::X25519);
    let plaintext = b"hello world";

    let pkg = encrypt_hybrid(plaintext, &kp.public_key, EccAlgorithm::X25519).unwrap();
    let decrypted = decrypt_hybrid(&pkg, &kp.private_key).unwrap();

    assert_eq!(decrypted, plaintext);
}

#[test]
fn x25519_roundtrip_empty_payload() {
    let kp = EccKeyPair::generate(EccAlgorithm::X25519);
    let plaintext: &[u8] = b"";

    let pkg = encrypt_hybrid(plaintext, &kp.public_key, EccAlgorithm::X25519).unwrap();
    let decrypted = decrypt_hybrid(&pkg, &kp.private_key).unwrap();

    assert_eq!(decrypted, plaintext);
}

#[test]
fn x25519_roundtrip_large_payload() {
    let kp = EccKeyPair::generate(EccAlgorithm::X25519);
    let plaintext: Vec<u8> = (0u8..=255).cycle().take(64 * 1024).collect();

    let pkg = encrypt_hybrid(&plaintext, &kp.public_key, EccAlgorithm::X25519).unwrap();
    let decrypted = decrypt_hybrid(&pkg, &kp.private_key).unwrap();

    assert_eq!(decrypted, plaintext);
}

#[test]
fn x25519_each_encryption_produces_unique_ciphertext() {
    let kp = EccKeyPair::generate(EccAlgorithm::X25519);
    let plaintext = b"same data";

    let pkg1 = encrypt_hybrid(plaintext, &kp.public_key, EccAlgorithm::X25519).unwrap();
    let pkg2 = encrypt_hybrid(plaintext, &kp.public_key, EccAlgorithm::X25519).unwrap();

    // Ephemeral keys differ each time
    assert_ne!(pkg1.ephemeral_public_key, pkg2.ephemeral_public_key);
    // Encrypted data differs (fresh nonces)
    assert_ne!(pkg1.encrypted_data, pkg2.encrypted_data);
}

#[test]
fn x25519_package_version_is_1() {
    let kp = EccKeyPair::generate(EccAlgorithm::X25519);
    let pkg = encrypt_hybrid(b"test", &kp.public_key, EccAlgorithm::X25519).unwrap();
    assert_eq!(pkg.version, 1);
    assert_eq!(pkg.algorithm, EccAlgorithm::X25519);
}

// ─────────────────────────────────────────────────────────────
// P-256 encrypt / decrypt round-trips
// ─────────────────────────────────────────────────────────────

#[test]
fn p256_roundtrip_small_payload() {
    let kp = EccKeyPair::generate(EccAlgorithm::P256);
    let plaintext = b"hello p256";

    let pkg = encrypt_hybrid(plaintext, &kp.public_key, EccAlgorithm::P256).unwrap();
    let decrypted = decrypt_hybrid(&pkg, &kp.private_key).unwrap();

    assert_eq!(decrypted, plaintext);
}

#[test]
fn p256_roundtrip_binary_data() {
    let kp = EccKeyPair::generate(EccAlgorithm::P256);
    // Simulate Python bytecode (binary data)
    let plaintext: Vec<u8> = (0u8..255).collect();

    let pkg = encrypt_hybrid(&plaintext, &kp.public_key, EccAlgorithm::P256).unwrap();
    let decrypted = decrypt_hybrid(&pkg, &kp.private_key).unwrap();

    assert_eq!(decrypted, plaintext);
}

#[test]
fn p256_package_algorithm_field() {
    let kp = EccKeyPair::generate(EccAlgorithm::P256);
    let pkg = encrypt_hybrid(b"test", &kp.public_key, EccAlgorithm::P256).unwrap();
    assert_eq!(pkg.algorithm, EccAlgorithm::P256);
}

// ─────────────────────────────────────────────────────────────
// Error paths
// ─────────────────────────────────────────────────────────────

#[test]
fn decrypt_with_wrong_private_key_fails() {
    let kp = EccKeyPair::generate(EccAlgorithm::X25519);
    let wrong_kp = EccKeyPair::generate(EccAlgorithm::X25519);

    let pkg = encrypt_hybrid(b"secret", &kp.public_key, EccAlgorithm::X25519).unwrap();
    let result = decrypt_hybrid(&pkg, &wrong_kp.private_key);

    assert!(result.is_err());
}

#[test]
fn decrypt_p256_with_wrong_private_key_fails() {
    let kp = EccKeyPair::generate(EccAlgorithm::P256);
    let wrong_kp = EccKeyPair::generate(EccAlgorithm::P256);

    let pkg = encrypt_hybrid(b"secret", &kp.public_key, EccAlgorithm::P256).unwrap();
    let result = decrypt_hybrid(&pkg, &wrong_kp.private_key);

    assert!(result.is_err());
}

#[test]
fn encrypt_with_invalid_public_key_returns_err() {
    let result = encrypt_hybrid(b"data", "not_a_valid_hex_key", EccAlgorithm::X25519);
    assert!(result.is_err());
}

#[test]
fn encrypt_with_too_short_public_key_returns_err() {
    // Valid hex but wrong length (not 32 bytes)
    let short_key = "deadbeef";
    let result = encrypt_hybrid(b"data", short_key, EccAlgorithm::X25519);
    assert!(result.is_err());
}

#[test]
fn decrypt_with_tampered_data_fails_integrity_check() {
    let kp = EccKeyPair::generate(EccAlgorithm::X25519);
    let plaintext = b"important data";

    let mut pkg = encrypt_hybrid(plaintext, &kp.public_key, EccAlgorithm::X25519).unwrap();

    // Tamper with the encrypted data (corrupt the base64 content)
    pkg.encrypted_data = "dGFtcGVyZWQ=".to_string(); // base64("tampered")

    let result = decrypt_hybrid(&pkg, &kp.private_key);
    assert!(result.is_err());
}

// ─────────────────────────────────────────────────────────────
// Package integrity
// ─────────────────────────────────────────────────────────────

#[test]
fn original_hash_is_deterministic_for_same_input() {
    let kp = EccKeyPair::generate(EccAlgorithm::X25519);
    let plaintext = b"deterministic";

    let pkg1 = encrypt_hybrid(plaintext, &kp.public_key, EccAlgorithm::X25519).unwrap();
    let pkg2 = encrypt_hybrid(plaintext, &kp.public_key, EccAlgorithm::X25519).unwrap();

    // BLAKE3 hash of same input must be equal
    assert_eq!(pkg1.original_hash, pkg2.original_hash);
}

#[test]
fn original_hash_differs_for_different_inputs() {
    let kp = EccKeyPair::generate(EccAlgorithm::X25519);

    let pkg1 = encrypt_hybrid(b"input_a", &kp.public_key, EccAlgorithm::X25519).unwrap();
    let pkg2 = encrypt_hybrid(b"input_b", &kp.public_key, EccAlgorithm::X25519).unwrap();

    assert_ne!(pkg1.original_hash, pkg2.original_hash);
}

#[test]
fn package_serialization_roundtrip() {
    let kp = EccKeyPair::generate(EccAlgorithm::X25519);
    let pkg = encrypt_hybrid(b"test bytes", &kp.public_key, EccAlgorithm::X25519).unwrap();

    let json = serde_json::to_string(&pkg).unwrap();
    let restored =
        serde_json::from_str::<auroraview_protect::crypto::EncryptedPackage>(&json).unwrap();

    // After JSON roundtrip, decryption should still work
    let decrypted = decrypt_hybrid(&restored, &kp.private_key).unwrap();
    assert_eq!(decrypted, b"test bytes");
}
