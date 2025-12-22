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
//!
//! ## Content Hash
//!
//! The overlay includes a content hash (BLAKE3) computed from all assets.
//! This hash is used as a cache key at runtime, enabling:
//! - Cache reuse: Same content → same hash → skip extraction
//! - Conflict avoidance: Different content → different hash → new directory
//! - Multi-version support: Multiple versions can coexist

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
    /// Content hash (BLAKE3) of all assets - used as cache key
    /// Format: 16 hex chars (first 64 bits of BLAKE3 hash)
    pub content_hash: String,
    /// Embedded assets (file path -> content)
    #[serde(skip)]
    pub assets: Vec<(String, Vec<u8>)>,
}

impl OverlayData {
    /// Create new overlay data with configuration
    pub fn new(config: PackConfig) -> Self {
        Self {
            config,
            content_hash: String::new(),
            assets: Vec::new(),
        }
    }

    /// Add an asset to the overlay
    pub fn add_asset(&mut self, path: impl Into<String>, content: Vec<u8>) {
        self.assets.push((path.into(), content));
    }

    /// Compute and set the content hash from all assets
    ///
    /// The hash is computed by hashing all asset paths and contents in order.
    /// Returns the computed hash string (16 hex chars).
    pub fn compute_content_hash(&mut self) -> String {
        let mut hasher = blake3::Hasher::new();

        // Sort assets by path for deterministic hashing
        let mut sorted_assets: Vec<_> = self.assets.iter().collect();
        sorted_assets.sort_by(|a, b| a.0.cmp(&b.0));

        for (path, content) in &sorted_assets {
            // Hash the path
            hasher.update(path.as_bytes());
            hasher.update(&[0]); // Separator
                                 // Hash the content length (for robustness)
            hasher.update(&(content.len() as u64).to_le_bytes());
            // Hash the content
            hasher.update(content);
        }

        // Use first 64 bits (16 hex chars) for shorter, still-unique cache keys
        let hash = hasher.finalize();
        let short_hash = format!(
            "{:016x}",
            u64::from_le_bytes(hash.as_bytes()[..8].try_into().unwrap())
        );

        self.content_hash = short_hash.clone();
        short_hash
    }

    /// Get the content hash, computing it if not already set
    pub fn get_content_hash(&mut self) -> String {
        if self.content_hash.is_empty() {
            self.compute_content_hash();
        }
        self.content_hash.clone()
    }
}

/// Metadata stored in the overlay (config + content hash)
///
/// This is what gets serialized to JSON and stored in the overlay.
/// It's separate from OverlayData to avoid serializing assets.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OverlayMetadata {
    /// Pack configuration
    #[serde(flatten)]
    config: PackConfig,
    /// Content hash (BLAKE3) of all assets
    content_hash: String,
}

/// Writer for appending overlay data to executables
pub struct OverlayWriter;

impl OverlayWriter {
    /// Write overlay data to an executable
    ///
    /// This appends the overlay to the end of the file without modifying
    /// the original executable content.
    ///
    /// The content hash is computed before writing if not already set.
    pub fn write(exe_path: &Path, data: &OverlayData) -> PackResult<()> {
        // Clone and compute hash if needed
        let mut data = data.clone();
        let content_hash = data.get_content_hash();

        let file = File::options().append(true).open(exe_path)?;
        let mut writer = BufWriter::new(file);

        // Get the current end of file (where overlay starts)
        let overlay_start = writer.seek(SeekFrom::End(0))?;

        // Create a metadata object that includes the hash
        let metadata = OverlayMetadata {
            config: data.config.clone(),
            content_hash: content_hash.clone(),
        };
        let metadata_json = serde_json::to_vec(&metadata)?;

        // Compress config with zstd
        let config_compressed = zstd::encode_all(&metadata_json[..], 3)
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

        // Explicitly drop writer and sync to ensure file is fully written
        // This is important on Windows before other tools (like rcedit) modify the file
        let file = writer
            .into_inner()
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        file.sync_all()?;
        drop(file);

        tracing::info!(
            "Overlay written: config={} bytes, assets={} bytes, hash={}",
            config_compressed.len(),
            assets_compressed.len(),
            content_hash
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

        // Parse overlay metadata
        let metadata: OverlayMetadata = serde_json::from_slice(&config_json)?;
        let config = metadata.config;
        let content_hash = metadata.content_hash;

        tracing::debug!("Overlay content hash: {}", content_hash);

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

        Ok(Some(OverlayData {
            config,
            content_hash,
            assets,
        }))
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
