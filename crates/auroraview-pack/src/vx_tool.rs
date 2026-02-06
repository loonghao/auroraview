//! vx Tool Manager
//!
//! This module provides functionality to download and manage the vx tool,
//! which is used for managing Python packages via `vx uv pip install`.
//!
//! vx is downloaded from: https://github.com/loonghao/vx/releases
//! Latest version is automatically detected from GitHub API.

#![allow(dead_code)]

use crate::{PackError, PackResult};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};

/// Default vx version fallback when API is unavailable
const VX_DEFAULT_VERSION: &str = "0.6.27";

/// GitHub API URL for latest release
const VX_GITHUB_API_URL: &str = "https://api.github.com/repos/loonghao/vx/releases/latest";

/// Cache duration for version check (24 hours)
const VERSION_CACHE_DURATION: Duration = Duration::from_secs(24 * 60 * 60);

/// Version cache structure for storing latest version info
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct VersionCache {
    version: String,
    timestamp: SystemTime,
}

/// vx download URL templates
#[cfg(target_os = "windows")]
const VX_DOWNLOAD_URL: &str =
    "https://github.com/loonghao/vx/releases/download/vx-v{version}/vx-x86_64-pc-windows-msvc.zip";

#[cfg(target_os = "linux")]
const VX_DOWNLOAD_URL: &str =
    "https://github.com/loonghao/vx/releases/download/vx-v{version}/vx-x86_64-unknown-linux-gnu.tar.gz";

#[cfg(target_os = "macos")]
const VX_DOWNLOAD_URL: &str =
    "https://github.com/loonghao/vx/releases/download/vx-v{version}/vx-x86_64-apple-darwin.tar.gz";

/// vx tool manager
///
/// This struct wraps the vx tool for managing Python packages.
pub struct VxTool {
    /// Path to the vx executable
    vx_path: PathBuf,
    /// Version of vx being used
    version: String,
}

impl VxTool {
    /// Create a new VxTool, downloading vx if necessary
    pub fn new() -> PackResult<Self> {
        let vx_path = Self::ensure_vx()?;
        let version = Self::detect_version(&vx_path)?;
        Ok(Self { vx_path, version })
    }

    /// Create a VxTool with a custom vx path
    pub fn with_vx_path(path: PathBuf) -> PackResult<Self> {
        if !path.exists() {
            return Err(PackError::Config(format!(
                "vx not found at: {}",
                path.display()
            )));
        }
        let version = Self::detect_version(&path)?;
        Ok(Self {
            vx_path: path,
            version,
        })
    }

    /// Get the path to the vx executable
    pub fn path(&self) -> &Path {
        &self.vx_path
    }

    /// Get the version of vx
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Minimum expected size for vx executable (should be ~7MB)
    const VX_MIN_SIZE: u64 = 1_000_000;

    /// Detect vx version by running `vx --version`
    fn detect_version(vx_path: &Path) -> PackResult<String> {
        let output = Command::new(vx_path)
            .args(["--version"])
            .output()
            .map_err(|e| PackError::Config(format!("Failed to run vx --version: {}", e)))?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(version)
        } else {
            Ok(VX_DEFAULT_VERSION.to_string())
        }
    }

    /// Get the cache directory for vx
    fn get_cache_dir() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("auroraview")
            .join("tools")
    }

    /// Get the version cache file path
    fn get_version_cache_path() -> PathBuf {
        Self::get_cache_dir().join("vx_version_cache.json")
    }

    /// Fetch the latest vx version from GitHub API
    fn fetch_latest_version() -> PackResult<String> {
        tracing::info!("Fetching latest vx version from GitHub...");

        let response = ureq::get(VX_GITHUB_API_URL)
            .set("User-Agent", "auroraview-pack")
            .call()
            .map_err(|e| PackError::Config(format!("Failed to fetch vx version: {}", e)))?;

        let json: serde_json::Value =
            serde_json::from_reader(response.into_reader()).map_err(|e| {
                PackError::Config(format!("Failed to parse vx version response: {}", e))
            })?;

        let tag_name = json
            .get("tag_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PackError::Config("Missing tag_name in vx release".to_string()))?;

        // Extract version from tag (e.g., "vx-v0.6.27" -> "0.6.27")
        let version = tag_name
            .trim_start_matches("vx-v")
            .trim_start_matches('v')
            .to_string();

        tracing::info!("Latest vx version: {}", version);
        Ok(version)
    }

    /// Get the cached or latest vx version
    fn get_version() -> String {
        // Check cache first
        let cache_path = Self::get_version_cache_path();
        if let Ok(content) = fs::read_to_string(&cache_path) {
            if let Ok(cache) = serde_json::from_str::<VersionCache>(&content) {
                if let Ok(elapsed) = cache.timestamp.elapsed() {
                    if elapsed < VERSION_CACHE_DURATION {
                        tracing::debug!("Using cached vx version: {}", cache.version);
                        return cache.version;
                    }
                }
            }
        }

        // Fetch latest version
        match Self::fetch_latest_version() {
            Ok(version) => {
                // Update cache
                let cache = VersionCache {
                    version: version.clone(),
                    timestamp: SystemTime::now(),
                };
                let _ = fs::create_dir_all(cache_path.parent().unwrap_or(Path::new(".")));
                let _ = fs::write(
                    &cache_path,
                    serde_json::to_string_pretty(&cache).unwrap_or_default(),
                );
                version
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to fetch latest vx version: {}, using default {}",
                    e,
                    VX_DEFAULT_VERSION
                );
                VX_DEFAULT_VERSION.to_string()
            }
        }
    }

    /// Ensure vx is available, downloading if necessary
    fn ensure_vx() -> PackResult<PathBuf> {
        let cache_dir = Self::get_cache_dir();
        fs::create_dir_all(&cache_dir)?;

        #[cfg(target_os = "windows")]
        let vx_exe_name = "vx.exe";
        #[cfg(not(target_os = "windows"))]
        let vx_exe_name = "vx";

        let vx_path = cache_dir.join(vx_exe_name);

        // Get the version to use
        let version = Self::get_version();

        // Check if already downloaded and valid (and matches version)
        if vx_path.exists() {
            if let Ok(metadata) = fs::metadata(&vx_path) {
                if metadata.len() >= Self::VX_MIN_SIZE {
                    // Check if version matches
                    if let Ok(detected_version) = Self::detect_version(&vx_path) {
                        if detected_version.contains(&version) {
                            tracing::debug!(
                                "Using cached vx {} at: {}",
                                version,
                                vx_path.display()
                            );
                            return Ok(vx_path);
                        }
                        tracing::info!(
                            "vx version mismatch (cached: {}, required: {}), updating...",
                            detected_version,
                            version
                        );
                    }
                } else {
                    tracing::warn!(
                        "Cached vx is too small ({} bytes), re-downloading...",
                        metadata.len()
                    );
                    let _ = fs::remove_file(&vx_path);
                }
            }
        }

        // Download vx
        tracing::info!("Downloading vx v{}...", version);
        let url = VX_DOWNLOAD_URL.replace("{version}", &version);

        let archive_data = Self::download_file(&url)?;

        // Extract the executable from the archive
        let exe_data = Self::extract_executable(&archive_data)?;

        // Validate downloaded size
        if (exe_data.len() as u64) < Self::VX_MIN_SIZE {
            return Err(PackError::Config(format!(
                "Downloaded vx is too small ({} bytes), expected at least {} bytes",
                exe_data.len(),
                Self::VX_MIN_SIZE
            )));
        }

        let mut file = fs::File::create(&vx_path)?;
        file.write_all(&exe_data)?;

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&vx_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&vx_path, perms)?;
        }

        tracing::info!(
            "vx v{} downloaded to: {} ({} bytes)",
            version,
            vx_path.display(),
            exe_data.len()
        );
        Ok(vx_path)
    }

    /// Execute a vx command with args
    pub fn exec(&self, args: &[&str]) -> PackResult<std::process::Output> {
        Command::new(&self.vx_path)
            .args(args)
            .output()
            .map_err(|e| PackError::Config(format!("Failed to run vx: {}", e)))
    }

    /// Execute a vx uv pip install command
    pub fn uv_pip_install(
        &self,
        packages: &[String],
        target_dir: &Path,
        no_deps: bool,
    ) -> PackResult<bool> {
        if packages.is_empty() {
            return Ok(true);
        }

        let mut args = vec![
            "uv".to_string(),
            "pip".to_string(),
            "install".to_string(),
            "--target".to_string(),
            target_dir.to_string_lossy().to_string(),
            "--no-compile".to_string(),
        ];

        if no_deps {
            args.push("--no-deps".to_string());
        }

        let status = Command::new(&self.vx_path)
            .args(&args)
            .args(packages)
            .status()
            .map_err(|e| PackError::Config(format!("Failed to run vx uv pip: {}", e)))?;

        Ok(status.success())
    }

    /// Execute vx uv pip install from requirements file
    pub fn uv_pip_install_requirements(
        &self,
        requirements_file: &Path,
        target_dir: &Path,
    ) -> PackResult<bool> {
        if !requirements_file.exists() {
            return Ok(true);
        }

        let status = Command::new(&self.vx_path)
            .args([
                "uv".to_string(),
                "pip".to_string(),
                "install".to_string(),
                "--target".to_string(),
                target_dir.to_string_lossy().to_string(),
                "--no-compile".to_string(),
                "-r".to_string(),
                requirements_file.to_string_lossy().to_string(),
            ])
            .status()
            .map_err(|e| PackError::Config(format!("Failed to run vx uv pip: {}", e)))?;

        Ok(status.success())
    }

    /// Create a virtual environment using vx uv
    pub fn uv_venv(&self, venv_path: &Path, python_version: Option<&str>) -> PackResult<bool> {
        let mut args = vec!["uv".to_string(), "venv".to_string()];

        if let Some(version) = python_version {
            args.push(format!("--python={}", version));
        }

        args.push(venv_path.to_string_lossy().to_string());

        let status = Command::new(&self.vx_path)
            .args(&args)
            .status()
            .map_err(|e| PackError::Config(format!("Failed to create venv with vx uv: {}", e)))?;

        Ok(status.success())
    }

    /// Download a file from URL
    fn download_file(url: &str) -> PackResult<Vec<u8>> {
        #[cfg(target_os = "windows")]
        {
            let temp_file = std::env::temp_dir().join("vx-download.zip");
            let output = Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-NonInteractive",
                    "-Command",
                    &format!(
                        "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; \
                         Invoke-WebRequest -Uri '{}' -OutFile '{}' -UseBasicParsing",
                        url,
                        temp_file.display()
                    ),
                ])
                .output()
                .map_err(|e| PackError::Config(format!("Failed to run PowerShell: {}", e)))?;

            if !output.status.success() {
                return Err(PackError::Config(format!(
                    "Failed to download vx: {}",
                    String::from_utf8_lossy(&output.stderr)
                )));
            }

            let data = fs::read(&temp_file)?;
            let _ = fs::remove_file(&temp_file);
            Ok(data)
        }

        #[cfg(not(target_os = "windows"))]
        {
            let output = Command::new("curl")
                .args(["-fsSL", url])
                .output()
                .map_err(|e| PackError::Config(format!("Failed to run curl: {}", e)))?;

            if !output.status.success() {
                return Err(PackError::Config(format!(
                    "Failed to download vx: {}",
                    String::from_utf8_lossy(&output.stderr)
                )));
            }

            Ok(output.stdout)
        }
    }

    /// Extract the vx executable from the downloaded archive
    fn extract_executable(archive_data: &[u8]) -> PackResult<Vec<u8>> {
        #[cfg(target_os = "windows")]
        {
            // Windows: ZIP archive
            use std::io::{Cursor, Read};
            let cursor = Cursor::new(archive_data);
            let mut archive = zip::ZipArchive::new(cursor)
                .map_err(|e| PackError::Config(format!("Failed to open zip archive: {}", e)))?;

            for i in 0..archive.len() {
                let mut file = archive
                    .by_index(i)
                    .map_err(|e| PackError::Config(format!("Failed to read zip entry: {}", e)))?;

                let name = file.name().to_string();
                if name.ends_with("vx.exe") || name == "vx.exe" {
                    let mut data = Vec::new();
                    file.read_to_end(&mut data).map_err(|e| {
                        PackError::Config(format!("Failed to extract vx.exe: {}", e))
                    })?;
                    return Ok(data);
                }
            }

            Err(PackError::Config("vx.exe not found in archive".to_string()))
        }

        #[cfg(not(target_os = "windows"))]
        {
            // Unix: tar.gz archive
            use flate2::read::GzDecoder;
            use std::io::{Cursor, Read};
            use tar::Archive;

            let cursor = Cursor::new(archive_data);
            let gz = GzDecoder::new(cursor);
            let mut archive = Archive::new(gz);

            for entry in archive
                .entries()
                .map_err(|e| PackError::Config(format!("Failed to read tar archive: {}", e)))?
            {
                let mut entry = entry
                    .map_err(|e| PackError::Config(format!("Failed to read tar entry: {}", e)))?;

                let path = entry
                    .path()
                    .map_err(|e| PackError::Config(format!("Failed to get entry path: {}", e)))?;

                let name = path.to_string_lossy();
                if name.ends_with("/vx") || name == "vx" {
                    let mut data = Vec::new();
                    entry
                        .read_to_end(&mut data)
                        .map_err(|e| PackError::Config(format!("Failed to extract vx: {}", e)))?;
                    return Ok(data);
                }
            }

            Err(PackError::Config("vx not found in archive".to_string()))
        }
    }

    /// Install packages using vx uv pip install (backward compatible alias)
    pub fn pip_install(&self, packages: &[String], target_dir: &Path) -> PackResult<bool> {
        self.uv_pip_install(packages, target_dir, false)
    }

    /// Run a generic command through vx (e.g., "vx npm install", "vx go build")
    pub fn run(&self, tool: &str, args: &[&str]) -> PackResult<bool> {
        let mut cmd_args = vec![tool.to_string()];
        cmd_args.extend(args.iter().map(|&s| s.to_string()));

        let status = Command::new(&self.vx_path)
            .args(&cmd_args)
            .status()
            .map_err(|e| PackError::Config(format!("Failed to run vx {}: {}", tool, e)))?;

        Ok(status.success())
    }

    /// Check if vx is available in the system PATH
    pub fn is_system_vx_available() -> bool {
        Command::new("vx")
            .args(["--version"])
            .output()
            .is_ok_and(|o| o.status.success())
    }

    /// Get the latest vx version (public API)
    pub fn latest_version() -> String {
        Self::get_version()
    }

    /// Force update to latest version (ignore cache)
    pub fn update_to_latest() -> PackResult<Self> {
        // Clear version cache
        let cache_path = Self::get_version_cache_path();
        let _ = fs::remove_file(&cache_path);

        // Remove existing vx executable to force re-download
        let vx_path = Self::get_cache_dir().join(if cfg!(target_os = "windows") {
            "vx.exe"
        } else {
            "vx"
        });
        if vx_path.exists() {
            let _ = fs::remove_file(&vx_path);
        }

        Self::new()
    }
}
