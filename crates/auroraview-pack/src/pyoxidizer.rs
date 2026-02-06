//! PyOxidizer integration for Python embedding
//!
//! This module provides integration with PyOxidizer for creating standalone
//! executables with embedded Python runtime.
//!
//! We use the AuroraView-maintained fork of PyOxidizer:
//! https://github.com/loonghao/PyOxidizer

use crate::error::{PackError, PackResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// PyOxidizer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PyOxidizerConfig {
    /// Path to PyOxidizer executable (or "pyoxidizer" to use from PATH)
    #[serde(default = "default_pyoxidizer_path")]
    pub executable: String,

    /// Python version to embed (e.g., "3.11")
    #[serde(default = "default_python_version")]
    pub python_version: String,

    /// Target triple (e.g., "x86_64-pc-windows-msvc")
    #[serde(default)]
    pub target: Option<String>,

    /// Build in release mode
    #[serde(default = "default_true")]
    pub release: bool,

    /// Python distribution flavor
    #[serde(default)]
    pub distribution_flavor: DistributionFlavor,

    /// Bytecode optimization level (0, 1, or 2)
    #[serde(default = "default_optimize")]
    pub optimize: u8,

    /// Include pip in the bundle
    #[serde(default)]
    pub include_pip: bool,

    /// Include setuptools in the bundle
    #[serde(default)]
    pub include_setuptools: bool,

    /// Filesystem importer fallback
    #[serde(default)]
    pub filesystem_importer: bool,

    /// Additional PyOxidizer config options
    #[serde(default)]
    pub extra_config: HashMap<String, String>,
}

fn default_pyoxidizer_path() -> String {
    "pyoxidizer".to_string()
}

fn default_python_version() -> String {
    "3.10".to_string()
}

fn default_optimize() -> u8 {
    1
}

fn default_true() -> bool {
    true
}

impl Default for PyOxidizerConfig {
    fn default() -> Self {
        Self {
            executable: default_pyoxidizer_path(),
            python_version: default_python_version(),
            target: None,
            release: true,
            distribution_flavor: DistributionFlavor::default(),
            optimize: default_optimize(),
            include_pip: false,
            include_setuptools: false,
            filesystem_importer: false,
            extra_config: HashMap::new(),
        }
    }
}

/// Python distribution flavor for PyOxidizer
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DistributionFlavor {
    /// Standalone static distribution (recommended for single-file exe)
    #[default]
    Standalone,
    /// Standalone dynamic distribution
    StandaloneDynamic,
    /// Use system Python
    System,
}

impl DistributionFlavor {
    /// Get the PyOxidizer flavor string
    pub fn as_str(&self) -> &'static str {
        match self {
            DistributionFlavor::Standalone => "standalone",
            DistributionFlavor::StandaloneDynamic => "standalone_dynamic",
            DistributionFlavor::System => "system",
        }
    }
}

/// External binary configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalBinary {
    /// Source path to the binary
    pub source: PathBuf,

    /// Destination path in the bundle (relative to app root)
    #[serde(default)]
    pub dest: Option<String>,

    /// Whether to make the binary executable (Unix only)
    #[serde(default = "default_true")]
    pub executable: bool,
}

/// Resource file configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceFile {
    /// Source path (file or directory)
    pub source: PathBuf,

    /// Destination path in the bundle (relative to resources root)
    #[serde(default)]
    pub dest: Option<String>,

    /// Glob pattern for filtering files (if source is directory)
    #[serde(default)]
    pub pattern: Option<String>,

    /// Exclude patterns
    #[serde(default)]
    pub exclude: Vec<String>,
}

/// PyOxidizer build context
pub struct PyOxidizerBuilder {
    config: PyOxidizerConfig,
    work_dir: PathBuf,
    app_name: String,
    entry_point: String,
    python_paths: Vec<PathBuf>,
    packages: Vec<String>,
    external_binaries: Vec<ExternalBinary>,
    resources: Vec<ResourceFile>,
    env_vars: HashMap<String, String>,
}

impl PyOxidizerBuilder {
    /// Create a new PyOxidizer builder
    pub fn new(
        config: PyOxidizerConfig,
        work_dir: impl Into<PathBuf>,
        app_name: impl Into<String>,
    ) -> Self {
        Self {
            config,
            work_dir: work_dir.into(),
            app_name: app_name.into(),
            entry_point: String::new(),
            python_paths: Vec::new(),
            packages: Vec::new(),
            external_binaries: Vec::new(),
            resources: Vec::new(),
            env_vars: HashMap::new(),
        }
    }

    /// Set the Python entry point (e.g., "myapp.main:run")
    pub fn entry_point(mut self, entry_point: impl Into<String>) -> Self {
        self.entry_point = entry_point.into();
        self
    }

    /// Add Python source paths
    pub fn python_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.python_paths = paths;
        self
    }

    /// Add pip packages to install
    pub fn packages(mut self, packages: Vec<String>) -> Self {
        self.packages = packages;
        self
    }

    /// Add external binaries
    pub fn external_binaries(mut self, binaries: Vec<ExternalBinary>) -> Self {
        self.external_binaries = binaries;
        self
    }

    /// Add resource files
    pub fn resources(mut self, resources: Vec<ResourceFile>) -> Self {
        self.resources = resources;
        self
    }

    /// Set environment variables
    pub fn env_vars(mut self, env: HashMap<String, String>) -> Self {
        self.env_vars = env;
        self
    }

    /// Check if PyOxidizer is available
    pub fn check_available(&self) -> PackResult<String> {
        let output = Command::new(&self.config.executable)
            .arg("--version")
            .output()
            .map_err(|e| {
                PackError::Build(format!(
                    "PyOxidizer not found at '{}': {}. \
                    Install from https://github.com/loonghao/PyOxidizer",
                    self.config.executable, e
                ))
            })?;

        if !output.status.success() {
            return Err(PackError::Build(
                "PyOxidizer version check failed".to_string(),
            ));
        }

        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(version)
    }

    /// Generate the pyoxidizer.bzl configuration file
    pub fn generate_config(&self) -> PackResult<String> {
        let mut config = String::new();

        // Header
        config.push_str("# Generated by AuroraView Pack\n");
        config.push_str("# https://github.com/loonghao/PyOxidizer\n\n");

        // Python distribution
        config.push_str(&format!(
            r#"def make_dist():
    return default_python_distribution(
        flavor = "{}",
        python_version = "{}",
    )

"#,
            self.config.distribution_flavor.as_str(),
            self.config.python_version
        ));

        // Main function
        config.push_str("def make_exe(dist):\n");
        config.push_str("    policy = dist.make_python_packaging_policy()\n");

        // Optimization level
        config.push_str(&format!(
            "    policy.bytecode_optimize_level_one = {}\n",
            self.config.optimize >= 1
        ));
        config.push_str(&format!(
            "    policy.bytecode_optimize_level_two = {}\n",
            self.config.optimize >= 2
        ));

        // Filesystem importer
        if self.config.filesystem_importer {
            config.push_str("    policy.allow_files = True\n");
        }

        config.push('\n');

        // Create executable
        config.push_str(&format!(
            r#"    exe = dist.to_python_executable(
        name = "{}",
        packaging_policy = policy,
        config = PythonInterpreterConfig(
            run_module = "{}",
        ),
    )

"#,
            self.app_name,
            self.get_run_module()
        ));

        // Add Python source paths
        for path in &self.python_paths {
            let path_str = path.to_string_lossy().replace('\\', "/");
            config.push_str(&format!(
                r#"    exe.add_python_resources(exe.read_package_root(
        path = "{}",
        packages = [""],
    ))

"#,
                path_str
            ));
        }

        // Add pip packages
        if !self.packages.is_empty() {
            config.push_str("    exe.add_python_resources(exe.pip_install([\n");
            for pkg in &self.packages {
                config.push_str(&format!("        \"{}\",\n", pkg));
            }
            config.push_str("    ]))\n\n");
        }

        // Include pip if requested
        if self.config.include_pip {
            config.push_str("    exe.add_python_resources(exe.pip_install([\"pip\"]))\n");
        }
        if self.config.include_setuptools {
            config.push_str("    exe.add_python_resources(exe.pip_install([\"setuptools\"]))\n");
        }

        config.push_str("    return exe\n\n");

        // Install function
        config.push_str("def make_install(exe):\n");
        config.push_str("    files = FileManifest()\n");
        config.push_str("    files.add_python_resource(\".\", exe)\n\n");

        // Add external binaries
        for binary in &self.external_binaries {
            let src = binary.source.to_string_lossy().replace('\\', "/");
            let dest = binary.dest.clone().unwrap_or_else(|| {
                binary
                    .source
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            });
            config.push_str(&format!(
                "    files.add_file(FileContent(path = \"{}\"), dest = \"{}\")\n",
                src, dest
            ));
        }

        // Add resources
        for resource in &self.resources {
            let src = resource.source.to_string_lossy().replace('\\', "/");
            let dest = resource
                .dest
                .clone()
                .unwrap_or_else(|| "resources".to_string());

            if resource.source.is_dir() {
                config.push_str(&format!(
                    "    files.add_manifest(glob(include = [\"{}/**/*\"], strip_prefix = \"{}\"))\n",
                    src, src
                ));
            } else {
                config.push_str(&format!(
                    "    files.add_file(FileContent(path = \"{}\"), dest = \"{}\")\n",
                    src, dest
                ));
            }
        }

        config.push_str("\n    return files\n\n");

        // Register targets
        config.push_str(
            r#"
register_target("dist", make_dist)
register_target("exe", make_exe, depends = ["dist"])
register_target("install", make_install, depends = ["exe"], default = True)

resolve_targets()
"#,
        );

        Ok(config)
    }

    /// Get the run module from entry point
    fn get_run_module(&self) -> String {
        // Convert "myapp.main:run" to "myapp.main"
        self.entry_point
            .split(':')
            .next()
            .unwrap_or(&self.entry_point)
            .to_string()
    }

    /// Build the PyOxidizer project
    pub fn build(&self, output_dir: &Path) -> PackResult<PathBuf> {
        // Check PyOxidizer is available
        let version = self.check_available()?;
        tracing::info!("Using PyOxidizer: {}", version);

        // Create work directory
        std::fs::create_dir_all(&self.work_dir)?;

        // Generate config file
        let config_content = self.generate_config()?;
        let config_path = self.work_dir.join("pyoxidizer.bzl");
        std::fs::write(&config_path, &config_content)?;
        tracing::debug!("Generated PyOxidizer config: {}", config_path.display());

        // Run PyOxidizer build
        let mut cmd = Command::new(&self.config.executable);
        cmd.arg("build");

        if self.config.release {
            cmd.arg("--release");
        }

        if let Some(ref target) = self.config.target {
            cmd.args(["--target", target]);
        }

        cmd.current_dir(&self.work_dir);

        // Set environment variables
        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }

        tracing::info!("Running PyOxidizer build...");
        let status = cmd
            .status()
            .map_err(|e| PackError::Build(format!("Failed to run PyOxidizer: {}", e)))?;

        if !status.success() {
            return Err(PackError::Build(format!(
                "PyOxidizer build failed with status: {}",
                status
            )));
        }

        // Find the built executable
        let build_dir = self.work_dir.join("build");
        let exe_name = self.get_exe_name();

        // Look for the executable in common locations
        let possible_paths = [
            build_dir.join("install").join(&exe_name),
            build_dir.join(&exe_name),
            build_dir
                .join("x86_64-pc-windows-msvc")
                .join("release")
                .join("install")
                .join(&exe_name),
            build_dir
                .join("x86_64-unknown-linux-gnu")
                .join("release")
                .join("install")
                .join(&exe_name),
            build_dir
                .join("x86_64-apple-darwin")
                .join("release")
                .join("install")
                .join(&exe_name),
        ];

        let built_exe = possible_paths.iter().find(|p| p.exists()).ok_or_else(|| {
            PackError::Build(format!(
                "Built executable not found. Searched: {:?}",
                possible_paths
            ))
        })?;

        // Copy to output directory
        std::fs::create_dir_all(output_dir)?;
        let output_exe = output_dir.join(&exe_name);
        std::fs::copy(built_exe, &output_exe)?;

        tracing::info!("PyOxidizer build complete: {}", output_exe.display());

        Ok(output_exe)
    }

    /// Get the executable name for the current platform
    fn get_exe_name(&self) -> String {
        #[cfg(target_os = "windows")]
        {
            format!("{}.exe", self.app_name)
        }
        #[cfg(not(target_os = "windows"))]
        {
            self.app_name.clone()
        }
    }
}

/// Check if PyOxidizer is installed and available
pub fn check_pyoxidizer() -> PackResult<String> {
    let builder =
        PyOxidizerBuilder::new(PyOxidizerConfig::default(), std::env::temp_dir(), "check");
    builder.check_available()
}

/// Get the recommended PyOxidizer installation instructions
pub fn installation_instructions() -> &'static str {
    r#"
PyOxidizer Installation Instructions
=====================================

AuroraView uses a maintained fork of PyOxidizer for Python embedding.

Option 1: Install from source (recommended)
-------------------------------------------
git clone https://github.com/loonghao/PyOxidizer.git
cd PyOxidizer
git checkout auroraview-maintained
cargo install --path pyoxidizer

Option 2: Download pre-built binary
-----------------------------------
Visit: https://github.com/loonghao/PyOxidizer/releases
Download the appropriate binary for your platform.

Verify installation:
-------------------
pyoxidizer --version
"#
}
