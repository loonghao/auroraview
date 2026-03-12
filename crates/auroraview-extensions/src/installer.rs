use crate::error::{ExtensionError, ExtensionResult};
use crate::manifest::Manifest;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionSourceKind {
    Archive,
    Crx,
}

#[derive(Debug, Clone)]
pub struct ResolvedExtensionSource {
    pub original_url: String,
    pub download_url: String,
    pub kind: ExtensionSourceKind,
    pub store_extension_id: Option<String>,
}

pub fn resolve_extension_source_url(url: &str) -> ExtensionResult<ResolvedExtensionSource> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err(ExtensionError::InvalidArgument(
            "Extension source URL cannot be empty".to_string(),
        ));
    }

    let lower = trimmed.to_ascii_lowercase();
    if lower.ends_with(".crx") {
        return Ok(ResolvedExtensionSource {
            original_url: trimmed.to_string(),
            download_url: trimmed.to_string(),
            kind: ExtensionSourceKind::Crx,
            store_extension_id: None,
        });
    }

    if let Some(ext_id) = extract_store_extension_id(trimmed) {
        let download_url = if lower.contains("microsoftedge.microsoft.com/addons") {
            build_edge_store_download_url(&ext_id)
        } else {
            build_chrome_store_download_url(&ext_id)
        };

        return Ok(ResolvedExtensionSource {
            original_url: trimmed.to_string(),
            download_url,
            kind: ExtensionSourceKind::Crx,
            store_extension_id: Some(ext_id),
        });
    }

    Ok(ResolvedExtensionSource {
        original_url: trimmed.to_string(),
        download_url: trimmed.to_string(),
        kind: ExtensionSourceKind::Archive,
        store_extension_id: None,
    })
}

pub fn archive_suffix_from_url(url: &str, kind: ExtensionSourceKind) -> &'static str {
    if kind == ExtensionSourceKind::Crx {
        return "crx";
    }

    let normalized = url
        .split('?')
        .next()
        .unwrap_or(url)
        .split('#')
        .next()
        .unwrap_or(url)
        .to_ascii_lowercase();

    if normalized.ends_with(".tar.gz") {
        "tar.gz"
    } else if normalized.ends_with(".tgz") {
        "tgz"
    } else if normalized.ends_with(".tar") {
        "tar"
    } else {
        "zip"
    }
}

pub fn extract_store_extension_id(url: &str) -> Option<String> {
    let parsed = url::Url::parse(url).ok()?;
    let host = parsed.host_str()?.to_ascii_lowercase();
    if !host.contains("chromewebstore.google.com")
        && !host.contains("chrome.google.com")
        && !host.contains("microsoftedge.microsoft.com")
    {
        return None;
    }

    let segments: Vec<_> = parsed
        .path_segments()
        .map(|s| s.filter(|part| !part.is_empty()).collect::<Vec<_>>())
        .unwrap_or_default();

    let candidate = segments.last().copied()?;
    if is_valid_extension_id(candidate) {
        Some(candidate.to_string())
    } else {
        None
    }
}

pub fn is_valid_extension_id(id: &str) -> bool {
    id.len() == 32 && id.chars().all(|c| c.is_ascii_lowercase())
}

pub fn build_chrome_store_download_url(extension_id: &str) -> String {
    format!(
        "https://clients2.google.com/service/update2/crx?response=redirect&prodversion=120.0.0.0&acceptformat=crx3&x=id%3D{}%26installsource%3Dondemand%26uc",
        extension_id
    )
}

pub fn build_edge_store_download_url(extension_id: &str) -> String {
    format!(
        "https://edge.microsoft.com/extensionwebstorebase/v1/crx?response=redirect&x=id%3D{}%26installsource%3Dondemand%26uc",
        extension_id
    )
}

pub fn extract_crx_archive(crx_path: &Path, dest: &Path) -> ExtensionResult<()> {
    let data = fs::read(crx_path)?;
    extract_crx_data(&data, dest)
}

pub fn extract_crx_data(data: &[u8], dest: &Path) -> ExtensionResult<()> {
    if data.len() < 12 {
        return Err(ExtensionError::InvalidArgument(
            "CRX file too small".to_string(),
        ));
    }

    if &data[0..4] != b"Cr24" {
        return Err(ExtensionError::InvalidArgument(
            "Invalid CRX magic header".to_string(),
        ));
    }

    let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    let zip_offset = match version {
        2 => {
            if data.len() < 16 {
                return Err(ExtensionError::InvalidArgument(
                    "Invalid CRX2 header".to_string(),
                ));
            }
            let pub_key_len = u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize;
            let sig_len = u32::from_le_bytes([data[12], data[13], data[14], data[15]]) as usize;
            16usize
                .checked_add(pub_key_len)
                .and_then(|v| v.checked_add(sig_len))
                .ok_or_else(|| {
                    ExtensionError::InvalidArgument("CRX2 header overflow".to_string())
                })?
        }
        3 => {
            let header_len = u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize;
            12usize.checked_add(header_len).ok_or_else(|| {
                ExtensionError::InvalidArgument("CRX3 header overflow".to_string())
            })?
        }
        _ => {
            return Err(ExtensionError::InvalidArgument(format!(
                "Unsupported CRX version: {}",
                version
            )));
        }
    };

    if zip_offset >= data.len() {
        return Err(ExtensionError::InvalidArgument(
            "CRX header length exceeds file size".to_string(),
        ));
    }

    extract_zip_data(&data[zip_offset..], dest)
}

pub fn validate_extension_dir(path: &Path) -> ExtensionResult<Manifest> {
    let manifest_path = path.join("manifest.json");
    if !manifest_path.exists() {
        return Err(ExtensionError::NotFound(format!(
            "Extension manifest not found: {}",
            manifest_path.display()
        )));
    }

    let manifest = Manifest::from_file(&manifest_path)?;
    manifest.validate()?;
    Ok(manifest)
}

pub fn find_extension_root(root: &Path) -> Option<PathBuf> {
    if root.join("manifest.json").exists() {
        return Some(root.to_path_buf());
    }

    let mut best_match: Option<PathBuf> = None;

    for entry in WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        if entry.file_name().to_string_lossy() != "manifest.json" {
            continue;
        }

        let Some(parent) = entry.path().parent() else {
            continue;
        };

        let is_better_match = best_match
            .as_ref()
            .map(|current| parent.components().count() < current.components().count())
            .unwrap_or(true);

        if is_better_match {
            best_match = Some(parent.to_path_buf());
        }
    }

    best_match
}

fn extract_zip_data(zip_data: &[u8], dest: &Path) -> ExtensionResult<()> {
    fs::create_dir_all(dest)?;

    let cursor = std::io::Cursor::new(zip_data);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| ExtensionError::Runtime(format!("Failed to parse CRX ZIP payload: {}", e)))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| ExtensionError::Runtime(format!("Failed to read ZIP entry: {}", e)))?;

        let output_rel = match file.enclosed_name() {
            Some(path) => path.to_path_buf(),
            None => continue,
        };
        let output_path = dest.join(output_rel);

        if file.name().ends_with('/') {
            fs::create_dir_all(&output_path)?;
            continue;
        }

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut output = fs::File::create(&output_path)?;
        std::io::copy(&mut file, &mut output)?;
    }

    Ok(())
}
