//! Runtime code generator for protected Python scripts
//!
//! Generates the aurora_runtime module that handles decryption and execution
//! of protected Python code at runtime.

use crate::bytecode::EncryptedModule;
use crate::crypto::EccAlgorithm;
use crate::ProtectConfig;

/// Generator for the aurora_runtime module
pub struct RuntimeGenerator {
    config: ProtectConfig,
}

impl RuntimeGenerator {
    /// Create a new runtime generator
    pub fn new(config: ProtectConfig) -> Self {
        Self { config }
    }

    /// Generate the Python runtime module code
    ///
    /// This module is responsible for:
    /// 1. Decrypting bytecode at runtime
    /// 2. Executing decrypted code in memory
    /// 3. Clearing memory after execution
    /// 4. Anti-debugging checks
    pub fn generate_python_runtime(&self, key: &[u8; 32]) -> String {
        let key_parts = self.obfuscate_key(key);

        format!(
            r#"# -*- coding: utf-8 -*-
"""
Aurora Runtime - Protected Code Loader
DO NOT MODIFY THIS FILE
"""
import sys
import types
import marshal
import base64
from typing import Any, Optional

# Obfuscated key parts
_K0 = bytes([{k0}])
_K1 = bytes([{k1}])
_K2 = bytes([{k2}])
_K3 = bytes([{k3}])
_X0 = bytes([{x0}])
_X1 = bytes([{x1}])
_X2 = bytes([{x2}])
_X3 = bytes([{x3}])

def _reconstruct_key() -> bytes:
    """Reconstruct the encryption key from obfuscated parts."""
    key = bytearray(32)
    for i, (k, x) in enumerate([(_K0, _X0), (_K1, _X1), (_K2, _X2), (_K3, _X3)]):
        for j in range(8):
            key[i * 8 + j] = k[j] ^ x[j]
    return bytes(key)

def _aes_decrypt(data: bytes, key: bytes) -> bytes:
    """
    AES-256-GCM decryption.
    
    Format: [nonce: 12 bytes][ciphertext + tag]
    """
    try:
        from cryptography.hazmat.primitives.ciphers.aead import AESGCM
        nonce = data[:12]
        ciphertext = data[12:]
        aesgcm = AESGCM(key)
        return aesgcm.decrypt(nonce, ciphertext, None)
    except ImportError:
        # Fallback to PyCryptodome
        from Crypto.Cipher import AES
        nonce = data[:12]
        tag = data[-16:]
        ciphertext = data[12:-16]
        cipher = AES.new(key, AES.MODE_GCM, nonce=nonce)
        return cipher.decrypt_and_verify(ciphertext, tag)

{anti_debug}

{integrity_check}

{expiration_check}

{machine_bind}

def __aurora__(
    module_name: str,
    module_file: str,
    encrypted_data: bytes,
    _globals: Optional[dict] = None
) -> Any:
    """
    Main entry point for protected code execution.
    
    Args:
        module_name: The __name__ of the calling module
        module_file: The __file__ of the calling module
        encrypted_data: The encrypted bytecode
        _globals: Optional globals dict to use
    
    Returns:
        The result of executing the protected code
    """
    {runtime_checks}
    
    # Reconstruct key
    key = _reconstruct_key()
    
    try:
        # Decrypt bytecode
        bytecode = _aes_decrypt(encrypted_data, key)
        
        # Unmarshal code object
        code = marshal.loads(bytecode)
        
        # Prepare execution environment
        if _globals is None:
            _globals = {{
                '__name__': module_name,
                '__file__': module_file,
                '__builtins__': __builtins__,
            }}
        
        # Execute in protected context
        try:
            exec(code, _globals)
        finally:
            # Clear sensitive data from memory
            _clear_frame_locals()
        
        return _globals
        
    finally:
        # Clear key from memory
        key = b'\x00' * 32
        del key

def _clear_frame_locals():
    """Clear local variables from the current frame to prevent memory inspection."""
    import gc
    frame = sys._getframe(1)
    if hasattr(frame, 'f_locals'):
        for key in list(frame.f_locals.keys()):
            if not key.startswith('__'):
                try:
                    frame.f_locals[key] = None
                except:
                    pass
    gc.collect()

# Module-level protection
def _protect_module():
    """Apply module-level protections."""
    import sys
    
    # Prevent module from being inspected
    class _ProtectedModule(types.ModuleType):
        def __repr__(self):
            return "<module 'aurora_runtime' (protected)>"
        
        def __dir__(self):
            return ['__aurora__']
    
    # Replace this module with protected version
    protected = _ProtectedModule(__name__)
    protected.__aurora__ = __aurora__
    sys.modules[__name__] = protected

_protect_module()
"#,
            k0 = key_parts.0[0]
                .iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            k1 = key_parts.0[1]
                .iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            k2 = key_parts.0[2]
                .iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            k3 = key_parts.0[3]
                .iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            x0 = key_parts.1[0]
                .iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            x1 = key_parts.1[1]
                .iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            x2 = key_parts.1[2]
                .iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            x3 = key_parts.1[3]
                .iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            anti_debug = self.generate_anti_debug(),
            integrity_check = self.generate_integrity_check(),
            expiration_check = self.generate_expiration_check(),
            machine_bind = self.generate_machine_bind(),
            runtime_checks = self.generate_runtime_checks(),
        )
    }

    /// Obfuscate the key into parts with XOR masks
    fn obfuscate_key(&self, key: &[u8; 32]) -> ([[u8; 8]; 4], [[u8; 8]; 4]) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let mut parts = [[0u8; 8]; 4];
        let mut xor_keys = [[0u8; 8]; 4];

        for i in 0..4 {
            let start = i * 8;
            let xor_key: [u8; 8] = rng.gen();
            xor_keys[i] = xor_key;

            for j in 0..8 {
                parts[i][j] = key[start + j] ^ xor_key[j];
            }
        }

        (parts, xor_keys)
    }

    /// Generate anti-debugging code
    fn generate_anti_debug(&self) -> String {
        if !self.config.anti_debug {
            return String::new();
        }

        r#"
def _check_debugger() -> bool:
    """Check if a debugger is attached."""
    import sys
    
    # Check for common debugger traces
    if sys.gettrace() is not None:
        return True
    
    # Check for debugger modules
    debugger_modules = ['pdb', 'pydevd', 'debugpy', '_pydevd_bundle']
    for mod in debugger_modules:
        if mod in sys.modules:
            return True
    
    # Check for breakpoint function override
    if hasattr(sys, 'breakpointhook'):
        return True
    
    return False

def _anti_debug():
    """Terminate if debugger detected."""
    if _check_debugger():
        import os
        os._exit(1)
"#
        .to_string()
    }

    /// Generate integrity check code
    fn generate_integrity_check(&self) -> String {
        if !self.config.integrity_check {
            return String::new();
        }

        r#"
def _verify_integrity(data: bytes, expected_hash: str) -> bool:
    """Verify data integrity using BLAKE3."""
    try:
        import hashlib
        actual_hash = hashlib.blake2b(data, digest_size=32).hexdigest()
        return actual_hash == expected_hash
    except:
        return False
"#
        .to_string()
    }

    /// Generate expiration check code
    fn generate_expiration_check(&self) -> String {
        match &self.config.expires_at {
            Some(date) => format!(
                r#"
def _check_expiration() -> bool:
    """Check if the license has expired."""
    from datetime import datetime
    expiry = datetime.fromisoformat("{}")
    return datetime.now() > expiry

def _enforce_expiration():
    """Terminate if expired."""
    if _check_expiration():
        raise RuntimeError("License has expired")
"#,
                date
            ),
            None => String::new(),
        }
    }

    /// Generate machine binding code
    fn generate_machine_bind(&self) -> String {
        if self.config.bind_machines.is_empty() {
            return String::new();
        }

        let machines = self
            .config
            .bind_machines
            .iter()
            .map(|m| format!("\"{}\"", m))
            .collect::<Vec<_>>()
            .join(", ");

        format!(
            r#"
def _get_machine_id() -> str:
    """Get a unique machine identifier."""
    import platform
    import hashlib
    
    # Combine multiple system identifiers
    info = [
        platform.node(),
        platform.machine(),
        platform.processor(),
    ]
    
    # Try to get MAC address
    try:
        import uuid
        info.append(str(uuid.getnode()))
    except:
        pass
    
    combined = "|".join(info)
    return hashlib.sha256(combined.encode()).hexdigest()[:16]

def _check_machine_binding() -> bool:
    """Check if running on an allowed machine."""
    allowed = [{}]
    current = _get_machine_id()
    return current in allowed

def _enforce_machine_binding():
    """Terminate if not on allowed machine."""
    if not _check_machine_binding():
        raise RuntimeError("This software is not licensed for this machine")
"#,
            machines
        )
    }

    /// Generate runtime checks call
    fn generate_runtime_checks(&self) -> String {
        let mut checks = Vec::new();

        if self.config.anti_debug {
            checks.push("_anti_debug()");
        }

        if self.config.expires_at.is_some() {
            checks.push("_enforce_expiration()");
        }

        if !self.config.bind_machines.is_empty() {
            checks.push("_enforce_machine_binding()");
        }

        if checks.is_empty() {
            "pass".to_string()
        } else {
            checks.join("\n    ")
        }
    }

    /// Generate bytecode bootstrap module for hybrid encryption
    ///
    /// This generates a Python module that:
    /// 1. Contains the embedded private key (obfuscated)
    /// 2. Decrypts the AES key using ECC private key
    /// 3. Decrypts bytecode using AES-256-GCM
    /// 4. Executes via marshal.loads() + exec()
    pub fn generate_bytecode_bootstrap(
        &self,
        private_key: &[u8],
        algorithm: EccAlgorithm,
        modules: &[EncryptedModule],
    ) -> String {
        // Obfuscate the private key
        let key_parts = self.obfuscate_variable_key(private_key);
        let key_len = private_key.len();

        // Generate module data as embedded JSON
        let modules_json = serde_json::to_string(modules).unwrap_or_default();

        let algorithm_str = match algorithm {
            EccAlgorithm::X25519 => "x25519",
            EccAlgorithm::P256 => "p256",
        };

        format!(
            r#"# -*- coding: utf-8 -*-
"""
Aurora Bootstrap - Encrypted Bytecode Loader
DO NOT MODIFY THIS FILE

This module provides runtime decryption and execution of protected Python code.
The private key is embedded and obfuscated within this file.
"""
import sys
import types
import marshal
import base64
import json
import hashlib
from typing import Any, Dict, Optional

# ============================================================================
# Obfuscated Private Key
# ============================================================================

_KP = [
{key_parts}
]

_XK = [
{xor_keys}
]

_KL = {key_len}

def _rk():
    """Reconstruct the private key from obfuscated parts."""
    k = bytearray()
    for i in range(len(_KP)):
        for j in range(8):
            k.append(_KP[i][j] ^ _XK[i][j])
    return bytes(k[:_KL])

# ============================================================================
# Cryptographic Functions
# ============================================================================

_ALGORITHM = "{algorithm}"

def _derive_key_x25519(shared_secret: bytes) -> bytes:
    """Derive encryption key from X25519 shared secret using BLAKE3."""
    try:
        import blake3
        return blake3.derive_key("aurora-protect-x25519-v1", shared_secret)
    except ImportError:
        # Fallback to hashlib
        return hashlib.blake2b(
            b"aurora-protect-x25519-v1" + shared_secret,
            digest_size=32
        ).digest()

def _derive_key_p256(shared_secret: bytes) -> bytes:
    """Derive encryption key from P-256 shared secret using BLAKE3."""
    try:
        import blake3
        return blake3.derive_key("aurora-protect-p256-v1", shared_secret)
    except ImportError:
        return hashlib.blake2b(
            b"aurora-protect-p256-v1" + shared_secret,
            digest_size=32
        ).digest()

def _x25519_ecdh(private_key: bytes, public_key: bytes) -> bytes:
    """Perform X25519 ECDH key exchange."""
    try:
        from cryptography.hazmat.primitives.asymmetric.x25519 import X25519PrivateKey, X25519PublicKey
        priv = X25519PrivateKey.from_private_bytes(private_key)
        pub = X25519PublicKey.from_public_bytes(public_key)
        return priv.exchange(pub)
    except ImportError:
        # Fallback to nacl
        import nacl.bindings
        return nacl.bindings.crypto_scalarmult(private_key, public_key)

def _p256_ecdh(private_key: bytes, public_key: bytes) -> bytes:
    """Perform P-256 ECDH key exchange."""
    from cryptography.hazmat.primitives.asymmetric import ec
    from cryptography.hazmat.primitives.serialization import Encoding, PublicFormat
    from cryptography.hazmat.backends import default_backend
    
    # Load private key
    priv = ec.derive_private_key(
        int.from_bytes(private_key, 'big'),
        ec.SECP256R1(),
        default_backend()
    )
    
    # Load public key from compressed point
    from cryptography.hazmat.primitives.serialization import load_der_public_key
    pub = ec.EllipticCurvePublicKey.from_encoded_point(ec.SECP256R1(), public_key)
    
    # Perform ECDH
    shared = priv.exchange(ec.ECDH(), pub)
    return shared

def _chacha20_decrypt(data: bytes, key: bytes) -> bytes:
    """ChaCha20-Poly1305 decryption."""
    from cryptography.hazmat.primitives.ciphers.aead import ChaCha20Poly1305
    nonce = data[:12]
    ciphertext = data[12:]
    cipher = ChaCha20Poly1305(key)
    return cipher.decrypt(nonce, ciphertext, None)

def _aes_gcm_decrypt(data: bytes, key: bytes) -> bytes:
    """AES-256-GCM decryption."""
    try:
        from cryptography.hazmat.primitives.ciphers.aead import AESGCM
        nonce = data[:12]
        ciphertext = data[12:]
        cipher = AESGCM(key)
        return cipher.decrypt(nonce, ciphertext, None)
    except ImportError:
        from Crypto.Cipher import AES
        nonce = data[:12]
        tag = data[-16:]
        ciphertext = data[12:-16]
        cipher = AES.new(key, AES.MODE_GCM, nonce=nonce)
        return cipher.decrypt_and_verify(ciphertext, tag)

def _decrypt_aes_key(encrypted_key: bytes, ephemeral_public: bytes, private_key: bytes) -> bytes:
    """Decrypt the AES key using ECC private key."""
    if _ALGORITHM == "x25519":
        shared_secret = _x25519_ecdh(private_key, ephemeral_public)
        derived_key = _derive_key_x25519(shared_secret)
        return _chacha20_decrypt(encrypted_key, derived_key)
    else:  # p256
        shared_secret = _p256_ecdh(private_key, ephemeral_public)
        derived_key = _derive_key_p256(shared_secret)
        return _aes_gcm_decrypt(encrypted_key, derived_key)

def _decrypt_bytecode(package: dict, private_key: bytes) -> bytes:
    """Decrypt bytecode from an encrypted package."""
    # Decode hex/base64 fields
    ephemeral_public = bytes.fromhex(package["ephemeral_public_key"])
    encrypted_key = bytes.fromhex(package["encrypted_key"])
    encrypted_data = base64.b64decode(package["encrypted_data"])
    original_hash = package["original_hash"]
    
    # Step 1: Decrypt AES key
    aes_key = _decrypt_aes_key(encrypted_key, ephemeral_public, private_key)
    
    # Step 2: Decrypt bytecode
    bytecode = _aes_gcm_decrypt(encrypted_data, aes_key)
    
    # Step 3: Verify integrity
    try:
        import blake3
        computed_hash = blake3.blake3(bytecode).hexdigest()
    except ImportError:
        computed_hash = hashlib.blake2b(bytecode, digest_size=32).hexdigest()
    
    if computed_hash != original_hash:
        raise RuntimeError("Integrity check failed: bytecode hash mismatch")
    
    return bytecode

# ============================================================================
# Module Loading
# ============================================================================

# Embedded encrypted modules
_MODULES_DATA = '''{modules_json}'''

_MODULES: Dict[str, dict] = {{}}
_LOADED: Dict[str, types.ModuleType] = {{}}

def _init_modules():
    """Initialize modules from embedded data."""
    global _MODULES
    if not _MODULES:
        _MODULES = {{m["name"]: m for m in json.loads(_MODULES_DATA)}}

def _load_module(name: str) -> types.ModuleType:
    """Load and execute an encrypted module."""
    _init_modules()
    
    if name in _LOADED:
        return _LOADED[name]
    
    if name not in _MODULES:
        raise ImportError(f"No encrypted module named '{{name}}'")
    
    module_data = _MODULES[name]
    private_key = _rk()
    
    try:
        # Decrypt bytecode
        bytecode = _decrypt_bytecode(module_data["package"], private_key)
        
        # Unmarshal code object
        code = marshal.loads(bytecode)
        
        # Create module
        module = types.ModuleType(name)
        module.__file__ = module_data["path"]
        module.__loader__ = _AuroraLoader()
        
        # Execute module code
        exec(code, module.__dict__)
        
        # Cache and register
        _LOADED[name] = module
        sys.modules[name] = module
        
        return module
    finally:
        # Clear sensitive data
        private_key = b'\\x00' * len(private_key)
        del private_key

class _AuroraLoader:
    """Custom loader for encrypted modules."""
    
    def create_module(self, spec):
        return None
    
    def exec_module(self, module):
        pass

class _AuroraFinder:
    """Custom finder for encrypted modules."""
    
    def find_module(self, name, path=None):
        _init_modules()
        if name in _MODULES:
            return _AuroraLoader()
        return None
    
    def find_spec(self, name, path, target=None):
        _init_modules()
        if name in _MODULES:
            from importlib.machinery import ModuleSpec
            return ModuleSpec(name, _AuroraLoader())
        return None

# Install custom finder
sys.meta_path.insert(0, _AuroraFinder())

# ============================================================================
# Public API
# ============================================================================

def __aurora_load__(name: str) -> types.ModuleType:
    """Load an encrypted module by name."""
    return _load_module(name)

def __aurora_exec__(
    module_name: str,
    module_file: str,
    encrypted_package: dict,
    _globals: Optional[dict] = None
) -> dict:
    """
    Decrypt and execute protected code directly.
    
    Args:
        module_name: The __name__ for the module
        module_file: The __file__ for the module
        encrypted_package: The encrypted package dict
        _globals: Optional globals dict
    
    Returns:
        The module's globals dict after execution
    """
    private_key = _rk()
    
    try:
        # Decrypt bytecode
        bytecode = _decrypt_bytecode(encrypted_package, private_key)
        
        # Unmarshal code object
        code = marshal.loads(bytecode)
        
        # Prepare execution environment
        if _globals is None:
            _globals = {{
                '__name__': module_name,
                '__file__': module_file,
                '__builtins__': __builtins__,
            }}
        
        # Execute
        exec(code, _globals)
        return _globals
    finally:
        private_key = b'\\x00' * len(private_key)
        del private_key

def __aurora_list__() -> list:
    """List all available encrypted modules."""
    _init_modules()
    return list(_MODULES.keys())

# ============================================================================
# Module Protection
# ============================================================================

def _protect_bootstrap():
    """Protect this bootstrap module from inspection."""
    class _ProtectedModule(types.ModuleType):
        def __repr__(self):
            return "<module '__aurora_bootstrap__' (protected)>"
        
        def __dir__(self):
            return ['__aurora_load__', '__aurora_exec__', '__aurora_list__']
    
    protected = _ProtectedModule(__name__)
    protected.__aurora_load__ = __aurora_load__
    protected.__aurora_exec__ = __aurora_exec__
    protected.__aurora_list__ = __aurora_list__
    sys.modules[__name__] = protected

_protect_bootstrap()
"#,
            key_parts = key_parts
                .0
                .iter()
                .map(|p| format!(
                    "    bytes([{}]),",
                    p.iter()
                        .map(|b| b.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
                .collect::<Vec<_>>()
                .join("\n"),
            xor_keys = key_parts
                .1
                .iter()
                .map(|x| format!(
                    "    bytes([{}]),",
                    x.iter()
                        .map(|b| b.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
                .collect::<Vec<_>>()
                .join("\n"),
            key_len = key_len,
            algorithm = algorithm_str,
            modules_json = modules_json.replace('\\', "\\\\").replace('\'', "\\'"),
        )
    }

    /// Obfuscate a variable-length key into parts with XOR masks
    fn obfuscate_variable_key(&self, key: &[u8]) -> (Vec<[u8; 8]>, Vec<[u8; 8]>) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        // Pad to multiple of 8
        let padded_len = key.len().div_ceil(8) * 8;
        let mut padded = vec![0u8; padded_len];
        padded[..key.len()].copy_from_slice(key);

        let num_parts = padded_len / 8;
        let mut parts = Vec::with_capacity(num_parts);
        let mut xor_keys = Vec::with_capacity(num_parts);

        for i in 0..num_parts {
            let start = i * 8;
            let xor_key: [u8; 8] = rng.gen();
            let mut part = [0u8; 8];

            for j in 0..8 {
                part[j] = padded[start + j] ^ xor_key[j];
            }

            parts.push(part);
            xor_keys.push(xor_key);
        }

        (parts, xor_keys)
    }

    /// Generate the Rust runtime extension code (for PyO3)
    ///
    /// This provides a native implementation of the decryption logic
    /// which is harder to reverse engineer than pure Python.
    pub fn generate_rust_extension(&self) -> String {
        r#"//! Aurora Runtime - Native Python Extension
//!
//! This module provides native decryption and code execution
//! for protected Python scripts.

use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyModule};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};

/// Decrypt and execute protected Python code
#[pyfunction]
fn __aurora__(
    py: Python<'_>,
    module_name: &str,
    module_file: &str,
    encrypted_data: &[u8],
    globals: Option<&PyDict>,
) -> PyResult<PyObject> {
    // Reconstruct key (embedded at build time)
    let key = reconstruct_key();
    
    // Decrypt
    let bytecode = decrypt(encrypted_data, &key)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            format!("Decryption failed: {}", e)
        ))?;
    
    // Unmarshal code object
    let marshal = py.import("marshal")?;
    let code = marshal.call_method1("loads", (PyBytes::new(py, &bytecode),))?;
    
    // Prepare globals
    let exec_globals = match globals {
        Some(g) => g.copy()?,
        None => {
            let g = PyDict::new(py);
            g.set_item("__name__", module_name)?;
            g.set_item("__file__", module_file)?;
            g.set_item("__builtins__", py.import("builtins")?)?;
            g
        }
    };
    
    // Execute
    py.import("builtins")?
        .call_method1("exec", (code, exec_globals))?;
    
    Ok(exec_globals.into())
}

fn reconstruct_key() -> [u8; 32] {
    // Key parts will be embedded here at build time
    include!(concat!(env!("OUT_DIR"), "/key_parts.rs"))
}

fn decrypt(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, String> {
    if data.len() < 28 {
        return Err("Data too short".to_string());
    }
    
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| e.to_string())?;
    
    let nonce = Nonce::from_slice(&data[..12]);
    let ciphertext = &data[12..];
    
    cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))
}

/// Aurora Runtime module
#[pymodule]
fn aurora_runtime(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(__aurora__, m)?)?;
    Ok(())
}
"#
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_runtime() {
        let config = ProtectConfig::default();
        let generator = RuntimeGenerator::new(config);

        let key = [0u8; 32];
        let runtime = generator.generate_python_runtime(&key);

        assert!(runtime.contains("def __aurora__"));
        assert!(runtime.contains("_reconstruct_key"));
        assert!(runtime.contains("_aes_decrypt"));
    }

    #[test]
    fn test_generate_with_anti_debug() {
        let config = ProtectConfig {
            anti_debug: true,
            ..Default::default()
        };
        let generator = RuntimeGenerator::new(config);

        let key = [0u8; 32];
        let runtime = generator.generate_python_runtime(&key);

        assert!(runtime.contains("_check_debugger"));
        assert!(runtime.contains("_anti_debug"));
    }

    #[test]
    fn test_generate_with_expiration() {
        let config = ProtectConfig {
            expires_at: Some("2025-12-31".to_string()),
            ..Default::default()
        };
        let generator = RuntimeGenerator::new(config);

        let key = [0u8; 32];
        let runtime = generator.generate_python_runtime(&key);

        assert!(runtime.contains("_check_expiration"));
        assert!(runtime.contains("2025-12-31"));
    }
}
