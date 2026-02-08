//! Download and cache management for external dependencies
//!
//! This module provides functionality for:
//! - Downloading external dependencies (vx runtime, assets, etc.)
//! - Caching downloaded artifacts
//! - Checksum verification (SHA256/SHA512)
//! - Security controls (domain whitelist, HTTPS enforcement)
//! - Extraction (zip, tar.gz) with strip_components support

use crate::error::{PackError, PackResult};
use sha2::{Digest, Sha256, Sha512};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Download manager for external dependencies
pub struct Downloader {
    /// Cache directory for downloaded artifacts
    cache_dir: PathBuf,
    /// Allow insecure HTTP downloads
    allow_insecure: bool,
    /// Allowed domains for downloads
    allowed_domains: Vec<String>,
    /// Block downloads from unknown domains
    block_unknown_domains: bool,
    /// Require checksum for all downloads
    require_checksum: bool,
    /// Offline mode (only use cache)
    offline: bool,
}

impl Downloader {
    /// Create a new downloader
    pub fn new(cache_dir: impl AsRef<Path>) -> Self {
        Self {
            cache_dir: cache_dir.as_ref().to_path_buf(),
            allow_insecure: false,
            allowed_domains: vec![],
            block_unknown_domains: false,
            require_checksum: false,
            offline: std::env::var("AURORAVIEW_OFFLINE")
                .map(|v| v == "1" || v.to_lowercase() == "true")
                .unwrap_or(false),
        }
    }

    /// Set insecure mode
    pub fn allow_insecure(mut self, allow: bool) -> Self {
        self.allow_insecure = allow;
        self
    }

    /// Set allowed domains
    pub fn allowed_domains(mut self, domains: Vec<String>) -> Self {
        self.allowed_domains = domains;
        self
    }

    /// Set whether to block unknown domains
    pub fn block_unknown_domains(mut self, block: bool) -> Self {
        self.block_unknown_domains = block;
        self
    }

    /// Set whether to require checksum
    pub fn require_checksum(mut self, require: bool) -> Self {
        self.require_checksum = require;
        self
    }

    /// Download a file with caching and verification
    pub fn download(&self, name: &str, url: &str, checksum: Option<&str>) -> PackResult<PathBuf> {
        // RFC 0003: Structured logging for vx phases
        info!(
            target: "auroraview::vx::download",
            name = %name,
            url = %url,
            has_checksum = checksum.is_some(),
            offline = self.offline,
            "Starting download"
        );

        // Check if we're in offline mode
        if self.offline {
            info!(
                target: "auroraview::vx::download",
                name = %name,
                "Offline mode: checking cache only"
            );
            return self.get_from_cache(name, checksum);
        }

        // Validate URL
        self.validate_url(url)?;

        // Check cache first
        if let Ok(cached) = self.get_from_cache(name, checksum) {
            info!(
                target: "auroraview::vx::download",
                name = %name,
                path = %cached.display(),
                "Using cached artifact"
            );
            return Ok(cached);
        }

        // Download the file
        info!(
            target: "auroraview::vx::download",
            name = %name,
            url = %url,
            "Downloading from remote"
        );
        let content = self.fetch_url(url)?;

        // Verify checksum if provided
        if let Some(expected) = checksum {
            self.verify_checksum(&content, expected)?;
            info!(
                target: "auroraview::vx::download",
                name = %name,
                checksum = %expected,
                "Checksum verification passed"
            );
        } else if self.require_checksum {
            warn!(
                target: "auroraview::vx::download",
                name = %name,
                "Checksum required but not provided - failing fast"
            );
            return Err(PackError::Config(format!(
                "Checksum required but not provided for {}",
                name
            )));
        } else {
            warn!("No checksum provided for {}, skipping verification", name);
        }

        // Save to cache
        self.save_to_cache(name, &content)?;

        // Return cached path
        self.get_cache_path(name)
    }

    /// Extract an archive to a destination
    pub fn extract(
        &self,
        archive_path: &Path,
        dest: &Path,
        strip_components: usize,
    ) -> PackResult<()> {
        info!(
            "Extracting {} to {} (strip: {})",
            archive_path.display(),
            dest.display(),
            strip_components
        );

        // Create destination directory
        fs::create_dir_all(dest)?;

        // Determine archive type by extension
        let ext = archive_path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| {
                PackError::Config(format!(
                    "Cannot determine archive type: {}",
                    archive_path.display()
                ))
            })?;

        match ext.to_lowercase().as_str() {
            "zip" => self.extract_zip(archive_path, dest, strip_components),
            "gz" | "tgz" => self.extract_tar_gz(archive_path, dest, strip_components),
            "tar" => self.extract_tar(archive_path, dest, strip_components),
            _ => Err(PackError::Config(format!(
                "Unsupported archive format: {}",
                ext
            ))),
        }
    }

    // ====================================================================
    // Private methods
    // ====================================================================

    /// Validate URL against security rules
    fn validate_url(&self, url: &str) -> PackResult<()> {
        // Parse URL
        let parsed = url::Url::parse(url)
            .map_err(|e| PackError::Config(format!("Invalid URL {}: {}", url, e)))?;

        // Check scheme (HTTPS required unless insecure mode)
        if !self.allow_insecure && parsed.scheme() != "https" {
            warn!(
                target: "auroraview::vx::security",
                url = %url,
                scheme = %parsed.scheme(),
                "Insecure protocol blocked"
            );
            return Err(PackError::Config(format!(
                "Insecure URL scheme ({}), HTTPS required. Set allow_insecure=true to bypass.",
                parsed.scheme()
            )));
        }

        // Check domain whitelist
        if !self.allowed_domains.is_empty() {
            if let Some(host) = parsed.host_str() {
                let allowed = self.allowed_domains.iter().any(|d| host.contains(d));
                if !allowed {
                    if self.block_unknown_domains {
                        warn!(
                            target: "auroraview::vx::security",
                            url = %url,
                            host = %host,
                            allowed_domains = ?self.allowed_domains,
                            "Domain not in whitelist - blocking"
                        );
                        return Err(PackError::Config(format!(
                            "Domain {} not in allowed list: {:?}",
                            host, self.allowed_domains
                        )));
                    } else {
                        warn!(
                            target: "auroraview::vx::security",
                            url = %url,
                            host = %host,
                            "Domain {} not in allowed list, proceeding anyway", host
                        );
                    }
                } else {
                    info!(
                        target: "auroraview::vx::security",
                        url = %url,
                        host = %host,
                        "Domain validation passed"
                    );
                }
            }
        }

        Ok(())
    }

    /// Fetch URL content
    fn fetch_url(&self, url: &str) -> PackResult<Vec<u8>> {
        let mut response = ureq::get(url)
            .call()
            .map_err(|e| PackError::Config(format!("Failed to download {}: {}", url, e)))?;

        let buffer = response
            .body_mut()
            .read_to_vec()
            .map_err(|e| PackError::Config(format!("Failed to read response: {}", e)))?;

        debug!("Downloaded {} bytes from {}", buffer.len(), url);
        Ok(buffer)
    }

    /// Verify checksum
    fn verify_checksum(&self, content: &[u8], expected: &str) -> PackResult<()> {
        // Determine hash algorithm by length
        let actual = if expected.len() == 64 {
            // SHA256
            let mut hasher = Sha256::new();
            hasher.update(content);
            format!("{:x}", hasher.finalize())
        } else if expected.len() == 128 {
            // SHA512
            let mut hasher = Sha512::new();
            hasher.update(content);
            format!("{:x}", hasher.finalize())
        } else {
            return Err(PackError::Config(format!(
                "Invalid checksum length: {} (expected 64 for SHA256 or 128 for SHA512)",
                expected.len()
            )));
        };

        if actual.to_lowercase() != expected.to_lowercase() {
            return Err(PackError::Config(format!(
                "Checksum mismatch:\n  Expected: {}\n  Actual:   {}",
                expected, actual
            )));
        }

        info!("Checksum verified successfully");
        Ok(())
    }

    /// Get cache path for a named artifact
    fn get_cache_path(&self, name: &str) -> PackResult<PathBuf> {
        let path = self.cache_dir.join(name);
        if !path.exists() {
            return Err(PackError::Config(format!(
                "Artifact not found in cache: {}",
                name
            )));
        }
        Ok(path)
    }

    /// Get artifact from cache (with optional checksum verification)
    fn get_from_cache(&self, name: &str, checksum: Option<&str>) -> PackResult<PathBuf> {
        let path = self.cache_dir.join(name);
        if !path.exists() {
            return Err(PackError::Config(format!("Cache miss: {}", name)));
        }

        // Verify checksum if provided
        if let Some(expected) = checksum {
            let content = fs::read(&path)?;
            self.verify_checksum(&content, expected)?;
        }

        Ok(path)
    }

    /// Save content to cache
    fn save_to_cache(&self, name: &str, content: &[u8]) -> PackResult<()> {
        fs::create_dir_all(&self.cache_dir)?;
        let path = self.cache_dir.join(name);
        let mut file = fs::File::create(&path)?;
        file.write_all(content)?;
        info!("Saved to cache: {} ({} bytes)", name, content.len());
        Ok(())
    }

    /// Extract zip archive
    fn extract_zip(
        &self,
        archive_path: &Path,
        dest: &Path,
        strip_components: usize,
    ) -> PackResult<()> {
        let file = fs::File::open(archive_path)?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| PackError::Config(format!("Failed to read zip: {}", e)))?;

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| PackError::Config(format!("Failed to read zip entry: {}", e)))?;

            let file_path = file.mangled_name();
            let stripped = self.strip_path_components(&file_path, strip_components);

            if let Some(output_path) = stripped {
                let full_path = dest.join(output_path);

                if file.is_dir() {
                    fs::create_dir_all(&full_path)?;
                } else {
                    if let Some(parent) = full_path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    let mut outfile = fs::File::create(&full_path)?;
                    io::copy(&mut file, &mut outfile)?;

                    // Set executable permissions on Unix
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        if let Some(mode) = file.unix_mode() {
                            if mode & 0o111 != 0 {
                                let mut perms = outfile.metadata()?.permissions();
                                perms.set_mode(mode);
                                fs::set_permissions(&full_path, perms)?;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Extract tar.gz archive
    fn extract_tar_gz(
        &self,
        archive_path: &Path,
        dest: &Path,
        strip_components: usize,
    ) -> PackResult<()> {
        let file = fs::File::open(archive_path)?;
        let decoder = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(decoder);

        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?;
            let stripped = self.strip_path_components(&path, strip_components);

            if let Some(output_path) = stripped {
                let full_path = dest.join(output_path);
                entry.unpack(&full_path)?;
            }
        }

        Ok(())
    }

    /// Extract tar archive
    fn extract_tar(
        &self,
        archive_path: &Path,
        dest: &Path,
        strip_components: usize,
    ) -> PackResult<()> {
        let file = fs::File::open(archive_path)?;
        let mut archive = tar::Archive::new(file);

        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?;
            let stripped = self.strip_path_components(&path, strip_components);

            if let Some(output_path) = stripped {
                let full_path = dest.join(output_path);
                entry.unpack(&full_path)?;
            }
        }

        Ok(())
    }

    /// Strip N path components from the beginning
    fn strip_path_components(&self, path: &Path, n: usize) -> Option<PathBuf> {
        if n == 0 {
            return Some(path.to_path_buf());
        }

        let components: Vec<_> = path.components().skip(n).collect();
        if components.is_empty() {
            None
        } else {
            Some(components.iter().collect())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_downloader_creation() {
        let temp = TempDir::new().unwrap();
        let downloader = Downloader::new(temp.path());
        assert_eq!(downloader.cache_dir, temp.path());
        assert!(!downloader.allow_insecure);
        assert!(!downloader.require_checksum);
    }

    #[test]
    fn test_url_validation_https() {
        let temp = TempDir::new().unwrap();
        let downloader = Downloader::new(temp.path());

        assert!(downloader
            .validate_url("https://example.com/file.zip")
            .is_ok());
        assert!(downloader
            .validate_url("http://example.com/file.zip")
            .is_err());
    }

    #[test]
    fn test_url_validation_insecure() {
        let temp = TempDir::new().unwrap();
        let downloader = Downloader::new(temp.path()).allow_insecure(true);

        assert!(downloader
            .validate_url("http://example.com/file.zip")
            .is_ok());
    }

    #[test]
    fn test_checksum_sha256() {
        let temp = TempDir::new().unwrap();
        let downloader = Downloader::new(temp.path());
        let content = b"hello world";

        // Correct SHA256
        let sha256 = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        assert!(downloader.verify_checksum(content, sha256).is_ok());

        // Wrong checksum
        let wrong = "0000000000000000000000000000000000000000000000000000000000000000";
        assert!(downloader.verify_checksum(content, wrong).is_err());
    }

    #[test]
    fn test_strip_path_components() {
        let temp = TempDir::new().unwrap();
        let downloader = Downloader::new(temp.path());

        let path = Path::new("a/b/c/file.txt");

        assert_eq!(
            downloader.strip_path_components(path, 0),
            Some(PathBuf::from("a/b/c/file.txt"))
        );
        assert_eq!(
            downloader.strip_path_components(path, 1),
            Some(PathBuf::from("b/c/file.txt"))
        );
        assert_eq!(
            downloader.strip_path_components(path, 2),
            Some(PathBuf::from("c/file.txt"))
        );
        assert_eq!(
            downloader.strip_path_components(path, 3),
            Some(PathBuf::from("file.txt"))
        );
        assert_eq!(downloader.strip_path_components(path, 4), None);
    }
}
