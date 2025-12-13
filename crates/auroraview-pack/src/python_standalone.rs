//! Python Standalone Runtime Integration
//!
//! Downloads and embeds python-build-standalone distributions for
//! fully offline, single-file executable deployment.
//!
//! # Design
//!
//! 1. **Pack time**: Download pre-built Python from python-build-standalone
//! 2. **Embed**: Compress and store in overlay alongside app code
//! 3. **Runtime**: Extract to cache directory on first run, reuse thereafter
//!
//! # Supported Distributions
//!
//! - Windows x86_64: `cpython-{version}+{release}-x86_64-pc-windows-msvc-install_only.tar.gz`
//! - Linux x86_64: `cpython-{version}+{release}-x86_64-unknown-linux-gnu-install_only.tar.gz`
//! - macOS x86_64: `cpython-{version}+{release}-x86_64-apple-darwin-install_only.tar.gz`
//! - macOS arm64: `cpython-{version}+{release}-aarch64-apple-darwin-install_only.tar.gz`

use crate::{PackError, PackResult};
use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};

/// Python standalone distribution configuration
#[derive(Debug, Clone)]
pub struct PythonStandaloneConfig {
    /// Python version (e.g., "3.11", "3.12")
    pub version: String,
    /// Release tag (e.g., "20241206")
    pub release: Option<String>,
    /// Target platform (auto-detected if None)
    pub target: Option<String>,
    /// Cache directory for downloaded distributions
    pub cache_dir: Option<PathBuf>,
}

impl Default for PythonStandaloneConfig {
    fn default() -> Self {
        Self {
            version: "3.11".to_string(),
            release: None,
            target: None,
            cache_dir: None,
        }
    }
}

/// Target platform for Python distribution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PythonTarget {
    WindowsX64,
    LinuxX64,
    MacOSX64,
    MacOSArm64,
}

impl PythonTarget {
    /// Detect current platform
    pub fn current() -> PackResult<Self> {
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        return Ok(Self::WindowsX64);

        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        return Ok(Self::LinuxX64);

        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        return Ok(Self::MacOSX64);

        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        return Ok(Self::MacOSArm64);

        #[cfg(not(any(
            all(target_os = "windows", target_arch = "x86_64"),
            all(target_os = "linux", target_arch = "x86_64"),
            all(target_os = "macos", target_arch = "x86_64"),
            all(target_os = "macos", target_arch = "aarch64"),
        )))]
        Err(PackError::Config(
            "Unsupported platform for Python standalone".to_string(),
        ))
    }

    /// Get the triple string for this target
    pub fn triple(&self) -> &'static str {
        match self {
            Self::WindowsX64 => "x86_64-pc-windows-msvc",
            Self::LinuxX64 => "x86_64-unknown-linux-gnu",
            Self::MacOSX64 => "x86_64-apple-darwin",
            Self::MacOSArm64 => "aarch64-apple-darwin",
        }
    }

    /// Get the Python executable name
    pub fn python_exe(&self) -> &'static str {
        match self {
            Self::WindowsX64 => "python.exe",
            _ => "python3",
        }
    }

    /// Get the relative path to Python executable within the distribution
    pub fn python_path(&self) -> &'static str {
        match self {
            Self::WindowsX64 => "python/python.exe",
            _ => "python/bin/python3",
        }
    }
}

/// Python standalone distribution manager
pub struct PythonStandalone {
    config: PythonStandaloneConfig,
    target: PythonTarget,
}

impl PythonStandalone {
    /// Create a new Python standalone manager
    pub fn new(config: PythonStandaloneConfig) -> PackResult<Self> {
        let target = if let Some(ref target_str) = config.target {
            match target_str.as_str() {
                "x86_64-pc-windows-msvc" => PythonTarget::WindowsX64,
                "x86_64-unknown-linux-gnu" => PythonTarget::LinuxX64,
                "x86_64-apple-darwin" => PythonTarget::MacOSX64,
                "aarch64-apple-darwin" => PythonTarget::MacOSArm64,
                _ => return Err(PackError::Config(format!("Unknown target: {}", target_str))),
            }
        } else {
            PythonTarget::current()?
        };

        Ok(Self { config, target })
    }

    /// Get the download URL for the Python distribution
    pub fn download_url(&self) -> String {
        let version = &self.config.version;
        let release = self
            .config
            .release
            .clone()
            .unwrap_or_else(get_latest_release);
        let triple = self.target.triple();

        format!(
            "https://github.com/indygreg/python-build-standalone/releases/download/{release}/cpython-{version}+{release}-{triple}-install_only.tar.gz"
        )
    }

    /// Get the cache directory for downloaded distributions
    pub fn cache_dir(&self) -> PathBuf {
        self.config.cache_dir.clone().unwrap_or_else(|| {
            dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("AuroraView")
                .join("python-standalone")
        })
    }

    /// Get the cached distribution path
    pub fn cached_path(&self) -> PathBuf {
        let filename = format!(
            "cpython-{}-{}.tar.gz",
            self.config.version,
            self.target.triple()
        );
        self.cache_dir().join(filename)
    }

    /// Download the Python distribution if not cached
    pub fn download(&self) -> PackResult<PathBuf> {
        let cache_path = self.cached_path();

        // Check if already cached
        if cache_path.exists() {
            tracing::info!("Using cached Python distribution: {}", cache_path.display());
            return Ok(cache_path);
        }

        // Create cache directory
        fs::create_dir_all(self.cache_dir())?;

        let url = self.download_url();
        tracing::info!("Downloading Python distribution from: {}", url);

        // Download using system tools (curl/wget/powershell)
        download_file(&url, &cache_path)?;

        tracing::info!("Downloaded to: {}", cache_path.display());
        Ok(cache_path)
    }

    /// Extract the Python distribution to a directory
    pub fn extract(&self, dest_dir: &Path) -> PackResult<PathBuf> {
        let archive_path = self.download()?;

        tracing::info!("Extracting Python to: {}", dest_dir.display());

        // Create destination directory
        fs::create_dir_all(dest_dir)?;

        // Extract tar.gz
        extract_tar_gz(&archive_path, dest_dir)?;

        // Return path to python executable
        let python_path = dest_dir.join(self.target.python_path());
        if !python_path.exists() {
            return Err(PackError::Config(format!(
                "Python executable not found at: {}",
                python_path.display()
            )));
        }

        Ok(python_path)
    }

    /// Get the Python distribution as bytes for embedding
    pub fn get_distribution_bytes(&self) -> PackResult<Vec<u8>> {
        let archive_path = self.download()?;
        let content = fs::read(&archive_path)?;
        Ok(content)
    }

    /// Get target information
    pub fn target(&self) -> PythonTarget {
        self.target
    }

    /// Get Python version
    pub fn version(&self) -> &str {
        &self.config.version
    }
}

/// Get the latest release tag from python-build-standalone
fn get_latest_release() -> String {
    // Default to a known stable release
    // In production, this could query the GitHub API
    "20241206".to_string()
}

/// Download a file using system tools
fn download_file(url: &str, dest: &Path) -> PackResult<()> {
    // Try different download methods based on platform
    #[cfg(target_os = "windows")]
    {
        // Use PowerShell on Windows
        let status = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                &format!(
                    "Invoke-WebRequest -Uri '{}' -OutFile '{}' -UseBasicParsing",
                    url,
                    dest.display()
                ),
            ])
            .status()
            .map_err(|e| PackError::Download(format!("Failed to run PowerShell: {}", e)))?;

        if !status.success() {
            return Err(PackError::Download(format!(
                "PowerShell download failed with status: {}",
                status
            )));
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Try curl first, then wget
        let curl_result = std::process::Command::new("curl")
            .args(["-fsSL", "-o", dest.to_str().unwrap_or("."), url])
            .status();

        match curl_result {
            Ok(status) if status.success() => {}
            _ => {
                // Fallback to wget
                let wget_status = std::process::Command::new("wget")
                    .args(["-q", "-O", dest.to_str().unwrap_or("."), url])
                    .status()
                    .map_err(|e| {
                        PackError::Download(format!("Failed to download (no curl/wget): {}", e))
                    })?;

                if !wget_status.success() {
                    return Err(PackError::Download(format!(
                        "wget download failed with status: {}",
                        wget_status
                    )));
                }
            }
        }
    }

    Ok(())
}

/// Extract a tar.gz archive
fn extract_tar_gz(archive_path: &Path, dest_dir: &Path) -> PackResult<()> {
    let file = File::open(archive_path)?;
    let reader = BufReader::new(file);

    // Decompress gzip
    let decoder = flate2::read::GzDecoder::new(reader);

    // Extract tar
    let mut archive = tar::Archive::new(decoder);
    archive.unpack(dest_dir)?;

    Ok(())
}

/// Runtime: Extract embedded Python distribution to cache
pub fn extract_runtime(
    python_archive: &[u8],
    app_name: &str,
    version: &str,
) -> PackResult<PathBuf> {
    let cache_dir = get_runtime_cache_dir(app_name);
    let version_marker = cache_dir.join(".version");

    // Check if already extracted with correct version
    if version_marker.exists() {
        if let Ok(cached_version) = fs::read_to_string(&version_marker) {
            if cached_version.trim() == version {
                let python_path = get_python_exe_path(&cache_dir);
                if python_path.exists() {
                    tracing::debug!("Using cached Python runtime: {}", cache_dir.display());
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

    // Decompress and extract
    let decoder = flate2::read::GzDecoder::new(python_archive);
    let mut archive = tar::Archive::new(decoder);
    archive.unpack(&cache_dir)?;

    // Write version marker
    fs::write(&version_marker, version)?;

    let python_path = get_python_exe_path(&cache_dir);
    if !python_path.exists() {
        return Err(PackError::Config(format!(
            "Python executable not found after extraction: {}",
            python_path.display()
        )));
    }

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&python_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&python_path, perms)?;
    }

    Ok(python_path)
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
fn get_python_exe_path(cache_dir: &Path) -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        cache_dir.join("python").join("python.exe")
    }
    #[cfg(not(target_os = "windows"))]
    {
        cache_dir.join("python").join("bin").join("python3")
    }
}

/// Metadata stored in overlay for Python runtime
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PythonRuntimeMeta {
    /// Python version
    pub version: String,
    /// Target platform triple
    pub target: String,
    /// Size of the compressed archive
    pub archive_size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_detection() {
        // Should not panic on supported platforms
        let result = PythonTarget::current();
        #[cfg(any(
            all(target_os = "windows", target_arch = "x86_64"),
            all(target_os = "linux", target_arch = "x86_64"),
            all(target_os = "macos", target_arch = "x86_64"),
            all(target_os = "macos", target_arch = "aarch64"),
        ))]
        assert!(result.is_ok());
    }

    #[test]
    fn test_download_url() {
        let config = PythonStandaloneConfig {
            version: "3.11.11".to_string(),
            release: Some("20241206".to_string()),
            target: Some("x86_64-pc-windows-msvc".to_string()),
            cache_dir: None,
        };

        let standalone = PythonStandalone::new(config).unwrap();
        let url = standalone.download_url();

        assert!(url.contains("cpython-3.11.11"));
        assert!(url.contains("20241206"));
        assert!(url.contains("x86_64-pc-windows-msvc"));
    }

    #[test]
    fn test_python_paths() {
        assert_eq!(PythonTarget::WindowsX64.python_exe(), "python.exe");
        assert_eq!(PythonTarget::LinuxX64.python_exe(), "python3");
        assert_eq!(PythonTarget::WindowsX64.python_path(), "python/python.exe");
        assert_eq!(PythonTarget::LinuxX64.python_path(), "python/bin/python3");
    }

    #[test]
    fn test_target_triples() {
        assert_eq!(PythonTarget::WindowsX64.triple(), "x86_64-pc-windows-msvc");
        assert_eq!(PythonTarget::LinuxX64.triple(), "x86_64-unknown-linux-gnu");
        assert_eq!(PythonTarget::MacOSX64.triple(), "x86_64-apple-darwin");
        assert_eq!(PythonTarget::MacOSArm64.triple(), "aarch64-apple-darwin");
    }

    #[test]
    fn test_macos_python_paths() {
        assert_eq!(PythonTarget::MacOSX64.python_exe(), "python3");
        assert_eq!(PythonTarget::MacOSArm64.python_exe(), "python3");
        assert_eq!(PythonTarget::MacOSX64.python_path(), "python/bin/python3");
        assert_eq!(PythonTarget::MacOSArm64.python_path(), "python/bin/python3");
    }

    #[test]
    fn test_config_default() {
        let config = PythonStandaloneConfig::default();
        assert_eq!(config.version, "3.11");
        assert!(config.release.is_none());
        assert!(config.target.is_none());
        assert!(config.cache_dir.is_none());
    }

    #[test]
    fn test_standalone_new_with_target() {
        let config = PythonStandaloneConfig {
            version: "3.12".to_string(),
            release: Some("20241206".to_string()),
            target: Some("x86_64-unknown-linux-gnu".to_string()),
            cache_dir: None,
        };

        let standalone = PythonStandalone::new(config).unwrap();
        assert_eq!(standalone.target(), PythonTarget::LinuxX64);
        assert_eq!(standalone.version(), "3.12");
    }

    #[test]
    fn test_standalone_invalid_target() {
        let config = PythonStandaloneConfig {
            version: "3.11".to_string(),
            release: None,
            target: Some("invalid-target".to_string()),
            cache_dir: None,
        };

        let result = PythonStandalone::new(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_cached_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = PythonStandaloneConfig {
            version: "3.11".to_string(),
            release: None,
            target: Some("x86_64-pc-windows-msvc".to_string()),
            cache_dir: Some(temp_dir.path().to_path_buf()),
        };

        let standalone = PythonStandalone::new(config).unwrap();
        let cached = standalone.cached_path();

        assert!(cached.to_string_lossy().contains("cpython-3.11"));
        assert!(cached.to_string_lossy().contains("x86_64-pc-windows-msvc"));
        assert!(cached.to_string_lossy().ends_with(".tar.gz"));
    }

    #[test]
    fn test_download_url_all_targets() {
        let targets = [
            ("x86_64-pc-windows-msvc", "x86_64-pc-windows-msvc"),
            ("x86_64-unknown-linux-gnu", "x86_64-unknown-linux-gnu"),
            ("x86_64-apple-darwin", "x86_64-apple-darwin"),
            ("aarch64-apple-darwin", "aarch64-apple-darwin"),
        ];

        for (target_str, expected) in targets {
            let config = PythonStandaloneConfig {
                version: "3.11".to_string(),
                release: Some("20241206".to_string()),
                target: Some(target_str.to_string()),
                cache_dir: None,
            };

            let standalone = PythonStandalone::new(config).unwrap();
            let url = standalone.download_url();

            assert!(
                url.contains(expected),
                "URL should contain {}: {}",
                expected,
                url
            );
            assert!(url.contains("install_only.tar.gz"));
            assert!(url.starts_with("https://github.com/indygreg/python-build-standalone"));
        }
    }

    #[test]
    fn test_runtime_meta_serialization() {
        let meta = PythonRuntimeMeta {
            version: "3.11.11".to_string(),
            target: "x86_64-pc-windows-msvc".to_string(),
            archive_size: 50_000_000,
        };

        let json = serde_json::to_string(&meta).unwrap();
        assert!(json.contains("3.11.11"));
        assert!(json.contains("x86_64-pc-windows-msvc"));
        assert!(json.contains("50000000"));

        let parsed: PythonRuntimeMeta = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.version, meta.version);
        assert_eq!(parsed.target, meta.target);
        assert_eq!(parsed.archive_size, meta.archive_size);
    }

    #[test]
    fn test_runtime_cache_dir() {
        let cache_dir = get_runtime_cache_dir("test-app");
        assert!(cache_dir.to_string_lossy().contains("AuroraView"));
        assert!(cache_dir.to_string_lossy().contains("runtime"));
        assert!(cache_dir.to_string_lossy().contains("test-app"));
    }

    #[test]
    fn test_cache_dir_custom() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = PythonStandaloneConfig {
            version: "3.11".to_string(),
            release: None,
            target: Some("x86_64-pc-windows-msvc".to_string()),
            cache_dir: Some(temp_dir.path().to_path_buf()),
        };

        let standalone = PythonStandalone::new(config).unwrap();
        assert_eq!(standalone.cache_dir(), temp_dir.path());
    }
}
