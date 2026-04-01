//! Integration tests for Protector struct, CompileOutput, CompileResult,
//! and related bytecode helpers that do not require a live Python runtime.

use auroraview_protect::{
    CompileResult, EncryptionConfig, ProtectConfig, ProtectError, Protector, ProtectionMethod,
};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

// ─────────────────────────────────────────────────────────────
// Protector construction & extension suffix
// ─────────────────────────────────────────────────────────────

#[test]
fn protector_extension_suffix_platform() {
    let ext = Protector::extension_suffix();
    if cfg!(target_os = "windows") {
        assert_eq!(ext, "pyd");
    } else {
        assert_eq!(ext, "so");
    }
    // Must be non-empty and contain only ASCII alnum
    assert!(!ext.is_empty());
    assert!(ext.chars().all(|c| c.is_ascii_alphanumeric()));
}

#[test]
fn protector_new_default_config() {
    let config = ProtectConfig::default();
    let _protector = Protector::new(config);
    // Construction should not panic
}

// ─────────────────────────────────────────────────────────────
// should_exclude via protect_directory (no Python needed)
// We create non-.py files so compile_to_bytecode is never called.
// ─────────────────────────────────────────────────────────────

/// Helper: write a text file and return its path.
fn write_file(dir: &Path, name: &str, content: &str) {
    fs::write(dir.join(name), content).expect("write_file");
}

#[test]
fn protect_directory_empty_dir_returns_empty_result() {
    let input = TempDir::new().unwrap();
    let output = TempDir::new().unwrap();
    let config = ProtectConfig::default();
    let protector = Protector::new(config);
    let result: CompileResult = protector
        .protect_directory(input.path(), output.path())
        .unwrap();
    assert!(result.compiled.is_empty());
    assert!(result.skipped.is_empty());
    assert_eq!(result.total_original_size, 0);
    assert_eq!(result.total_compiled_size, 0);
}

#[test]
fn protect_directory_ignores_non_python_files() {
    let input = TempDir::new().unwrap();
    let output = TempDir::new().unwrap();
    write_file(input.path(), "data.json", r#"{"key":"value"}"#);
    write_file(input.path(), "readme.txt", "hello");

    let config = ProtectConfig::default();
    let protector = Protector::new(config);
    let result = protector
        .protect_directory(input.path(), output.path())
        .unwrap();

    // protect_directory only processes .py files; non-.py files are not compiled or skipped
    assert_eq!(result.compiled.len(), 0);
    assert_eq!(result.skipped.len(), 0);
    assert_eq!(result.total_original_size, 0);
    assert_eq!(result.total_compiled_size, 0);
}

#[test]
fn protect_directory_creates_output_dir() {
    let input = TempDir::new().unwrap();
    let base = TempDir::new().unwrap();
    // Nested output that does not yet exist
    let output = base.path().join("nested").join("output");

    let config = ProtectConfig::default();
    let protector = Protector::new(config);
    let result = protector
        .protect_directory(input.path(), &output)
        .unwrap();

    assert!(output.exists());
    assert!(result.compiled.is_empty());
}

// ─────────────────────────────────────────────────────────────
// Exclusion logic – __init__.py always excluded
// ─────────────────────────────────────────────────────────────

/// The Protector's internal exclusion is exercised via protect_directory.
/// __init__.py should be skipped (copied as-is) regardless of other settings.
#[test]
fn protect_directory_skips_init_py_file() {
    let input = TempDir::new().unwrap();
    let output = TempDir::new().unwrap();
    write_file(
        input.path(),
        "__init__.py",
        "# package init\nVERSION = '1.0'\n",
    );

    let config = ProtectConfig::default();
    let protector = Protector::new(config);
    let result = protector
        .protect_directory(input.path(), output.path())
        .unwrap();

    // __init__.py is excluded → goes to skipped, copied as-is
    assert_eq!(result.compiled.len(), 0);
    assert_eq!(result.skipped.len(), 1);
    // The file itself must be copied to output
    assert!(output.path().join("__init__.py").exists());
}

#[test]
fn protect_directory_skips_test_files() {
    let input = TempDir::new().unwrap();
    let output = TempDir::new().unwrap();
    // Both test_*.py and *_test.py default patterns should exclude these
    write_file(input.path(), "test_utils.py", "def test_foo(): pass\n");
    write_file(input.path(), "utils_test.py", "def test_bar(): pass\n");

    let config = ProtectConfig::default();
    let protector = Protector::new(config);
    let result = protector
        .protect_directory(input.path(), output.path())
        .unwrap();

    // Both test files should be in skipped
    assert_eq!(result.compiled.len(), 0);
    assert_eq!(result.skipped.len(), 2);
}

// ─────────────────────────────────────────────────────────────
// Custom exclusion patterns
// ─────────────────────────────────────────────────────────────

#[test]
fn protect_directory_custom_glob_exclude() {
    let input = TempDir::new().unwrap();
    let output = TempDir::new().unwrap();
    write_file(input.path(), "vendor_lib.py", "# vendor code\n");

    let config = ProtectConfig {
        exclude: vec!["vendor_*".to_string()],
        ..ProtectConfig::default()
    };
    let protector = Protector::new(config);
    let result = protector
        .protect_directory(input.path(), output.path())
        .unwrap();

    // vendor_lib.py should be excluded
    assert!(result.compiled.is_empty());
    assert!(!result.skipped.is_empty());
}

#[test]
fn protect_directory_exclude_double_star_pattern() {
    let input = TempDir::new().unwrap();
    let output = TempDir::new().unwrap();

    // Create nested directory: vendor/some/deep.py
    let vendor_dir = input.path().join("vendor").join("some");
    fs::create_dir_all(&vendor_dir).unwrap();
    write_file(&vendor_dir, "deep.py", "# deep vendor\n");

    let config = ProtectConfig {
        exclude: vec!["**/vendor/**".to_string()],
        ..ProtectConfig::default()
    };
    let protector = Protector::new(config);
    let result = protector
        .protect_directory(input.path(), output.path())
        .unwrap();

    assert!(result.compiled.is_empty());
    assert!(!result.skipped.is_empty());
}

// ─────────────────────────────────────────────────────────────
// ProtectConfig builder API
// ─────────────────────────────────────────────────────────────

#[test]
fn protect_config_builder_chain() {
    let config = ProtectConfig::new()
        .python_path("/usr/bin/python3")
        .python_version("3.11")
        .optimization(1)
        .keep_temp(true)
        .target_dcc("maya")
        .exclude("**/generated/**");

    assert_eq!(config.python_path.as_deref(), Some("/usr/bin/python3"));
    assert_eq!(config.python_version.as_deref(), Some("3.11"));
    assert_eq!(config.optimization, 1);
    assert!(config.keep_temp);
    assert_eq!(config.target_dcc.as_deref(), Some("maya"));
    assert!(config.exclude.iter().any(|p| p == "**/generated/**"));
}

#[test]
fn protect_config_optimization_clamped_at_three() {
    let config = ProtectConfig::new().optimization(99);
    assert_eq!(config.optimization, 3);
}

#[test]
fn protect_config_default_optimization_is_two() {
    let config = ProtectConfig::default();
    assert_eq!(config.optimization, 2);
}

#[test]
fn protect_config_default_method_is_bytecode() {
    let config = ProtectConfig::default();
    assert_eq!(config.method, ProtectionMethod::Bytecode);
}

#[test]
fn protect_config_default_exclude_patterns_present() {
    let config = ProtectConfig::default();
    let exclude = &config.exclude;
    assert!(exclude.iter().any(|p| p.contains("test_*.py")));
    assert!(exclude.iter().any(|p| p.contains("__pycache__")));
    assert!(exclude.iter().any(|p| p.contains("setup.py")));
}

// ─────────────────────────────────────────────────────────────
// EncryptionConfig builder
// ─────────────────────────────────────────────────────────────

#[test]
fn encryption_config_enabled_builder() {
    let enc = EncryptionConfig::enabled();
    assert!(enc.enabled);
    assert_eq!(enc.algorithm, "x25519");
}

#[test]
fn encryption_config_with_p256() {
    let enc = EncryptionConfig::enabled().with_p256();
    assert_eq!(enc.algorithm, "p256");
}

#[test]
fn encryption_config_with_x25519() {
    let enc = EncryptionConfig::enabled().with_p256().with_x25519();
    assert_eq!(enc.algorithm, "x25519");
}

#[test]
fn encryption_config_with_keys() {
    let enc = EncryptionConfig::enabled()
        .with_keys("pub_hex".to_string(), "priv_hex".to_string());
    assert_eq!(enc.public_key.as_deref(), Some("pub_hex"));
    assert_eq!(enc.private_key.as_deref(), Some("priv_hex"));
}

#[test]
fn encryption_config_default_disabled() {
    let enc = EncryptionConfig::default();
    assert!(!enc.enabled);
    assert!(enc.public_key.is_none());
    assert!(enc.private_key.is_none());
}

// ─────────────────────────────────────────────────────────────
// ProtectError display
// ─────────────────────────────────────────────────────────────

#[test]
fn protect_error_display_compilation() {
    let err = ProtectError::Compilation("cython failed".to_string());
    assert!(err.to_string().contains("Compilation failed"));
    assert!(err.to_string().contains("cython failed"));
}

#[test]
fn protect_error_display_python_compile() {
    let err = ProtectError::PythonCompile("syntax error".to_string());
    assert!(err.to_string().contains("Python compile failed"));
    assert!(err.to_string().contains("syntax error"));
}

#[test]
fn protect_error_display_encryption() {
    let err = ProtectError::Encryption("bad key".to_string());
    assert!(err.to_string().contains("Encryption failed"));
}

#[test]
fn protect_error_display_decryption() {
    let err = ProtectError::Decryption("auth tag mismatch".to_string());
    assert!(err.to_string().contains("Decryption failed"));
}

#[test]
fn protect_error_display_file_not_found() {
    let err = ProtectError::FileNotFound("/no/such/file.py".to_string());
    assert!(err.to_string().contains("File not found"));
    assert!(err.to_string().contains("/no/such/file.py"));
}

#[test]
fn protect_error_display_config() {
    let err = ProtectError::Config("missing field".to_string());
    assert!(err.to_string().contains("Configuration error"));
    assert!(err.to_string().contains("missing field"));
}

#[test]
fn protect_error_display_key_generation() {
    let err = ProtectError::KeyGeneration("rng failure".to_string());
    assert!(err.to_string().contains("Key generation failed"));
}

#[test]
fn protect_error_display_invalid_key() {
    let err = ProtectError::InvalidKey("bad hex".to_string());
    assert!(err.to_string().contains("Invalid key format"));
    assert!(err.to_string().contains("bad hex"));
}

#[test]
fn protect_error_display_obfuscation() {
    let err = ProtectError::Obfuscation("parse error".to_string());
    assert!(err.to_string().contains("Obfuscation failed"));
}

#[test]
fn protect_error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
    let err: ProtectError = io_err.into();
    assert!(err.to_string().contains("IO error"));
}

// ─────────────────────────────────────────────────────────────
// protect_file – missing source returns error (no Python needed)
// ─────────────────────────────────────────────────────────────

#[test]
fn protect_file_missing_source_returns_error() {
    let output = TempDir::new().unwrap();
    let config = ProtectConfig::default();
    let protector = Protector::new(config);
    let result = protector.protect_file(Path::new("/nonexistent/file.py"), output.path());
    assert!(result.is_err());
}

// ─────────────────────────────────────────────────────────────
// protect_file – with Python (skip if unavailable)
// ─────────────────────────────────────────────────────────────

#[test]
fn protect_file_real_py_if_available() {
    let input = TempDir::new().unwrap();
    let output = TempDir::new().unwrap();
    let py_file = input.path().join("greet.py");
    fs::write(&py_file, "def greet(name): return f'Hi {name}'\n").unwrap();

    let config = ProtectConfig::default();
    let protector = Protector::new(config);
    match protector.protect_file(&py_file, output.path()) {
        Ok(compile_out) => {
            assert_eq!(compile_out.source, py_file);
            assert!(compile_out.output.exists());
            assert!(compile_out.original_size > 0);
            assert!(compile_out.compiled_size > 0);
        }
        Err(_) => {
            eprintln!("Skipping protect_file test: Python/py2pyd toolchain not available");
        }
    }
}

// ─────────────────────────────────────────────────────────────
// to_py2pyd_config round-trip
// ─────────────────────────────────────────────────────────────

#[test]
fn protect_config_to_py2pyd_config_fields() {
    let config = ProtectConfig::new()
        .python_path("/usr/bin/python3")
        .python_version("3.11")
        .optimization(1)
        .keep_temp(true)
        .target_dcc("houdini");

    let py2pyd_cfg = config.to_py2pyd_config();
    assert!(py2pyd_cfg
        .python_path
        .as_ref()
        .map(|p| p.to_string_lossy().contains("python3"))
        .unwrap_or(false));
    assert_eq!(py2pyd_cfg.python_version.as_deref(), Some("3.11"));
    assert_eq!(py2pyd_cfg.optimize_level, 1);
    assert!(py2pyd_cfg.keep_temp_files);
    assert_eq!(py2pyd_cfg.target_dcc.as_deref(), Some("houdini"));
}

// ─────────────────────────────────────────────────────────────
// VERSION constant
// ─────────────────────────────────────────────────────────────

#[test]
fn crate_version_is_semver() {
    let v = auroraview_protect::VERSION;
    // Must have at least one dot (e.g. "0.1.0")
    assert!(v.contains('.'), "VERSION '{}' is not semver", v);
    // Must not be empty
    assert!(!v.is_empty());
}
