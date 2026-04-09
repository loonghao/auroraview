//! Aurora Protect - Python Code Protection
//!
//! Provides multiple protection strategies for Python code:
//!
//! ## Protection Strategies
//!
//! ### 1. Cython Compilation (py2pyd)
//! Compiles Python to native machine code (.pyd/.so) using Cython.
//!
//! ### 2. Hybrid Encryption (ECC + AES-256-GCM)
//! Encrypts Python bytecode (.pyc) with AES-256-GCM, then encrypts the AES key
//! with ECC (X25519 or P-256) for secure distribution.
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                        Pack Time (Dev Machine)                  │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  .py ──► py_compile ──► .pyc bytecode                          │
//! │                              │                                  │
//! │                              ▼                                  │
//! │                    AES-256-GCM encrypt (fast)                   │
//! │                              │                                  │
//! │                              ▼                                  │
//! │                      encrypted .pyc.enc                         │
//! │                                                                 │
//! │  Also: AES key ──► ECC public key encrypt ──► encrypted key    │
//! └─────────────────────────────────────────────────────────────────┘
//!
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                       Runtime (User Machine)                    │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  1. encrypted key ──► embedded private key decrypt ──► AES key │
//! │                                                                 │
//! │  2. .pyc.enc ──► AES decrypt ──► .pyc bytecode (~GB/s)         │
//! │                                                                 │
//! │  3. marshal.loads() + exec() to execute                        │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use aurora_protect::{Protector, ProtectConfig, EncryptionConfig};
//! use aurora_protect::crypto::{EccKeyPair, EccAlgorithm, encrypt_hybrid};
//!
//! // Generate ECC key pair (do this once, save private key securely)
//! let key_pair = EccKeyPair::generate(EccAlgorithm::X25519);
//!
//! // Encrypt bytecode
//! let bytecode = std::fs::read("module.pyc")?;
//! let package = encrypt_hybrid(&bytecode, &key_pair.public_key, EccAlgorithm::X25519)?;
//!
//! // Save encrypted package
//! std::fs::write("module.pyc.enc", serde_json::to_string(&package)?)?;
//! ```

/// Python AST-based precise obfuscation.
pub mod ast_obfuscator;
/// Python bytecode compilation, encryption, and protection.
pub mod bytecode;
mod config;
/// Hybrid encryption (ECC + AES-256-GCM) for secure code distribution.
pub mod crypto;
mod error;
/// Python code obfuscation (name, control flow, string encryption).
pub mod obfuscator;
mod protector;
mod runtime_gen;
/// Python AST-based precise obfuscation.
pub mod ast_obfuscator;
/// Python code obfuscation (name, control flow, string encryption).
pub mod obfuscator;

/// Obfuscation level for Python code
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ObfuscationLevel {
    /// No obfuscation
    None,
    /// Basic: local variable renaming only
    Basic,
    /// Standard: variables + functions + classes
    Standard,
    /// Advanced: Standard + control flow obfuscation
    Advanced,
    /// Maximum: Advanced + string encryption + dead code insertion
    Maximum,
}

/// Obfuscation level for Python code
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ObfuscationLevel {
    /// No obfuscation
    None,
    /// Basic: local variable renaming only
    Basic,
    /// Standard: variables + functions + classes
    Standard,
    /// Advanced: Standard + control flow obfuscation
    Advanced,
    /// Maximum: Advanced + string encryption + dead code insertion
    Maximum,
}

/// Bytecode compilation and encryption types and utilities.
pub use bytecode::{
    compile_directory, compile_to_bytecode, encrypt_bytecode, protect_with_bytecode, BytecodeFile,
    BytecodeProtectionResult, CompilationResult, EncryptedModule, EncryptionResult,
};
/// Protection configuration: encryption settings, strategy selection.
pub use config::{EncryptionConfig, ProtectConfig, ProtectionMethod};
/// Error and result types for protection operations.
pub use error::{ProtectError, ProtectResult};
/// High-level protector: compilation output and batch processing.
pub use protector::{CompileOutput, CompileResult, Protector};
/// Runtime loader code generation for encrypted modules.
pub use runtime_gen::RuntimeGenerator;

/// Cython-based Python-to-native compilation (py2pyd).
pub use py2pyd;

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub use ast_obfuscator::AstObfuscator;
pub use obfuscator::{NameObfuscator, Obfuscator};
