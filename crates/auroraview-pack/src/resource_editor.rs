//! Windows executable resource editor
//!
//! This module provides functionality to modify Windows PE executable resources,
//! including icons, version information, and subsystem settings.
//!
//! It uses rcedit (https://github.com/electron/rcedit) as the underlying tool.

use crate::{PackError, PackResult};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

/// rcedit release version to download
const RCEDIT_VERSION: &str = "v2.0.0";

/// rcedit download URL template
const RCEDIT_DOWNLOAD_URL: &str =
    "https://github.com/electron/rcedit/releases/download/{version}/rcedit-x64.exe";

/// Windows executable resource editor
///
/// This struct wraps the rcedit tool for modifying PE resources.
pub struct ResourceEditor {
    /// Path to the rcedit executable
    rcedit_path: PathBuf,
}

impl ResourceEditor {
    /// Create a new ResourceEditor, downloading rcedit if necessary
    pub fn new() -> PackResult<Self> {
        let rcedit_path = Self::ensure_rcedit()?;
        Ok(Self { rcedit_path })
    }

    /// Create a ResourceEditor with a custom rcedit path
    pub fn with_rcedit_path(path: PathBuf) -> PackResult<Self> {
        if !path.exists() {
            return Err(PackError::ResourceEdit(format!(
                "rcedit not found at: {}",
                path.display()
            )));
        }
        Ok(Self { rcedit_path: path })
    }

    /// Minimum expected size for rcedit-x64.exe (should be ~1.3MB)
    const RCEDIT_MIN_SIZE: u64 = 500_000;

    /// Ensure rcedit is available, downloading if necessary
    fn ensure_rcedit() -> PackResult<PathBuf> {
        // Check cache directory
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("auroraview")
            .join("tools");

        fs::create_dir_all(&cache_dir)?;

        let rcedit_path = cache_dir.join("rcedit-x64.exe");

        // Check if already downloaded and valid
        if rcedit_path.exists() {
            // Verify file size to detect corrupted downloads
            if let Ok(metadata) = fs::metadata(&rcedit_path) {
                if metadata.len() >= Self::RCEDIT_MIN_SIZE {
                    tracing::debug!("Using cached rcedit at: {}", rcedit_path.display());
                    return Ok(rcedit_path);
                } else {
                    tracing::warn!(
                        "Cached rcedit is too small ({} bytes), re-downloading...",
                        metadata.len()
                    );
                    let _ = fs::remove_file(&rcedit_path);
                }
            }
        }

        // Download rcedit
        tracing::info!("Downloading rcedit {}...", RCEDIT_VERSION);
        let url = RCEDIT_DOWNLOAD_URL.replace("{version}", RCEDIT_VERSION);

        let response = Self::download_file(&url)?;

        // Validate downloaded size
        if (response.len() as u64) < Self::RCEDIT_MIN_SIZE {
            return Err(PackError::ResourceEdit(format!(
                "Downloaded rcedit is too small ({} bytes), expected at least {} bytes. \
                 Download may have failed.",
                response.len(),
                Self::RCEDIT_MIN_SIZE
            )));
        }

        let mut file = fs::File::create(&rcedit_path)?;
        file.write_all(&response)?;

        tracing::info!(
            "rcedit downloaded to: {} ({} bytes)",
            rcedit_path.display(),
            response.len()
        );
        Ok(rcedit_path)
    }

    /// Download a file from URL
    fn download_file(url: &str) -> PackResult<Vec<u8>> {
        // Use PowerShell to download on Windows (no extra dependencies)
        #[cfg(target_os = "windows")]
        {
            // Use Invoke-WebRequest with -OutFile to download binary correctly
            let temp_file = std::env::temp_dir().join("rcedit-download.exe");
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
                .map_err(|e| PackError::ResourceEdit(format!("Failed to run PowerShell: {}", e)))?;

            if !output.status.success() {
                return Err(PackError::ResourceEdit(format!(
                    "Failed to download rcedit: {}",
                    String::from_utf8_lossy(&output.stderr)
                )));
            }

            let data = fs::read(&temp_file)?;
            let _ = fs::remove_file(&temp_file);
            Ok(data)
        }

        #[cfg(not(target_os = "windows"))]
        {
            // On non-Windows, use curl
            let output = Command::new("curl")
                .args(["-fsSL", url])
                .output()
                .map_err(|e| PackError::ResourceEdit(format!("Failed to run curl: {}", e)))?;

            if !output.status.success() {
                return Err(PackError::ResourceEdit(format!(
                    "Failed to download rcedit: {}",
                    String::from_utf8_lossy(&output.stderr)
                )));
            }

            Ok(output.stdout)
        }
    }

    /// Set the icon of an executable
    ///
    /// # Arguments
    /// * `exe_path` - Path to the executable to modify
    /// * `icon_path` - Path to the .ico file
    pub fn set_icon(&self, exe_path: &Path, icon_path: &Path) -> PackResult<()> {
        if !icon_path.exists() {
            return Err(PackError::ResourceEdit(format!(
                "Icon file not found: {}",
                icon_path.display()
            )));
        }

        // Verify it's an .ico file
        let ext = icon_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext.to_lowercase() != "ico" {
            return Err(PackError::ResourceEdit(format!(
                "Icon must be an .ico file, got: {}",
                icon_path.display()
            )));
        }

        tracing::info!("Setting icon: {}", icon_path.display());

        // rcedit syntax: rcedit <exe> --set-icon <icon>
        let output = Command::new(&self.rcedit_path)
            .arg(exe_path)
            .args(["--set-icon", &icon_path.to_string_lossy()])
            .output()
            .map_err(|e| PackError::ResourceEdit(format!("Failed to run rcedit: {}", e)))?;

        if !output.status.success() {
            return Err(PackError::ResourceEdit(format!(
                "rcedit failed to set icon: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Set the Windows subsystem of an executable
    ///
    /// This directly modifies the PE header to change the subsystem field.
    /// rcedit doesn't support this, so we do it manually.
    ///
    /// # Arguments
    /// * `exe_path` - Path to the executable to modify
    /// * `console` - If true, set to CONSOLE subsystem (shows console window).
    ///   If false, set to WINDOWS subsystem (no console window)
    pub fn set_subsystem(&self, exe_path: &Path, console: bool) -> PackResult<()> {
        use std::io::{Read, Seek, SeekFrom, Write};

        // Windows subsystem values
        const IMAGE_SUBSYSTEM_WINDOWS_GUI: u16 = 2;
        const IMAGE_SUBSYSTEM_WINDOWS_CUI: u16 = 3;

        let subsystem_value = if console {
            IMAGE_SUBSYSTEM_WINDOWS_CUI
        } else {
            IMAGE_SUBSYSTEM_WINDOWS_GUI
        };

        tracing::info!(
            "Setting subsystem to: {} (value={})",
            if console { "console" } else { "windows" },
            subsystem_value
        );

        let mut file = fs::File::options().read(true).write(true).open(exe_path)?;

        // Read DOS header to get PE header offset
        let mut dos_header = [0u8; 64];
        file.read_exact(&mut dos_header)?;

        // Check DOS signature "MZ"
        if dos_header[0] != b'M' || dos_header[1] != b'Z' {
            return Err(PackError::ResourceEdit(
                "Invalid DOS header: not a valid PE file".to_string(),
            ));
        }

        // Get PE header offset from DOS header at offset 0x3C
        let pe_offset = u32::from_le_bytes([
            dos_header[0x3C],
            dos_header[0x3D],
            dos_header[0x3E],
            dos_header[0x3F],
        ]) as u64;

        // Seek to PE header
        file.seek(SeekFrom::Start(pe_offset))?;

        // Read PE signature
        let mut pe_sig = [0u8; 4];
        file.read_exact(&mut pe_sig)?;

        // Check PE signature "PE\0\0"
        if &pe_sig != b"PE\0\0" {
            return Err(PackError::ResourceEdit(
                "Invalid PE signature: not a valid PE file".to_string(),
            ));
        }

        // Read COFF header (20 bytes)
        let mut coff_header = [0u8; 20];
        file.read_exact(&mut coff_header)?;

        // Get size of optional header
        let optional_header_size = u16::from_le_bytes([coff_header[16], coff_header[17]]) as u64;

        if optional_header_size < 68 {
            return Err(PackError::ResourceEdit(
                "Optional header too small".to_string(),
            ));
        }

        // The subsystem field is at offset 68 in the optional header (for PE32)
        // or at offset 68 for PE32+ as well
        // Current position is at start of optional header
        // Subsystem is at offset 68 from start of optional header
        let subsystem_offset = pe_offset + 4 + 20 + 68;

        file.seek(SeekFrom::Start(subsystem_offset))?;
        file.write_all(&subsystem_value.to_le_bytes())?;
        file.sync_all()?;

        tracing::debug!("Subsystem field written at offset 0x{:X}", subsystem_offset);

        Ok(())
    }

    /// Set version string resource
    ///
    /// # Arguments
    /// * `exe_path` - Path to the executable to modify
    /// * `key` - Version string key (e.g., "FileDescription", "ProductName")
    /// * `value` - Value to set
    pub fn set_version_string(&self, exe_path: &Path, key: &str, value: &str) -> PackResult<()> {
        tracing::debug!("Setting version string {}: {}", key, value);

        // rcedit syntax: rcedit <exe> --set-version-string <key> <value>
        let output = Command::new(&self.rcedit_path)
            .arg(exe_path)
            .args(["--set-version-string", key, value])
            .output()
            .map_err(|e| PackError::ResourceEdit(format!("Failed to run rcedit: {}", e)))?;

        if !output.status.success() {
            return Err(PackError::ResourceEdit(format!(
                "rcedit failed to set version string: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Set file version
    ///
    /// # Arguments
    /// * `exe_path` - Path to the executable to modify
    /// * `version` - Version string (e.g., "1.0.0.0")
    pub fn set_file_version(&self, exe_path: &Path, version: &str) -> PackResult<()> {
        tracing::debug!("Setting file version: {}", version);

        // rcedit syntax: rcedit <exe> --set-file-version <version>
        let output = Command::new(&self.rcedit_path)
            .arg(exe_path)
            .args(["--set-file-version", version])
            .output()
            .map_err(|e| PackError::ResourceEdit(format!("Failed to run rcedit: {}", e)))?;

        if !output.status.success() {
            return Err(PackError::ResourceEdit(format!(
                "rcedit failed to set file version: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Set product version
    ///
    /// # Arguments
    /// * `exe_path` - Path to the executable to modify
    /// * `version` - Version string (e.g., "1.0.0.0")
    pub fn set_product_version(&self, exe_path: &Path, version: &str) -> PackResult<()> {
        tracing::debug!("Setting product version: {}", version);

        // rcedit syntax: rcedit <exe> --set-product-version <version>
        let output = Command::new(&self.rcedit_path)
            .arg(exe_path)
            .args(["--set-product-version", version])
            .output()
            .map_err(|e| PackError::ResourceEdit(format!("Failed to run rcedit: {}", e)))?;

        if !output.status.success() {
            return Err(PackError::ResourceEdit(format!(
                "rcedit failed to set product version: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Apply all resource modifications from a configuration
    pub fn apply_config(&self, exe_path: &Path, config: &ResourceConfig) -> PackResult<()> {
        // First, do all rcedit operations (icon, version info)
        // Then modify PE header for subsystem (must be last as it directly modifies the file)

        // Set icon if specified (uses rcedit)
        if let Some(ref icon_path) = config.icon {
            self.set_icon(exe_path, icon_path)?;
        }

        // Set version information (uses rcedit)
        if let Some(ref version) = config.file_version {
            self.set_file_version(exe_path, version)?;
        }

        if let Some(ref version) = config.product_version {
            self.set_product_version(exe_path, version)?;
        }

        if let Some(ref desc) = config.file_description {
            self.set_version_string(exe_path, "FileDescription", desc)?;
        }

        if let Some(ref name) = config.product_name {
            self.set_version_string(exe_path, "ProductName", name)?;
        }

        if let Some(ref company) = config.company_name {
            self.set_version_string(exe_path, "CompanyName", company)?;
        }

        if let Some(ref copyright) = config.copyright {
            self.set_version_string(exe_path, "LegalCopyright", copyright)?;
        }

        // Set subsystem LAST (directly modifies PE header, doesn't use rcedit)
        // Only modify if we need to hide console (console=false means GUI subsystem)
        if !config.console {
            self.set_subsystem(exe_path, config.console)?;
        }

        Ok(())
    }
}

/// Configuration for Windows executable resources
#[derive(Debug, Clone, Default)]
pub struct ResourceConfig {
    /// Path to the .ico icon file
    pub icon: Option<PathBuf>,

    /// Whether to show console window (default: false)
    pub console: bool,

    /// File version (e.g., "1.0.0.0")
    pub file_version: Option<String>,

    /// Product version (e.g., "1.0.0")
    pub product_version: Option<String>,

    /// File description
    pub file_description: Option<String>,

    /// Product name
    pub product_name: Option<String>,

    /// Company name
    pub company_name: Option<String>,

    /// Copyright string
    pub copyright: Option<String>,
}

impl ResourceConfig {
    /// Create a new empty configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the icon path
    pub fn with_icon(mut self, path: impl Into<PathBuf>) -> Self {
        self.icon = Some(path.into());
        self
    }

    /// Set whether to show console
    pub fn with_console(mut self, console: bool) -> Self {
        self.console = console;
        self
    }

    /// Set file version
    pub fn with_file_version(mut self, version: impl Into<String>) -> Self {
        self.file_version = Some(version.into());
        self
    }

    /// Set product version
    pub fn with_product_version(mut self, version: impl Into<String>) -> Self {
        self.product_version = Some(version.into());
        self
    }

    /// Set file description
    pub fn with_file_description(mut self, desc: impl Into<String>) -> Self {
        self.file_description = Some(desc.into());
        self
    }

    /// Set product name
    pub fn with_product_name(mut self, name: impl Into<String>) -> Self {
        self.product_name = Some(name.into());
        self
    }

    /// Set company name
    pub fn with_company_name(mut self, name: impl Into<String>) -> Self {
        self.company_name = Some(name.into());
        self
    }

    /// Set copyright
    pub fn with_copyright(mut self, copyright: impl Into<String>) -> Self {
        self.copyright = Some(copyright.into());
        self
    }

    /// Check if any resource modifications are configured
    pub fn has_modifications(&self) -> bool {
        self.icon.is_some()
            || !self.console // console=false means we need to modify subsystem
            || self.file_version.is_some()
            || self.product_version.is_some()
            || self.file_description.is_some()
            || self.product_name.is_some()
            || self.company_name.is_some()
            || self.copyright.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_config_builder() {
        let config = ResourceConfig::new()
            .with_icon("test.ico")
            .with_console(false)
            .with_file_version("1.0.0.0")
            .with_product_name("Test App");

        assert_eq!(config.icon, Some(PathBuf::from("test.ico")));
        assert!(!config.console);
        assert_eq!(config.file_version, Some("1.0.0.0".to_string()));
        assert_eq!(config.product_name, Some("Test App".to_string()));
        assert!(config.has_modifications());
    }

    #[test]
    fn test_empty_config_has_modifications() {
        // Empty config still has modifications because console defaults to false
        // which means we need to set subsystem to "windows"
        let config = ResourceConfig::new();
        assert!(config.has_modifications());

        // Config with console=true has no modifications if nothing else is set
        let config = ResourceConfig::new().with_console(true);
        assert!(!config.has_modifications());
    }
}
