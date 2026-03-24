//! Utility functions for packed mode
//!
//! Includes path helpers, environment variable injection, string escaping,
//! and environment isolation (rez-style).

use auroraview_pack::IsolationConfig;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

// Re-export escape_json_for_js from core
pub use auroraview_core::utils::escape_json_for_js;

/// Get the runtime cache directory with content hash
///
/// Uses hash-based caching for cache isolation:
/// - `%LOCALAPPDATA%/AuroraView/runtime/{app_name}/{content_hash}`
pub fn get_runtime_cache_dir_with_hash(app_name: &str, content_hash: &str) -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("AuroraView")
        .join("runtime")
        .join(app_name)
        .join(content_hash)
}

/// Get the Python executable path within the extracted runtime
pub fn get_python_exe_path(cache_dir: &Path) -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        cache_dir.join("python").join("python.exe")
    }
    #[cfg(not(target_os = "windows"))]
    {
        cache_dir.join("python").join("bin").join("python3")
    }
}

/// Get the directory for extracting Python files with content hash
///
/// Uses hash-based caching similar to uv:
/// - `%LOCALAPPDATA%/AuroraView/python/{app_name}/{content_hash}`
///
/// Benefits:
/// - Same content → same hash → skip extraction entirely
/// - Different content → different hash → no file lock conflicts
/// - Multiple versions can coexist safely
pub fn get_python_extract_dir_with_hash(content_hash: &str) -> PathBuf {
    let exe_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("auroraview"));
    let exe_name = exe_path
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("auroraview");

    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("AuroraView")
        .join("python")
        .join(exe_name)
        .join(content_hash)
}

/// Build module search paths from configuration
///
/// Expands special variables:
/// - `$EXTRACT_DIR` - The directory where Python files are extracted
/// - `$RESOURCES_DIR` - The resources directory
/// - `$SITE_PACKAGES` - The site-packages directory
pub fn build_module_search_paths(
    config_paths: &[String],
    extract_dir: &Path,
    resources_dir: &Path,
    site_packages_dir: &Path,
) -> Vec<String> {
    config_paths
        .iter()
        .map(|path| {
            path.replace("$EXTRACT_DIR", &extract_dir.to_string_lossy())
                .replace("$RESOURCES_DIR", &resources_dir.to_string_lossy())
                .replace("$SITE_PACKAGES", &site_packages_dir.to_string_lossy())
        })
        .filter(|path| {
            // Only include paths that exist
            let p = Path::new(path);
            if p.exists() {
                true
            } else {
                tracing::debug!("Module search path does not exist: {}", path);
                false
            }
        })
        .collect()
}

/// Get the WebView2 user data directory in AppData
///
/// Returns a path like: `%LOCALAPPDATA%/AuroraView/WebView2`
/// This prevents WebView2 from creating data folders in the current directory.
pub fn get_webview_data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("AuroraView")
        .join("WebView2")
}

/// Get the browser extensions directory for WebView2
///
/// Returns a path like: `%LOCALAPPDATA%/AuroraView/Extensions`
/// Extensions placed in this directory will be loaded when the WebView starts.
///
/// # WebView2 Extension Support
///
/// WebView2 supports loading unpacked Chrome extensions from a directory.
/// Each subdirectory in the extensions folder should contain a valid Chrome extension
/// (with manifest.json).
///
/// ## Directory Structure
/// ```text
/// %LOCALAPPDATA%/AuroraView/Extensions/
/// ├── my-extension-1/
/// │   ├── manifest.json
/// │   └── ...
/// └── my-extension-2/
///     ├── manifest.json
///     └── ...
/// ```
#[cfg(target_os = "windows")]
pub fn get_extensions_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("AuroraView")
        .join("Extensions")
}

/// Get extension enabled/disabled configuration file path
#[cfg(target_os = "windows")]
pub fn get_extension_config_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("AuroraView")
        .join("extension_config.json")
}

/// Load disabled extension IDs from config file
#[cfg(target_os = "windows")]
pub fn load_disabled_extensions() -> std::collections::HashSet<String> {
    let path = get_extension_config_path();
    if !path.exists() {
        return std::collections::HashSet::new();
    }

    let Ok(content) = std::fs::read_to_string(&path) else {
        tracing::warn!(
            "[Extensions] Failed to read config file: {}",
            path.display()
        );
        return std::collections::HashSet::new();
    };

    let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
        tracing::warn!(
            "[Extensions] Failed to parse config JSON: {}",
            path.display()
        );
        return std::collections::HashSet::new();
    };

    json.get("disabled_extensions")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

/// Check if the given directory contains at least one valid extension directory
#[cfg(target_os = "windows")]
pub fn has_extensions_in_dir(dir: &Path) -> bool {
    if !dir.exists() {
        return false;
    }

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join("manifest.json").exists() {
                return true;
            }
        }
    }
    false
}

/// Build per-app active extension directory that contains only enabled extensions
#[cfg(target_os = "windows")]
pub fn prepare_active_extensions_dir(runtime_enabled: bool) -> std::io::Result<PathBuf> {
    let source_dir = get_extensions_dir();
    let app_name = std::env::current_exe()
        .ok()
        .and_then(|p| {
            p.file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "auroraview".to_string());

    let active_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("AuroraView")
        .join("ExtensionsActive")
        .join(app_name);

    if active_dir.exists() {
        std::fs::remove_dir_all(&active_dir)?;
    }
    std::fs::create_dir_all(&active_dir)?;

    if !runtime_enabled || !source_dir.exists() {
        return Ok(active_dir);
    }

    let disabled = load_disabled_extensions();

    if let Ok(entries) = std::fs::read_dir(&source_dir) {
        for entry in entries.flatten() {
            let src_path = entry.path();
            if !src_path.is_dir() || !src_path.join("manifest.json").exists() {
                continue;
            }

            let ext_id = src_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_string();

            if disabled.contains(&ext_id) {
                tracing::info!("[Extensions] Skip disabled extension: {}", ext_id);
                continue;
            }

            let dst_path = active_dir.join(&ext_id);
            copy_dir_recursive(&src_path, &dst_path)?;
        }
    }

    Ok(active_dir)
}

#[cfg(target_os = "windows")]
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if path.is_dir() {
            copy_dir_recursive(&path, &dst_path)?;
        } else {
            if let Some(parent) = dst_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(&path, &dst_path)?;
        }
    }
    Ok(())
}

/// Inject environment variables from config
pub fn inject_environment_variables(env: &std::collections::HashMap<String, String>) {
    for (key, value) in env {
        tracing::debug!("Setting env: {}={}", key, value);
        std::env::set_var(key, value);
    }

    if !env.is_empty() {
        tracing::info!("Injected {} environment variables", env.len());
    }
}

/// Context for expanding path variables in isolation config
#[derive(Debug, Clone)]
pub struct IsolationContext {
    /// The directory where Python files are extracted
    pub extract_dir: PathBuf,
    /// The resources directory
    pub resources_dir: PathBuf,
    /// The Python home directory (parent of python.exe)
    pub python_home: PathBuf,
    /// The site-packages directory
    pub site_packages_dir: PathBuf,
    /// Built PYTHONPATH (from module_search_paths)
    pub pythonpath: String,
}

impl IsolationContext {
    /// Expand special variables in a path string
    pub fn expand_path(&self, path: &str) -> String {
        path.replace("$EXTRACT_DIR", &self.extract_dir.to_string_lossy())
            .replace("$RESOURCES_DIR", &self.resources_dir.to_string_lossy())
            .replace("$PYTHON_HOME", &self.python_home.to_string_lossy())
            .replace("$SITE_PACKAGES", &self.site_packages_dir.to_string_lossy())
    }
}

/// Build an isolated PATH string based on isolation config
///
/// When isolation is enabled:
/// - Starts with bundled paths (Python home, external binaries)
/// - Adds system essential paths from config
/// - Does NOT inherit host PATH
///
/// When isolation is disabled:
/// - Prepends bundled paths to existing PATH
pub fn build_isolated_path(isolation: &IsolationConfig, context: &IsolationContext) -> String {
    let separator = if cfg!(windows) { ";" } else { ":" };
    let mut paths: Vec<String> = Vec::new();

    // Always add Python home bin directory first (for python.exe)
    #[cfg(windows)]
    {
        paths.push(context.python_home.to_string_lossy().to_string());
        // Also add Scripts directory for pip-installed executables
        let scripts_dir = context.python_home.join("Scripts");
        if scripts_dir.exists() {
            paths.push(scripts_dir.to_string_lossy().to_string());
        }
    }
    #[cfg(not(windows))]
    {
        let bin_dir = context.python_home.join("bin");
        paths.push(bin_dir.to_string_lossy().to_string());
    }

    // Add extra paths from config (expanded)
    for extra in &isolation.extra_path {
        let expanded = context.expand_path(extra);
        if !expanded.is_empty() && Path::new(&expanded).exists() {
            paths.push(expanded);
        }
    }

    if isolation.path {
        // Add system essential paths
        for system_path in &isolation.system_path {
            if Path::new(system_path).exists() {
                paths.push(system_path.clone());
            }
        }
        tracing::debug!(
            "[IsolatedEnv] PATH isolated: {} entries (system essentials only)",
            paths.len()
        );
    } else {
        // Inherit host PATH
        if let Ok(host_path) = std::env::var("PATH") {
            paths.push(host_path);
        }
        tracing::debug!("[IsolatedEnv] PATH inherited from host");
    }

    paths.join(separator)
}

/// Build an isolated PYTHONPATH string based on isolation config
///
/// When isolation is enabled:
/// - Uses only the configured module_search_paths
/// - Does NOT inherit host PYTHONPATH
///
/// When isolation is disabled:
/// - Prepends bundled paths to existing PYTHONPATH
pub fn build_isolated_pythonpath(
    isolation: &IsolationConfig,
    context: &IsolationContext,
) -> String {
    let separator = if cfg!(windows) { ";" } else { ":" };
    let mut paths: Vec<String> = Vec::new();

    // Always add the pre-built pythonpath from module_search_paths
    if !context.pythonpath.is_empty() {
        for p in context.pythonpath.split(separator) {
            if !p.is_empty() {
                paths.push(p.to_string());
            }
        }
    }

    // Add extra PYTHONPATH entries from isolation config
    for extra in &isolation.extra_pythonpath {
        let expanded = context.expand_path(extra);
        if !expanded.is_empty() && Path::new(&expanded).exists() {
            paths.push(expanded);
        }
    }

    if !isolation.pythonpath {
        // Inherit host PYTHONPATH
        if let Ok(host_pythonpath) = std::env::var("PYTHONPATH") {
            if !host_pythonpath.is_empty() {
                paths.push(host_pythonpath);
            }
        }
        tracing::debug!("[IsolatedEnv] PYTHONPATH inherited from host");
    } else {
        tracing::debug!("[IsolatedEnv] PYTHONPATH isolated: {} entries", paths.len());
    }

    paths.join(separator)
}

/// Apply environment isolation to a Command
///
/// This configures the Command's environment based on the isolation settings.
/// It handles:
/// - PATH isolation
/// - PYTHONPATH isolation
/// - Environment variable inheritance
/// - Environment variable clearing
pub fn apply_isolation_to_command(
    cmd: &mut Command,
    isolation: &IsolationConfig,
    context: &IsolationContext,
) {
    // Build isolated paths
    let isolated_path = build_isolated_path(isolation, context);
    let isolated_pythonpath = build_isolated_pythonpath(isolation, context);

    // Clear environment if full isolation is needed
    if isolation.path || isolation.pythonpath {
        // Start with a minimal environment
        // We need to selectively inherit only what's specified
        let inherited_vars: HashMap<String, String> = isolation
            .inherit_env
            .iter()
            .filter_map(|key| std::env::var(key).ok().map(|v| (key.clone(), v)))
            .collect();

        // Clear all env and set only what we want
        cmd.env_clear();

        // Re-add inherited variables
        for (key, value) in &inherited_vars {
            cmd.env(key, value);
        }
    }

    // Set the isolated PATH
    cmd.env("PATH", &isolated_path);
    tracing::debug!("[IsolatedEnv] PATH={}", isolated_path);

    // Set the isolated PYTHONPATH
    cmd.env("PYTHONPATH", &isolated_pythonpath);
    tracing::debug!("[IsolatedEnv] PYTHONPATH={}", isolated_pythonpath);

    // Set PYTHONHOME for python-build-standalone
    cmd.env("PYTHONHOME", &context.python_home);

    // Clear any explicitly specified variables
    for var in &isolation.clear_env {
        cmd.env_remove(var);
        tracing::debug!("[IsolatedEnv] Cleared env: {}", var);
    }
}

/// Store isolation context in environment variables for child processes
///
/// This allows ProcessPlugin.spawn_ipc to access the isolation context
/// when spawning child processes.
pub fn store_isolation_context_in_env(context: &IsolationContext, isolation: &IsolationConfig) {
    // Store the pre-built isolated paths
    let isolated_path = build_isolated_path(isolation, context);
    let isolated_pythonpath = build_isolated_pythonpath(isolation, context);

    std::env::set_var("AURORAVIEW_ISOLATED_PATH", &isolated_path);
    std::env::set_var("AURORAVIEW_ISOLATED_PYTHONPATH", &isolated_pythonpath);
    std::env::set_var("AURORAVIEW_PYTHON_HOME", &context.python_home);
    std::env::set_var("AURORAVIEW_EXTRACT_DIR", &context.extract_dir);
    std::env::set_var("AURORAVIEW_RESOURCES_DIR", &context.resources_dir);

    // Store isolation flags
    let isolate_path_flag = if isolation.path { "1" } else { "0" };
    let isolate_pythonpath_flag = if isolation.pythonpath { "1" } else { "0" };

    std::env::set_var("AURORAVIEW_ISOLATE_PATH", isolate_path_flag);
    std::env::set_var("AURORAVIEW_ISOLATE_PYTHONPATH", isolate_pythonpath_flag);

    // Store inherit_env list as comma-separated
    std::env::set_var("AURORAVIEW_INHERIT_ENV", isolation.inherit_env.join(","));

    tracing::info!(
        "[IsolatedEnv] Stored isolation context: isolate_path={}, isolate_pythonpath={}",
        isolate_path_flag,
        isolate_pythonpath_flag
    );
    tracing::debug!("[IsolatedEnv] ISOLATED_PATH={}", isolated_path);
    tracing::debug!("[IsolatedEnv] ISOLATED_PYTHONPATH={}", isolated_pythonpath);
}
