//! Python code protector using py2pyd

use crate::{ProtectConfig, ProtectError, ProtectResult};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Result of compiling a single file
#[derive(Debug)]
pub struct CompileOutput {
    /// Original Python file path
    pub source: PathBuf,
    /// Compiled extension path (.pyd/.so)
    pub output: PathBuf,
    /// Original file size
    pub original_size: u64,
    /// Compiled file size
    pub compiled_size: u64,
}

/// Result of compiling a directory
#[derive(Debug)]
pub struct CompileResult {
    /// Successfully compiled files
    pub compiled: Vec<CompileOutput>,
    /// Files that were skipped (excluded or failed)
    pub skipped: Vec<PathBuf>,
    /// Total original size
    pub total_original_size: u64,
    /// Total compiled size
    pub total_compiled_size: u64,
}

/// Python code protector
pub struct Protector {
    config: ProtectConfig,
}

impl Protector {
    /// Create a new protector
    pub fn new(config: ProtectConfig) -> Self {
        Self { config }
    }

    /// Get the extension suffix for compiled files
    pub fn extension_suffix() -> &'static str {
        py2pyd::get_extension()
    }

    /// Compile a single Python file
    pub fn protect_file(&self, source: &Path, output_dir: &Path) -> ProtectResult<CompileOutput> {
        let original_size = fs::metadata(source)?.len();

        // Determine output path
        let stem = source
            .file_stem()
            .ok_or_else(|| ProtectError::FileNotFound(source.display().to_string()))?
            .to_string_lossy();
        let output_path = output_dir.join(format!("{}.{}", stem, Self::extension_suffix()));

        // Ensure output directory exists
        fs::create_dir_all(output_dir)?;

        // Use py2pyd to compile
        let py2pyd_config = self.config.to_py2pyd_config();
        py2pyd::compile_file(source, &output_path, &py2pyd_config)?;

        let compiled_size = fs::metadata(&output_path)?.len();

        Ok(CompileOutput {
            source: source.to_path_buf(),
            output: output_path,
            original_size,
            compiled_size,
        })
    }

    /// Compile all Python files in a directory
    pub fn protect_directory(
        &self,
        input_dir: &Path,
        output_dir: &Path,
    ) -> ProtectResult<CompileResult> {
        let mut compiled = Vec::new();
        let mut skipped = Vec::new();
        let mut total_original_size = 0u64;
        let mut total_compiled_size = 0u64;

        // Ensure output directory exists
        fs::create_dir_all(output_dir)?;

        for entry in WalkDir::new(input_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "py") && e.path().is_file())
        {
            let rel_path = entry.path().strip_prefix(input_dir).unwrap_or(entry.path());

            // Check exclusions
            if self.should_exclude(rel_path) {
                tracing::debug!("Skipping excluded file: {}", rel_path.display());
                skipped.push(entry.path().to_path_buf());

                // Copy excluded files as-is
                let dest = output_dir.join(rel_path);
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(entry.path(), &dest)?;
                continue;
            }

            // Determine output subdirectory
            let output_subdir = if let Some(parent) = rel_path.parent() {
                output_dir.join(parent)
            } else {
                output_dir.to_path_buf()
            };

            match self.protect_file(entry.path(), &output_subdir) {
                Ok(output) => {
                    tracing::info!(
                        "Compiled: {} -> {} ({} -> {} bytes)",
                        output.source.display(),
                        output.output.display(),
                        output.original_size,
                        output.compiled_size
                    );
                    total_original_size += output.original_size;
                    total_compiled_size += output.compiled_size;
                    compiled.push(output);
                }
                Err(e) => {
                    tracing::warn!("Failed to compile {}: {}", entry.path().display(), e);
                    skipped.push(entry.path().to_path_buf());

                    // Copy failed files as-is
                    let dest = output_dir.join(rel_path);
                    if let Some(parent) = dest.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::copy(entry.path(), &dest)?;
                }
            }
        }

        Ok(CompileResult {
            compiled,
            skipped,
            total_original_size,
            total_compiled_size,
        })
    }

    /// Batch compile using py2pyd's batch function
    pub fn protect_batch(
        &self,
        input_pattern: &str,
        output_dir: &Path,
        recursive: bool,
    ) -> ProtectResult<()> {
        let py2pyd_config = self.config.to_py2pyd_config();
        py2pyd::batch_compile(input_pattern, output_dir, &py2pyd_config, recursive)?;
        Ok(())
    }

    /// Check if a file should be excluded
    fn should_exclude(&self, path: &Path) -> bool {
        // Always exclude __init__.py (need to keep as .py for package structure)
        if path.file_name().is_some_and(|n| n == "__init__.py") {
            return true;
        }

        // Normalize to forward slashes for matching consistency
        let path_str = path.to_string_lossy().replace('\\', "/");

        for pattern in &self.config.exclude {
            if pattern.contains('*') || pattern.contains('?') {
                // Glob -> regex (supports **, *, ?)
                let re_str = glob_to_regex(pattern);
                if let Ok(re) = regex::Regex::new(&re_str) {
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
}

fn glob_to_regex(pattern: &str) -> String {
    // Normalize separators in pattern too
    let pattern = pattern.replace('\\', "/");
    let chars: Vec<char> = pattern.chars().collect();

    let mut out = String::from("^");
    let mut i = 0usize;

    while i < chars.len() {
        match chars[i] {
            // ** or **/
            '*' if i + 1 < chars.len() && chars[i + 1] == '*' => {
                // Handle **/ as "zero or more directories"
                if i + 2 < chars.len() && chars[i + 2] == '/' {
                    out.push_str("(?:.*/)?");
                    i += 3;
                } else {
                    out.push_str(".*");
                    i += 2;
                }
            }
            '*' => {
                // Match within a single path segment
                out.push_str("[^/]*");
                i += 1;
            }
            '?' => {
                out.push_str("[^/]");
                i += 1;
            }
            '/' => {
                out.push('/');
                i += 1;
            }
            // Escape regex metacharacters
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_suffix() {
        let ext = Protector::extension_suffix();
        #[cfg(target_os = "windows")]
        assert_eq!(ext, "pyd");
        #[cfg(not(target_os = "windows"))]
        assert_eq!(ext, "so");
    }

    #[test]
    fn test_should_exclude() {
        let config = ProtectConfig::default();
        let protector = Protector::new(config);

        assert!(protector.should_exclude(Path::new("__init__.py")));
        assert!(protector.should_exclude(Path::new("test_main.py")));
        assert!(protector.should_exclude(Path::new("main_test.py")));
        assert!(!protector.should_exclude(Path::new("main.py")));
        assert!(!protector.should_exclude(Path::new("utils.py")));
    }
}
