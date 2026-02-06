//! Extension bundling for packed applications
//!
//! This module handles Chrome extension bundling:
//! - Download from Chrome Web Store
//! - Bundle local extensions
//! - Embed extensions into overlay

#![allow(dead_code)]

// TODO: Fix ExtensionBundler when extension_downloader module is available
// TODO: Implement has_extensions() method for ExtensionsConfig
use super::traits::PackResult;
use crate::builder::common::ExtensionsConfig;
use crate::overlay::OverlayData;
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
        // TODO: Implement when ExtensionsConfig has has_extensions() method
        // if !self.config.has_extensions() {
        //     return Ok(BundleResult::default());
        // }

        let _ = overlay;
        Ok(BundleResult::default())
    }

    // TODO: Implement remaining extension bundling methods
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
}
