//! Unit tests for the hybrid encryption module (ECC + AES-256-GCM / ChaCha20-Poly1305).
//!
//! Covers EccKeyPair generation, encrypt_hybrid / decrypt_hybrid round-trips,
//! integrity verification, and error paths.

use auroraview_protect::crypto::{
    base64_decode, base64_encode, decrypt_aes_gcm, decrypt_hybrid, encrypt_aes_gcm, encrypt_hybrid,
    hex_decode, hex_encode, EccAlgorithm, EccKeyPair, KeyObfuscator, AES_KEY_SIZE,
    P256_PUBLIC_KEY_SIZE, X25519_PUBLIC_KEY_SIZE,
};
use rstest::*;

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

// ─────────────────────────────────────────────────────────────
// AES-256-GCM direct interface
// ─────────────────────────────────────────────────────────────

#[test]
fn aes_gcm_empty_plaintext_roundtrip() {
    let key: [u8; AES_KEY_SIZE] = rand::thread_rng().gen();
    let plaintext: &[u8] = b"";
    let encrypted = encrypt_aes_gcm(plaintext, &key).unwrap();
    let decrypted = decrypt_aes_gcm(&encrypted, &key).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn aes_gcm_single_byte_roundtrip() {
    let key: [u8; AES_KEY_SIZE] = rand::thread_rng().gen();
    let encrypted = encrypt_aes_gcm(b"\x00", &key).unwrap();
    let decrypted = decrypt_aes_gcm(&encrypted, &key).unwrap();
    assert_eq!(decrypted, b"\x00");
}

#[test]
fn aes_gcm_large_data_roundtrip() {
    let key: [u8; AES_KEY_SIZE] = rand::thread_rng().gen();
    let plaintext: Vec<u8> = (0u8..=255).cycle().take(1_024 * 1024).collect(); // 1 MiB
    let encrypted = encrypt_aes_gcm(&plaintext, &key).unwrap();
    let decrypted = decrypt_aes_gcm(&encrypted, &key).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn aes_gcm_each_encryption_unique_nonce() {
    let key: [u8; AES_KEY_SIZE] = rand::thread_rng().gen();
    let e1 = encrypt_aes_gcm(b"same", &key).unwrap();
    let e2 = encrypt_aes_gcm(b"same", &key).unwrap();
    assert_ne!(e1, e2); // different nonces
}

#[test]
fn aes_gcm_decrypt_wrong_key_fails() {
    let key1: [u8; AES_KEY_SIZE] = rand::thread_rng().gen();
    let key2: [u8; AES_KEY_SIZE] = rand::thread_rng().gen();
    let encrypted = encrypt_aes_gcm(b"secret", &key1).unwrap();
    let result = decrypt_aes_gcm(&encrypted, &key2);
    assert!(result.is_err());
}

#[test]
fn aes_gcm_decrypt_truncated_data_fails() {
    let key: [u8; AES_KEY_SIZE] = rand::thread_rng().gen();
    // too short to even contain nonce + tag
    let short: Vec<u8> = vec![0u8; 10];
    let result = decrypt_aes_gcm(&short, &key);
    assert!(result.is_err());
}

#[test]
fn aes_gcm_decrypt_tampered_ciphertext_fails() {
    let key: [u8; AES_KEY_SIZE] = rand::thread_rng().gen();
    let mut encrypted = encrypt_aes_gcm(b"payload", &key).unwrap();
    // Flip a byte in the ciphertext region
    let last = encrypted.len() - 1;
    encrypted[last] ^= 0xFF;
    let result = decrypt_aes_gcm(&encrypted, &key);
    assert!(result.is_err());
}

#[rstest]
#[case(1)]
#[case(15)]
#[case(16)]
#[case(100)]
#[case(1023)]
fn aes_gcm_various_plaintext_lengths(#[case] len: usize) {
    let key: [u8; AES_KEY_SIZE] = rand::thread_rng().gen();
    let plaintext: Vec<u8> = (0u8..=255).cycle().take(len).collect();
    let encrypted = encrypt_aes_gcm(&plaintext, &key).unwrap();
    let decrypted = decrypt_aes_gcm(&encrypted, &key).unwrap();
    assert_eq!(decrypted, plaintext);
}

// ─────────────────────────────────────────────────────────────
// KeyObfuscator edge cases
// ─────────────────────────────────────────────────────────────

#[test]
fn key_obfuscator_empty_key_roundtrip() {
    let key: &[u8] = b"";
    let obf = KeyObfuscator::new(key);
    let rec = obf.reconstruct(0);
    assert_eq!(rec, key);
}

#[test]
fn key_obfuscator_single_byte() {
    let key = [0xABu8];
    let obf = KeyObfuscator::new(&key);
    let rec = obf.reconstruct(1);
    assert_eq!(&rec, &key);
}

#[test]
fn key_obfuscator_all_zeros() {
    let key = [0u8; 32];
    let obf = KeyObfuscator::new(&key);
    let rec = obf.reconstruct(32);
    assert_eq!(&rec, &key);
}

#[test]
fn key_obfuscator_all_ones() {
    let key = [0xFFu8; 32];
    let obf = KeyObfuscator::new(&key);
    let rec = obf.reconstruct(32);
    assert_eq!(&rec, &key);
}

#[rstest]
#[case(7)]
#[case(8)]
#[case(9)]
#[case(31)]
#[case(32)]
#[case(33)]
#[case(64)]
fn key_obfuscator_various_lengths(#[case] len: usize) {
    let key: Vec<u8> = (0u8..=255).cycle().take(len).collect();
    let obf = KeyObfuscator::new(&key);
    let rec = obf.reconstruct(len);
    assert_eq!(rec, key);
}

#[test]
fn key_obfuscator_rust_code_is_non_empty() {
    let key = b"test-key-32-bytes-long__________";
    let obf = KeyObfuscator::new(key);
    let code = obf.generate_rust_code(key.len());
    assert!(code.contains("KEY_PARTS"));
    assert!(code.contains("XOR_KEYS"));
    assert!(code.contains("KEY_LEN"));
    assert!(code.contains("reconstruct_key"));
}

#[test]
fn key_obfuscator_python_code_is_non_empty() {
    let key = b"test-key-32-bytes-long__________";
    let obf = KeyObfuscator::new(key);
    let code = obf.generate_python_code(key.len());
    assert!(code.contains("_KP"));
    assert!(code.contains("_XK"));
    assert!(code.contains("_KL"));
    assert!(code.contains("_rk()"));
}

// ─────────────────────────────────────────────────────────────
// hex_encode / hex_decode utilities
// ─────────────────────────────────────────────────────────────

#[test]
fn hex_roundtrip_empty() {
    let bytes: &[u8] = b"";
    assert_eq!(hex_decode(&hex_encode(bytes)).unwrap(), bytes);
}

#[test]
fn hex_roundtrip_all_byte_values() {
    let bytes: Vec<u8> = (0u8..=255).collect();
    let encoded = hex_encode(&bytes);
    let decoded = hex_decode(&encoded).unwrap();
    assert_eq!(decoded, bytes);
}

#[test]
fn hex_decode_odd_length_returns_err() {
    let result = hex_decode("abc");
    assert!(result.is_err());
}

#[test]
fn hex_decode_invalid_char_returns_err() {
    let result = hex_decode("zz");
    assert!(result.is_err());
}

#[test]
fn hex_encode_known_value() {
    assert_eq!(hex_encode(&[0xDE, 0xAD, 0xBE, 0xEF]), "deadbeef");
}

// ─────────────────────────────────────────────────────────────
// base64_encode / base64_decode utilities
// ─────────────────────────────────────────────────────────────

#[test]
fn base64_roundtrip_empty() {
    let bytes: &[u8] = b"";
    assert_eq!(base64_decode(&base64_encode(bytes)).unwrap(), bytes);
}

#[test]
fn base64_roundtrip_binary() {
    let bytes: Vec<u8> = (0u8..=255).collect();
    assert_eq!(base64_decode(&base64_encode(&bytes)).unwrap(), bytes);
}

#[test]
fn base64_decode_invalid_returns_err() {
    let result = base64_decode("not-valid-base64!!!");
    assert!(result.is_err());
}

// ─────────────────────────────────────────────────────────────
// Cross-algorithm decryption must fail
// ─────────────────────────────────────────────────────────────

#[test]
fn cross_algorithm_decrypt_x25519_pkg_with_p256_key_fails() {
    let x_kp = EccKeyPair::generate(EccAlgorithm::X25519);
    let p_kp = EccKeyPair::generate(EccAlgorithm::P256);
    let pkg = encrypt_hybrid(b"data", &x_kp.public_key, EccAlgorithm::X25519).unwrap();
    // Trying to decrypt with P-256 private key (algorithm mismatch)
    let result = decrypt_hybrid(&pkg, &p_kp.private_key);
    assert!(result.is_err());
}

// ─────────────────────────────────────────────────────────────
// Concurrent encryption — no data races
// ─────────────────────────────────────────────────────────────

#[test]
fn concurrent_x25519_encrypt_no_panic() {
    use std::thread;

    let kp = EccKeyPair::generate(EccAlgorithm::X25519);
    let pub_key = kp.public_key.clone();

    let handles: Vec<_> = (0..8)
        .map(|i| {
            let pk = pub_key.clone();
            thread::spawn(move || {
                let data: Vec<u8> = (0u8..64).map(|b| b ^ (i as u8)).collect();
                let pkg = encrypt_hybrid(&data, &pk, EccAlgorithm::X25519).unwrap();
                assert_eq!(pkg.version, 1);
            })
        })
        .collect();

    for h in handles {
        h.join().expect("thread panicked");
    }
}

#[test]
fn concurrent_p256_encrypt_no_panic() {
    use std::thread;

    let kp = EccKeyPair::generate(EccAlgorithm::P256);
    let pub_key = kp.public_key.clone();

    let handles: Vec<_> = (0..8)
        .map(|i| {
            let pk = pub_key.clone();
            thread::spawn(move || {
                let data: Vec<u8> = (0u8..64).map(|b| b.wrapping_add(i as u8)).collect();
                let pkg = encrypt_hybrid(&data, &pk, EccAlgorithm::P256).unwrap();
                assert_eq!(pkg.algorithm, EccAlgorithm::P256);
            })
        })
        .collect();

    for h in handles {
        h.join().expect("thread panicked");
    }
}

#[test]
fn concurrent_key_generation_unique_keys() {
    use std::sync::{Arc, Mutex};
    use std::thread;

    let keys: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    let handles: Vec<_> = (0..8)
        .map(|_| {
            let k = Arc::clone(&keys);
            thread::spawn(move || {
                let kp = EccKeyPair::generate(EccAlgorithm::X25519);
                k.lock().unwrap().push(kp.public_key);
            })
        })
        .collect();

    for h in handles {
        h.join().expect("thread panicked");
    }

    let collected = keys.lock().unwrap();
    let mut sorted = collected.clone();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        collected.len(),
        "all public keys must be unique"
    );
}

// ─────────────────────────────────────────────────────────────
// EccAlgorithm display / debug
// ─────────────────────────────────────────────────────────────

#[test]
fn ecc_algorithm_debug_output() {
    assert!(format!("{:?}", EccAlgorithm::X25519).contains("X25519"));
    assert!(format!("{:?}", EccAlgorithm::P256).contains("P256"));
}

#[test]
fn ecc_algorithm_clone_equality() {
    let a = EccAlgorithm::X25519;
    let b = a;
    assert_eq!(a, b);
}

// ─────────────────────────────────────────────────────────────
// Tamper: corrupt ephemeral public key
// ─────────────────────────────────────────────────────────────

#[test]
fn decrypt_with_tampered_ephemeral_key_fails() {
    let kp = EccKeyPair::generate(EccAlgorithm::X25519);
    let mut pkg = encrypt_hybrid(b"payload", &kp.public_key, EccAlgorithm::X25519).unwrap();
    // Replace with all-zeros hex (32 bytes)
    pkg.ephemeral_public_key = "00".repeat(32);
    let result = decrypt_hybrid(&pkg, &kp.private_key);
    assert!(result.is_err());
}

#[test]
fn decrypt_with_tampered_encrypted_key_fails() {
    let kp = EccKeyPair::generate(EccAlgorithm::X25519);
    let mut pkg = encrypt_hybrid(b"payload", &kp.public_key, EccAlgorithm::X25519).unwrap();
    // Corrupt the encrypted key
    pkg.encrypted_key = "deadbeef".repeat(10);
    let result = decrypt_hybrid(&pkg, &kp.private_key);
    assert!(result.is_err());
}

// use rand for key generation in AES tests
use rand::Rng;
