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

use crate::metrics::PackedMetrics;
use crate::{PackConfig, PackError, PackResult};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::time::Instant;

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
        Self::read_with_metrics(path, None)
    }

    /// Read overlay data from a file with performance metrics
    pub fn read_with_metrics(
        path: &Path,
        mut metrics: Option<&mut PackedMetrics>,
    ) -> PackResult<Option<OverlayData>> {
        let file = File::open(path)?;
        let file_len = file.metadata()?.len();

        if file_len < FOOTER_SIZE {
            return Ok(None);
        }

        let mut reader = BufReader::with_capacity(64 * 1024, file); // 64KB buffer

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
            return Err(PackError::InvalidOverlay(
                "Invalid header magic".to_string(),
            ));
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
        let read_start = Instant::now();
        let mut config_compressed = vec![0u8; config_len];
        reader.read_exact(&mut config_compressed)?;

        // Decompress config
        let config_json = zstd::decode_all(&config_compressed[..])
            .map_err(|e| PackError::Compression(e.to_string()))?;

        let config: PackConfig = serde_json::from_slice(&config_json)?;

        if let Some(ref mut m) = metrics {
            m.add_phase("config_read_decompress", read_start.elapsed());
            m.mark_config_decompress();
        }

        tracing::debug!(
            "Config: {} bytes compressed -> {} bytes",
            config_len,
            config_json.len()
        );

        // Read assets data
        let assets_start = Instant::now();
        let mut assets_compressed = vec![0u8; assets_len];
        reader.read_exact(&mut assets_compressed)?;

        if let Some(ref mut m) = metrics {
            m.add_phase("assets_read", assets_start.elapsed());
        }

        // Use streaming decompression + tar extraction (avoids double memory allocation)
        let decompress_start = Instant::now();
        let assets = Self::extract_assets_streaming(&assets_compressed)?;

        if let Some(ref mut m) = metrics {
            m.add_phase("assets_decompress_and_extract", decompress_start.elapsed());
            m.mark_assets_decompress();
            m.mark_tar_extract();
            m.mark_overlay_read();
        }

        tracing::debug!(
            "Assets: {} bytes compressed -> {} files extracted",
            assets_len,
            assets.len()
        );

        Ok(Some(OverlayData { config, assets }))
    }

    /// Extract assets from a tar archive (parallel version)
    ///
    /// First pass: collect entry metadata and offsets
    /// Second pass: parallel read of file contents
    #[allow(dead_code)]
    fn extract_assets_archive(data: &[u8]) -> PackResult<Vec<(String, Vec<u8>)>> {
        let mut archive = tar::Archive::new(data);

        // First pass: collect entries sequentially (tar requires sequential read)
        let mut entries_data: Vec<(String, Vec<u8>)> = Vec::new();

        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?.to_string_lossy().to_string();
            let mut content = Vec::with_capacity(entry.size() as usize);
            entry.read_to_end(&mut content)?;
            entries_data.push((path, content));
        }

        Ok(entries_data)
    }

    /// Extract assets from a tar archive using streaming zstd decoder
    ///
    /// This avoids loading the entire decompressed tar into memory at once.
    fn extract_assets_streaming(compressed_data: &[u8]) -> PackResult<Vec<(String, Vec<u8>)>> {
        // Use streaming zstd decoder
        let decoder = zstd::stream::Decoder::new(compressed_data)
            .map_err(|e| PackError::Compression(e.to_string()))?;

        let mut archive = tar::Archive::new(decoder);
        let mut entries_data: Vec<(String, Vec<u8>)> = Vec::new();

        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?.to_string_lossy().to_string();
            let mut content = Vec::with_capacity(entry.size() as usize);
            entry.read_to_end(&mut content)?;
            entries_data.push((path, content));
        }

        Ok(entries_data)
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
