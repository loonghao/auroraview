//! Windows executable resource editor
//!
//! This module provides functionality to modify Windows PE executable resources,
//! including icons, version information, and subsystem settings.
//!
//! It uses rcedit (https://github.com/electron/rcedit) as the underlying tool,
//! managed through vx (https://github.com/loonghao/vx).

use crate::vx_tool::VxTool;
use crate::{PackError, PackResult};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Rcedit execution strategy
enum RceditRunner {
    /// Run rcedit through vx (`vx rcedit <args>`)
    Vx(VxTool),
    /// Run rcedit directly from a custom path
    Direct(PathBuf),
}

/// Windows executable resource editor
///
/// This struct wraps the rcedit tool for modifying PE resources.
/// By default, rcedit is managed through vx. A custom rcedit path
/// can be provided as a fallback.
pub struct ResourceEditor {
    runner: RceditRunner,
}

impl ResourceEditor {
    /// Create a new ResourceEditor using vx to manage rcedit.
    ///
    /// vx will automatically download and cache rcedit if not already installed.
    pub fn new() -> PackResult<Self> {
        let vx = VxTool::new()?;
        Ok(Self {
            runner: RceditRunner::Vx(vx),
        })
    }

    /// Create a ResourceEditor with a custom rcedit path.
    pub fn with_rcedit_path(path: PathBuf) -> PackResult<Self> {
        if !path.exists() {
            return Err(PackError::ResourceEdit(format!(
                "rcedit not found at: {}",
                path.display()
            )));
        }
        Ok(Self {
            runner: RceditRunner::Direct(path),
        })
    }

    /// Execute rcedit with the given arguments.
    fn run_rcedit(&self, args: &[&str]) -> PackResult<std::process::Output> {
        match &self.runner {
            RceditRunner::Vx(vx) => {
                let mut cmd_args = vec!["rcedit"];
                cmd_args.extend(args);
                tracing::debug!("Running: vx {}", cmd_args.join(" "));
                vx.exec(&cmd_args)
                    .map_err(|e| PackError::ResourceEdit(format!("Failed to run vx rcedit: {}", e)))
            }
            RceditRunner::Direct(path) => {
                tracing::debug!("Running: {} {}", path.display(), args.join(" "));
                Command::new(path)
                    .args(args)
                    .output()
                    .map_err(|e| PackError::ResourceEdit(format!("Failed to run rcedit: {}", e)))
            }
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

        let exe_str = exe_path.to_string_lossy();
        let icon_str = icon_path.to_string_lossy();
        let output = self.run_rcedit(&[&exe_str, "--set-icon", &icon_str])?;

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
        use std::fs;
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

        let exe_str = exe_path.to_string_lossy();
        let output = self.run_rcedit(&[&exe_str, "--set-version-string", key, value])?;

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

        let exe_str = exe_path.to_string_lossy();
        let output = self.run_rcedit(&[&exe_str, "--set-file-version", version])?;

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

        let exe_str = exe_path.to_string_lossy();
        let output = self.run_rcedit(&[&exe_str, "--set-product-version", version])?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!("rcedit stderr: {}", stderr);

            // Check if this is a file lock issue
            if stderr.contains("cannot open")
                || stderr.contains("permission denied")
                || stderr.contains("Access is denied")
            {
                return Err(PackError::ResourceEdit("Cannot modify executable (file may be locked by antivirus or another process). \
                     Try: 1) Close all running instances, 2) Temporarily disable antivirus, 3) Run as administrator".to_string()));
            }

            // Check if this is an invalid format issue
            if stderr.contains("not a valid PE file") {
                return Err(PackError::ResourceEdit(format!(
                    "Executable is not a valid PE file: {}. Check if the file was copied correctly.",
                    exe_path.display()
                )));
            }

            return Err(PackError::ResourceEdit(format!(
                "rcedit failed to set product version: {}",
                stderr
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
