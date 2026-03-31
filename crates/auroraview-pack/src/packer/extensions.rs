//! Extension bundling for packed applications
//!
//! This module handles Chrome extension bundling:
//! - Download from Chrome Web Store
//! - Bundle local extensions
//! - Embed extensions into overlay

#![allow(dead_code)]

use super::traits::PackResult;
use crate::builder::common::ExtensionsConfig;
use crate::overlay::OverlayData;
use crate::PackError;
use std::path::Path;

/// Extension bundler for packaging Chrome extensions
pub struct ExtensionBundler<'a> {
    /// Extension configuration
    config: &'a ExtensionsConfig,
    /// Output directory for temp files
    output_dir: &'a Path,
}

impl<'a> ExtensionBundler<'a> {
    /// Create a new extension bundler
    pub fn new(config: &'a ExtensionsConfig, output_dir: &'a Path) -> Self {
        Self { config, output_dir }
    }

    /// Bundle all configured extensions into overlay
    pub fn bundle(&self, overlay: &mut OverlayData) -> PackResult<BundleResult> {
        if !self.config.has_extensions() {
            return Ok(BundleResult::default());
        }

        let mut result = BundleResult::default();

        // Bundle local extensions
        for ext in &self.config.local {
            match self.bundle_local_extension(&ext.path, overlay) {
                Ok(ext_id) => {
                    result.local_count += 1;
                    result.extension_ids.push(ext_id);
                    tracing::info!(
                        "Bundled local extension: {}",
                        ext.name.as_deref().unwrap_or("unknown")
                    );
                }
                Err(e) => {
                    let name = ext.name.as_deref().unwrap_or("unknown");
                    tracing::warn!("Failed to bundle local extension '{}': {}", name, e);
                    result.failed.push((name.to_string(), e.to_string()));
                }
            }
        }

        // Bundle store extensions (download placeholder)
        for ext in &self.config.store {
            match self.bundle_store_extension(&ext.id, ext.version.as_deref(), overlay) {
                Ok(()) => {
                    result.store_count += 1;
                    result.extension_ids.push(ext.id.clone());
                    tracing::info!(
                        "Bundled store extension: {} ({})",
                        ext.name.as_deref().unwrap_or("unknown"),
                        ext.id
                    );
                }
                Err(e) => {
                    let name = ext.name.as_deref().unwrap_or(&ext.id);
                    tracing::warn!("Failed to bundle store extension '{}': {}", name, e);
                    result.failed.push((name.to_string(), e.to_string()));
                }
            }
        }

        Ok(result)
    }

    /// Bundle a local extension from a directory path
    fn bundle_local_extension(
        &self,
        ext_path: &Path,
        overlay: &mut OverlayData,
    ) -> PackResult<String> {
        let abs_path = if ext_path.is_absolute() {
            ext_path.to_path_buf()
        } else {
            self.output_dir.join(ext_path)
        };

        if !abs_path.exists() {
            return Err(PackError::Build(format!(
                "Extension path does not exist: {}",
                abs_path.display()
            )));
        }

        // Read manifest.json to get extension ID
        let manifest_path = abs_path.join("manifest.json");
        if !manifest_path.exists() {
            return Err(PackError::Build(format!(
                "No manifest.json found in extension: {}",
                abs_path.display()
            )));
        }

        let manifest_content = std::fs::read_to_string(&manifest_path)
            .map_err(|e| PackError::Build(format!("Failed to read manifest.json: {}", e)))?;

        let manifest: serde_json::Value = serde_json::from_str(&manifest_content)
            .map_err(|e| PackError::Build(format!("Invalid manifest.json: {}", e)))?;

        let ext_name = manifest
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let ext_id = sanitize_extension_id(ext_name);

        // Walk extension directory and add files to overlay
        self.add_directory_to_overlay(&abs_path, &format!("extensions/{}", ext_id), overlay)?;

        Ok(ext_id)
    }

    /// Bundle a store extension by ID (placeholder for future Chrome Web Store download)
    fn bundle_store_extension(
        &self,
        ext_id: &str,
        _version: Option<&str>,
        overlay: &mut OverlayData,
    ) -> PackResult<()> {
        // Check if extension was pre-downloaded to output dir
        let cached_path = self.output_dir.join("extensions").join(ext_id);
        if cached_path.exists() {
            self.add_directory_to_overlay(
                &cached_path,
                &format!("extensions/{}", ext_id),
                overlay,
            )?;
            return Ok(());
        }

        // Store extension download not yet implemented
        Err(PackError::Build(format!(
            "Chrome Web Store download not yet implemented. \
             Pre-download extension '{}' to {}/extensions/{}/",
            ext_id,
            self.output_dir.display(),
            ext_id
        )))
    }

    /// Recursively add directory contents to overlay
    fn add_directory_to_overlay(
        &self,
        dir: &Path,
        prefix: &str,
        overlay: &mut OverlayData,
    ) -> PackResult<()> {
        for entry in std::fs::read_dir(dir)
            .map_err(|e| PackError::Build(format!("Failed to read directory: {}", e)))?
        {
            let entry =
                entry.map_err(|e| PackError::Build(format!("Failed to read entry: {}", e)))?;
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let asset_key = format!("{}/{}", prefix, name);

            if path.is_dir() {
                self.add_directory_to_overlay(&path, &asset_key, overlay)?;
            } else {
                let content = std::fs::read(&path).map_err(|e| {
                    PackError::Build(format!("Failed to read file {}: {}", path.display(), e))
                })?;
                overlay.add_asset(asset_key, content);
            }
        }
        Ok(())
    }
}

/// Sanitize an extension name to a valid ID
fn sanitize_extension_id(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

/// Result of extension bundling
#[derive(Debug, Default)]
pub struct BundleResult {
    /// Number of store extensions bundled
    pub store_count: usize,
    /// Number of local extensions bundled
    pub local_count: usize,
    /// Extension IDs that were bundled
    pub extension_ids: Vec<String>,
    /// Failed extensions with error messages
    pub failed: Vec<(String, String)>,
}

impl BundleResult {
    /// Total number of bundled extensions
    pub fn total_count(&self) -> usize {
        self.store_count + self.local_count
    }

    /// Check if any extensions failed
    pub fn has_failures(&self) -> bool {
        !self.failed.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bundle_result_default() {
        let result = BundleResult::default();
        assert_eq!(result.total_count(), 0);
        assert!(!result.has_failures());
    }

    #[test]
    fn test_bundle_result_counts() {
        let result = BundleResult {
            store_count: 2,
            local_count: 1,
            extension_ids: vec!["ext1".to_string(), "ext2".to_string(), "local1".to_string()],
            failed: vec![],
        };
        assert_eq!(result.total_count(), 3);
        assert!(!result.has_failures());
    }

    #[test]
    fn test_bundle_result_with_failures() {
        let result = BundleResult {
            store_count: 1,
            local_count: 0,
            extension_ids: vec!["ext1".to_string()],
            failed: vec![("ext2".to_string(), "download failed".to_string())],
        };
        assert!(result.has_failures());
        assert_eq!(result.total_count(), 1);
    }

    #[test]
    fn test_sanitize_extension_id() {
        assert_eq!(sanitize_extension_id("My Extension"), "my_extension");
        assert_eq!(sanitize_extension_id("dark-reader"), "dark-reader");
        assert_eq!(
            sanitize_extension_id("Extension@v2.0!"),
            "extension_v2_0_"
        );
    }

    #[test]
    fn test_bundle_disabled_config_returns_empty() {
        let config = ExtensionsConfig::default();
        assert!(!config.has_extensions());
    }

    #[test]
    fn test_bundle_enabled_but_empty() {
        let config = ExtensionsConfig {
            enabled: true,
            store: vec![],
            local: vec![],
        };
        assert!(!config.has_extensions());
        assert_eq!(config.extension_count(), 0);
    }

    #[test]
    fn test_has_extensions_with_store() {
        use crate::builder::common::StoreExtension;
        let config = ExtensionsConfig {
            enabled: true,
            store: vec![StoreExtension {
                id: "abc123".to_string(),
                version: None,
                name: Some("Test Extension".to_string()),
            }],
            local: vec![],
        };
        assert!(config.has_extensions());
        assert_eq!(config.extension_count(), 1);
    }
}
