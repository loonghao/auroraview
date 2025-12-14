//! Asset bundling for frontend mode

use crate::{PackError, PackResult};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Collection of assets to be embedded
#[derive(Debug, Default)]
pub struct AssetBundle {
    /// Assets as (relative_path, content) pairs
    assets: Vec<(String, Vec<u8>)>,
    /// Total uncompressed size
    total_size: u64,
}

impl AssetBundle {
    /// Create an empty bundle
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an asset to the bundle
    pub fn add(&mut self, path: impl Into<String>, content: Vec<u8>) {
        let content_len = content.len() as u64;
        self.assets.push((path.into(), content));
        self.total_size += content_len;
    }

    /// Get all assets
    pub fn assets(&self) -> &[(String, Vec<u8>)] {
        &self.assets
    }

    /// Get the number of assets
    pub fn len(&self) -> usize {
        self.assets.len()
    }

    /// Check if the bundle is empty
    pub fn is_empty(&self) -> bool {
        self.assets.is_empty()
    }

    /// Get total uncompressed size
    pub fn total_size(&self) -> u64 {
        self.total_size
    }

    /// Convert to owned assets vector
    pub fn into_assets(self) -> Vec<(String, Vec<u8>)> {
        self.assets
    }
}

/// Builder for creating asset bundles from directories
pub struct BundleBuilder {
    /// Root directory for assets
    root: std::path::PathBuf,
    /// File extensions to include (empty = all)
    extensions: Vec<String>,
    /// Patterns to exclude
    exclude_patterns: Vec<String>,
}

impl BundleBuilder {
    /// Create a new bundle builder for a directory
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            extensions: Vec::new(),
            exclude_patterns: vec![
                ".git".to_string(),
                ".gitignore".to_string(),
                ".DS_Store".to_string(),
                "Thumbs.db".to_string(),
                "*.map".to_string(),
            ],
        }
    }

    /// Only include files with these extensions
    pub fn with_extensions(mut self, extensions: &[&str]) -> Self {
        self.extensions = extensions.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add patterns to exclude
    pub fn exclude(mut self, patterns: &[&str]) -> Self {
        self.exclude_patterns
            .extend(patterns.iter().map(|s| s.to_string()));
        self
    }

    /// Build the asset bundle
    pub fn build(&self) -> PackResult<AssetBundle> {
        if !self.root.exists() {
            return Err(PackError::FrontendNotFound(self.root.clone()));
        }

        let mut bundle = AssetBundle::new();

        // If root is a file, just add it as index.html
        if self.root.is_file() {
            let content = fs::read(&self.root)?;
            bundle.add("index.html", content);
            return Ok(bundle);
        }

        // Walk directory
        for entry in WalkDir::new(&self.root)
            .follow_links(true)
            .into_iter()
            .filter_entry(|e| !self.should_exclude(e))
        {
            let entry = entry.map_err(|e| PackError::Bundle(e.to_string()))?;

            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();

            // Check extension filter
            if !self.extensions.is_empty() {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if !self.extensions.iter().any(|e| e == ext) {
                    continue;
                }
            }

            // Get relative path
            let relative = path
                .strip_prefix(&self.root)
                .map_err(|e| PackError::Bundle(e.to_string()))?;

            // Normalize path separators to forward slashes
            let relative_str = relative.to_string_lossy().replace('\\', "/");

            // Read content
            let content = fs::read(path)?;

            tracing::debug!("Adding asset: {} ({} bytes)", relative_str, content.len());
            bundle.add(relative_str, content);
        }

        if bundle.is_empty() {
            return Err(PackError::Bundle(format!(
                "No assets found in: {}",
                self.root.display()
            )));
        }

        tracing::info!(
            "Bundle created: {} files, {} bytes total",
            bundle.len(),
            bundle.total_size()
        );

        Ok(bundle)
    }

    /// Check if an entry should be excluded
    fn should_exclude(&self, entry: &walkdir::DirEntry) -> bool {
        let name = entry.file_name().to_string_lossy();

        for pattern in &self.exclude_patterns {
            if let Some(suffix) = pattern.strip_prefix('*') {
                // Wildcard pattern (e.g., "*.map")
                if name.ends_with(suffix) {
                    return true;
                }
            } else if name == pattern.as_str() {
                return true;
            }
        }

        false
    }
}


