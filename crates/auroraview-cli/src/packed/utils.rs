//! Utility functions for packed mode
//!
//! Includes path helpers, environment variable injection, string escaping,
//! and environment isolation (rez-style).

use auroraview_pack::IsolationConfig;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Escape JSON string for embedding in JavaScript
pub fn escape_json_for_js(json: &str) -> String {
    json.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

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
/// ```
/// %LOCALAPPDATA%/AuroraView/Extensions/
/// ├── my-extension-1/
/// │   ├── manifest.json
/// │   └── ...
/// └── my-extension-2/
///     ├── manifest.json
///     └── ...
/// ```
#[allow(dead_code)]
pub fn get_extensions_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("AuroraView")
        .join("Extensions")
}

/// Check if the extensions directory exists and has any extensions
#[allow(dead_code)]
pub fn has_extensions() -> bool {
    let ext_dir = get_extensions_dir();
    tracing::debug!(
        "[Extensions] Checking extensions directory: {}",
        ext_dir.display()
    );

    if !ext_dir.exists() {
        tracing::debug!("[Extensions] Extensions directory does not exist");
        return false;
    }

    // Check if there are any subdirectories with manifest.json
    if let Ok(entries) = std::fs::read_dir(&ext_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let manifest_path = path.join("manifest.json");
            tracing::debug!(
                "[Extensions] Checking: {} (is_dir={}, manifest_exists={})",
                path.display(),
                path.is_dir(),
                manifest_path.exists()
            );
            if path.is_dir() && manifest_path.exists() {
                tracing::info!("[Extensions] Found extension at: {}", path.display());
                return true;
            }
        }
    }
    tracing::debug!("[Extensions] No valid extensions found");
    false
}

/// List installed extensions in the extensions directory
#[allow(dead_code)]
pub fn list_extensions() -> Vec<ExtensionInfo> {
    let ext_dir = get_extensions_dir();
    let mut extensions = Vec::new();

    if !ext_dir.exists() {
        return extensions;
    }

    if let Ok(entries) = std::fs::read_dir(&ext_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let manifest_path = path.join("manifest.json");

            if path.is_dir() && manifest_path.exists() {
                // Read manifest to get extension info
                if let Ok(manifest_content) = std::fs::read_to_string(&manifest_path) {
                    if let Ok(manifest) =
                        serde_json::from_str::<serde_json::Value>(&manifest_content)
                    {
                        let name = manifest
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown")
                            .to_string();
                        let version = manifest
                            .get("version")
                            .and_then(|v| v.as_str())
                            .unwrap_or("0.0.0")
                            .to_string();
                        let description = manifest
                            .get("description")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();

                        extensions.push(ExtensionInfo {
                            id: path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown")
                                .to_string(),
                            name,
                            version,
                            description,
                            path: path.to_string_lossy().to_string(),
                        });
                    }
                }
            }
        }
    }

    extensions
}

/// Information about an installed extension
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[allow(dead_code)]
pub struct ExtensionInfo {
    /// Extension folder name (used as ID)
    pub id: String,
    /// Extension name from manifest
    pub name: String,
    /// Extension version from manifest
    pub version: String,
    /// Extension description from manifest
    pub description: String,
    /// Full path to extension folder
    pub path: String,
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
