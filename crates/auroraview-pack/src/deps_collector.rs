//! Python Dependency Collector
//!
//! Automatically discovers and collects Python dependencies similar to PyInstaller.
//! This module analyzes Python source files to find imports and collects the
//! corresponding packages from the current Python environment.

use crate::{PackError, PackResult};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Collected dependency information
#[derive(Debug, Clone)]
pub struct CollectedDeps {
    /// Paths to collected packages/modules
    pub paths: Vec<PathBuf>,
    /// Total size in bytes
    pub total_size: u64,
    /// Number of files collected
    pub file_count: usize,
    /// Package names that were collected
    pub packages: Vec<String>,
}

/// Python dependency collector
pub struct DepsCollector {
    /// Python executable to use
    python_exe: PathBuf,
    /// Packages to exclude
    exclude_packages: HashSet<String>,
    /// Additional packages to include
    include_packages: HashSet<String>,
}

impl DepsCollector {
    /// Create a new dependency collector
    pub fn new() -> Self {
        Self {
            python_exe: PathBuf::from("python"),
            exclude_packages: default_excludes(),
            include_packages: HashSet::new(),
        }
    }

    /// Set the Python executable to use
    pub fn python_exe(mut self, path: impl Into<PathBuf>) -> Self {
        self.python_exe = path.into();
        self
    }

    /// Add packages to exclude
    pub fn exclude(mut self, packages: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for pkg in packages {
            self.exclude_packages.insert(pkg.into());
        }
        self
    }

    /// Add packages to include (even if not detected)
    pub fn include(mut self, packages: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for pkg in packages {
            self.include_packages.insert(pkg.into());
        }
        self
    }

    /// Analyze a Python file and discover its dependencies
    pub fn analyze_file(&self, file_path: &Path) -> PackResult<Vec<String>> {
        let script = r#"
import sys
import ast
import importlib.util

def get_imports(file_path):
    with open(file_path, 'r', encoding='utf-8') as f:
        try:
            tree = ast.parse(f.read())
        except SyntaxError:
            return []
    
    imports = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            for alias in node.names:
                imports.add(alias.name.split('.')[0])
        elif isinstance(node, ast.ImportFrom):
            if node.module:
                imports.add(node.module.split('.')[0])
    
    return list(imports)

file_path = sys.argv[1]
for imp in get_imports(file_path):
    print(imp)
"#;

        let output = Command::new(&self.python_exe)
            .args(["-c", script, file_path.to_str().unwrap_or("")])
            .output()
            .map_err(|e| PackError::Config(format!("Failed to run Python: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("Failed to analyze {}: {}", file_path.display(), stderr);
            return Ok(Vec::new());
        }

        let imports: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(imports)
    }

    /// Get the installation path for a package
    pub fn get_package_path(&self, package_name: &str) -> PackResult<Option<PathBuf>> {
        let script = format!(
            r#"
import importlib.util
import os

spec = importlib.util.find_spec("{}")
if spec and spec.origin:
    # Get the package directory
    origin = spec.origin
    if origin.endswith('__init__.py'):
        print(os.path.dirname(origin))
    else:
        print(origin)
elif spec and spec.submodule_search_locations:
    for loc in spec.submodule_search_locations:
        print(loc)
        break
"#,
            package_name
        );

        let output = Command::new(&self.python_exe)
            .args(["-c", &script])
            .output()
            .map_err(|e| PackError::Config(format!("Failed to run Python: {}", e)))?;

        if !output.status.success() {
            return Ok(None);
        }

        let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if path_str.is_empty() {
            return Ok(None);
        }

        Ok(Some(PathBuf::from(path_str)))
    }

    /// Collect all dependencies for a Python entry point
    pub fn collect(&self, entry_files: &[PathBuf], dest_dir: &Path) -> PackResult<CollectedDeps> {
        let mut all_imports = HashSet::new();

        // Analyze all entry files
        for file in entry_files {
            if file.exists() && file.extension().is_some_and(|e| e == "py") {
                let imports = self.analyze_file(file)?;
                all_imports.extend(imports);
            }
        }

        // Add explicitly included packages
        all_imports.extend(self.include_packages.iter().cloned());

        // Filter out excluded and stdlib packages
        let packages_to_collect: Vec<String> = all_imports
            .into_iter()
            .filter(|p| !self.exclude_packages.contains(p))
            .filter(|p| !is_stdlib(p))
            .collect();

        tracing::info!(
            "Discovered {} packages to collect: {:?}",
            packages_to_collect.len(),
            packages_to_collect
        );

        let mut collected = CollectedDeps {
            paths: Vec::new(),
            total_size: 0,
            file_count: 0,
            packages: Vec::new(),
        };

        std::fs::create_dir_all(dest_dir)?;

        for package in &packages_to_collect {
            if let Some(pkg_path) = self.get_package_path(package)? {
                let result = self.copy_package(&pkg_path, dest_dir, package)?;
                collected.paths.push(result.0);
                collected.total_size += result.1;
                collected.file_count += result.2;
                collected.packages.push(package.clone());
            } else {
                tracing::warn!("Package not found: {}", package);
            }
        }

        Ok(collected)
    }

    /// Copy a package to the destination directory
    fn copy_package(
        &self,
        src: &Path,
        dest_dir: &Path,
        package_name: &str,
    ) -> PackResult<(PathBuf, u64, usize)> {
        let mut total_size = 0u64;
        let mut file_count = 0usize;

        if src.is_file() {
            // Single file module (e.g., yaml.py)
            let dest = dest_dir.join(src.file_name().unwrap_or_default());
            std::fs::copy(src, &dest)?;
            total_size = std::fs::metadata(&dest)?.len();
            file_count = 1;
            tracing::debug!("Collected module: {} -> {}", src.display(), dest.display());
            return Ok((dest, total_size, file_count));
        }

        // Directory package
        let dest = dest_dir.join(package_name);
        std::fs::create_dir_all(&dest)?;

        for entry in walkdir::WalkDir::new(src)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            let rel_path = path.strip_prefix(src).unwrap_or(path);
            let dest_path = dest.join(rel_path);

            if path.is_dir() {
                std::fs::create_dir_all(&dest_path)?;
            } else if path.is_file() {
                // Skip __pycache__ and .pyc files
                if rel_path
                    .to_string_lossy()
                    .contains("__pycache__")
                {
                    continue;
                }
                if path.extension().is_some_and(|e| e == "pyc") {
                    continue;
                }

                if let Some(parent) = dest_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::copy(path, &dest_path)?;
                total_size += std::fs::metadata(&dest_path)?.len();
                file_count += 1;
            }
        }

        tracing::debug!(
            "Collected package: {} ({} files, {} bytes)",
            package_name,
            file_count,
            total_size
        );

        Ok((dest, total_size, file_count))
    }

    /// Collect site-packages for specific packages using pip
    pub fn collect_with_pip(
        &self,
        packages: &[String],
        dest_dir: &Path,
    ) -> PackResult<CollectedDeps> {
        if packages.is_empty() {
            return Ok(CollectedDeps {
                paths: Vec::new(),
                total_size: 0,
                file_count: 0,
                packages: Vec::new(),
            });
        }

        std::fs::create_dir_all(dest_dir)?;

        tracing::info!("Installing {} packages with pip...", packages.len());

        // Use pip to install packages to dest_dir
        let status = Command::new(&self.python_exe)
            .args([
                "-m",
                "pip",
                "install",
                "--target",
                dest_dir.to_str().unwrap_or("."),
                "--no-compile",
                "--no-deps",
            ])
            .args(packages)
            .status()
            .map_err(|e| PackError::Config(format!("Failed to run pip: {}", e)))?;

        if !status.success() {
            tracing::warn!("pip install exited with non-zero status");
        }

        // Calculate collected stats
        let mut total_size = 0u64;
        let mut file_count = 0usize;

        for entry in walkdir::WalkDir::new(dest_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
        {
            total_size += std::fs::metadata(entry.path())?.len();
            file_count += 1;
        }

        Ok(CollectedDeps {
            paths: vec![dest_dir.to_path_buf()],
            total_size,
            file_count,
            packages: packages.to_vec(),
        })
    }
}

impl Default for DepsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Default packages to exclude (stdlib and common dev packages)
fn default_excludes() -> HashSet<String> {
    [
        // Test frameworks
        "pytest",
        "unittest",
        "nose",
        "coverage",
        // Dev tools
        "pip",
        "setuptools",
        "wheel",
        "pkg_resources",
        // Type checking
        "mypy",
        "typing_extensions",
        // Linting
        "pylint",
        "flake8",
        "black",
        "ruff",
        // Build tools
        "build",
        "twine",
        "maturin",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

/// Check if a module is part of Python standard library
fn is_stdlib(module: &str) -> bool {
    // Common stdlib modules
    const STDLIB: &[&str] = &[
        "abc",
        "aifc",
        "argparse",
        "array",
        "ast",
        "asynchat",
        "asyncio",
        "asyncore",
        "atexit",
        "audioop",
        "base64",
        "bdb",
        "binascii",
        "binhex",
        "bisect",
        "builtins",
        "bz2",
        "calendar",
        "cgi",
        "cgitb",
        "chunk",
        "cmath",
        "cmd",
        "code",
        "codecs",
        "codeop",
        "collections",
        "colorsys",
        "compileall",
        "concurrent",
        "configparser",
        "contextlib",
        "contextvars",
        "copy",
        "copyreg",
        "cProfile",
        "crypt",
        "csv",
        "ctypes",
        "curses",
        "dataclasses",
        "datetime",
        "dbm",
        "decimal",
        "difflib",
        "dis",
        "distutils",
        "doctest",
        "email",
        "encodings",
        "enum",
        "errno",
        "faulthandler",
        "fcntl",
        "filecmp",
        "fileinput",
        "fnmatch",
        "fractions",
        "ftplib",
        "functools",
        "gc",
        "getopt",
        "getpass",
        "gettext",
        "glob",
        "graphlib",
        "grp",
        "gzip",
        "hashlib",
        "heapq",
        "hmac",
        "html",
        "http",
        "idlelib",
        "imaplib",
        "imghdr",
        "imp",
        "importlib",
        "inspect",
        "io",
        "ipaddress",
        "itertools",
        "json",
        "keyword",
        "lib2to3",
        "linecache",
        "locale",
        "logging",
        "lzma",
        "mailbox",
        "mailcap",
        "marshal",
        "math",
        "mimetypes",
        "mmap",
        "modulefinder",
        "multiprocessing",
        "netrc",
        "nis",
        "nntplib",
        "numbers",
        "operator",
        "optparse",
        "os",
        "ossaudiodev",
        "pathlib",
        "pdb",
        "pickle",
        "pickletools",
        "pipes",
        "pkgutil",
        "platform",
        "plistlib",
        "poplib",
        "posix",
        "posixpath",
        "pprint",
        "profile",
        "pstats",
        "pty",
        "pwd",
        "py_compile",
        "pyclbr",
        "pydoc",
        "queue",
        "quopri",
        "random",
        "re",
        "readline",
        "reprlib",
        "resource",
        "rlcompleter",
        "runpy",
        "sched",
        "secrets",
        "select",
        "selectors",
        "shelve",
        "shlex",
        "shutil",
        "signal",
        "site",
        "smtpd",
        "smtplib",
        "sndhdr",
        "socket",
        "socketserver",
        "spwd",
        "sqlite3",
        "ssl",
        "stat",
        "statistics",
        "string",
        "stringprep",
        "struct",
        "subprocess",
        "sunau",
        "symtable",
        "sys",
        "sysconfig",
        "syslog",
        "tabnanny",
        "tarfile",
        "telnetlib",
        "tempfile",
        "termios",
        "test",
        "textwrap",
        "threading",
        "time",
        "timeit",
        "tkinter",
        "token",
        "tokenize",
        "trace",
        "traceback",
        "tracemalloc",
        "tty",
        "turtle",
        "turtledemo",
        "types",
        "typing",
        "unicodedata",
        "unittest",
        "urllib",
        "uu",
        "uuid",
        "venv",
        "warnings",
        "wave",
        "weakref",
        "webbrowser",
        "winreg",
        "winsound",
        "wsgiref",
        "xdrlib",
        "xml",
        "xmlrpc",
        "zipapp",
        "zipfile",
        "zipimport",
        "zlib",
        "_thread",
        "__future__",
    ];

    STDLIB.contains(&module)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_stdlib() {
        assert!(is_stdlib("os"));
        assert!(is_stdlib("sys"));
        assert!(is_stdlib("json"));
        assert!(is_stdlib("pathlib"));
        assert!(!is_stdlib("requests"));
        assert!(!is_stdlib("numpy"));
        assert!(!is_stdlib("auroraview"));
    }

    #[test]
    fn test_default_excludes() {
        let excludes = default_excludes();
        assert!(excludes.contains("pytest"));
        assert!(excludes.contains("pip"));
        assert!(excludes.contains("setuptools"));
    }

    #[test]
    fn test_collector_builder() {
        let collector = DepsCollector::new()
            .python_exe("python3")
            .exclude(["test_pkg"])
            .include(["extra_pkg"]);

        assert_eq!(collector.python_exe, PathBuf::from("python3"));
        assert!(collector.exclude_packages.contains("test_pkg"));
        assert!(collector.include_packages.contains("extra_pkg"));
    }
}
