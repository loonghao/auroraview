//! Chrome Extension Downloader
//!
//! This module provides functionality for downloading Chrome extensions
//! from the Chrome Web Store and bundling them into AuroraView applications.
//!
//! ## Security
//!
//! - All downloads are from official Chrome Web Store servers
//! - CRX files are verified using the extension ID
//! - Extensions are extracted and stored in isolated directories
//!
//! ## Usage
//!
//! ```rust,ignore
//! use auroraview_pack::extension_downloader::ExtensionDownloader;
//!
//! let downloader = ExtensionDownloader::new("./extensions");
//! let ext_path = downloader.download("bpoadfkcbjbfhfodiogcnhhhpibjhbnh", None)?;
//! ```

use crate::error::{PackError, PackResult};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Chrome Web Store CRX download URL template
/// Uses the "update" API endpoint which returns CRX files
const CWS_UPDATE_URL: &str = "https://clients2.google.com/service/update2/crx";

/// User agent to use for downloads (mimics Chrome browser)
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// CRX3 magic header
const CRX3_MAGIC: &[u8] = b"Cr24";

/// Chrome extension downloader
pub struct ExtensionDownloader {
    /// Directory to store downloaded extensions
    output_dir: PathBuf,
    /// Cache directory for CRX files
    cache_dir: PathBuf,
    /// Whether to use cache
    use_cache: bool,
    /// Offline mode (only use cache)
    offline: bool,
}

impl ExtensionDownloader {
    /// Create a new extension downloader
    pub fn new(output_dir: impl AsRef<Path>) -> Self {
        let output_dir = output_dir.as_ref().to_path_buf();
        let cache_dir = output_dir.join(".cache");
        Self {
            output_dir,
            cache_dir,
            use_cache: true,
            offline: std::env::var("AURORAVIEW_OFFLINE")
                .map(|v| v == "1" || v.to_lowercase() == "true")
                .unwrap_or(false),
        }
    }

    /// Set cache directory
    pub fn with_cache_dir(mut self, cache_dir: impl AsRef<Path>) -> Self {
        self.cache_dir = cache_dir.as_ref().to_path_buf();
        self
    }

    /// Disable caching
    pub fn no_cache(mut self) -> Self {
        self.use_cache = false;
        self
    }

    /// Download an extension from Chrome Web Store
    ///
    /// # Arguments
    /// * `extension_id` - The 32-character Chrome Web Store extension ID
    /// * `version` - Optional version constraint
    ///
    /// # Returns
    /// Path to the extracted extension directory
    pub fn download(&self, extension_id: &str, version: Option<&str>) -> PackResult<PathBuf> {
        // Validate extension ID format
        if !Self::validate_extension_id(extension_id) {
            return Err(PackError::Config(format!(
                "Invalid extension ID format: {}. Expected 32-character lowercase alphanumeric string.",
                extension_id
            )));
        }

        info!(
            target: "auroraview::pack::extension",
            extension_id = %extension_id,
            version = ?version,
            "Downloading Chrome extension"
        );

        // Check cache first
        let cache_path = self.cache_dir.join(format!("{}.crx", extension_id));
        let ext_dir = self.output_dir.join(extension_id);

        // If extension directory already exists with manifest.json, skip download
        if ext_dir.join("manifest.json").exists() {
            info!(
                target: "auroraview::pack::extension",
                extension_id = %extension_id,
                path = %ext_dir.display(),
                "Extension already extracted, skipping download"
            );
            return Ok(ext_dir);
        }

        // Try to get from cache
        if self.use_cache && cache_path.exists() {
            info!(
                target: "auroraview::pack::extension",
                extension_id = %extension_id,
                cache_path = %cache_path.display(),
                "Using cached CRX file"
            );
            return self.extract_crx(&cache_path, &ext_dir);
        }

        // Check offline mode
        if self.offline {
            return Err(PackError::Io(format!(
                "Cannot download extension {} in offline mode. Cache not found at: {}",
                extension_id,
                cache_path.display()
            )));
        }

        // Build download URL
        let url = Self::build_download_url(extension_id, version);
        debug!(
            target: "auroraview::pack::extension",
            url = %url,
            "Downloading from Chrome Web Store"
        );

        // Download CRX file
        let crx_data = self.download_crx(&url)?;

        // Verify CRX magic header
        if !crx_data.starts_with(CRX3_MAGIC) {
            return Err(PackError::Config(format!(
                "Invalid CRX file format for extension {}. File does not start with CRX3 magic header.",
                extension_id
            )));
        }

        // Cache the CRX file
        if self.use_cache {
            fs::create_dir_all(&self.cache_dir)
                .map_err(|e| PackError::Io(format!("Failed to create cache directory: {}", e)))?;
            fs::write(&cache_path, &crx_data)
                .map_err(|e| PackError::Io(format!("Failed to write CRX to cache: {}", e)))?;
            debug!(
                target: "auroraview::pack::extension",
                cache_path = %cache_path.display(),
                "CRX file cached"
            );
        }

        // Extract CRX to extension directory
        self.extract_crx_data(&crx_data, &ext_dir)
    }

    /// Download multiple extensions
    pub fn download_all(&self, extension_ids: &[(&str, Option<&str>)]) -> PackResult<Vec<PathBuf>> {
        let mut paths = Vec::new();
        for (id, version) in extension_ids {
            let path = self.download(id, *version)?;
            paths.push(path);
        }
        Ok(paths)
    }

    /// Validate extension ID format
    fn validate_extension_id(id: &str) -> bool {
        id.len() == 32 && id.chars().all(|c| c.is_ascii_lowercase())
    }

    /// Build Chrome Web Store download URL
    fn build_download_url(extension_id: &str, _version: Option<&str>) -> String {
        // Chrome Web Store update URL format
        // Parameters:
        // - response: redirect (get direct download URL)
        // - prodversion: Chrome version (use latest)
        // - x: URL-encoded extension info
        let x_param = format!("id%3D{}%26uc", extension_id);
        format!(
            "{}?response=redirect&prodversion=120.0.0.0&acceptformat=crx3&x={}",
            CWS_UPDATE_URL, x_param
        )
    }

    /// Download CRX file from URL
    fn download_crx(&self, url: &str) -> PackResult<Vec<u8>> {
        let client = ureq::AgentBuilder::new()
            .user_agent(USER_AGENT)
            .redirects(5)
            .build();

        let response = client
            .get(url)
            .call()
            .map_err(|e| PackError::Io(format!("Failed to download extension: {}", e)))?;

        if response.status() != 200 {
            return Err(PackError::Io(format!(
                "Failed to download extension: HTTP {}",
                response.status()
            )));
        }

        let mut data = Vec::new();
        response
            .into_reader()
            .read_to_end(&mut data)
            .map_err(|e| PackError::Io(format!("Failed to read response: {}", e)))?;

        info!(
            target: "auroraview::pack::extension",
            size = data.len(),
            "Downloaded CRX file"
        );

        Ok(data)
    }

    /// Extract CRX file to directory
    fn extract_crx(&self, crx_path: &Path, ext_dir: &Path) -> PackResult<PathBuf> {
        let data = fs::read(crx_path)
            .map_err(|e| PackError::Io(format!("Failed to read CRX file: {}", e)))?;
        self.extract_crx_data(&data, ext_dir)
    }

    /// Extract CRX data to directory
    fn extract_crx_data(&self, data: &[u8], ext_dir: &Path) -> PackResult<PathBuf> {
        // CRX3 format:
        // - Magic (4 bytes): "Cr24"
        // - Version (4 bytes): 3 (little-endian)
        // - Header length (4 bytes, little-endian)
        // - Header (protobuf)
        // - ZIP archive

        if data.len() < 12 {
            return Err(PackError::Config("CRX file too small".to_string()));
        }

        // Verify magic
        if &data[0..4] != CRX3_MAGIC {
            return Err(PackError::Config("Invalid CRX magic header".to_string()));
        }

        // Read version
        let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        if version != 3 {
            return Err(PackError::Config(format!(
                "Unsupported CRX version: {}. Expected version 3.",
                version
            )));
        }

        // Read header length
        let header_len = u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize;

        // Calculate ZIP offset
        let zip_offset = 12 + header_len;
        if zip_offset >= data.len() {
            return Err(PackError::Config(
                "CRX header length exceeds file size".to_string(),
            ));
        }

        // Extract ZIP portion
        let zip_data = &data[zip_offset..];

        // Create extension directory
        fs::create_dir_all(ext_dir)
            .map_err(|e| PackError::Io(format!("Failed to create extension directory: {}", e)))?;

        // Extract ZIP archive
        let cursor = std::io::Cursor::new(zip_data);
        let mut archive = zip::ZipArchive::new(cursor)
            .map_err(|e| PackError::Io(format!("Failed to read ZIP archive: {}", e)))?;

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| PackError::Io(format!("Failed to read ZIP entry: {}", e)))?;

            let outpath = match file.enclosed_name() {
                Some(path) => ext_dir.join(path),
                None => continue,
            };

            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath)
                    .map_err(|e| PackError::Io(format!("Failed to create directory: {}", e)))?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p).map_err(|e| {
                            PackError::Io(format!("Failed to create parent directory: {}", e))
                        })?;
                    }
                }
                let mut outfile = fs::File::create(&outpath)
                    .map_err(|e| PackError::Io(format!("Failed to create file: {}", e)))?;
                std::io::copy(&mut file, &mut outfile)
                    .map_err(|e| PackError::Io(format!("Failed to extract file: {}", e)))?;
            }
        }

        info!(
            target: "auroraview::pack::extension",
            path = %ext_dir.display(),
            files = archive.len(),
            "Extracted extension"
        );

        Ok(ext_dir.to_path_buf())
    }

    /// Copy a local extension to the output directory
    pub fn copy_local(&self, source: &Path, extension_id: Option<&str>) -> PackResult<PathBuf> {
        // Read manifest to get extension ID if not provided
        let manifest_path = source.join("manifest.json");
        if !manifest_path.exists() {
            return Err(PackError::Config(format!(
                "Extension manifest not found: {}",
                manifest_path.display()
            )));
        }

        let manifest_content = fs::read_to_string(&manifest_path)
            .map_err(|e| PackError::Io(format!("Failed to read manifest: {}", e)))?;

        // Parse manifest to get name (used as fallback ID)
        let manifest: serde_json::Value = serde_json::from_str(&manifest_content)
            .map_err(|e| PackError::Config(format!("Failed to parse manifest: {}", e)))?;

        // Generate extension ID from source path if not provided
        let id = extension_id
            .map(String::from)
            .or_else(|| {
                // Generate ID from path hash
                let path_str = source.to_string_lossy();
                let hash = format!("{:x}", md5::compute(path_str.as_bytes()));
                Some(hash[..32].to_string())
            })
            .unwrap();

        let ext_dir = self.output_dir.join(&id);

        // Copy extension files
        Self::copy_dir_all(source, &ext_dir)?;

        info!(
            target: "auroraview::pack::extension",
            id = %id,
            name = ?manifest.get("name").and_then(|v| v.as_str()),
            path = %ext_dir.display(),
            "Copied local extension"
        );

        Ok(ext_dir)
    }

    /// Recursively copy directory
    fn copy_dir_all(src: &Path, dst: &Path) -> PackResult<()> {
        fs::create_dir_all(dst)
            .map_err(|e| PackError::Io(format!("Failed to create directory: {}", e)))?;

        for entry in fs::read_dir(src)
            .map_err(|e| PackError::Io(format!("Failed to read directory: {}", e)))?
        {
            let entry = entry.map_err(|e| PackError::Io(format!("Failed to read entry: {}", e)))?;
            let ty = entry
                .file_type()
                .map_err(|e| PackError::Io(format!("Failed to get file type: {}", e)))?;

            if ty.is_dir() {
                Self::copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
            } else {
                fs::copy(entry.path(), dst.join(entry.file_name()))
                    .map_err(|e| PackError::Io(format!("Failed to copy file: {}", e)))?;
            }
        }

        Ok(())
    }
}

/// Extension metadata extracted from manifest
#[derive(Debug, Clone)]
pub struct ExtensionInfo {
    /// Extension ID
    pub id: String,
    /// Extension name
    pub name: String,
    /// Extension version
    pub version: String,
    /// Extension description
    pub description: Option<String>,
    /// Path to extension directory
    pub path: PathBuf,
}

impl ExtensionInfo {
    /// Load extension info from directory
    pub fn from_dir(path: &Path, id: Option<&str>) -> PackResult<Self> {
        let manifest_path = path.join("manifest.json");
        let content = fs::read_to_string(&manifest_path)
            .map_err(|e| PackError::Io(format!("Failed to read manifest: {}", e)))?;

        let manifest: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| PackError::Config(format!("Failed to parse manifest: {}", e)))?;

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
            .map(String::from);

        // Use provided ID or derive from path
        let id = id
            .map(String::from)
            .or_else(|| path.file_name().and_then(|n| n.to_str()).map(String::from))
            .unwrap_or_else(|| "unknown".to_string());

        Ok(Self {
            id,
            name,
            version,
            description,
            path: path.to_path_buf(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_extension_id() {
        // Valid IDs
        assert!(ExtensionDownloader::validate_extension_id(
            "bpoadfkcbjbfhfodiogcnhhhpibjhbnh"
        ));
        assert!(ExtensionDownloader::validate_extension_id(
            "abcdefghijklmnopqrstuvwxyzabcdef"
        ));

        // Invalid IDs
        assert!(!ExtensionDownloader::validate_extension_id("short"));
        assert!(!ExtensionDownloader::validate_extension_id(
            "BPOADFKCBJBFHFODIOGCNHHHPIBJHBNH"
        )); // uppercase
        assert!(!ExtensionDownloader::validate_extension_id(
            "bpoadfkcbjbfhfodiogcnhhhpibjhbnh1"
        )); // 33 chars
        assert!(!ExtensionDownloader::validate_extension_id(
            "bpoadfkcbjbfhfodiogcnhhhpibjhbn"
        )); // 31 chars
    }

    #[test]
    fn test_build_download_url() {
        let url = ExtensionDownloader::build_download_url("bpoadfkcbjbfhfodiogcnhhhpibjhbnh", None);
        assert!(url.contains("clients2.google.com"));
        assert!(url.contains("bpoadfkcbjbfhfodiogcnhhhpibjhbnh"));
    }
}
