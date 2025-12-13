//! Main packer implementation

use crate::bundle::BundleBuilder;
use crate::config::{BundleStrategy, PythonBundleConfig};
use crate::deps_collector::DepsCollector;
use crate::overlay::{OverlayData, OverlayWriter};
use crate::python_standalone::{PythonRuntimeMeta, PythonStandalone, PythonStandaloneConfig};
use crate::{Manifest, PackConfig, PackError, PackMode, PackResult};
use std::fs;
use std::path::{Path, PathBuf};

/// Result of a pack operation
#[derive(Debug)]
pub struct PackOutput {
    /// Path to the generated executable or directory
    pub executable: PathBuf,
    /// Size of the executable in bytes
    pub size: u64,
    /// Number of embedded assets (for frontend mode)
    pub asset_count: usize,
    /// Number of Python files bundled (for fullstack mode)
    pub python_file_count: usize,
    /// Pack mode used
    pub mode: String,
}

/// Main packer for creating standalone executables
pub struct Packer {
    config: PackConfig,
}

impl Packer {
    /// Create a new packer with configuration
    pub fn new(config: PackConfig) -> Self {
        Self { config }
    }

    /// Create a packer from a manifest file
    pub fn from_manifest(manifest: &Manifest, base_dir: &Path) -> PackResult<Self> {
        let config = PackConfig::from_manifest(manifest, base_dir)?;
        Ok(Self::new(config))
    }

    /// Generate a pack project directory (for backward compatibility)
    ///
    /// This is an alias for `pack()` that returns the output path.
    pub fn generate(&self) -> PackResult<PathBuf> {
        let output = self.pack()?;
        Ok(output.executable)
    }

    /// Pack the application into a standalone executable
    ///
    /// This copies the current auroraview executable and appends
    /// configuration and assets as overlay data.
    pub fn pack(&self) -> PackResult<PackOutput> {
        // Validate configuration
        self.validate()?;

        // Ensure output directory exists
        fs::create_dir_all(&self.config.output_dir)?;

        match &self.config.mode {
            PackMode::Url { .. } | PackMode::Frontend { .. } => self.pack_simple(),
            PackMode::FullStack {
                frontend_path,
                python,
            } => self.pack_fullstack(frontend_path, python),
        }
    }

    /// Pack URL or Frontend mode (simple overlay approach)
    fn pack_simple(&self) -> PackResult<PackOutput> {
        // Determine output path
        let exe_name = self.get_exe_name();
        let output_path = self.config.output_dir.join(&exe_name);

        tracing::info!("Packing to: {}", output_path.display());

        // Get the current executable
        let current_exe = std::env::current_exe()?;

        // Copy executable to output
        fs::copy(&current_exe, &output_path)?;

        // Create overlay data
        let mut overlay = OverlayData::new(self.config.clone());

        // Bundle assets if in frontend mode
        let asset_count = if let PackMode::Frontend { ref path } = self.config.mode {
            let bundle = BundleBuilder::new(path).build()?;
            let count = bundle.len();

            for (path, content) in bundle.into_assets() {
                overlay.add_asset(path, content);
            }

            count
        } else {
            0
        };

        // Write overlay to executable
        OverlayWriter::write(&output_path, &overlay)?;

        // Get final size
        let size = fs::metadata(&output_path)?.len();

        tracing::info!(
            "Pack complete: {} ({:.2} MB)",
            output_path.display(),
            size as f64 / (1024.0 * 1024.0)
        );

        Ok(PackOutput {
            executable: output_path,
            size,
            asset_count,
            python_file_count: 0,
            mode: self.config.mode.name().to_string(),
        })
    }

    /// Pack FullStack mode (frontend + Python backend)
    fn pack_fullstack(
        &self,
        frontend_path: &Path,
        python: &PythonBundleConfig,
    ) -> PackResult<PackOutput> {
        match python.strategy {
            BundleStrategy::Standalone => self.pack_fullstack_standalone(frontend_path, python),
            BundleStrategy::PyOxidizer => self.pack_fullstack_pyoxidizer(frontend_path, python),
            BundleStrategy::Embedded => self.pack_fullstack_embedded(frontend_path, python),
            BundleStrategy::Portable => self.pack_fullstack_portable(frontend_path, python),
            BundleStrategy::System => self.pack_fullstack_system(frontend_path, python),
        }
    }

    /// Pack FullStack with standalone Python runtime (default)
    ///
    /// This creates a single executable with:
    /// - Embedded Python runtime (from python-build-standalone)
    /// - All Python code and dependencies
    /// - Frontend assets
    ///
    /// At runtime, the Python distribution is extracted to a cache directory
    /// on first run and reused thereafter. This provides:
    /// - Single-file distribution
    /// - Fully offline operation
    /// - No system Python required
    fn pack_fullstack_standalone(
        &self,
        frontend_path: &Path,
        python: &PythonBundleConfig,
    ) -> PackResult<PackOutput> {
        let exe_name = self.get_exe_name();
        let output_path = self.config.output_dir.join(&exe_name);

        tracing::info!(
            "Packing fullstack (standalone) to: {}",
            output_path.display()
        );

        // Download Python distribution
        let standalone_config = PythonStandaloneConfig {
            version: python.version.clone(),
            release: None, // Use latest
            target: None,  // Auto-detect
            cache_dir: None,
        };

        let standalone = PythonStandalone::new(standalone_config)?;
        tracing::info!(
            "Downloading Python {} for {}...",
            standalone.version(),
            standalone.target().triple()
        );

        let python_archive = standalone.get_distribution_bytes()?;
        let python_meta = PythonRuntimeMeta {
            version: python.version.clone(),
            target: standalone.target().triple().to_string(),
            archive_size: python_archive.len() as u64,
        };

        tracing::info!(
            "Python distribution size: {:.2} MB",
            python_archive.len() as f64 / (1024.0 * 1024.0)
        );

        // Get the current executable
        let current_exe = std::env::current_exe()?;
        fs::copy(&current_exe, &output_path)?;

        // Create overlay data
        let mut overlay = OverlayData::new(self.config.clone());

        // Add Python runtime metadata
        let meta_json = serde_json::to_vec(&python_meta)?;
        overlay.add_asset("python_runtime.json".to_string(), meta_json);

        // Add Python distribution archive
        overlay.add_asset("python_runtime.tar.gz".to_string(), python_archive);

        // Bundle frontend assets
        let frontend_bundle = BundleBuilder::new(frontend_path).build()?;
        let asset_count = frontend_bundle.len();
        for (path, content) in frontend_bundle.into_assets() {
            overlay.add_asset(format!("frontend/{}", path), content);
        }

        // Bundle Python code
        let python_file_count = self.bundle_python_code(&mut overlay, python)?;

        // Collect additional resources from hooks
        let resource_count = self.collect_hook_resources(&mut overlay)?;
        if resource_count > 0 {
            tracing::info!("Collected {} resource files from hooks", resource_count);
        }

        // Write overlay to executable
        OverlayWriter::write(&output_path, &overlay)?;

        let size = fs::metadata(&output_path)?.len();

        tracing::info!(
            "Pack complete: {} ({:.2} MB, {} assets, {} python files, {} resources)",
            output_path.display(),
            size as f64 / (1024.0 * 1024.0),
            asset_count,
            python_file_count,
            resource_count
        );

        Ok(PackOutput {
            executable: output_path,
            size,
            asset_count,
            python_file_count,
            mode: "fullstack-standalone".to_string(),
        })
    }

    /// Pack FullStack with PyOxidizer (single-file executable with embedded Python)
    ///
    /// This uses PyOxidizer to create a standalone executable with:
    /// - Embedded Python interpreter
    /// - All Python dependencies
    /// - Frontend assets
    /// - External binaries and resources
    fn pack_fullstack_pyoxidizer(
        &self,
        frontend_path: &Path,
        python: &PythonBundleConfig,
    ) -> PackResult<PackOutput> {
        use crate::pyoxidizer::{
            DistributionFlavor, ExternalBinary, PyOxidizerBuilder, PyOxidizerConfig, ResourceFile,
        };

        tracing::info!("Packing fullstack with PyOxidizer...");

        // Create work directory
        let work_dir = self.config.output_dir.join(".pyoxidizer-build");
        fs::create_dir_all(&work_dir)?;

        // Configure PyOxidizer
        let mut pyox_config = PyOxidizerConfig {
            python_version: python.version.clone(),
            optimize: python.optimize,
            include_pip: python.include_pip,
            include_setuptools: python.include_setuptools,
            ..Default::default()
        };

        // Set custom PyOxidizer path if specified
        if let Some(ref path) = python.pyoxidizer_path {
            pyox_config.executable = path.to_string_lossy().to_string();
        }

        // Set distribution flavor
        if let Some(ref flavor) = python.distribution_flavor {
            pyox_config.distribution_flavor = match flavor.as_str() {
                "standalone" => DistributionFlavor::Standalone,
                "standalone_dynamic" => DistributionFlavor::StandaloneDynamic,
                "system" => DistributionFlavor::System,
                _ => DistributionFlavor::Standalone,
            };
        }

        // Build external binaries list
        let external_binaries: Vec<ExternalBinary> = python
            .external_bin
            .iter()
            .map(|path| ExternalBinary {
                source: path.clone(),
                dest: None,
                executable: true,
            })
            .collect();

        // Build resources list (including frontend)
        let mut resources: Vec<ResourceFile> = vec![ResourceFile {
            source: frontend_path.to_path_buf(),
            dest: Some("frontend".to_string()),
            pattern: None,
            exclude: Vec::new(),
        }];

        // Add additional resources from config
        for res_path in &python.resources {
            resources.push(ResourceFile {
                source: res_path.clone(),
                dest: None,
                pattern: None,
                exclude: Vec::new(),
            });
        }

        // Read packages from requirements.txt if specified
        let mut packages = python.packages.clone();
        if let Some(ref req_path) = python.requirements {
            if req_path.exists() {
                let content = fs::read_to_string(req_path)?;
                for line in content.lines() {
                    let line = line.trim();
                    if !line.is_empty() && !line.starts_with('#') {
                        packages.push(line.to_string());
                    }
                }
            }
        }

        // Create builder
        let builder = PyOxidizerBuilder::new(pyox_config, &work_dir, &self.config.output_name)
            .entry_point(&python.entry_point)
            .python_paths(python.include_paths.clone())
            .packages(packages)
            .external_binaries(external_binaries)
            .resources(resources)
            .env_vars(self.config.env.clone());

        // Build with PyOxidizer
        let output_exe = builder.build(&self.config.output_dir)?;

        // Get frontend asset count for reporting
        let frontend_bundle = BundleBuilder::new(frontend_path).build()?;
        let asset_count = frontend_bundle.len();

        // Count Python files
        let mut python_file_count = 0;
        for include_path in &python.include_paths {
            if include_path.is_file() {
                python_file_count += 1;
            } else if include_path.is_dir() {
                python_file_count += walkdir::WalkDir::new(include_path)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().is_some_and(|ext| ext == "py"))
                    .count();
            }
        }

        let size = fs::metadata(&output_exe)?.len();

        tracing::info!(
            "PyOxidizer pack complete: {} ({:.2} MB)",
            output_exe.display(),
            size as f64 / (1024.0 * 1024.0)
        );

        // Cleanup work directory (optional, keep for debugging)
        if !self.config.debug {
            let _ = fs::remove_dir_all(&work_dir);
        }

        Ok(PackOutput {
            executable: output_exe,
            size,
            asset_count,
            python_file_count,
            mode: "fullstack-pyoxidizer".to_string(),
        })
    }

    /// Pack FullStack with embedded Python (overlay approach)
    ///
    /// This bundles everything into a single executable using the overlay format.
    /// Python code is stored as assets and executed via embedded Python interpreter.
    fn pack_fullstack_embedded(
        &self,
        frontend_path: &Path,
        python: &PythonBundleConfig,
    ) -> PackResult<PackOutput> {
        let exe_name = self.get_exe_name();
        let output_path = self.config.output_dir.join(&exe_name);

        tracing::info!("Packing fullstack (embedded) to: {}", output_path.display());

        // Get the current executable
        let current_exe = std::env::current_exe()?;
        fs::copy(&current_exe, &output_path)?;

        // Create overlay data
        let mut overlay = OverlayData::new(self.config.clone());

        // Bundle frontend assets
        let frontend_bundle = BundleBuilder::new(frontend_path).build()?;
        let asset_count = frontend_bundle.len();
        for (path, content) in frontend_bundle.into_assets() {
            overlay.add_asset(format!("frontend/{}", path), content);
        }

        // Bundle Python code
        let python_file_count = self.bundle_python_code(&mut overlay, python)?;

        // Write overlay to executable
        OverlayWriter::write(&output_path, &overlay)?;

        let size = fs::metadata(&output_path)?.len();

        tracing::info!(
            "Pack complete: {} ({:.2} MB, {} assets, {} python files)",
            output_path.display(),
            size as f64 / (1024.0 * 1024.0),
            asset_count,
            python_file_count
        );

        Ok(PackOutput {
            executable: output_path,
            size,
            asset_count,
            python_file_count,
            mode: "fullstack-embedded".to_string(),
        })
    }

    /// Pack FullStack with portable Python runtime
    ///
    /// This creates a directory structure with:
    /// - app.exe (the launcher)
    /// - python/ (embedded Python runtime)
    /// - lib/ (Python packages)
    /// - frontend/ (web assets)
    /// - backend/ (Python source code)
    fn pack_fullstack_portable(
        &self,
        frontend_path: &Path,
        python: &PythonBundleConfig,
    ) -> PackResult<PackOutput> {
        let output_dir = self.config.output_dir.join(&self.config.output_name);
        fs::create_dir_all(&output_dir)?;

        tracing::info!("Packing fullstack (portable) to: {}", output_dir.display());

        // Copy launcher executable
        let exe_name = self.get_exe_name();
        let exe_path = output_dir.join(&exe_name);
        let current_exe = std::env::current_exe()?;
        fs::copy(&current_exe, &exe_path)?;

        // Create overlay for launcher config
        let overlay = OverlayData::new(self.config.clone());
        OverlayWriter::write(&exe_path, &overlay)?;

        // Copy frontend assets
        let frontend_dir = output_dir.join("frontend");
        fs::create_dir_all(&frontend_dir)?;
        let frontend_bundle = BundleBuilder::new(frontend_path).build()?;
        let asset_count = frontend_bundle.len();
        for (path, content) in frontend_bundle.into_assets() {
            let dest = frontend_dir.join(&path);
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&dest, content)?;
        }

        // Copy Python backend code
        let backend_dir = output_dir.join("backend");
        fs::create_dir_all(&backend_dir)?;
        let python_file_count = self.copy_python_code(&backend_dir, python)?;

        // Install Python packages
        let lib_dir = output_dir.join("lib");
        fs::create_dir_all(&lib_dir)?;
        self.install_python_packages(&lib_dir, python)?;

        // Calculate total size
        let size = calculate_dir_size(&output_dir)?;

        tracing::info!(
            "Pack complete: {} ({:.2} MB, {} assets, {} python files)",
            output_dir.display(),
            size as f64 / (1024.0 * 1024.0),
            asset_count,
            python_file_count
        );

        Ok(PackOutput {
            executable: exe_path,
            size,
            asset_count,
            python_file_count,
            mode: "fullstack-portable".to_string(),
        })
    }

    /// Pack FullStack with system Python
    ///
    /// This creates a minimal package that relies on system Python.
    fn pack_fullstack_system(
        &self,
        frontend_path: &Path,
        python: &PythonBundleConfig,
    ) -> PackResult<PackOutput> {
        let output_dir = self.config.output_dir.join(&self.config.output_name);
        fs::create_dir_all(&output_dir)?;

        tracing::info!("Packing fullstack (system) to: {}", output_dir.display());

        // Copy launcher executable
        let exe_name = self.get_exe_name();
        let exe_path = output_dir.join(&exe_name);
        let current_exe = std::env::current_exe()?;
        fs::copy(&current_exe, &exe_path)?;

        // Create overlay for launcher config
        let overlay = OverlayData::new(self.config.clone());
        OverlayWriter::write(&exe_path, &overlay)?;

        // Copy frontend assets
        let frontend_dir = output_dir.join("frontend");
        fs::create_dir_all(&frontend_dir)?;
        let frontend_bundle = BundleBuilder::new(frontend_path).build()?;
        let asset_count = frontend_bundle.len();
        for (path, content) in frontend_bundle.into_assets() {
            let dest = frontend_dir.join(&path);
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&dest, content)?;
        }

        // Copy Python backend code
        let backend_dir = output_dir.join("backend");
        fs::create_dir_all(&backend_dir)?;
        let python_file_count = self.copy_python_code(&backend_dir, python)?;

        // Generate requirements.txt for user to install
        self.generate_requirements_file(&output_dir, python)?;

        let size = calculate_dir_size(&output_dir)?;

        tracing::info!(
            "Pack complete: {} ({:.2} MB, {} assets, {} python files)",
            output_dir.display(),
            size as f64 / (1024.0 * 1024.0),
            asset_count,
            python_file_count
        );

        Ok(PackOutput {
            executable: exe_path,
            size,
            asset_count,
            python_file_count,
            mode: "fullstack-system".to_string(),
        })
    }

    /// Bundle Python code into overlay
    fn bundle_python_code(
        &self,
        overlay: &mut OverlayData,
        python: &PythonBundleConfig,
    ) -> PackResult<usize> {
        let mut count = 0;
        let mut entry_files = Vec::new();

        for include_path in &python.include_paths {
            if include_path.is_file() {
                // Single file
                let content = fs::read(include_path)?;
                let name = include_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("main.py");
                overlay.add_asset(format!("python/{}", name), content);
                count += 1;
                entry_files.push(include_path.clone());
            } else if include_path.is_dir() {
                // Directory - walk and add all .py files
                for entry in walkdir::WalkDir::new(include_path)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().is_some_and(|ext| ext == "py"))
                {
                    // Skip excluded patterns
                    let rel_path = entry
                        .path()
                        .strip_prefix(include_path)
                        .unwrap_or(entry.path());

                    // Check if path matches any exclude pattern
                    let path_str = rel_path.to_string_lossy();
                    let should_exclude = python.exclude.iter().any(|pattern| {
                        // Simple glob matching
                        if pattern.contains('*') {
                            let pattern = pattern.replace("*", "");
                            path_str.contains(&pattern)
                        } else {
                            path_str.contains(pattern)
                        }
                    });

                    if should_exclude {
                        continue;
                    }

                    let content = fs::read(entry.path())?;
                    overlay.add_asset(
                        format!("python/{}", rel_path.to_string_lossy().replace('\\', "/")),
                        content,
                    );
                    count += 1;

                    // Track main entry files for dependency analysis
                    if rel_path.to_string_lossy() == "main.py"
                        || rel_path.to_string_lossy().ends_with("/main.py")
                    {
                        entry_files.push(entry.path().to_path_buf());
                    }
                }
            }
        }

        // Collect Python dependencies
        let deps_count = self.collect_python_deps(overlay, python, &entry_files)?;
        count += deps_count;

        Ok(count)
    }

    /// Collect Python dependencies and add to overlay
    fn collect_python_deps(
        &self,
        overlay: &mut OverlayData,
        python: &PythonBundleConfig,
        entry_files: &[PathBuf],
    ) -> PackResult<usize> {
        // Build list of packages to include
        let mut packages_to_collect: Vec<String> = python.packages.clone();

        // Always include auroraview if not explicitly excluded
        if !python.exclude.iter().any(|e| e == "auroraview") {
            packages_to_collect.push("auroraview".to_string());
        }

        // Read from requirements.txt if specified
        if let Some(ref req_path) = python.requirements {
            if req_path.exists() {
                let content = fs::read_to_string(req_path)?;
                for line in content.lines() {
                    let line = line.trim();
                    if !line.is_empty() && !line.starts_with('#') {
                        // Extract package name (before any version specifier)
                        let pkg_name = line
                            .split(['=', '>', '<', '!', '[', ';'])
                            .next()
                            .unwrap_or(line)
                            .trim();
                        if !pkg_name.is_empty() {
                            packages_to_collect.push(pkg_name.to_string());
                        }
                    }
                }
            }
        }

        if packages_to_collect.is_empty() && entry_files.is_empty() {
            return Ok(0);
        }

        tracing::info!(
            "Collecting Python dependencies: {:?}",
            packages_to_collect
        );

        // Create temp directory for collecting deps
        let temp_dir = std::env::temp_dir().join(format!(
            "auroraview-deps-{}",
            std::process::id()
        ));
        fs::create_dir_all(&temp_dir)?;

        // Use DepsCollector to collect packages
        let collector = DepsCollector::new()
            .include(packages_to_collect.iter().cloned())
            .exclude(python.exclude.iter().cloned());

        let collected = collector.collect(entry_files, &temp_dir)?;

        tracing::info!(
            "Collected {} packages ({} files, {:.2} MB)",
            collected.packages.len(),
            collected.file_count,
            collected.total_size as f64 / (1024.0 * 1024.0)
        );

        // Add collected files to overlay
        let mut count = 0;
        for entry in walkdir::WalkDir::new(&temp_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
        {
            let rel_path = entry
                .path()
                .strip_prefix(&temp_dir)
                .unwrap_or(entry.path());
            let content = fs::read(entry.path())?;
            overlay.add_asset(
                format!("python/{}", rel_path.to_string_lossy().replace('\\', "/")),
                content,
            );
            count += 1;
        }

        // Cleanup temp directory
        let _ = fs::remove_dir_all(&temp_dir);

        Ok(count)
    }

    /// Copy Python code to output directory
    fn copy_python_code(&self, dest_dir: &Path, python: &PythonBundleConfig) -> PackResult<usize> {
        let mut count = 0;

        for include_path in &python.include_paths {
            if include_path.is_file() {
                let name = include_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("main.py");
                fs::copy(include_path, dest_dir.join(name))?;
                count += 1;
            } else if include_path.is_dir() {
                for entry in walkdir::WalkDir::new(include_path)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().is_some_and(|ext| ext == "py"))
                {
                    let rel_path = entry
                        .path()
                        .strip_prefix(include_path)
                        .unwrap_or(entry.path());
                    let dest = dest_dir.join(rel_path);
                    if let Some(parent) = dest.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::copy(entry.path(), &dest)?;
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Install Python packages using pip
    fn install_python_packages(
        &self,
        lib_dir: &Path,
        python: &PythonBundleConfig,
    ) -> PackResult<()> {
        let mut packages = python.packages.clone();

        // Read from requirements.txt if specified
        if let Some(ref req_path) = python.requirements {
            if req_path.exists() {
                let content = fs::read_to_string(req_path)?;
                for line in content.lines() {
                    let line = line.trim();
                    if !line.is_empty() && !line.starts_with('#') {
                        packages.push(line.to_string());
                    }
                }
            }
        }

        if packages.is_empty() {
            return Ok(());
        }

        tracing::info!("Installing {} Python packages...", packages.len());

        // Use pip to install packages to lib_dir
        let status = std::process::Command::new("pip")
            .args([
                "install",
                "--target",
                lib_dir.to_str().unwrap_or("."),
                "--no-deps",
            ])
            .args(&packages)
            .status();

        match status {
            Ok(s) if s.success() => {
                tracing::info!("Python packages installed successfully");
                Ok(())
            }
            Ok(s) => {
                tracing::warn!("pip install exited with status: {}", s);
                Ok(()) // Continue even if pip fails
            }
            Err(e) => {
                tracing::warn!("Failed to run pip: {}", e);
                Ok(()) // Continue even if pip is not available
            }
        }
    }

    /// Collect additional resources from hooks configuration
    ///
    /// This processes the `hooks.collect` entries from the manifest,
    /// expanding glob patterns and adding matched files to the overlay.
    fn collect_hook_resources(&self, overlay: &mut OverlayData) -> PackResult<usize> {
        let hooks = match &self.config.hooks {
            Some(h) => h,
            None => return Ok(0),
        };

        let mut count = 0;

        for pattern in &hooks.collect_files {
            // Expand glob pattern
            let entries = glob::glob(&pattern.source).map_err(|e| {
                PackError::Config(format!("Invalid glob pattern '{}': {}", pattern.source, e))
            })?;

            for entry in entries {
                let path = entry
                    .map_err(|e| PackError::Config(format!("Failed to read glob entry: {}", e)))?;

                if !path.is_file() {
                    continue;
                }

                // Determine destination path
                let dest_path = if let Some(ref dest) = pattern.dest {
                    if pattern.preserve_structure {
                        // Preserve relative path structure under dest
                        let file_name = path.file_name().unwrap_or_default();
                        format!("{}/{}", dest, file_name.to_string_lossy())
                    } else {
                        // Just use filename under dest
                        let file_name = path.file_name().unwrap_or_default();
                        format!("{}/{}", dest, file_name.to_string_lossy())
                    }
                } else {
                    // Use original filename
                    path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                };

                // Read and add file
                let content = fs::read(&path)?;
                tracing::debug!("Collecting resource: {} -> {}", path.display(), dest_path);
                overlay.add_asset(dest_path, content);
                count += 1;
            }
        }

        Ok(count)
    }

    /// Generate requirements.txt file
    fn generate_requirements_file(
        &self,
        output_dir: &Path,
        python: &PythonBundleConfig,
    ) -> PackResult<()> {
        let mut packages = python.packages.clone();

        if let Some(ref req_path) = python.requirements {
            if req_path.exists() {
                let content = fs::read_to_string(req_path)?;
                for line in content.lines() {
                    let line = line.trim();
                    if !line.is_empty() && !line.starts_with('#') {
                        packages.push(line.to_string());
                    }
                }
            }
        }

        if !packages.is_empty() {
            let req_file = output_dir.join("requirements.txt");
            fs::write(&req_file, packages.join("\n"))?;
            tracing::info!(
                "Generated requirements.txt with {} packages",
                packages.len()
            );
        }

        Ok(())
    }

    /// Validate the configuration
    fn validate(&self) -> PackResult<()> {
        match &self.config.mode {
            PackMode::Url { url } => {
                if url.is_empty() {
                    return Err(PackError::InvalidUrl("URL cannot be empty".to_string()));
                }
            }
            PackMode::Frontend { path } => {
                if !path.exists() {
                    return Err(PackError::FrontendNotFound(path.clone()));
                }

                // Check for index.html
                let index_path = if path.is_dir() {
                    path.join("index.html")
                } else {
                    path.clone()
                };

                if !index_path.exists() {
                    return Err(PackError::FrontendNotFound(index_path));
                }
            }
            PackMode::FullStack {
                frontend_path,
                python,
            } => {
                // Validate frontend
                if !frontend_path.exists() {
                    return Err(PackError::FrontendNotFound(frontend_path.clone()));
                }

                let index_path = if frontend_path.is_dir() {
                    frontend_path.join("index.html")
                } else {
                    frontend_path.clone()
                };

                if !index_path.exists() {
                    return Err(PackError::FrontendNotFound(index_path));
                }

                // Validate Python entry point
                if python.entry_point.is_empty() {
                    return Err(PackError::Config(
                        "Python entry_point is required for fullstack mode".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Get the output executable name with platform extension
    fn get_exe_name(&self) -> String {
        #[cfg(target_os = "windows")]
        {
            format!("{}.exe", self.config.output_name)
        }
        #[cfg(not(target_os = "windows"))]
        {
            self.config.output_name.clone()
        }
    }
}

/// Calculate total size of a directory recursively
fn calculate_dir_size(path: &Path) -> PackResult<u64> {
    let mut total = 0;
    for entry in walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
    {
        total += entry.metadata().map(|m| m.len()).unwrap_or(0);
    }
    Ok(total)
}

impl PackConfig {
    /// Create PackConfig from a Manifest
    pub fn from_manifest(manifest: &Manifest, base_dir: &Path) -> PackResult<Self> {
        // Check if this is a fullstack configuration
        let is_fullstack = manifest.is_fullstack();

        let mode = if let Some(ref url) = manifest.app.url {
            PackMode::Url { url: url.clone() }
        } else if let Some(ref frontend_path) = manifest.app.frontend_path {
            // Resolve path relative to manifest location
            let resolved = if frontend_path.is_absolute() {
                frontend_path.clone()
            } else {
                base_dir.join(frontend_path)
            };

            if is_fullstack {
                // FullStack mode: frontend + Python
                let python_config = manifest.python.as_ref().ok_or_else(|| {
                    PackError::Config("Python config required for fullstack mode".to_string())
                })?;

                // Parse strategy from string
                let strategy = match python_config.strategy.as_str() {
                    "standalone" => BundleStrategy::Standalone,
                    "pyoxidizer" => BundleStrategy::PyOxidizer,
                    "embedded" => BundleStrategy::Embedded,
                    "portable" => BundleStrategy::Portable,
                    "system" => BundleStrategy::System,
                    _ => BundleStrategy::Standalone, // Default to Standalone
                };

                let python = PythonBundleConfig {
                    entry_point: python_config
                        .entry_point
                        .clone()
                        .unwrap_or_else(|| "main:run".to_string()),
                    include_paths: python_config
                        .include_paths
                        .iter()
                        .map(|p| {
                            if p.is_absolute() {
                                p.clone()
                            } else {
                                base_dir.join(p)
                            }
                        })
                        .collect(),
                    packages: python_config.packages.clone(),
                    requirements: python_config.requirements.as_ref().map(|p| {
                        if p.is_absolute() {
                            p.clone()
                        } else {
                            base_dir.join(p)
                        }
                    }),
                    strategy,
                    version: python_config.version.clone(),
                    optimize: python_config.optimize,
                    exclude: python_config.exclude.clone(),
                    external_bin: python_config
                        .external_bin
                        .iter()
                        .map(|p| {
                            if p.is_absolute() {
                                p.clone()
                            } else {
                                base_dir.join(p)
                            }
                        })
                        .collect(),
                    resources: python_config
                        .resources
                        .iter()
                        .map(|p| {
                            if p.is_absolute() {
                                p.clone()
                            } else {
                                base_dir.join(p)
                            }
                        })
                        .collect(),
                    include_pip: python_config.include_pip,
                    include_setuptools: python_config.include_setuptools,
                    distribution_flavor: python_config
                        .pyoxidizer
                        .as_ref()
                        .and_then(|p| p.flavor.clone()),
                    pyoxidizer_path: python_config
                        .pyoxidizer
                        .as_ref()
                        .and_then(|p| p.executable.clone()),
                };

                PackMode::FullStack {
                    frontend_path: resolved,
                    python: Box::new(python),
                }
            } else {
                PackMode::Frontend { path: resolved }
            }
        } else {
            return Err(PackError::Config(
                "Either 'url' or 'frontend_path' must be specified".to_string(),
            ));
        };

        let window = crate::WindowConfig {
            title: manifest.app.title.clone(),
            width: manifest.window.width,
            height: manifest.window.height,
            min_width: manifest.window.min_width,
            min_height: manifest.window.min_height,
            start_position: match &manifest.window.start_position {
                crate::StartPosition::Named(s) if s == "center" => {
                    crate::WindowStartPosition::Center
                }
                crate::StartPosition::Named(_) => crate::WindowStartPosition::Center,
                crate::StartPosition::Position { x, y } => {
                    crate::WindowStartPosition::Position { x: *x, y: *y }
                }
            },
            resizable: manifest.window.resizable,
            frameless: manifest.window.frameless,
            transparent: manifest.window.transparent,
            always_on_top: manifest.window.always_on_top,
        };

        let icon_path = manifest.get_icon_path().map(|p| {
            if p.is_absolute() {
                p.clone()
            } else {
                base_dir.join(p)
            }
        });

        // Build environment variables from runtime config
        let mut env = std::collections::HashMap::new();
        if let Some(ref runtime) = manifest.runtime {
            env.extend(runtime.env.clone());
        }
        // Also include Python env if present
        if let Some(ref python) = manifest.python {
            env.extend(python.env.clone());
        }

        // Build license config
        let license = manifest.license.as_ref().map(|l| crate::LicenseConfig {
            enabled: l.enabled,
            expires_at: l.expires_at.clone(),
            require_token: l.require_token,
            embedded_token: l.embedded_token.clone(),
            validation_url: l.validation_url.clone(),
            allowed_machines: l.allowed_machines.clone(),
            grace_period_days: l.grace_period_days,
            expiration_message: l.expiration_message.clone(),
        });

        // Build hooks config from manifest
        let hooks = manifest.hooks.as_ref().map(|h| crate::HooksConfig {
            before_collect: h.before_collect.clone(),
            collect_files: h
                .collect
                .iter()
                .map(|c| crate::CollectPattern {
                    source: if std::path::Path::new(&c.source).is_absolute() {
                        c.source.clone()
                    } else {
                        base_dir.join(&c.source).to_string_lossy().to_string()
                    },
                    dest: c.dest.clone(),
                    preserve_structure: c.preserve_structure,
                })
                .collect(),
            after_pack: h.after_pack.clone(),
        });

        Ok(Self {
            mode,
            output_name: manifest.package.name.clone(),
            output_dir: manifest
                .build
                .out_dir
                .clone()
                .unwrap_or_else(|| PathBuf::from(".")),
            window,
            target_platform: crate::TargetPlatform::Current,
            debug: manifest.debug.enabled,
            allow_new_window: manifest.app.allow_new_window,
            user_agent: manifest.app.user_agent.clone(),
            inject_js: manifest.inject.as_ref().and_then(|i| i.js_code.clone()),
            inject_css: manifest.inject.as_ref().and_then(|i| i.css_code.clone()),
            icon_path,
            env,
            license,
            hooks,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_packer_validate_url() {
        let config = PackConfig::url("https://example.com");
        let packer = Packer::new(config);
        assert!(packer.validate().is_ok());
    }

    #[test]
    fn test_packer_validate_empty_url() {
        let config = PackConfig::url("");
        let packer = Packer::new(config);
        assert!(packer.validate().is_err());
    }

    #[test]
    fn test_packer_validate_frontend() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("index.html"), "<html></html>").unwrap();

        let config = PackConfig::frontend(temp.path());
        let packer = Packer::new(config);
        assert!(packer.validate().is_ok());
    }

    #[test]
    fn test_packer_validate_frontend_missing() {
        let config = PackConfig::frontend("/nonexistent/path");
        let packer = Packer::new(config);
        assert!(packer.validate().is_err());
    }

    #[test]
    fn test_exe_name() {
        let config = PackConfig::url("example.com").with_output("my-app");
        let packer = Packer::new(config);

        #[cfg(target_os = "windows")]
        assert_eq!(packer.get_exe_name(), "my-app.exe");

        #[cfg(not(target_os = "windows"))]
        assert_eq!(packer.get_exe_name(), "my-app");
    }
}
