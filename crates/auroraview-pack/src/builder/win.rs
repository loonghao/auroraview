//! Windows Builder

use super::common::{BuildContext, BuildOutput, BuildResult};
use super::traits::{Builder, BuilderCapability};
use crate::bundle::BundleBuilder;
use crate::common::BundleStrategy;
use crate::config::PythonBundleConfig;
use crate::deps_collector::DepsCollector;
use crate::overlay::{OverlayData, OverlayWriter};
use crate::python_standalone::{PythonRuntimeMeta, PythonStandalone, PythonStandaloneConfig};
use crate::{PackConfig, PackError, PackMode};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Windows platform builder
pub struct WinBuilder {
    /// Use portable mode (no installer)
    portable: bool,
}

impl WinBuilder {
    pub fn new() -> Self {
        Self { portable: false }
    }

    pub fn portable(mut self, value: bool) -> Self {
        self.portable = value;
        self
    }

    fn get_exe_name(ctx: &BuildContext) -> String {
        let name = ctx
            .config
            .target
            .output_name
            .clone()
            .unwrap_or_else(|| ctx.config.app.name.clone());
        // Only append .exe on Windows; on macOS/Linux executables have no extension
        if cfg!(target_os = "windows") {
            if name.ends_with(".exe") {
                name
            } else {
                format!("{}.exe", name)
            }
        } else {
            // Strip .exe suffix if present (e.g., from config)
            name.strip_suffix(".exe").unwrap_or(&name).to_string()
        }
    }
}

impl Default for WinBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder for WinBuilder {
    fn id(&self) -> &'static str {
        "win"
    }
    fn name(&self) -> &'static str {
        "Windows"
    }
    fn targets(&self) -> &'static [&'static str] {
        &["win", "windows", "win64", "win32"]
    }

    fn capabilities(&self) -> Vec<BuilderCapability> {
        vec![
            BuilderCapability::Standalone,
            BuilderCapability::Portable,
            BuilderCapability::CodeSign,
            BuilderCapability::PythonEmbed,
            BuilderCapability::Extensions,
            BuilderCapability::DevTools,
        ]
    }

    fn is_available(&self) -> bool {
        cfg!(target_os = "windows")
    }

    fn validate(&self, ctx: &BuildContext) -> BuildResult<()> {
        if let Some(ref fe) = ctx.config.frontend {
            match fe {
                super::common::FrontendConfig::Path { path } => {
                    if !path.exists() {
                        return Err(PackError::FrontendNotFound(path.clone()));
                    }
                }
                super::common::FrontendConfig::Url { url } => {
                    if url.is_empty() {
                        return Err(PackError::InvalidUrl("URL cannot be empty".into()));
                    }
                }
            }
        }
        Ok(())
    }

    fn build(&self, ctx: &mut BuildContext) -> BuildResult<BuildOutput> {
        ctx.ensure_temp_dir()?;
        let exe_name = Self::get_exe_name(ctx);
        let output_path = ctx.output_dir.join(&exe_name);

        tracing::info!("Building Windows executable: {}", output_path.display());

        // Copy current exe as base
        let current_exe = std::env::current_exe()?;
        fs::create_dir_all(&ctx.output_dir)?;
        fs::copy(&current_exe, &output_path)?;

        // Build overlay config
        let overlay_config = self.build_overlay_config(ctx);
        let mut overlay = OverlayData::new(overlay_config.clone());

        // Bundle frontend
        if let Some(super::common::FrontendConfig::Path { path }) = &ctx.config.frontend {
            let bundle = BundleBuilder::new(path).build()?;
            for (p, content) in bundle.into_assets() {
                overlay.add_asset(p, content);
            }
        }

        // Add Python runtime for FullStack mode with Standalone strategy
        if let PackMode::FullStack { ref python, .. } = &overlay_config.mode {
            if python.strategy == BundleStrategy::Standalone {
                self.add_python_runtime(&mut overlay, &python.version)?;
            }
            // Add Python source files from include_paths
            self.add_python_files(&mut overlay, python)?;

            // Install pip packages if specified
            if !python.packages.is_empty() {
                self.add_pip_packages(&mut overlay, python, &ctx.output_dir)?;
            }
        }

        // Add collected assets
        for (path, content) in &ctx.assets {
            overlay.add_asset(path.clone(), content.clone());
        }

        // Apply Windows resources
        #[cfg(target_os = "windows")]
        self.apply_resources(ctx, &output_path)?;

        // Write overlay
        OverlayWriter::write(&output_path, &overlay)?;

        let size = fs::metadata(&output_path)?.len();
        tracing::info!(
            "Build complete: {} ({:.2} MB)",
            output_path.display(),
            size as f64 / 1_048_576.0
        );

        Ok(BuildOutput::new(output_path, "exe")
            .with_size(size)
            .with_assets(ctx.assets.len())
            .with_duration(ctx.elapsed()))
    }
}

impl WinBuilder {
    fn build_overlay_config(&self, ctx: &BuildContext) -> PackConfig {
        // First, try to get the original PackConfig from context metadata
        // This preserves the full configuration including FullStack mode
        if let Some(pack_config) = ctx.get_metadata::<PackConfig>("pack_config") {
            return pack_config;
        }

        // Fallback: build a basic config from BuildContext
        let (mode, mut config) = match &ctx.config.frontend {
            Some(super::common::FrontendConfig::Url { url }) => {
                (PackMode::Url { url: url.clone() }, PackConfig::url(url))
            }
            Some(super::common::FrontendConfig::Path { path }) => (
                PackMode::Frontend { path: path.clone() },
                PackConfig::frontend(path),
            ),
            None => {
                let url = "about:blank";
                (PackMode::Url { url: url.into() }, PackConfig::url(url))
            }
        };

        config.mode = mode;
        config.output_dir = ctx.output_dir.clone();
        if let Some(ref name) = ctx.config.target.output_name {
            config.output_name = name.clone();
        }
        config
    }

    /// Add Python standalone runtime to overlay for FullStack mode
    fn add_python_runtime(&self, overlay: &mut OverlayData, version: &str) -> BuildResult<()> {
        tracing::info!("Adding Python {} standalone runtime...", version);

        let config = PythonStandaloneConfig {
            version: version.to_string(),
            ..Default::default()
        };

        let standalone = PythonStandalone::new(config)?;

        // Download Python distribution
        let archive_bytes = standalone.get_distribution_bytes()?;
        let archive_size = archive_bytes.len();

        tracing::info!(
            "Python runtime downloaded: {:.2} MB",
            archive_size as f64 / 1_048_576.0
        );

        // Create runtime metadata
        let meta = PythonRuntimeMeta {
            version: version.to_string(),
            target: standalone.target().triple().to_string(),
            archive_size: archive_size as u64,
        };

        // Add metadata and archive to overlay
        let meta_json = serde_json::to_vec(&meta).map_err(|e| {
            PackError::Config(format!("Failed to serialize Python runtime meta: {}", e))
        })?;

        overlay.add_asset("python_runtime.json", meta_json);
        overlay.add_asset("python_runtime.tar.gz", archive_bytes);

        tracing::info!("Python runtime added to overlay");
        Ok(())
    }

    /// Add Python source files from include_paths to overlay
    fn add_python_files(
        &self,
        overlay: &mut OverlayData,
        python: &PythonBundleConfig,
    ) -> BuildResult<()> {
        if python.include_paths.is_empty() {
            tracing::debug!("No Python include_paths specified, skipping Python file collection");
            return Ok(());
        }

        tracing::info!(
            "Collecting Python files from {} include paths...",
            python.include_paths.len()
        );

        let mut total_files = 0;
        let mut total_size: u64 = 0;

        for include_path in &python.include_paths {
            if !include_path.exists() {
                tracing::warn!(
                    "Python include_path does not exist: {}",
                    include_path.display()
                );
                continue;
            }

            tracing::debug!("Scanning Python path: {}", include_path.display());

            for entry in WalkDir::new(include_path)
                .follow_links(true)
                .into_iter()
                .filter_entry(|e| {
                    let name = e.file_name().to_string_lossy();
                    // Skip common excluded directories
                    !matches!(
                        name.as_ref(),
                        "__pycache__"
                            | ".git"
                            | ".venv"
                            | "venv"
                            | "node_modules"
                            | ".tox"
                            | ".pytest_cache"
                            | ".mypy_cache"
                            | "dist"
                            | "build"
                            | "*.egg-info"
                    )
                })
            {
                let entry = match entry {
                    Ok(e) => e,
                    Err(e) => {
                        tracing::warn!("Failed to read directory entry: {}", e);
                        continue;
                    }
                };

                if !entry.file_type().is_file() {
                    continue;
                }

                let path = entry.path();
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

                // Include Python files and common data files
                if !matches!(
                    ext,
                    "py" | "pyd" | "so" | "json" | "yaml" | "yml" | "toml" | "txt" | "md"
                ) {
                    continue;
                }

                // Skip excluded patterns
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                let should_exclude = python.exclude.iter().any(|pattern| {
                    // Simple glob matching
                    if let Some(stripped) = pattern.strip_prefix('*') {
                        file_name.ends_with(stripped)
                    } else if pattern.ends_with('*') {
                        file_name.starts_with(&pattern[..pattern.len() - 1])
                    } else {
                        file_name == pattern || path.to_string_lossy().contains(pattern)
                    }
                });

                if should_exclude {
                    continue;
                }

                // Read file content
                let content = match fs::read(path) {
                    Ok(c) => c,
                    Err(e) => {
                        tracing::warn!("Failed to read file {}: {}", path.display(), e);
                        continue;
                    }
                };

                // Get relative path from include_path
                let relative = match path.strip_prefix(include_path) {
                    Ok(r) => r,
                    Err(_) => continue,
                };

                // Add to overlay with "python/" prefix so extraction knows where to put it
                let asset_path =
                    format!("python/{}", relative.to_string_lossy().replace('\\', "/"));

                tracing::debug!(
                    "Adding Python file: {} ({} bytes)",
                    asset_path,
                    content.len()
                );
                total_size += content.len() as u64;
                total_files += 1;

                overlay.add_asset(asset_path, content);
            }
        }

        tracing::info!(
            "Added {} Python files ({:.2} KB) to overlay",
            total_files,
            total_size as f64 / 1024.0
        );

        Ok(())
    }

    /// Install pip packages and add them to overlay
    fn add_pip_packages(
        &self,
        overlay: &mut OverlayData,
        python: &PythonBundleConfig,
        output_dir: &Path,
    ) -> BuildResult<()> {
        tracing::info!(
            "Installing {} pip packages: {:?}",
            python.packages.len(),
            python.packages
        );

        // Create a temporary directory for pip installation
        let temp_site_packages = output_dir.join("_temp_site_packages");
        if temp_site_packages.exists() {
            fs::remove_dir_all(&temp_site_packages)?;
        }
        fs::create_dir_all(&temp_site_packages)?;

        // Use DepsCollector to install packages with dependencies
        let collector = DepsCollector::new();
        let result = collector.collect_with_pip(&python.packages, &temp_site_packages)?;

        tracing::info!(
            "Installed {} files ({:.2} MB) from pip packages",
            result.file_count,
            result.total_size as f64 / 1_048_576.0
        );

        // Add all installed files to overlay under "python/site-packages/"
        for entry in WalkDir::new(&temp_site_packages)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            let relative = match path.strip_prefix(&temp_site_packages) {
                Ok(r) => r,
                Err(_) => continue,
            };

            let content = match fs::read(path) {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!("Failed to read file {}: {}", path.display(), e);
                    continue;
                }
            };

            // Add to overlay with "python/site-packages/" prefix
            let asset_path = format!(
                "python/site-packages/{}",
                relative.to_string_lossy().replace('\\', "/")
            );
            overlay.add_asset(asset_path, content);
        }

        // Clean up temporary directory
        if let Err(e) = fs::remove_dir_all(&temp_site_packages) {
            tracing::warn!("Failed to clean up temp directory: {}", e);
        }

        tracing::info!("Pip packages added to overlay");
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn apply_resources(&self, ctx: &BuildContext, exe_path: &std::path::Path) -> BuildResult<()> {
        use crate::resource_editor::{ResourceConfig, ResourceEditor};

        let mut res = ResourceConfig::default();
        if let Some(ref icon) = ctx.config.app.icon {
            res.icon = Some(icon.clone());
        }
        res.product_name = Some(ctx.config.app.name.clone());
        res.file_description = ctx.config.app.description.clone();
        res.product_version = Some(ctx.config.app.version.clone());

        if res.has_modifications() {
            let editor = ResourceEditor::new()?;
            editor.apply_config(exe_path, &res)?;
        }
        Ok(())
    }
}
