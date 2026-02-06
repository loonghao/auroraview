//! Collect plugin for handling [[hooks.collect]] entries
//!
//! This plugin handles resource collection configured in [hooks.collect] sections
//! of auroraview.pack.toml manifest files.

use super::traits::{PackContext, PackHook, PackPlugin, PackResult};
use crate::common::CollectPattern;
use crate::PackError;
use glob::glob;
use std::path::{Path, PathBuf};

/// Collect plugin for gathering additional files
pub struct CollectPlugin {
    base_dir: PathBuf,
}

impl CollectPlugin {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }
}

impl PackPlugin for CollectPlugin {
    fn name(&self) -> &'static str {
        "resource-collector"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn hooks(&self) -> Vec<PackHook> {
        vec![PackHook::AfterCollect]
    }

    fn on_hook(&self, hook: PackHook, context: &mut PackContext) -> PackResult<()> {
        if hook != PackHook::AfterCollect {
            return Ok(());
        }

        // Get hooks.collect configuration from config
        let collect_patterns = match &context.config.hooks {
            Some(ref h) if !h.collect.is_empty() => {
                tracing::debug!("CollectPlugin: found {} patterns", h.collect.len());
                h.collect.clone()
            }
            Some(_) => {
                tracing::debug!("CollectPlugin: hooks config exists but collect is empty");
                return Ok(());
            }
            None => {
                tracing::debug!("CollectPlugin: no hooks config found");
                return Ok(());
            }
        };

        tracing::info!(
            "CollectPlugin: collecting {} patterns",
            collect_patterns.len()
        );

        // Collect files configured in [[hooks.collect]]
        self.collect_files(&collect_patterns, context)
    }
}

impl CollectPlugin {
    fn collect_files(
        &self,
        patterns: &[CollectPattern],
        context: &mut PackContext,
    ) -> PackResult<()> {
        for pattern in patterns {
            tracing::info!(
                "CollectPlugin: processing pattern '{}' -> dest: {:?}",
                pattern.source,
                pattern.dest
            );

            // Resolve source pattern relative to base_dir
            let source_pattern = &pattern.source;
            let source_path = Path::new(source_pattern);
            let full_source = if source_path.is_absolute() {
                // Absolute path (works on all platforms including Windows)
                PathBuf::from(source_pattern)
            } else if let Some(stripped) = source_pattern.strip_prefix("./") {
                // Relative path with ./
                self.base_dir.join(stripped)
            } else {
                // Relative path
                self.base_dir.join(source_pattern)
            };

            tracing::info!("CollectPlugin: resolved path '{}'", full_source.display());

            // Handle dest option
            let dest_path = pattern.dest.as_ref().map(|d| PathBuf::from(d.as_str()));

            // Handle glob patterns
            if source_pattern.contains('*') || source_pattern.contains('?') {
                tracing::info!("CollectPlugin: using glob pattern");
                self.collect_glob(&full_source, &dest_path, pattern, context)?;
            } else {
                tracing::info!("CollectPlugin: processing single file/directory");
                self.collect_single_file(&full_source, &dest_path, pattern, context)?;
            }
        }
        Ok(())
    }

    fn collect_glob(
        &self,
        source: &Path,
        dest: &Option<PathBuf>,
        pattern: &CollectPattern,
        context: &mut PackContext,
    ) -> PackResult<()> {
        // Convert Windows backslashes to forward slashes for glob compatibility
        let glob_pattern = source.to_string_lossy().replace('\\', "/");

        tracing::info!("CollectPlugin: glob pattern: {}", glob_pattern);

        match glob(&glob_pattern) {
            Ok(entries) => {
                let mut count = 0;
                for entry in entries.flatten() {
                    if entry.is_file() {
                        count += 1;
                        tracing::debug!("CollectPlugin: found file: {}", entry.display());

                        // Read file content
                        let content = std::fs::read(&entry).map_err(|e| {
                            PackError::Config(format!(
                                "Failed to read file {}: {}",
                                entry.display(),
                                e
                            ))
                        })?;

                        // Calculate relative path from source
                        let source_parent = source.parent().unwrap_or_else(|| Path::new("."));
                        let relative = entry
                            .strip_prefix(source_parent)
                            .unwrap_or(&entry)
                            .to_string_lossy()
                            .to_string();

                        // Build destination path
                        let empty_path = PathBuf::from("");
                        let dest_ref = dest.as_ref().unwrap_or(&empty_path);
                        let dest_file = if pattern.preserve_structure {
                            dest_ref.join(&relative)
                        } else {
                            dest_ref.join(entry.file_name().unwrap())
                        };

                        // Add to context assets
                        let asset_path = dest_file.to_string_lossy().to_string();
                        tracing::info!("CollectPlugin: adding asset: {}", asset_path);
                        context.add_asset(asset_path, content);
                    }
                }
                tracing::info!("CollectPlugin: collected {} files from glob", count);
            }
            Err(e) => {
                tracing::warn!("Glob pattern failed for {}: {}", glob_pattern, e);
            }
        }
        Ok(())
    }

    fn collect_single_file(
        &self,
        source: &Path,
        dest: &Option<PathBuf>,
        pattern: &CollectPattern,
        context: &mut PackContext,
    ) -> PackResult<()> {
        if !source.exists() {
            tracing::warn!("CollectPlugin: source not found: {}", source.display());
            return Ok(());
        }

        tracing::info!("CollectPlugin: collecting from: {}", source.display());

        let empty_path = PathBuf::from("");
        let dest_ref = dest.as_ref().unwrap_or(&empty_path);

        if source.is_file() {
            // Single file
            tracing::info!("CollectPlugin: reading single file: {}", source.display());
            let content = std::fs::read(source).map_err(|e| {
                PackError::Config(format!("Failed to read file {}: {}", source.display(), e))
            })?;

            let dest_file = dest_ref.join(source.file_name().unwrap());
            let asset_path = dest_file.to_string_lossy().to_string();
            tracing::info!("CollectPlugin: adding single file asset: {}", asset_path);
            context.add_asset(asset_path, content);
        } else if source.is_dir() {
            // Directory - collect recursively
            tracing::info!("CollectPlugin: collecting directory recursively");
            self.collect_directory(source, dest_ref, pattern, context)?;
        } else {
            tracing::warn!(
                "CollectPlugin: source is neither file nor directory: {}",
                source.display()
            );
        }

        Ok(())
    }

    fn collect_directory(
        &self,
        source: &Path,
        dest: &Path,
        pattern: &CollectPattern,
        context: &mut PackContext,
    ) -> PackResult<()> {
        let mut count = 0;
        for entry in walkdir::WalkDir::new(source) {
            match entry {
                Ok(e) => {
                    let path = e.path();

                    if path.is_file() {
                        count += 1;
                        tracing::debug!("CollectPlugin: directory entry: {}", path.display());

                        let content = std::fs::read(path).map_err(|e| {
                            PackError::Config(format!(
                                "Failed to read file {}: {}",
                                path.display(),
                                e
                            ))
                        })?;

                        let relative = path
                            .strip_prefix(source.parent().unwrap_or_else(|| Path::new(".")))
                            .unwrap_or(path)
                            .to_string_lossy()
                            .to_string();

                        let dest_file = if pattern.preserve_structure {
                            dest.join(&relative)
                        } else {
                            dest.join(path.file_name().unwrap())
                        };

                        let asset_path = dest_file.to_string_lossy().to_string();
                        tracing::info!("CollectPlugin: adding directory asset: {}", asset_path);
                        context.add_asset(asset_path, content);
                    }
                }
                Err(e) => {
                    tracing::warn!("Error walking directory {}: {}", source.display(), e);
                }
            }
        }

        tracing::info!("CollectPlugin: collected {} files from directory", count);
        Ok(())
    }
}

impl Default for CollectPlugin {
    fn default() -> Self {
        Self {
            base_dir: PathBuf::from("."),
        }
    }
}
