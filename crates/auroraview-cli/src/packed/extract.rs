//! File extraction module
//!
//! Handles extraction of Python runtime and resources from overlay data.

use anyhow::{Context, Result};
use auroraview_pack::{OverlayData, PythonRuntimeMeta};
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::utils::{get_python_exe_path, get_runtime_cache_dir};

/// Extract embedded Python runtime for standalone mode
pub fn extract_standalone_python(overlay: &OverlayData) -> Result<PathBuf> {
    // Find Python runtime metadata
    let meta_data = overlay
        .assets
        .iter()
        .find(|(path, _)| path == "python_runtime.json")
        .map(|(_, content)| content.clone())
        .ok_or_else(|| anyhow::anyhow!("Python runtime metadata not found in overlay"))?;

    let meta: PythonRuntimeMeta = serde_json::from_slice(&meta_data)
        .with_context(|| "Failed to parse Python runtime metadata")?;

    // Find Python runtime archive
    let archive_data = overlay
        .assets
        .iter()
        .find(|(path, _)| path == "python_runtime.tar.gz")
        .map(|(_, content)| content.clone())
        .ok_or_else(|| anyhow::anyhow!("Python runtime archive not found in overlay"))?;

    tracing::info!(
        "Extracting Python {} runtime ({:.2} MB)...",
        meta.version,
        archive_data.len() as f64 / (1024.0 * 1024.0)
    );

    // Get app name for cache directory
    let exe_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("auroraview"));
    let app_name = exe_path
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("auroraview");

    // Extract to cache directory
    let cache_dir = get_runtime_cache_dir(app_name);
    let version_marker = cache_dir.join(".version");

    // Check if already extracted with correct version
    if version_marker.exists() {
        if let Ok(cached_version) = fs::read_to_string(&version_marker) {
            if cached_version.trim() == meta.version {
                let python_path = get_python_exe_path(&cache_dir);
                if python_path.exists() {
                    tracing::info!("Using cached Python runtime: {}", cache_dir.display());
                    return Ok(python_path);
                }
            }
        }
    }

    // Clean up old extraction if exists
    if cache_dir.exists() {
        fs::remove_dir_all(&cache_dir)?;
    }
    fs::create_dir_all(&cache_dir)?;

    tracing::info!("Extracting Python runtime to: {}", cache_dir.display());

    // Decompress and extract tar.gz
    let decoder = flate2::read::GzDecoder::new(&archive_data[..]);
    let mut archive = tar::Archive::new(decoder);
    archive
        .unpack(&cache_dir)
        .with_context(|| "Failed to extract Python runtime")?;

    // Write version marker
    fs::write(&version_marker, &meta.version)?;

    let python_path = get_python_exe_path(&cache_dir);
    if !python_path.exists() {
        return Err(anyhow::anyhow!(
            "Python executable not found after extraction: {}",
            python_path.display()
        ));
    }

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&python_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&python_path, perms)?;
    }

    tracing::info!("Python runtime ready: {}", python_path.display());
    Ok(python_path)
}

/// Extract resource directories from overlay assets (parallel version)
///
/// Resources are stored with prefixes like "resources/examples/", "resources/data/", etc.
/// This function extracts them to the resources directory and returns the path.
pub fn extract_resources_parallel(overlay: &OverlayData, base_dir: &Path) -> Result<PathBuf> {
    let resources_dir = base_dir.join("resources");
    fs::create_dir_all(&resources_dir)?;

    // Collect resource files to extract
    let resource_assets: Vec<_> = overlay
        .assets
        .iter()
        .filter_map(|(path, content)| {
            if path.starts_with("resources/") {
                let rel_path = path.strip_prefix("resources/").unwrap_or(path);
                Some((resources_dir.join(rel_path), content))
            } else if path.starts_with("examples/") {
                Some((resources_dir.join(path), content))
            } else {
                None
            }
        })
        .collect();

    if resource_assets.is_empty() {
        return Ok(resources_dir);
    }

    // Pre-create all directories in batch
    let dirs: HashSet<PathBuf> = resource_assets
        .iter()
        .filter_map(|(path, _)| path.parent().map(|p| p.to_path_buf()))
        .collect();

    for dir in &dirs {
        fs::create_dir_all(dir)?;
    }

    // Parallel file extraction with file locking handling
    let results: Vec<Result<(), anyhow::Error>> = resource_assets
        .par_iter()
        .map(|(dest_path, content)| {
            match fs::write(dest_path, content) {
                Ok(_) => Ok(()),
                Err(e) => {
                    // Check if it's a file locking error (os error 32 on Windows)
                    if e.raw_os_error() == Some(32) {
                        // File is locked, check if existing file has same content
                        if let Ok(existing_content) = fs::read(dest_path) {
                            if existing_content == content.as_slice() {
                                tracing::debug!(
                                    "Resource file {} is locked but content matches, skipping",
                                    dest_path.display()
                                );
                                return Ok(());
                            }
                        }
                        Err(anyhow::anyhow!(
                            "Resource file {} is locked by another process",
                            dest_path.display()
                        ))
                    } else {
                        Err(e).with_context(|| format!("Failed to write: {}", dest_path.display()))
                    }
                }
            }
        })
        .collect();

    // Check for errors
    for result in results {
        result?;
    }

    tracing::info!(
        "Extracted {} resource files to: {}",
        resource_assets.len(),
        resources_dir.display()
    );

    Ok(resources_dir)
}

#[allow(dead_code)]
/// Extract resource directories from overlay assets (sequential version, kept for reference)
pub fn extract_resources(overlay: &OverlayData, base_dir: &Path) -> Result<PathBuf> {
    let resources_dir = base_dir.join("resources");
    fs::create_dir_all(&resources_dir)?;

    let mut resource_count = 0;

    for (path, content) in &overlay.assets {
        // Check for resources with "resources/" prefix (from hooks.collect)
        if path.starts_with("resources/") {
            let rel_path = path.strip_prefix("resources/").unwrap_or(path);
            let dest_path = resources_dir.join(rel_path);

            // Create parent directories
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(&dest_path, content)?;
            resource_count += 1;
            tracing::debug!("Extracted resource: {}", rel_path);
        }
        // Also check for "examples/" prefix directly (legacy support)
        else if path.starts_with("examples/") {
            let dest_path = resources_dir.join(path);

            // Create parent directories
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(&dest_path, content)?;
            resource_count += 1;
            tracing::debug!("Extracted resource: {}", path);
        }
    }

    if resource_count > 0 {
        tracing::info!(
            "Extracted {} resource files to: {}",
            resource_count,
            resources_dir.display()
        );
    }

    Ok(resources_dir)
}
