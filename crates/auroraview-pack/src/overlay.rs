//! Overlay data format for packed executables
//!
//! The overlay is appended to the end of the executable and contains
//! the configuration and assets needed to run as a standalone app.
//!
//! ## Format
//!
//! ```text
//! [Original Executable]
//! [Overlay Header]
//!   - Magic: "AVPK" (4 bytes)
//!   - Version: u32 LE (4 bytes)
//!   - Config Length: u64 LE (8 bytes)
//!   - Assets Length: u64 LE (8 bytes)
//! [Config Data] (JSON, zstd compressed)
//! [Assets Data] (tar archive, zstd compressed)
//! [Footer]
//!   - Overlay Start Offset: u64 LE (8 bytes)
//!   - Magic: "AVPK" (4 bytes)
//! ```

use crate::{PackConfig, PackError, PackResult};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

/// Magic bytes for overlay identification
pub const OVERLAY_MAGIC: &[u8; 4] = b"AVPK";

/// Current overlay format version
pub const OVERLAY_VERSION: u32 = 1;

/// Footer size in bytes (offset: 8 + magic: 4)
const FOOTER_SIZE: u64 = 12;

/// Header size in bytes (magic: 4 + version: 4 + config_len: 8 + assets_len: 8)
#[allow(dead_code)]
const HEADER_SIZE: u64 = 24;

/// Overlay data containing configuration and assets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayData {
    /// Pack configuration
    pub config: PackConfig,
    /// Embedded assets (file path -> content)
    #[serde(skip)]
    pub assets: Vec<(String, Vec<u8>)>,
}

impl OverlayData {
    /// Create new overlay data with configuration
    pub fn new(config: PackConfig) -> Self {
        Self {
            config,
            assets: Vec::new(),
        }
    }

    /// Add an asset to the overlay
    pub fn add_asset(&mut self, path: impl Into<String>, content: Vec<u8>) {
        self.assets.push((path.into(), content));
    }
}

/// Writer for appending overlay data to executables
pub struct OverlayWriter;

impl OverlayWriter {
    /// Write overlay data to an executable
    ///
    /// This appends the overlay to the end of the file without modifying
    /// the original executable content.
    pub fn write(exe_path: &Path, data: &OverlayData) -> PackResult<()> {
        let file = File::options().append(true).open(exe_path)?;
        let mut writer = BufWriter::new(file);

        // Get the current end of file (where overlay starts)
        let overlay_start = writer.seek(SeekFrom::End(0))?;

        // Serialize config to JSON
        let config_json = serde_json::to_vec(&data.config)?;

        // Compress config with zstd
        let config_compressed = zstd::encode_all(&config_json[..], 3)
            .map_err(|e| PackError::Compression(e.to_string()))?;

        // Create tar archive for assets
        let assets_tar = Self::create_assets_archive(&data.assets)?;

        // Compress assets with zstd
        let assets_compressed = zstd::encode_all(&assets_tar[..], 3)
            .map_err(|e| PackError::Compression(e.to_string()))?;

        // Write header
        writer.write_all(OVERLAY_MAGIC)?;
        writer.write_all(&OVERLAY_VERSION.to_le_bytes())?;
        writer.write_all(&(config_compressed.len() as u64).to_le_bytes())?;
        writer.write_all(&(assets_compressed.len() as u64).to_le_bytes())?;

        // Write data
        writer.write_all(&config_compressed)?;
        writer.write_all(&assets_compressed)?;

        // Write footer
        writer.write_all(&overlay_start.to_le_bytes())?;
        writer.write_all(OVERLAY_MAGIC)?;

        writer.flush()?;

        tracing::info!(
            "Overlay written: config={} bytes, assets={} bytes",
            config_compressed.len(),
            assets_compressed.len()
        );

        Ok(())
    }

    /// Create a tar archive from assets
    fn create_assets_archive(assets: &[(String, Vec<u8>)]) -> PackResult<Vec<u8>> {
        let mut archive = tar::Builder::new(Vec::new());

        for (path, content) in assets {
            let mut header = tar::Header::new_gnu();
            header.set_path(path)?;
            header.set_size(content.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            archive.append(&header, &content[..])?;
        }

        archive
            .into_inner()
            .map_err(|e| PackError::Bundle(e.to_string()))
    }
}

/// Reader for extracting overlay data from executables
pub struct OverlayReader;

impl OverlayReader {
    /// Check if a file has overlay data
    pub fn has_overlay(path: &Path) -> PackResult<bool> {
        let file = File::open(path)?;
        let file_len = file.metadata()?.len();

        if file_len < FOOTER_SIZE {
            return Ok(false);
        }

        let mut reader = BufReader::new(file);
        reader.seek(SeekFrom::End(-(FOOTER_SIZE as i64)))?;

        // Read footer
        let mut offset_bytes = [0u8; 8];
        let mut magic = [0u8; 4];
        reader.read_exact(&mut offset_bytes)?;
        reader.read_exact(&mut magic)?;

        Ok(&magic == OVERLAY_MAGIC)
    }

    /// Read overlay data from a file
    pub fn read(path: &Path) -> PackResult<Option<OverlayData>> {
        let file = File::open(path)?;
        let file_len = file.metadata()?.len();

        if file_len < FOOTER_SIZE {
            return Ok(None);
        }

        let mut reader = BufReader::new(file);

        // Read footer
        reader.seek(SeekFrom::End(-(FOOTER_SIZE as i64)))?;
        let mut offset_bytes = [0u8; 8];
        let mut magic = [0u8; 4];
        reader.read_exact(&mut offset_bytes)?;
        reader.read_exact(&mut magic)?;

        if &magic != OVERLAY_MAGIC {
            return Ok(None);
        }

        let overlay_start = u64::from_le_bytes(offset_bytes);

        // Seek to overlay start and read header
        reader.seek(SeekFrom::Start(overlay_start))?;

        let mut header_magic = [0u8; 4];
        let mut version_bytes = [0u8; 4];
        let mut config_len_bytes = [0u8; 8];
        let mut assets_len_bytes = [0u8; 8];

        reader.read_exact(&mut header_magic)?;
        reader.read_exact(&mut version_bytes)?;
        reader.read_exact(&mut config_len_bytes)?;
        reader.read_exact(&mut assets_len_bytes)?;

        if &header_magic != OVERLAY_MAGIC {
            return Err(PackError::InvalidOverlay("Invalid header magic".to_string()));
        }

        let version = u32::from_le_bytes(version_bytes);
        if version != OVERLAY_VERSION {
            return Err(PackError::InvalidOverlay(format!(
                "Unsupported version: {} (expected {})",
                version, OVERLAY_VERSION
            )));
        }

        let config_len = u64::from_le_bytes(config_len_bytes) as usize;
        let assets_len = u64::from_le_bytes(assets_len_bytes) as usize;

        // Read config data
        let mut config_compressed = vec![0u8; config_len];
        reader.read_exact(&mut config_compressed)?;

        // Decompress config
        let config_json = zstd::decode_all(&config_compressed[..])
            .map_err(|e| PackError::Compression(e.to_string()))?;

        let config: PackConfig = serde_json::from_slice(&config_json)?;

        // Read assets data
        let mut assets_compressed = vec![0u8; assets_len];
        reader.read_exact(&mut assets_compressed)?;

        // Decompress assets
        let assets_tar = zstd::decode_all(&assets_compressed[..])
            .map_err(|e| PackError::Compression(e.to_string()))?;

        // Extract assets from tar
        let assets = Self::extract_assets_archive(&assets_tar)?;

        Ok(Some(OverlayData { config, assets }))
    }

    /// Extract assets from a tar archive
    fn extract_assets_archive(data: &[u8]) -> PackResult<Vec<(String, Vec<u8>)>> {
        let mut archive = tar::Archive::new(data);
        let mut assets = Vec::new();

        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry
                .path()?
                .to_string_lossy()
                .to_string();
            let mut content = Vec::new();
            entry.read_to_end(&mut content)?;
            assets.push((path, content));
        }

        Ok(assets)
    }

    /// Get the original executable size (before overlay)
    pub fn get_original_size(path: &Path) -> PackResult<Option<u64>> {
        let file = File::open(path)?;
        let file_len = file.metadata()?.len();

        if file_len < FOOTER_SIZE {
            return Ok(None);
        }

        let mut reader = BufReader::new(file);
        reader.seek(SeekFrom::End(-(FOOTER_SIZE as i64)))?;

        let mut offset_bytes = [0u8; 8];
        let mut magic = [0u8; 4];
        reader.read_exact(&mut offset_bytes)?;
        reader.read_exact(&mut magic)?;

        if &magic != OVERLAY_MAGIC {
            return Ok(None);
        }

        Ok(Some(u64::from_le_bytes(offset_bytes)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_overlay_roundtrip() {
        // Create a temp file with some content
        let temp = NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), b"fake executable content").unwrap();

        // Create overlay data
        let config = PackConfig::url("https://example.com").with_title("Test App");
        let mut data = OverlayData::new(config);
        data.add_asset("index.html", b"<html></html>".to_vec());
        data.add_asset("style.css", b"body { }".to_vec());

        // Write overlay
        OverlayWriter::write(temp.path(), &data).unwrap();

        // Verify overlay exists
        assert!(OverlayReader::has_overlay(temp.path()).unwrap());

        // Read overlay
        let read_data = OverlayReader::read(temp.path()).unwrap().unwrap();
        assert_eq!(read_data.config.window.title, "Test App");
        assert_eq!(read_data.assets.len(), 2);

        // Verify original size
        let original_size = OverlayReader::get_original_size(temp.path()).unwrap().unwrap();
        assert_eq!(original_size, b"fake executable content".len() as u64);
    }

    #[test]
    fn test_no_overlay() {
        let temp = NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), b"just a regular file").unwrap();

        assert!(!OverlayReader::has_overlay(temp.path()).unwrap());
        assert!(OverlayReader::read(temp.path()).unwrap().is_none());
    }
}
