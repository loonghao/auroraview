//! Python bytecode compilation and encryption

use crate::crypto::{encrypt_hybrid, EccAlgorithm, EccKeyPair, EncryptedPackage};
use crate::{ProtectConfig, ProtectError, ProtectResult, RuntimeGenerator};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

/// Result of compiling a single Python file to bytecode
#[derive(Debug, Clone)]
pub struct BytecodeFile {
    pub source: PathBuf,
    pub relative_path: PathBuf,
    pub module_name: String,
    pub bytecode: Vec<u8>,
    pub source_size: u64,
}

/// Result of compiling a directory of Python files
#[derive(Debug)]
pub struct CompilationResult {
    pub compiled: Vec<BytecodeFile>,
    pub skipped: Vec<PathBuf>,
    pub total_source_size: u64,
    pub total_bytecode_size: u64,
}

/// Compile a single Python file to bytecode
pub fn compile_to_bytecode(
    source: &Path,
    python_exe: Option<&str>,
    optimize: u8,
) -> ProtectResult<Vec<u8>> {
    let python = python_exe.unwrap_or("python");
    let optimize = optimize.min(2);
    let temp_dir = std::env::temp_dir().join(format!("aurora-pyc-{}", std::process::id()));
    fs::create_dir_all(&temp_dir)?;

    let source_name = source
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("module");
    let pyc_path = temp_dir.join(format!("{}.pyc", source_name));

    let script = format!(
        "import py_compile\npy_compile.compile(r'{}', r'{}', doraise=True, optimize={})\nwith open(r'{}', 'rb') as f:\n    f.seek(16)\n    code = f.read()\nwith open(r'{}.code', 'wb') as f:\n    f.write(code)",
        source.display().to_string().replace('\\', "\\\\"),
        pyc_path.display().to_string().replace('\\', "\\\\"),
        optimize,
        pyc_path.display().to_string().replace('\\', "\\\\"),
        pyc_path.display().to_string().replace('\\', "\\\\")
    );

    let output = Command::new(python)
        .args(["-c", &script])
        .output()
        .map_err(|e| ProtectError::PythonCompile(format!("Failed to run Python: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let _ = fs::remove_dir_all(&temp_dir);
        return Err(ProtectError::PythonCompile(format!(
            "py_compile failed: {}",
            stderr
        )));
    }

    let code_path = format!("{}.code", pyc_path.display());
    let bytecode = fs::read(&code_path).map_err(|e| {
        let _ = fs::remove_dir_all(&temp_dir);
        ProtectError::PythonCompile(format!("Failed to read bytecode: {}", e))
    })?;

    let _ = fs::remove_dir_all(&temp_dir);
    Ok(bytecode)
}

/// Compile all Python files in a directory to bytecode
pub fn compile_directory(
    input_dir: &Path,
    config: &ProtectConfig,
) -> ProtectResult<CompilationResult> {
    let python_exe = config.python_path.as_deref();
    let optimize = config.optimization;
    let mut compiled = Vec::new();
    let mut skipped = Vec::new();
    let mut total_source_size = 0u64;
    let mut total_bytecode_size = 0u64;

    for entry in WalkDir::new(input_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "py") && e.path().is_file())
    {
        let source = entry.path();
        let rel_path = source.strip_prefix(input_dir).unwrap_or(source);

        if should_exclude(rel_path, &config.exclude) {
            skipped.push(source.to_path_buf());
            continue;
        }

        let source_size = fs::metadata(source)?.len();
        total_source_size += source_size;

        match compile_to_bytecode(source, python_exe, optimize) {
            Ok(bytecode) => {
                let bytecode_size = bytecode.len() as u64;
                total_bytecode_size += bytecode_size;
                compiled.push(BytecodeFile {
                    source: source.to_path_buf(),
                    relative_path: rel_path.to_path_buf(),
                    module_name: path_to_module_name(rel_path),
                    bytecode,
                    source_size,
                });
            }
            Err(e) => {
                tracing::warn!("Failed to compile {}: {}", source.display(), e);
                skipped.push(source.to_path_buf());
            }
        }
    }

    Ok(CompilationResult {
        compiled,
        skipped,
        total_source_size,
        total_bytecode_size,
    })
}

/// Encrypted module data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedModule {
    pub name: String,
    pub path: String,
    pub package: EncryptedPackage,
}

/// Result of encrypting compiled bytecode
#[derive(Debug)]
pub struct EncryptionResult {
    pub modules: Vec<EncryptedModule>,
    pub key_pair: EccKeyPair,
    pub total_encrypted_size: u64,
}

/// Encrypt compiled bytecode files
pub fn encrypt_bytecode(
    compiled: &[BytecodeFile],
    algorithm: EccAlgorithm,
    key_pair: Option<EccKeyPair>,
) -> ProtectResult<EncryptionResult> {
    let key_pair = key_pair.unwrap_or_else(|| EccKeyPair::generate(algorithm));
    let mut modules = Vec::with_capacity(compiled.len());
    let mut total_encrypted_size = 0u64;

    for file in compiled {
        let package = encrypt_hybrid(&file.bytecode, &key_pair.public_key, algorithm)?;
        total_encrypted_size += package.encrypted_data.len() as u64;
        modules.push(EncryptedModule {
            name: file.module_name.clone(),
            path: file.relative_path.to_string_lossy().replace('\\', "/"),
            package,
        });
    }

    Ok(EncryptionResult {
        modules,
        key_pair,
        total_encrypted_size,
    })
}

/// Complete result of protecting Python code
#[derive(Debug)]
pub struct BytecodeProtectionResult {
    pub files_compiled: usize,
    pub files_skipped: usize,
    pub source_size: u64,
    pub bytecode_size: u64,
    pub encrypted_size: u64,
    pub key_pair: EccKeyPair,
    pub bootstrap_path: PathBuf,
    pub modules_path: PathBuf,
}

/// Protect Python code using bytecode encryption
pub fn protect_with_bytecode(
    input_dir: &Path,
    output_dir: &Path,
    config: &ProtectConfig,
) -> ProtectResult<BytecodeProtectionResult> {
    fs::create_dir_all(output_dir)?;
    let compilation = compile_directory(input_dir, config)?;

    if compilation.compiled.is_empty() {
        return Err(ProtectError::Config("No Python files found".to_string()));
    }

    let algorithm = config
        .encryption
        .algorithm
        .parse::<EccAlgorithm>()
        .unwrap_or_default();
    let key_pair = match (
        &config.encryption.public_key,
        &config.encryption.private_key,
    ) {
        (Some(pub_key), Some(priv_key)) => EccKeyPair {
            algorithm,
            public_key: pub_key.clone(),
            private_key: priv_key.clone(),
        },
        _ => EccKeyPair::generate(algorithm),
    };

    let encryption = encrypt_bytecode(&compilation.compiled, algorithm, Some(key_pair.clone()))?;
    let generator = RuntimeGenerator::new(config.clone());
    let private_key_bytes = encryption.key_pair.private_key_bytes()?;
    let bootstrap_code =
        generator.generate_bytecode_bootstrap(&private_key_bytes, algorithm, &encryption.modules);

    let bootstrap_path = output_dir.join("__aurora_bootstrap__.py");
    fs::write(&bootstrap_path, &bootstrap_code)?;

    let modules_json = serde_json::to_string_pretty(&encryption.modules)
        .map_err(|e| ProtectError::Config(format!("Failed to serialize: {}", e)))?;
    let modules_path = output_dir.join("__aurora_modules__.json");
    fs::write(&modules_path, &modules_json)?;

    copy_non_python_files(input_dir, output_dir, &config.exclude)?;

    for skipped_file in &compilation.skipped {
        if let Ok(rel_path) = skipped_file.strip_prefix(input_dir) {
            let dest = output_dir.join(rel_path);
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(skipped_file, &dest)?;
        }
    }

    Ok(BytecodeProtectionResult {
        files_compiled: compilation.compiled.len(),
        files_skipped: compilation.skipped.len(),
        source_size: compilation.total_source_size,
        bytecode_size: compilation.total_bytecode_size,
        encrypted_size: encryption.total_encrypted_size,
        key_pair: encryption.key_pair,
        bootstrap_path,
        modules_path,
    })
}

fn should_exclude(path: &Path, exclude_patterns: &[String]) -> bool {
    if path.file_name().is_some_and(|n| n == "__init__.py") {
        return true;
    }
    let path_str = path.to_string_lossy().replace('\\', "/");
    for pattern in exclude_patterns {
        if pattern.contains('*') || pattern.contains('?') {
            if let Ok(re) = regex::Regex::new(&glob_to_regex(pattern)) {
                if re.is_match(&path_str) {
                    return true;
                }
            }
        } else if path_str.contains(pattern) {
            return true;
        }
    }
    false
}

fn glob_to_regex(pattern: &str) -> String {
    let pattern = pattern.replace('\\', "/");
    let mut out = String::from("^");
    let chars: Vec<char> = pattern.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '*' if i + 1 < chars.len() && chars[i + 1] == '*' => {
                if i + 2 < chars.len() && chars[i + 2] == '/' {
                    out.push_str("(?:.*/)?");
                    i += 3;
                } else {
                    out.push_str(".*");
                    i += 2;
                }
            }
            '*' => {
                out.push_str("[^/]*");
                i += 1;
            }
            '?' => {
                out.push_str("[^/]");
                i += 1;
            }
            '.' | '+' | '(' | ')' | '|' | '^' | '$' | '{' | '}' | '[' | ']' | '\\' => {
                out.push('\\');
                out.push(chars[i]);
                i += 1;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    out.push('$');
    out
}

fn path_to_module_name(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .trim_end_matches(".py")
        .replace('/', ".")
}

fn copy_non_python_files(
    input_dir: &Path,
    output_dir: &Path,
    exclude_patterns: &[String],
) -> ProtectResult<()> {
    for entry in WalkDir::new(input_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
    {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "py") {
            continue;
        }
        let rel_path = path.strip_prefix(input_dir).unwrap_or(path);
        if should_exclude(rel_path, exclude_patterns) {
            continue;
        }
        let dest = output_dir.join(rel_path);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(path, &dest)?;
    }
    Ok(())
}
