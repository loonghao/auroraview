//! Cryptographic operations for code protection
//!
//! Implements hybrid encryption scheme:
//! - **Pack time**: .py → .pyc → AES-256-GCM encrypt → .pyc.enc;
//!   AES key → ECC public key encrypt → encrypted key
//! - **Runtime**: encrypted key → ECC private key decrypt → AES key;
//!   .pyc.enc → AES decrypt → .pyc → marshal.loads() + exec()
//!
//! Supported algorithms:
//! - X25519 + ChaCha20-Poly1305 (default, fast, modern)
//! - P-256 + AES-256-GCM (FIPS compliant)

use crate::{ProtectError, ProtectResult};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce as AesNonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Nonce as ChachaNonce};
use p256::{
    ecdh::EphemeralSecret as P256EphemeralSecret,
    elliptic_curve::sec1::{FromEncodedPoint, ToEncodedPoint},
    PublicKey as P256PublicKey,
};
use rand::rngs::OsRng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519StaticSecret};

// ============================================================================
// Constants
// ============================================================================

/// AES key size in bytes (256 bits)
pub const AES_KEY_SIZE: usize = 32;

/// AES-GCM nonce size in bytes (96 bits)
pub const AES_NONCE_SIZE: usize = 12;

/// ChaCha20-Poly1305 nonce size (96 bits)
pub const CHACHA_NONCE_SIZE: usize = 12;

/// X25519 public key size (32 bytes)
pub const X25519_PUBLIC_KEY_SIZE: usize = 32;

/// X25519 private key size (32 bytes)
pub const X25519_PRIVATE_KEY_SIZE: usize = 32;

/// P-256 compressed public key size (33 bytes)
pub const P256_PUBLIC_KEY_SIZE: usize = 33;

/// P-256 private key size (32 bytes)
pub const P256_PRIVATE_KEY_SIZE: usize = 32;

// ============================================================================
// Key Types
// ============================================================================

/// ECC algorithm type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum EccAlgorithm {
    /// X25519 ECDH + ChaCha20-Poly1305 (default)
    #[default]
    X25519,
    /// NIST P-256 ECDH + AES-256-GCM
    P256,
}

impl std::str::FromStr for EccAlgorithm {
    type Err = ProtectError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "x25519" => Ok(Self::X25519),
            "p256" | "p-256" | "secp256r1" => Ok(Self::P256),
            _ => Err(ProtectError::InvalidKey(format!(
                "Unknown algorithm: {}. Supported: x25519, p256",
                s
            ))),
        }
    }
}

/// ECC key pair for hybrid encryption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EccKeyPair {
    /// Algorithm used
    pub algorithm: EccAlgorithm,
    /// Public key (hex encoded)
    pub public_key: String,
    /// Private key (hex encoded) - KEEP SECRET!
    pub private_key: String,
}

impl EccKeyPair {
    /// Generate a new X25519 key pair
    pub fn generate_x25519() -> Self {
        let secret = X25519StaticSecret::random_from_rng(OsRng);
        let public = X25519PublicKey::from(&secret);

        Self {
            algorithm: EccAlgorithm::X25519,
            public_key: hex_encode(public.as_bytes()),
            private_key: hex_encode(secret.as_bytes()),
        }
    }

    /// Generate a new P-256 key pair
    pub fn generate_p256() -> Self {
        let secret = p256::SecretKey::random(&mut OsRng);
        let public = secret.public_key();

        // Use compressed point encoding (33 bytes)
        let public_bytes = public.to_encoded_point(true);

        Self {
            algorithm: EccAlgorithm::P256,
            public_key: hex_encode(public_bytes.as_bytes()),
            private_key: hex_encode(secret.to_bytes().as_slice()),
        }
    }

    /// Generate a key pair with the specified algorithm
    pub fn generate(algorithm: EccAlgorithm) -> Self {
        match algorithm {
            EccAlgorithm::X25519 => Self::generate_x25519(),
            EccAlgorithm::P256 => Self::generate_p256(),
        }
    }

    /// Get public key bytes
    pub fn public_key_bytes(&self) -> ProtectResult<Vec<u8>> {
        hex_decode(&self.public_key)
    }

    /// Get private key bytes
    pub fn private_key_bytes(&self) -> ProtectResult<Vec<u8>> {
        hex_decode(&self.private_key)
    }
}

// ============================================================================
// Encrypted Package Format
// ============================================================================

/// Encrypted bytecode package
///
/// Contains everything needed to decrypt at runtime:
/// - Encrypted AES key (encrypted with ECC public key)
/// - Encrypted bytecode (encrypted with AES key)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedPackage {
    /// Version of the encryption format
    pub version: u8,

    /// Algorithm used for key encryption
    pub algorithm: EccAlgorithm,

    /// Ephemeral public key used for ECDH (hex encoded)
    /// This is generated fresh for each encryption
    pub ephemeral_public_key: String,

    /// Encrypted AES key (hex encoded)
    /// Format: [nonce (12 bytes)][ciphertext + tag]
    pub encrypted_key: String,

    /// Encrypted bytecode (base64 encoded for size efficiency)
    /// Format: [nonce (12 bytes)][ciphertext + tag]
    pub encrypted_data: String,

    /// Original file hash for integrity verification (BLAKE3, hex)
    pub original_hash: String,
}

// ============================================================================
// Hybrid Encryption (Pack Time)
// ============================================================================

/// Encrypt bytecode using hybrid encryption
///
/// # Flow
/// 1. Generate random AES-256 key
/// 2. Encrypt bytecode with AES-256-GCM
/// 3. Generate ephemeral ECC key pair
/// 4. Derive shared secret via ECDH with recipient's public key
/// 5. Encrypt AES key with derived key
///
/// # Arguments
/// * `bytecode` - The marshaled Python bytecode (.pyc content)
/// * `recipient_public_key` - Recipient's ECC public key (hex encoded)
/// * `algorithm` - ECC algorithm to use
pub fn encrypt_hybrid(
    bytecode: &[u8],
    recipient_public_key: &str,
    algorithm: EccAlgorithm,
) -> ProtectResult<EncryptedPackage> {
    // 1. Generate random AES key
    let aes_key: [u8; AES_KEY_SIZE] = rand::thread_rng().gen();

    // 2. Encrypt bytecode with AES-256-GCM
    let encrypted_data = encrypt_aes_gcm(bytecode, &aes_key)?;

    // 3. Compute original hash
    let original_hash = blake3::hash(bytecode);

    // 4. Encrypt AES key with ECC
    let (ephemeral_public_key, encrypted_key) = match algorithm {
        EccAlgorithm::X25519 => encrypt_key_x25519(&aes_key, recipient_public_key)?,
        EccAlgorithm::P256 => encrypt_key_p256(&aes_key, recipient_public_key)?,
    };

    Ok(EncryptedPackage {
        version: 1,
        algorithm,
        ephemeral_public_key,
        encrypted_key: hex_encode(&encrypted_key),
        encrypted_data: base64_encode(&encrypted_data),
        original_hash: original_hash.to_hex().to_string(),
    })
}

/// Encrypt AES key using X25519 ECDH + ChaCha20-Poly1305
fn encrypt_key_x25519(
    aes_key: &[u8; AES_KEY_SIZE],
    recipient_public_key: &str,
) -> ProtectResult<(String, Vec<u8>)> {
    // Parse recipient's public key
    let recipient_bytes = hex_decode(recipient_public_key)?;
    if recipient_bytes.len() != X25519_PUBLIC_KEY_SIZE {
        return Err(ProtectError::InvalidKey(format!(
            "X25519 public key must be {} bytes, got {}",
            X25519_PUBLIC_KEY_SIZE,
            recipient_bytes.len()
        )));
    }
    let recipient_public: [u8; 32] = recipient_bytes.try_into().unwrap();
    let recipient_public = X25519PublicKey::from(recipient_public);

    // Generate ephemeral key pair
    let ephemeral_secret = X25519StaticSecret::random_from_rng(OsRng);
    let ephemeral_public = X25519PublicKey::from(&ephemeral_secret);

    // ECDH: derive shared secret
    let shared_secret = ephemeral_secret.diffie_hellman(&recipient_public);

    // Derive encryption key from shared secret using BLAKE3
    let derived_key = blake3::derive_key("aurora-protect-x25519-v1", shared_secret.as_bytes());

    // Encrypt AES key with ChaCha20-Poly1305
    let cipher = ChaCha20Poly1305::new_from_slice(&derived_key)
        .map_err(|e| ProtectError::Encryption(e.to_string()))?;

    let nonce_bytes: [u8; CHACHA_NONCE_SIZE] = rand::thread_rng().gen();
    let nonce = ChachaNonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, aes_key.as_slice())
        .map_err(|e| ProtectError::Encryption(e.to_string()))?;

    // Format: [nonce][ciphertext + tag]
    let mut encrypted = Vec::with_capacity(CHACHA_NONCE_SIZE + ciphertext.len());
    encrypted.extend_from_slice(&nonce_bytes);
    encrypted.extend_from_slice(&ciphertext);

    Ok((hex_encode(ephemeral_public.as_bytes()), encrypted))
}

/// Encrypt AES key using P-256 ECDH + AES-256-GCM
fn encrypt_key_p256(
    aes_key: &[u8; AES_KEY_SIZE],
    recipient_public_key: &str,
) -> ProtectResult<(String, Vec<u8>)> {
    // Parse recipient's public key
    let recipient_bytes = hex_decode(recipient_public_key)?;
    let recipient_point = p256::EncodedPoint::from_bytes(&recipient_bytes)
        .map_err(|e| ProtectError::InvalidKey(format!("Invalid P-256 public key: {}", e)))?;
    let recipient_public = P256PublicKey::from_encoded_point(&recipient_point);
    if recipient_public.is_none().into() {
        return Err(ProtectError::InvalidKey(
            "Invalid P-256 public key point".to_string(),
        ));
    }
    let recipient_public = recipient_public.unwrap();

    // Generate ephemeral key pair
    let ephemeral_secret = P256EphemeralSecret::random(&mut OsRng);
    let ephemeral_public = ephemeral_secret.public_key();

    // ECDH: derive shared secret
    let shared_secret = ephemeral_secret.diffie_hellman(&recipient_public);

    // Derive encryption key from shared secret using BLAKE3
    let derived_key =
        blake3::derive_key("aurora-protect-p256-v1", shared_secret.raw_secret_bytes());

    // Encrypt AES key with AES-256-GCM
    let cipher = Aes256Gcm::new_from_slice(&derived_key)
        .map_err(|e| ProtectError::Encryption(e.to_string()))?;

    let nonce_bytes: [u8; AES_NONCE_SIZE] = rand::thread_rng().gen();
    let nonce = AesNonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, aes_key.as_slice())
        .map_err(|e| ProtectError::Encryption(e.to_string()))?;

    // Format: [nonce][ciphertext + tag]
    let mut encrypted = Vec::with_capacity(AES_NONCE_SIZE + ciphertext.len());
    encrypted.extend_from_slice(&nonce_bytes);
    encrypted.extend_from_slice(&ciphertext);

    // Use compressed point encoding
    let ephemeral_public_bytes = ephemeral_public.to_encoded_point(true);

    Ok((hex_encode(ephemeral_public_bytes.as_bytes()), encrypted))
}

// ============================================================================
// Hybrid Decryption (Runtime)
// ============================================================================

/// Decrypt bytecode using hybrid decryption
///
/// # Flow
/// 1. Derive shared secret via ECDH with ephemeral public key
/// 2. Decrypt AES key
/// 3. Decrypt bytecode with AES key
/// 4. Verify integrity
///
/// # Arguments
/// * `package` - The encrypted package
/// * `private_key` - Recipient's private key (hex encoded)
pub fn decrypt_hybrid(package: &EncryptedPackage, private_key: &str) -> ProtectResult<Vec<u8>> {
    // 1. Decrypt AES key
    let aes_key = match package.algorithm {
        EccAlgorithm::X25519 => decrypt_key_x25519(
            &hex_decode(&package.encrypted_key)?,
            &package.ephemeral_public_key,
            private_key,
        )?,
        EccAlgorithm::P256 => decrypt_key_p256(
            &hex_decode(&package.encrypted_key)?,
            &package.ephemeral_public_key,
            private_key,
        )?,
    };

    // 2. Decrypt bytecode
    let encrypted_data = base64_decode(&package.encrypted_data)?;
    let bytecode = decrypt_aes_gcm(&encrypted_data, &aes_key)?;

    // 3. Verify integrity
    let computed_hash = blake3::hash(&bytecode);
    if computed_hash.to_hex().to_string() != package.original_hash {
        return Err(ProtectError::Decryption(
            "Integrity check failed: bytecode hash mismatch".to_string(),
        ));
    }

    Ok(bytecode)
}

/// Decrypt AES key using X25519 ECDH
fn decrypt_key_x25519(
    encrypted_key: &[u8],
    ephemeral_public_key: &str,
    private_key: &str,
) -> ProtectResult<[u8; AES_KEY_SIZE]> {
    if encrypted_key.len() < CHACHA_NONCE_SIZE + 16 {
        return Err(ProtectError::Decryption(
            "Encrypted key too short".to_string(),
        ));
    }

    // Parse keys
    let ephemeral_bytes = hex_decode(ephemeral_public_key)?;
    let ephemeral_public: [u8; 32] = ephemeral_bytes
        .try_into()
        .map_err(|_| ProtectError::InvalidKey("Invalid ephemeral public key length".to_string()))?;
    let ephemeral_public = X25519PublicKey::from(ephemeral_public);

    let private_bytes = hex_decode(private_key)?;
    let private_key_arr: [u8; 32] = private_bytes
        .try_into()
        .map_err(|_| ProtectError::InvalidKey("Invalid private key length".to_string()))?;
    let static_secret = X25519StaticSecret::from(private_key_arr);

    // ECDH: derive shared secret
    let shared_secret = static_secret.diffie_hellman(&ephemeral_public);

    // Derive decryption key
    let derived_key = blake3::derive_key("aurora-protect-x25519-v1", shared_secret.as_bytes());

    // Decrypt AES key
    let cipher = ChaCha20Poly1305::new_from_slice(&derived_key)
        .map_err(|e| ProtectError::Decryption(e.to_string()))?;

    let nonce = ChachaNonce::from_slice(&encrypted_key[..CHACHA_NONCE_SIZE]);
    let ciphertext = &encrypted_key[CHACHA_NONCE_SIZE..];

    let aes_key_bytes = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| ProtectError::Decryption(format!("Key decryption failed: {}", e)))?;

    aes_key_bytes
        .try_into()
        .map_err(|_| ProtectError::Decryption("Invalid AES key length".to_string()))
}

/// Decrypt AES key using P-256 ECDH
fn decrypt_key_p256(
    encrypted_key: &[u8],
    ephemeral_public_key: &str,
    private_key: &str,
) -> ProtectResult<[u8; AES_KEY_SIZE]> {
    if encrypted_key.len() < AES_NONCE_SIZE + 16 {
        return Err(ProtectError::Decryption(
            "Encrypted key too short".to_string(),
        ));
    }

    // Parse ephemeral public key
    let ephemeral_bytes = hex_decode(ephemeral_public_key)?;
    let ephemeral_point = p256::EncodedPoint::from_bytes(&ephemeral_bytes)
        .map_err(|e| ProtectError::InvalidKey(format!("Invalid ephemeral public key: {}", e)))?;
    let ephemeral_public = P256PublicKey::from_encoded_point(&ephemeral_point);
    if ephemeral_public.is_none().into() {
        return Err(ProtectError::InvalidKey(
            "Invalid ephemeral public key point".to_string(),
        ));
    }
    let ephemeral_public = ephemeral_public.unwrap();

    // Parse private key
    let private_bytes = hex_decode(private_key)?;
    let secret_key = p256::SecretKey::from_bytes((&private_bytes[..]).into())
        .map_err(|e| ProtectError::InvalidKey(format!("Invalid P-256 private key: {}", e)))?;

    // ECDH: derive shared secret
    let shared_secret =
        p256::ecdh::diffie_hellman(secret_key.to_nonzero_scalar(), ephemeral_public.as_affine());

    // Derive decryption key
    let derived_key =
        blake3::derive_key("aurora-protect-p256-v1", shared_secret.raw_secret_bytes());

    // Decrypt AES key
    let cipher = Aes256Gcm::new_from_slice(&derived_key)
        .map_err(|e| ProtectError::Decryption(e.to_string()))?;

    let nonce = AesNonce::from_slice(&encrypted_key[..AES_NONCE_SIZE]);
    let ciphertext = &encrypted_key[AES_NONCE_SIZE..];

    let aes_key_bytes = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| ProtectError::Decryption(format!("Key decryption failed: {}", e)))?;

    aes_key_bytes
        .try_into()
        .map_err(|_| ProtectError::Decryption("Invalid AES key length".to_string()))
}

// ============================================================================
// AES-256-GCM (Symmetric Encryption)
// ============================================================================

/// Encrypt data with AES-256-GCM
///
/// # Returns
/// Format: [nonce (12 bytes)][ciphertext + tag]
pub fn encrypt_aes_gcm(plaintext: &[u8], key: &[u8; AES_KEY_SIZE]) -> ProtectResult<Vec<u8>> {
    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|e| ProtectError::Encryption(e.to_string()))?;

    let nonce_bytes: [u8; AES_NONCE_SIZE] = rand::thread_rng().gen();
    let nonce = AesNonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| ProtectError::Encryption(e.to_string()))?;

    let mut result = Vec::with_capacity(AES_NONCE_SIZE + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

/// Decrypt data with AES-256-GCM
///
/// # Arguments
/// * `encrypted` - Format: [nonce (12 bytes)][ciphertext + tag]
pub fn decrypt_aes_gcm(encrypted: &[u8], key: &[u8; AES_KEY_SIZE]) -> ProtectResult<Vec<u8>> {
    if encrypted.len() < AES_NONCE_SIZE + 16 {
        return Err(ProtectError::Decryption(
            "Encrypted data too short".to_string(),
        ));
    }

    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|e| ProtectError::Decryption(e.to_string()))?;

    let nonce = AesNonce::from_slice(&encrypted[..AES_NONCE_SIZE]);
    let ciphertext = &encrypted[AES_NONCE_SIZE..];

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| ProtectError::Decryption(format!("Decryption failed: {}", e)))
}

// ============================================================================
// Key Obfuscation (for embedding private key in binary)
// ============================================================================

/// Obfuscate key storage in binary
///
/// The key is split into multiple parts and XOR'd with random constants
/// to make static analysis harder.
pub struct KeyObfuscator {
    parts: Vec<[u8; 8]>,
    xor_keys: Vec<[u8; 8]>,
}

impl KeyObfuscator {
    /// Create a new key obfuscator with the given key
    pub fn new(key: &[u8]) -> Self {
        let mut rng = rand::thread_rng();

        // Pad key to multiple of 8
        let padded_len = key.len().div_ceil(8) * 8;
        let mut padded_key = vec![0u8; padded_len];
        padded_key[..key.len()].copy_from_slice(key);

        let num_parts = padded_len / 8;
        let mut parts = Vec::with_capacity(num_parts);
        let mut xor_keys = Vec::with_capacity(num_parts);

        for i in 0..num_parts {
            let start = i * 8;
            let mut part: [u8; 8] = padded_key[start..start + 8].try_into().unwrap();
            let xor_key: [u8; 8] = rng.gen();

            // XOR the part with the key
            for j in 0..8 {
                part[j] ^= xor_key[j];
            }

            parts.push(part);
            xor_keys.push(xor_key);
        }

        Self { parts, xor_keys }
    }

    /// Reconstruct the original key
    pub fn reconstruct(&self, original_len: usize) -> Vec<u8> {
        let mut key = Vec::with_capacity(self.parts.len() * 8);

        for i in 0..self.parts.len() {
            for j in 0..8 {
                key.push(self.parts[i][j] ^ self.xor_keys[i][j]);
            }
        }

        key.truncate(original_len);
        key
    }

    /// Generate Rust code that embeds the obfuscated key
    pub fn generate_rust_code(&self, key_len: usize) -> String {
        let mut code = String::new();
        code.push_str("// Auto-generated obfuscated key - DO NOT EDIT\n");
        code.push_str(&format!(
            "const KEY_PARTS: [[u8; 8]; {}] = [\n",
            self.parts.len()
        ));
        for part in &self.parts {
            code.push_str(&format!("    {:?},\n", part));
        }
        code.push_str("];\n\n");

        code.push_str(&format!(
            "const XOR_KEYS: [[u8; 8]; {}] = [\n",
            self.xor_keys.len()
        ));
        for xor_key in &self.xor_keys {
            code.push_str(&format!("    {:?},\n", xor_key));
        }
        code.push_str("];\n\n");

        code.push_str(&format!("const KEY_LEN: usize = {};\n\n", key_len));

        code.push_str(
            r#"
fn reconstruct_key() -> Vec<u8> {
    let mut key = Vec::with_capacity(KEY_PARTS.len() * 8);
    for i in 0..KEY_PARTS.len() {
        for j in 0..8 {
            key.push(KEY_PARTS[i][j] ^ XOR_KEYS[i][j]);
        }
    }
    key.truncate(KEY_LEN);
    key
}
"#,
        );

        code
    }

    /// Generate Python code that embeds the obfuscated key
    pub fn generate_python_code(&self, key_len: usize) -> String {
        let mut code = String::new();
        code.push_str("# Auto-generated obfuscated key - DO NOT EDIT\n");
        code.push_str("_KP = [\n");
        for part in &self.parts {
            code.push_str(&format!("    bytes({:?}),\n", part.as_slice()));
        }
        code.push_str("]\n\n");

        code.push_str("_XK = [\n");
        for xor_key in &self.xor_keys {
            code.push_str(&format!("    bytes({:?}),\n", xor_key.as_slice()));
        }
        code.push_str("]\n\n");

        code.push_str(&format!("_KL = {}\n\n", key_len));

        code.push_str(
            r#"
def _rk():
    k = bytearray()
    for i in range(len(_KP)):
        for j in range(8):
            k.append(_KP[i][j] ^ _XK[i][j])
    return bytes(k[:_KL])
"#,
        );

        code
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Hex encode bytes
pub fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Hex decode string
pub fn hex_decode(s: &str) -> ProtectResult<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return Err(ProtectError::InvalidKey(
            "Hex string must have even length".to_string(),
        ));
    }

    (0..s.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16)
                .map_err(|e| ProtectError::InvalidKey(format!("Invalid hex character: {}", e)))
        })
        .collect()
}

/// Base64 encode bytes
pub fn base64_encode(bytes: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

/// Base64 decode string
pub fn base64_decode(s: &str) -> ProtectResult<Vec<u8>> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(s)
        .map_err(|e| ProtectError::InvalidKey(format!("Invalid base64: {}", e)))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_x25519_hybrid_encryption() {
        // Generate key pair
        let key_pair = EccKeyPair::generate_x25519();

        // Test data
        let bytecode = b"Hello, World! This is Python bytecode.";

        // Encrypt
        let package = encrypt_hybrid(bytecode, &key_pair.public_key, EccAlgorithm::X25519).unwrap();

        // Decrypt
        let decrypted = decrypt_hybrid(&package, &key_pair.private_key).unwrap();

        assert_eq!(bytecode.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_p256_hybrid_encryption() {
        // Generate key pair
        let key_pair = EccKeyPair::generate_p256();

        // Test data
        let bytecode = b"Hello, World! This is Python bytecode.";

        // Encrypt
        let package = encrypt_hybrid(bytecode, &key_pair.public_key, EccAlgorithm::P256).unwrap();

        // Decrypt
        let decrypted = decrypt_hybrid(&package, &key_pair.private_key).unwrap();

        assert_eq!(bytecode.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_wrong_private_key_fails() {
        let key_pair1 = EccKeyPair::generate_x25519();
        let key_pair2 = EccKeyPair::generate_x25519();

        let bytecode = b"Secret data";
        let package =
            encrypt_hybrid(bytecode, &key_pair1.public_key, EccAlgorithm::X25519).unwrap();

        // Try to decrypt with wrong key
        let result = decrypt_hybrid(&package, &key_pair2.private_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_aes_gcm_roundtrip() {
        let key: [u8; 32] = rand::thread_rng().gen();
        let plaintext = b"Test data for AES-GCM encryption";

        let encrypted = encrypt_aes_gcm(plaintext, &key).unwrap();
        let decrypted = decrypt_aes_gcm(&encrypted, &key).unwrap();

        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_key_obfuscator() {
        let original_key = b"this_is_a_32_byte_secret_key!!!";
        let obfuscator = KeyObfuscator::new(original_key);
        let reconstructed = obfuscator.reconstruct(original_key.len());

        assert_eq!(original_key.as_slice(), reconstructed.as_slice());
    }

    #[test]
    fn test_key_obfuscator_variable_length() {
        // Test with different key lengths
        for len in [16, 32, 33, 48, 64] {
            let key: Vec<u8> = (0..len).map(|i| i as u8).collect();
            let obfuscator = KeyObfuscator::new(&key);
            let reconstructed = obfuscator.reconstruct(len);
            assert_eq!(key, reconstructed);
        }
    }

    #[test]
    fn test_integrity_check() {
        let key_pair = EccKeyPair::generate_x25519();
        let bytecode = b"Original bytecode";

        let mut package =
            encrypt_hybrid(bytecode, &key_pair.public_key, EccAlgorithm::X25519).unwrap();

        // Tamper with the hash
        package.original_hash =
            "0000000000000000000000000000000000000000000000000000000000000000".to_string();

        let result = decrypt_hybrid(&package, &key_pair.private_key);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Integrity check"));
    }

    #[test]
    fn test_serialization() {
        let key_pair = EccKeyPair::generate_x25519();
        let bytecode = b"Test bytecode";

        let package = encrypt_hybrid(bytecode, &key_pair.public_key, EccAlgorithm::X25519).unwrap();

        // Serialize to JSON
        let json = serde_json::to_string(&package).unwrap();

        // Deserialize
        let deserialized: EncryptedPackage = serde_json::from_str(&json).unwrap();

        // Decrypt
        let decrypted = decrypt_hybrid(&deserialized, &key_pair.private_key).unwrap();
        assert_eq!(bytecode.as_slice(), decrypted.as_slice());
    }
}
