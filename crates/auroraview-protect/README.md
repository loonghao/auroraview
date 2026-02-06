# AuroraView Protect

Python code protection toolkit providing multiple protection strategies for Python applications.

## Features

### Protection Strategies

1. **Bytecode Encryption** (Fast, Zero-Dependency)
   - Hybrid encryption: ECC (X25519/P-256) + AES-256-GCM
   - No C compiler required at runtime
   - ~GB/s decryption speed
   - Embed private key in launcher for transparent protection

2. **py2pyd Compilation** (Maximum Protection)
   - Compile Python to native machine code (.pyd/.so)
   - Uses Cython for compilation
   - DCC-aware (Maya, Houdini, 3ds Max, etc.)
   - Generates true binary extensions

### Key Benefits

- **DCC-Aware**: Automatic Python version detection for Maya, Houdini, Blender, etc.
- **Incremental Protection**: Smart caching to speed up repeated builds
- **Flexible Exclusion**: Glob patterns to skip specific files/directories
- **Cross-Platform**: Windows, macOS, Linux support

## Quick Start

### Installation

```bash
cargo add auroraview-protect
```

Or in `Cargo.toml`:

```toml
[dependencies]
auroraview-protect = "0.1"
```

### Basic Usage

#### 1. Bytecode Encryption

```rust
use auroraview_protect::{Protector, ProtectConfig, EncryptionConfig};
use auroraview_protect::crypto::{EccKeyPair, EccAlgorithm};

// Generate ECC key pair (once per project)
let key_pair = EccKeyPair::generate(EccAlgorithm::X25519);

// Save keys securely
std::fs::write("public.key", &key_pair.public_key)?;
std::fs::write("private.key", &key_pair.private_key)?;

// Protect Python files
let config = ProtectConfig {
    input_dir: "./src".into(),
    output_dir: "./dist".into(),
    encryption: Some(EncryptionConfig {
        public_key: key_pair.public_key,
        algorithm: EccAlgorithm::X25519,
    }),
    exclusions: vec!["test_*".to_string(), "*/__pycache__/*".to_string()],
    ..Default::default()
};

let protector = Protector::new(config);
protector.protect()?;
```

#### 2. Runtime Decryption (in your launcher)

```rust
use auroraview_protect::crypto::decrypt_hybrid;

// Load private key (embedded in your exe)
let private_key = include_bytes!("private.key");

// Decrypt .pyc.enc files at runtime
let encrypted = std::fs::read("module.pyc.enc")?;
let bytecode = decrypt_hybrid(&encrypted, private_key)?;

// Execute in Python
// python: exec(marshal.loads(bytecode))
```

#### 3. py2pyd Compilation

```rust
use auroraview_protect::{Protector, ProtectConfig, CompilationConfig};

let config = ProtectConfig {
    input_dir: "./src".into(),
    output_dir: "./dist".into(),
    compilation: Some(CompilationConfig {
        dcc: Some("maya2025".to_string()), // Auto-detects Python version
        ..Default::default()
    }),
    ..Default::default()
};

let protector = Protector::new(config);
protector.protect()?;
```

## Architecture

### Encryption Flow

```text
+---------------------------------------------------------------------+
|                        Pack Time (Dev Machine)                      |
+---------------------------------------------------------------------+
|  .py --> py_compile --> .pyc bytecode                               |
|                              |                                      |
|                              v                                      |
|                    AES-256-GCM encrypt (fast)                       |
|                              |                                      |
|                              v                                      |
|                      encrypted .pyc.enc                             |
|                                                                     |
|  Also: AES key --> ECC public key encrypt --> encrypted key        |
+---------------------------------------------------------------------+

+---------------------------------------------------------------------+
|                       Runtime (User Machine)                        |
+---------------------------------------------------------------------+
|  1. encrypted key --> embedded private key decrypt --> AES key     |
|                                                                     |
|  2. .pyc.enc --> AES decrypt --> .pyc bytecode (~GB/s)             |
|                                                                     |
|  3. marshal.loads() + exec() to execute                            |
+---------------------------------------------------------------------+
```

## Configuration

### ProtectConfig

```rust
pub struct ProtectConfig {
    /// Input directory containing Python files
    pub input_dir: PathBuf,
    
    /// Output directory for protected files
    pub output_dir: PathBuf,
    
    /// Encryption configuration (optional)
    pub encryption: Option<EncryptionConfig>,
    
    /// Compilation configuration (optional)
    pub compilation: Option<CompilationConfig>,
    
    /// File/directory exclusion patterns (glob)
    pub exclusions: Vec<String>,
    
    /// Enable incremental protection with caching
    pub incremental: bool,
}
```

### EncryptionConfig

```rust
pub struct EncryptionConfig {
    /// ECC public key for key encryption
    pub public_key: Vec<u8>,
    
    /// ECC algorithm: X25519 (fast) or P-256 (FIPS)
    pub algorithm: EccAlgorithm,
}
```

### CompilationConfig

```rust
pub struct CompilationConfig {
    /// DCC app name (e.g., "maya2025", "houdini20.5")
    /// If provided, auto-detects Python version
    pub dcc: Option<String>,
    
    /// Explicit Python version (e.g., "3.10")
    pub python_version: Option<String>,
    
    /// Custom Cython options
    pub cython_options: Vec<String>,
}
```

## DCC Support

Automatically detects Python version for:

- **Maya** 2022-2026 (Python 3.9-3.11)
- **Houdini** 19.0-20.5 (Python 3.9-3.11)
- **3ds Max** 2023-2026 (Python 3.9-3.12)
- **Blender** 3.0-4.3 (Python 3.10-3.11)
- **Nuke** 13.0-15.1 (Python 3.9-3.11)
- **Unreal Engine** 5.0-5.5 (Python 3.9-3.11)

## Performance

### Bytecode Encryption

| Operation | Speed |
|-----------|-------|
| Encryption (pack time) | ~200 MB/s |
| Decryption (runtime) | ~1 GB/s |
| Key generation | <1ms |

### py2pyd Compilation

- First build: ~5-30s per file (depends on code size)
- Incremental: <1s (uses cache)

## Security Considerations

### Bytecode Encryption

- **Private key must be embedded** in your launcher/exe
- Uses modern cryptography (X25519, AES-256-GCM)
- Protects against casual inspection, not advanced reverse engineering
- Best for: IP protection, license enforcement, obfuscation

### py2pyd Compilation

- Compiles to native machine code
- Much harder to reverse than bytecode
- Best for: Critical algorithms, core business logic

## License

MIT License - see [LICENSE](LICENSE) for details

## Contributing

Contributions welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Links

- **Repository**: https://github.com/loonghao/auroraview-protect
- **Documentation**: https://docs.rs/auroraview-protect
- **Crates.io**: https://crates.io/crates/auroraview-protect
- **Issues**: https://github.com/loonghao/auroraview-protect/issues
