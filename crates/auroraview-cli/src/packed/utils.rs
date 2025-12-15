//! Utility functions for packed mode
//!
//! Includes path helpers, environment variable injection, and string escaping.

use std::path::{Path, PathBuf};

/// Escape JSON string for embedding in JavaScript
pub fn escape_json_for_js(json: &str) -> String {
    json.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Get the runtime cache directory for an app
pub fn get_runtime_cache_dir(app_name: &str) -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("AuroraView")
        .join("runtime")
        .join(app_name)
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

/// Get the directory for extracting Python files
pub fn get_python_extract_dir() -> PathBuf {
    // Use a unique directory based on the executable path
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
