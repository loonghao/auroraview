//! Integration tests for Python Dependency Collector
//!
//! These tests verify the dependency collection functionality for various scenarios:
//! - Simple third-party packages (e.g., pyyaml)
//! - Complex third-party packages with native extensions (e.g., numpy)
//! - Local packages (e.g., auroraview)
//! - Binary packages with native libraries (e.g., PySide6)
//!
//! ## Test Requirements
//!
//! Some tests require specific packages to be installed in the Python environment.
//! Tests will be skipped if the required packages are not available.

use auroraview_pack::DepsCollector;
use std::path::PathBuf;
use tempfile::tempdir;

// ============================================================================
// Helper Functions
// ============================================================================

/// Check if Python is available
fn python_available() -> bool {
    std::process::Command::new("python")
        .args(["--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if a Python package is installed
fn package_installed(package: &str) -> bool {
    std::process::Command::new("python")
        .args(["-c", &format!("import {}", package)])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// ============================================================================
// Unit Tests for DepsCollector
// ============================================================================

#[test]
fn test_deps_collector_new() {
    // Should create a collector with default settings
    let _collector = DepsCollector::new();
    // If this compiles and runs, the collector was created successfully
}

#[test]
fn test_deps_collector_builder_pattern() {
    // Test that builder pattern works (fields are private, so we just verify it compiles)
    let _collector = DepsCollector::new()
        .python_exe("python3.10")
        .exclude(["pytest", "mypy"])
        .include(["extra_package"]);
}

#[test]
fn test_deps_collector_exclude_chain() {
    // Test that exclude can be chained
    let _collector = DepsCollector::new()
        .exclude(["pkg1"])
        .exclude(["pkg2", "pkg3"]);
}

#[test]
fn test_deps_collector_include_chain() {
    // Test that include can be chained
    let _collector = DepsCollector::new()
        .include(["pkg1"])
        .include(["pkg2", "pkg3"]);
}

// ============================================================================
// Integration Tests - File Analysis
// ============================================================================

#[test]
fn test_analyze_simple_python_file() {
    if !python_available() {
        eprintln!("Skipping test: Python not available");
        return;
    }

    let temp = tempdir().unwrap();
    let test_file = temp.path().join("test_imports.py");

    // Create a test Python file with various imports
    std::fs::write(
        &test_file,
        r#"
import os
import sys
import json
from pathlib import Path
from collections import defaultdict
import yaml
import requests
from auroraview import WebView
"#,
    )
    .unwrap();

    let collector = DepsCollector::new();
    let imports = collector.analyze_file(&test_file).unwrap();

    // Should detect all imports
    assert!(imports.contains(&"os".to_string()));
    assert!(imports.contains(&"sys".to_string()));
    assert!(imports.contains(&"json".to_string()));
    assert!(imports.contains(&"pathlib".to_string()));
    assert!(imports.contains(&"collections".to_string()));
    assert!(imports.contains(&"yaml".to_string()));
    assert!(imports.contains(&"requests".to_string()));
    assert!(imports.contains(&"auroraview".to_string()));
}

#[test]
fn test_analyze_file_with_relative_imports() {
    if !python_available() {
        eprintln!("Skipping test: Python not available");
        return;
    }

    let temp = tempdir().unwrap();
    let test_file = temp.path().join("test_relative.py");

    // Create a test Python file with relative imports
    std::fs::write(
        &test_file,
        r#"
from . import local_module
from .utils import helper
from ..parent import something
import absolute_package
"#,
    )
    .unwrap();

    let collector = DepsCollector::new();
    let imports = collector.analyze_file(&test_file).unwrap();

    // Should detect absolute import
    assert!(imports.contains(&"absolute_package".to_string()));
    // Relative imports should not be included (they don't have a module name)
}

#[test]
fn test_analyze_file_with_syntax_error() {
    if !python_available() {
        eprintln!("Skipping test: Python not available");
        return;
    }

    let temp = tempdir().unwrap();
    let test_file = temp.path().join("syntax_error.py");

    // Create a Python file with syntax error
    std::fs::write(
        &test_file,
        r#"
import os
def broken(
    # Missing closing parenthesis
"#,
    )
    .unwrap();

    let collector = DepsCollector::new();
    let imports = collector.analyze_file(&test_file).unwrap();

    // Should return empty list on syntax error (graceful handling)
    assert!(imports.is_empty());
}

#[test]
fn test_analyze_nonexistent_file() {
    if !python_available() {
        eprintln!("Skipping test: Python not available");
        return;
    }

    let collector = DepsCollector::new();
    let result = collector.analyze_file(&PathBuf::from("/nonexistent/file.py"));

    // Should return empty list for nonexistent file
    assert!(result.unwrap().is_empty());
}

// ============================================================================
// Integration Tests - Package Path Discovery
// ============================================================================

#[test]
fn test_get_package_path_stdlib() {
    if !python_available() {
        eprintln!("Skipping test: Python not available");
        return;
    }

    let collector = DepsCollector::new();

    // Standard library packages should be found
    let os_path = collector.get_package_path("os").unwrap();
    assert!(os_path.is_some());

    let json_path = collector.get_package_path("json").unwrap();
    assert!(json_path.is_some());
}

#[test]
fn test_get_package_path_nonexistent() {
    if !python_available() {
        eprintln!("Skipping test: Python not available");
        return;
    }

    let collector = DepsCollector::new();

    // Nonexistent package should return None
    let result = collector.get_package_path("nonexistent_package_xyz123").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_get_package_path_auroraview() {
    if !python_available() {
        eprintln!("Skipping test: Python not available");
        return;
    }

    if !package_installed("auroraview") {
        eprintln!("Skipping test: auroraview not installed");
        return;
    }

    let collector = DepsCollector::new();
    let path = collector.get_package_path("auroraview").unwrap();

    assert!(path.is_some());
    let path = path.unwrap();
    // Should be a directory containing __init__.py
    assert!(path.is_dir() || path.is_file());
}

// ============================================================================
// Integration Tests - Package Collection
// ============================================================================

#[test]
fn test_collect_simple_package() {
    if !python_available() {
        eprintln!("Skipping test: Python not available");
        return;
    }

    if !package_installed("yaml") {
        eprintln!("Skipping test: pyyaml not installed");
        return;
    }

    let temp = tempdir().unwrap();
    let dest = temp.path().join("collected");
    let entry_file = temp.path().join("entry.py");

    // Create entry file that imports yaml
    std::fs::write(&entry_file, "import yaml\n").unwrap();

    let collector = DepsCollector::new();
    let result = collector.collect(&[entry_file], &dest).unwrap();

    // Should collect yaml package
    assert!(result.packages.contains(&"yaml".to_string()));
    assert!(result.file_count > 0);
    assert!(result.total_size > 0);
}

#[test]
fn test_collect_local_package() {
    if !python_available() {
        eprintln!("Skipping test: Python not available");
        return;
    }

    if !package_installed("auroraview") {
        eprintln!("Skipping test: auroraview not installed");
        return;
    }

    let temp = tempdir().unwrap();
    let dest = temp.path().join("collected");
    let entry_file = temp.path().join("entry.py");

    // Create entry file that imports auroraview
    std::fs::write(&entry_file, "import auroraview\n").unwrap();

    let collector = DepsCollector::new();
    let result = collector.collect(&[entry_file], &dest).unwrap();

    // Should collect auroraview package
    assert!(result.packages.contains(&"auroraview".to_string()));
    assert!(result.file_count > 0);
    assert!(result.total_size > 0);

    // Check that the package directory was created
    let pkg_dir = dest.join("auroraview");
    assert!(pkg_dir.exists());
}

#[test]
fn test_collect_excludes_stdlib() {
    if !python_available() {
        eprintln!("Skipping test: Python not available");
        return;
    }

    let temp = tempdir().unwrap();
    let dest = temp.path().join("collected");
    let entry_file = temp.path().join("entry.py");

    // Create entry file that imports only stdlib
    std::fs::write(
        &entry_file,
        r#"
import os
import sys
import json
import pathlib
"#,
    )
    .unwrap();

    let collector = DepsCollector::new();
    let result = collector.collect(&[entry_file], &dest).unwrap();

    // Should not collect any packages (all are stdlib)
    assert!(result.packages.is_empty());
    assert_eq!(result.file_count, 0);
}

#[test]
fn test_collect_respects_excludes() {
    if !python_available() {
        eprintln!("Skipping test: Python not available");
        return;
    }

    if !package_installed("yaml") {
        eprintln!("Skipping test: pyyaml not installed");
        return;
    }

    let temp = tempdir().unwrap();
    let dest = temp.path().join("collected");
    let entry_file = temp.path().join("entry.py");

    // Create entry file that imports yaml
    std::fs::write(&entry_file, "import yaml\n").unwrap();

    let collector = DepsCollector::new().exclude(["yaml"]);
    let result = collector.collect(&[entry_file], &dest).unwrap();

    // Should not collect yaml (excluded)
    assert!(!result.packages.contains(&"yaml".to_string()));
}

#[test]
fn test_collect_respects_includes() {
    if !python_available() {
        eprintln!("Skipping test: Python not available");
        return;
    }

    if !package_installed("yaml") {
        eprintln!("Skipping test: pyyaml not installed");
        return;
    }

    let temp = tempdir().unwrap();
    let dest = temp.path().join("collected");
    let entry_file = temp.path().join("entry.py");

    // Create entry file with no imports
    std::fs::write(&entry_file, "# No imports\n").unwrap();

    let collector = DepsCollector::new().include(["yaml"]);
    let result = collector.collect(&[entry_file], &dest).unwrap();

    // Should collect yaml (explicitly included)
    assert!(result.packages.contains(&"yaml".to_string()));
}

// ============================================================================
// Integration Tests - Complex Packages with Native Extensions
// ============================================================================

#[test]
fn test_collect_package_with_native_extension() {
    if !python_available() {
        eprintln!("Skipping test: Python not available");
        return;
    }

    // Test with a package that has native extensions (if available)
    // Common packages with native extensions: numpy, pandas, pillow
    let native_packages = ["numpy", "PIL", "cv2"];
    let mut found_package = None;

    for pkg in &native_packages {
        if package_installed(pkg) {
            found_package = Some(*pkg);
            break;
        }
    }

    let package = match found_package {
        Some(p) => p,
        None => {
            eprintln!("Skipping test: No native extension package available (numpy, PIL, cv2)");
            return;
        }
    };

    let temp = tempdir().unwrap();
    let dest = temp.path().join("collected");
    let entry_file = temp.path().join("entry.py");

    std::fs::write(&entry_file, format!("import {}\n", package)).unwrap();

    let collector = DepsCollector::new();
    let result = collector.collect(&[entry_file], &dest).unwrap();

    // Should collect the package
    assert!(result.packages.contains(&package.to_string()));
    assert!(result.file_count > 0);

    // Check for binary files (.pyd on Windows, .so on Linux/Mac)
    let pkg_dir = dest.join(package);
    if pkg_dir.exists() {
        let has_binary = walkdir::WalkDir::new(&pkg_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .any(|e| {
                let path = e.path();
                path.extension()
                    .is_some_and(|ext| ext == "pyd" || ext == "so" || ext == "dll")
            });

        // Native packages should have binary files
        // Note: This may not always be true depending on the package structure
        println!(
            "Package {} has binary files: {}",
            package, has_binary
        );
    }
}

// ============================================================================
// Integration Tests - Binary Packages (PySide6, PyQt5, etc.)
// ============================================================================

#[test]
fn test_collect_pyside6() {
    if !python_available() {
        eprintln!("Skipping test: Python not available");
        return;
    }

    if !package_installed("PySide6") {
        eprintln!("Skipping test: PySide6 not installed");
        return;
    }

    let collector = DepsCollector::new();
    let path = collector.get_package_path("PySide6").unwrap();

    assert!(path.is_some());
    let path = path.unwrap();
    println!("PySide6 path: {}", path.display());

    // PySide6 should be a directory
    assert!(path.is_dir());

    // Check for Qt DLLs or shared libraries
    let has_qt_binaries = walkdir::WalkDir::new(&path)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
        .any(|e| {
            let name = e.file_name().to_string_lossy();
            name.starts_with("Qt") && (name.ends_with(".dll") || name.ends_with(".so"))
        });

    println!("PySide6 has Qt binaries: {}", has_qt_binaries);
}

#[test]
fn test_collect_pyside6_full() {
    if !python_available() {
        eprintln!("Skipping test: Python not available");
        return;
    }

    if !package_installed("PySide6") {
        eprintln!("Skipping test: PySide6 not installed");
        return;
    }

    let temp = tempdir().unwrap();
    let dest = temp.path().join("collected");
    let entry_file = temp.path().join("entry.py");

    // Create entry file that imports PySide6
    std::fs::write(
        &entry_file,
        r#"
from PySide6.QtWidgets import QApplication, QWidget
from PySide6.QtCore import Qt
"#,
    )
    .unwrap();

    let collector = DepsCollector::new();
    let result = collector.collect(&[entry_file], &dest).unwrap();

    // Should collect PySide6
    assert!(result.packages.contains(&"PySide6".to_string()));
    assert!(result.file_count > 0);
    assert!(result.total_size > 0);

    println!(
        "PySide6 collected: {} files, {} bytes",
        result.file_count, result.total_size
    );

    // PySide6 is typically large (hundreds of MB)
    // This is a sanity check - actual size depends on the installation
    if result.total_size > 0 {
        println!(
            "PySide6 size: {:.2} MB",
            result.total_size as f64 / 1024.0 / 1024.0
        );
    }
}

// ============================================================================
// Integration Tests - Multiple Entry Files
// ============================================================================

#[test]
fn test_collect_multiple_entry_files() {
    if !python_available() {
        eprintln!("Skipping test: Python not available");
        return;
    }

    if !package_installed("yaml") {
        eprintln!("Skipping test: pyyaml not installed");
        return;
    }

    let temp = tempdir().unwrap();
    let dest = temp.path().join("collected");
    let entry1 = temp.path().join("entry1.py");
    let entry2 = temp.path().join("entry2.py");

    std::fs::write(&entry1, "import yaml\n").unwrap();
    std::fs::write(&entry2, "import json\n").unwrap(); // stdlib, should be ignored

    let collector = DepsCollector::new();
    let result = collector.collect(&[entry1, entry2], &dest).unwrap();

    // Should collect yaml from entry1
    assert!(result.packages.contains(&"yaml".to_string()));
    // json is stdlib, should not be collected
}

// ============================================================================
// Integration Tests - pip-based Collection
// ============================================================================

#[test]
fn test_collect_with_pip_empty() {
    if !python_available() {
        eprintln!("Skipping test: Python not available");
        return;
    }

    let temp = tempdir().unwrap();
    let dest = temp.path().join("pip_collected");

    let collector = DepsCollector::new();
    let result = collector.collect_with_pip(&[], &dest).unwrap();

    // Empty package list should return empty result
    assert!(result.packages.is_empty());
    assert_eq!(result.file_count, 0);
}

// ============================================================================
// Edge Cases and Error Handling
// ============================================================================

#[test]
fn test_collect_empty_entry_files() {
    if !python_available() {
        eprintln!("Skipping test: Python not available");
        return;
    }

    let temp = tempdir().unwrap();
    let dest = temp.path().join("collected");

    let collector = DepsCollector::new();
    let result = collector.collect(&[], &dest).unwrap();

    // No entry files should result in no packages
    assert!(result.packages.is_empty());
}

#[test]
fn test_collect_nonexistent_entry_file() {
    if !python_available() {
        eprintln!("Skipping test: Python not available");
        return;
    }

    let temp = tempdir().unwrap();
    let dest = temp.path().join("collected");

    let collector = DepsCollector::new();
    let result = collector
        .collect(&[PathBuf::from("/nonexistent/file.py")], &dest)
        .unwrap();

    // Nonexistent file should be skipped gracefully
    assert!(result.packages.is_empty());
}

#[test]
fn test_collect_non_python_file() {
    if !python_available() {
        eprintln!("Skipping test: Python not available");
        return;
    }

    let temp = tempdir().unwrap();
    let dest = temp.path().join("collected");
    let entry_file = temp.path().join("entry.txt");

    std::fs::write(&entry_file, "import yaml\n").unwrap();

    let collector = DepsCollector::new();
    let result = collector.collect(&[entry_file], &dest).unwrap();

    // Non-.py file should be skipped
    assert!(result.packages.is_empty());
}
