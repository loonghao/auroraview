//! Extension handling for WebView module
//!
//! This module contains Windows-specific functions for installing
//! bundled extensions and handling extension resource requests.

#[cfg(target_os = "windows")]
use anyhow::Context as AnyhowContext;

#[cfg(target_os = "windows")]
use std::borrow::Cow;

#[cfg(target_os = "windows")]
use mime_guess::from_path;
#[cfg(target_os = "windows")]
use wry::http::Response;

#[cfg(target_os = "windows")]
use crate::packed::utils::get_extensions_dir;
#[cfg(target_os = "windows")]
use auroraview_pack::OverlayData;

/// Install bundled extensions from overlay assets into shared extension directory
#[cfg(target_os = "windows")]
pub fn install_bundled_extensions_from_assets(overlay: &OverlayData) -> anyhow::Result<usize> {
    let extensions_dir = get_extensions_dir();
    std::fs::create_dir_all(&extensions_dir).with_context(|| {
        format!(
            "Failed to create extensions dir: {}",
            extensions_dir.display()
        )
    })?;

    let mut installed = 0usize;

    for (asset_path, content) in &overlay.assets {
        if !asset_path.starts_with("extensions/") {
            continue;
        }

        let relative = asset_path.trim_start_matches("extensions/");
        if relative.is_empty() {
            continue;
        }

        let target_path = extensions_dir.join(relative.replace('/', "\\"));
        if let Some(parent) = target_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create dir: {}", parent.display()))?;
        }

        std::fs::write(&target_path, content).with_context(|| {
            format!("Failed to write extension asset: {}", target_path.display())
        })?;

        installed += 1;
    }

    Ok(installed)
}

/// Handle extension resource requests in the custom protocol
///
/// Maps URLs like `https://auroraview.localhost/extension/{extensionId}/{path}`
/// to local files in `%LOCALAPPDATA%/AuroraView/Extensions/{extensionId}/{path}`
#[cfg(target_os = "windows")]
pub fn handle_extension_resource_request(
    ext_path: &str,
    allowed_origin: &str,
) -> Response<Cow<'static, [u8]>> {
    /// Build an error response for extension protocol handlers.
    fn ext_error(status: u16, body: &'static [u8]) -> Response<Cow<'static, [u8]>> {
        Response::builder()
            .status(status)
            .body(Cow::Borrowed(body))
            .expect("hardcoded status/body should always produce a valid response")
    }

    tracing::debug!("[Protocol] extension resource request: {}", ext_path);

    // Parse extension ID and resource path
    // Format: {extensionId}/{path/to/resource}
    let parts: Vec<&str> = ext_path.splitn(2, '/').collect();
    if parts.is_empty() {
        tracing::warn!("[Protocol] Invalid extension path: {}", ext_path);
        return ext_error(400, b"Bad Request: Invalid extension path");
    }

    let extension_id = parts[0];
    let resource_path = if parts.len() > 1 {
        parts[1]
    } else {
        "index.html"
    };

    // Get the extensions directory
    let extensions_dir = get_extensions_dir();

    // Build full path to the resource
    let full_path = extensions_dir.join(extension_id).join(resource_path);

    tracing::debug!(
        "[Protocol] Extension resource: {} -> {:?}",
        ext_path,
        full_path
    );

    // Security check: ensure the path is within the extension directory
    let canonical_ext_dir = match extensions_dir.join(extension_id).canonicalize() {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!(
                "[Protocol] Extension directory not found: {} ({})",
                extension_id,
                e
            );
            return ext_error(404, b"Extension not found");
        }
    };

    let canonical_full_path = match full_path.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!(
                "[Protocol] Extension resource not found: {:?} ({})",
                full_path,
                e
            );
            return ext_error(404, b"Resource not found");
        }
    };

    // Verify the resource is within the extension directory (prevent directory traversal)
    if !canonical_full_path.starts_with(&canonical_ext_dir) {
        tracing::warn!(
            "[Protocol] Directory traversal attempt in extension: {:?}",
            full_path
        );
        return ext_error(403, b"Forbidden: Directory traversal");
    }

    // Read and serve the file
    match std::fs::read(&full_path) {
        Ok(data) => {
            let mime_type = from_path(&full_path).first_or_octet_stream().to_string();
            tracing::debug!(
                "[Protocol] Loaded extension resource: {} ({} bytes, {})",
                ext_path,
                data.len(),
                mime_type
            );

            Response::builder()
                .status(200)
                .header("Content-Type", mime_type)
                .header("Access-Control-Allow-Origin", allowed_origin)
                .body(Cow::Owned(data))
                .expect("valid 200 response with content-type and CORS headers")
        }
        Err(e) => {
            tracing::warn!(
                "[Protocol] Failed to read extension resource: {:?} ({})",
                full_path,
                e
            );
            ext_error(404, b"Resource not found")
        }
    }
}
