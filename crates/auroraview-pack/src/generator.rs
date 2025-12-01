//! Pack generator - creates standalone executable projects

use std::fs;
use std::path::{Path, PathBuf};

use askama::Template;

use crate::pyembed_integration::{generate_pyoxidizer_config, PyEmbedConfig};
use crate::templates::{
    CargoTomlPyembedTemplate, CargoTomlTemplate, MainFrontendTemplate, MainFullstackTemplate,
    MainUrlTemplate,
};
use crate::{PackConfig, PackError, PackMode, PackResult};

/// Pack generator creates standalone executable projects
pub struct PackGenerator {
    config: PackConfig,
}

impl PackGenerator {
    /// Create a new pack generator with the given configuration
    pub fn new(config: PackConfig) -> Self {
        Self { config }
    }

    /// Validate the pack configuration
    pub fn validate(&self) -> PackResult<()> {
        match &self.config.mode {
            PackMode::Url { url } => {
                // Validate URL format
                if !url.contains('.') && !url.starts_with("http") {
                    return Err(PackError::InvalidUrl(format!(
                        "Invalid URL format: {}. Expected a valid URL like 'example.com' or 'https://example.com'",
                        url
                    )));
                }
            }
            PackMode::Frontend { path } => {
                // Validate frontend path exists
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
                backend_entry,
            } => {
                // Validate frontend
                if !frontend_path.exists() {
                    return Err(PackError::FrontendNotFound(frontend_path.clone()));
                }
                // Validate backend entry format (module:function)
                if !backend_entry.contains(':') {
                    return Err(PackError::InvalidBackendEntry(backend_entry.clone()));
                }
                // Note: PyOxidizer integration will be added later
                tracing::warn!(
                    "Full-stack mode with Python backend is experimental and requires PyOxidizer"
                );
            }
        }
        Ok(())
    }

    /// Generate the pack project
    ///
    /// This creates a new Rust project with embedded resources that can be
    /// compiled into a standalone executable.
    pub fn generate(&self) -> PackResult<PathBuf> {
        self.validate()?;

        // Create output directory
        let project_dir = self.config.output_dir.join(&self.config.output_name);
        fs::create_dir_all(&project_dir)?;

        tracing::info!("Generating pack project in: {}", project_dir.display());

        match &self.config.mode {
            PackMode::Url { url } => self.generate_url_mode(&project_dir, url),
            PackMode::Frontend { path } => self.generate_frontend_mode(&project_dir, path),
            PackMode::FullStack {
                frontend_path,
                backend_entry,
            } => self.generate_fullstack_mode(&project_dir, frontend_path, backend_entry),
        }
    }

    /// Generate URL-only mode project
    fn generate_url_mode(&self, project_dir: &Path, url: &str) -> PackResult<PathBuf> {
        tracing::info!("Generating URL mode project for: {}", url);

        // Generate Cargo.toml (no embedded assets for URL mode)
        let cargo_toml = self.generate_cargo_toml(false)?;
        fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;

        // Generate main.rs for URL mode
        let main_rs = self.generate_url_main_rs(url)?;
        let src_dir = project_dir.join("src");
        fs::create_dir_all(&src_dir)?;
        fs::write(src_dir.join("main.rs"), main_rs)?;

        tracing::info!("URL mode project generated successfully");
        Ok(project_dir.to_path_buf())
    }

    /// Generate frontend-only mode project
    fn generate_frontend_mode(&self, project_dir: &Path, frontend: &Path) -> PackResult<PathBuf> {
        tracing::info!(
            "Generating frontend mode project from: {}",
            frontend.display()
        );

        // Generate Cargo.toml with rust-embed
        let cargo_toml = self.generate_cargo_toml(true)?;
        fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;

        // Copy frontend assets
        let assets_dir = project_dir.join("assets");
        self.copy_frontend_assets(frontend, &assets_dir)?;

        // Generate main.rs for frontend mode
        let main_rs = self.generate_frontend_main_rs()?;
        let src_dir = project_dir.join("src");
        fs::create_dir_all(&src_dir)?;
        fs::write(src_dir.join("main.rs"), main_rs)?;

        tracing::info!("Frontend mode project generated successfully");
        Ok(project_dir.to_path_buf())
    }

    /// Generate full-stack mode project (with PyOxidizer/pyembed)
    fn generate_fullstack_mode(
        &self,
        project_dir: &Path,
        frontend: &Path,
        backend_entry: &str,
    ) -> PackResult<PathBuf> {
        tracing::info!(
            "Generating full-stack mode project with Python backend: {}",
            backend_entry
        );

        // Parse backend entry (module:function)
        let parts: Vec<&str> = backend_entry.split(':').collect();
        let (backend_module, backend_func) = if parts.len() == 2 {
            (parts[0], parts[1])
        } else {
            return Err(PackError::InvalidBackendEntry(backend_entry.to_string()));
        };

        // Generate Cargo.toml with pyembed dependency
        let cargo_toml = self.generate_cargo_toml_pyembed(true)?;
        fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;

        // Copy frontend assets
        let assets_dir = project_dir.join("assets");
        self.copy_frontend_assets(frontend, &assets_dir)?;

        // Generate main.rs for full-stack mode
        let main_rs = self.generate_fullstack_main_rs(backend_module, backend_func)?;
        let src_dir = project_dir.join("src");
        fs::create_dir_all(&src_dir)?;
        fs::write(src_dir.join("main.rs"), main_rs)?;

        // Generate build.rs for pyo3-build-config
        let build_rs = self.generate_build_rs()?;
        fs::write(project_dir.join("build.rs"), build_rs)?;

        // Generate pyoxidizer.bzl configuration
        let pyembed_config = PyEmbedConfig::with_entry_point(backend_entry);
        let pyoxidizer_config = generate_pyoxidizer_config(&self.config, &pyembed_config)?;
        fs::write(project_dir.join("pyoxidizer.bzl"), pyoxidizer_config)?;

        tracing::info!("Full-stack mode project generated successfully");
        tracing::info!(
            "To build: cd {} && cargo build --release",
            project_dir.display()
        );
        tracing::info!("Note: Ensure Python development headers are available");

        Ok(project_dir.to_path_buf())
    }

    /// Copy frontend assets to target directory
    fn copy_frontend_assets(&self, src: &Path, dst: &Path) -> PackResult<()> {
        fs::create_dir_all(dst)?;

        if src.is_dir() {
            self.copy_dir_recursive(src, dst)?;
        } else {
            // Single file, copy it as index.html
            fs::copy(src, dst.join("index.html"))?;
        }

        Ok(())
    }

    /// Recursively copy directory
    fn copy_dir_recursive(&self, src: &Path, dst: &Path) -> PackResult<()> {
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if src_path.is_dir() {
                fs::create_dir_all(&dst_path)?;
                self.copy_dir_recursive(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }
        Ok(())
    }

    /// Generate Cargo.toml using Askama template
    fn generate_cargo_toml(&self, embed_assets: bool) -> PackResult<String> {
        let template = CargoTomlTemplate {
            name: &self.config.output_name,
            embed_assets,
        };
        template
            .render()
            .map_err(|e| PackError::TemplateError(e.to_string()))
    }

    /// Generate main.rs for URL mode using Askama template
    fn generate_url_main_rs(&self, url: &str) -> PackResult<String> {
        let normalized_url = if url.starts_with("http://") || url.starts_with("https://") {
            url.to_string()
        } else {
            format!("https://{}", url)
        };

        let template = MainUrlTemplate {
            title: &self.config.window_title,
            url: &normalized_url,
            width: self.config.window_width,
            height: self.config.window_height,
        };
        template
            .render()
            .map_err(|e| PackError::TemplateError(e.to_string()))
    }

    /// Generate main.rs for frontend mode using Askama template
    fn generate_frontend_main_rs(&self) -> PackResult<String> {
        let template = MainFrontendTemplate {
            title: &self.config.window_title,
            width: self.config.window_width,
            height: self.config.window_height,
        };
        template
            .render()
            .map_err(|e| PackError::TemplateError(e.to_string()))
    }

    /// Generate Cargo.toml with pyembed for full-stack mode
    fn generate_cargo_toml_pyembed(&self, embed_assets: bool) -> PackResult<String> {
        let template = CargoTomlPyembedTemplate {
            name: &self.config.output_name,
            embed_assets,
        };
        template
            .render()
            .map_err(|e| PackError::TemplateError(e.to_string()))
    }

    /// Generate main.rs for full-stack mode
    fn generate_fullstack_main_rs(
        &self,
        backend_module: &str,
        backend_func: &str,
    ) -> PackResult<String> {
        let template = MainFullstackTemplate {
            title: &self.config.window_title,
            width: self.config.window_width,
            height: self.config.window_height,
            backend_entry: true,
            backend_module,
            backend_func,
        };
        template
            .render()
            .map_err(|e| PackError::TemplateError(e.to_string()))
    }

    /// Generate build.rs for pyo3-build-config
    fn generate_build_rs(&self) -> PackResult<String> {
        Ok(r#"//! Build script for pyembed integration
//!
//! This configures pyo3-build-config to find the Python installation

fn main() {
    // pyo3-build-config will automatically detect the Python installation
    // and set the necessary environment variables for linking
    println!("cargo:rerun-if-env-changed=PYO3_PYTHON");
    println!("cargo:rerun-if-env-changed=PYTHON_SYS_EXECUTABLE");
}
"#
        .to_string())
    }
}
