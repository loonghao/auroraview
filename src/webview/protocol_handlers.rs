//! Protocol handlers for custom URI schemes

use mime_guess::from_path;
use std::borrow::Cow;
use std::fs;
use std::path::Path;
use wry::http::{Request, Response};

/// Handle auroraview:// protocol requests
///
/// Maps URLs like `auroraview://css/style.css` to `{asset_root}/css/style.css`
pub fn handle_auroraview_protocol(
    asset_root: &Path,
    request: Request<Vec<u8>>,
) -> Response<Cow<'static, [u8]>> {
    // Only handle GET requests
    if request.method() != "GET" {
        return Response::builder()
            .status(405)
            .body(Cow::Borrowed(b"Method Not Allowed" as &[u8]))
            .unwrap();
    }

    // Extract path from URI
    let uri = request.uri();

    // For custom protocols, we need to extract the path from the full URI
    // Examples:
    // - "auroraview://file.txt" -> uri.to_string() = "auroraview://file.txt/"
    // - uri.path() may return "/" instead of the actual path
    // So we parse the URI string directly
    let uri_str = uri.to_string();
    let path = if let Some(idx) = uri_str.find("://") {
        // Extract everything after "://"
        let after_scheme = &uri_str[idx + 3..];
        // Remove trailing slash if present
        after_scheme.trim_end_matches('/')
    } else {
        // Fallback to path() method
        uri.path().trim_start_matches('/')
    };

    // Build full path
    let full_path = asset_root.join(path);

    tracing::debug!(
        "[Protocol] auroraview:// request: {} -> {:?}",
        uri,
        full_path
    );

    // Security check: prevent directory traversal
    // Canonicalize both paths to resolve .. and symlinks
    let canonical_asset_root = match asset_root.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("[Protocol] Failed to canonicalize asset_root: {}", e);
            #[cfg(test)]
            eprintln!("[Protocol] ERROR: asset_root.canonicalize() failed: {}", e);
            return Response::builder()
                .status(500)
                .body(Cow::Borrowed(b"Internal Server Error" as &[u8]))
                .unwrap();
        }
    };

    let canonical_full_path = match full_path.canonicalize() {
        Ok(p) => p,
        Err(_e) => {
            // File doesn't exist or can't be accessed
            tracing::warn!("[Protocol] File not found or inaccessible: {:?}", full_path);
            #[cfg(test)]
            eprintln!(
                "[Protocol] full_path.canonicalize() failed: {} (path: {:?})",
                _e, full_path
            );
            return Response::builder()
                .status(404)
                .body(Cow::Borrowed(b"Not Found" as &[u8]))
                .unwrap();
        }
    };

    #[cfg(test)]
    {
        eprintln!(
            "[Protocol] canonical_asset_root = {:?}",
            canonical_asset_root
        );
        eprintln!("[Protocol] canonical_full_path = {:?}", canonical_full_path);
        eprintln!(
            "[Protocol] starts_with check = {}",
            canonical_full_path.starts_with(&canonical_asset_root)
        );
    }

    if !canonical_full_path.starts_with(&canonical_asset_root) {
        tracing::warn!(
            "[Protocol] Directory traversal attempt: {:?} not in {:?}",
            canonical_full_path,
            canonical_asset_root
        );
        #[cfg(test)]
        eprintln!("[Protocol] Returning 403 Forbidden");
        return Response::builder()
            .status(403)
            .body(Cow::Borrowed(b"Forbidden" as &[u8]))
            .unwrap();
    }

    // Read file
    match fs::read(&full_path) {
        Ok(data) => {
            let mime_type = guess_mime_type(&full_path);
            tracing::debug!(
                "[Protocol] Loaded {} ({} bytes, {})",
                path,
                data.len(),
                mime_type
            );

            Response::builder()
                .status(200)
                .header("Content-Type", mime_type.as_str())
                .body(Cow::Owned(data))
                .unwrap()
        }
        Err(e) => {
            tracing::warn!("[Protocol] File not found: {:?} ({})", full_path, e);
            Response::builder()
                .status(404)
                .body(Cow::Borrowed(b"Not Found" as &[u8]))
                .unwrap()
        }
    }
}

/// Handle custom protocol requests using user-provided callback
///
/// Calls the Python callback and converts the result to HTTP response
#[allow(clippy::type_complexity)]
pub fn handle_custom_protocol(
    callback: &dyn Fn(&str) -> Option<(Vec<u8>, String, u16)>,
    request: Request<Vec<u8>>,
) -> Response<Cow<'static, [u8]>> {
    let uri = request.uri().to_string();

    tracing::debug!("[Protocol] Custom protocol request: {}", uri);

    match callback(&uri) {
        Some((data, mime_type, status)) => {
            tracing::debug!(
                "[Protocol] Custom handler returned {} bytes (status: {})",
                data.len(),
                status
            );

            Response::builder()
                .status(status)
                .header("Content-Type", mime_type)
                .body(Cow::Owned(data))
                .unwrap()
        }
        None => {
            tracing::warn!("[Protocol] Custom handler returned None for: {}", uri);
            Response::builder()
                .status(404)
                .body(Cow::Borrowed(b"Not Found" as &[u8]))
                .unwrap()
        }
    }
}

/// Guess MIME type from file extension using mime_guess crate
///
/// This function uses the `mime_guess` crate which maintains a comprehensive
/// database of MIME types based on file extensions. It supports 1000+ file types
/// and is regularly updated.
fn guess_mime_type(path: &Path) -> String {
    from_path(path).first_or_octet_stream().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    /// Unit test: Verify MIME type detection for common web file types
    #[rstest]
    #[case("test.html", "text/html")]
    #[case("test.css", "text/css")]
    #[case("test.js", "text/javascript")] // mime_guess uses text/javascript (RFC 9239)
    #[case("test.png", "image/png")]
    #[case("test.unknown", "application/octet-stream")] // Unknown extension defaults to octet-stream
    #[case("test.json", "application/json")]
    #[case("test.svg", "image/svg+xml")]
    #[case("test.woff2", "font/woff2")]
    #[case("test.mp4", "video/mp4")]
    fn test_guess_mime_type(#[case] filename: &str, #[case] expected_mime: &str) {
        assert_eq!(guess_mime_type(Path::new(filename)), expected_mime);
    }

    /// Unit test: Verify MIME type detection for DCC-specific file formats
    #[rstest]
    #[case("model.fbx", "application/octet-stream")] // FBX not in mime_guess database
    #[case("scene.usd", "application/octet-stream")] // USD not in mime_guess database
    #[case("texture.exr", "application/octet-stream")] // EXR not in mime_guess database
    #[case("geometry.obj", "application/x-tgif")] // OBJ is registered as TGIF format in mime_guess
    fn test_guess_mime_type_dcc_formats(#[case] filename: &str, #[case] expected_mime: &str) {
        assert_eq!(guess_mime_type(Path::new(filename)), expected_mime);
    }

    /// Unit test: Verify MIME type detection for modern web formats
    #[rstest]
    #[case("image.avif", "image/avif")]
    #[case("image.webp", "image/webp")]
    #[case("app.wasm", "application/wasm")]
    #[case("script.ts", "video/vnd.dlna.mpeg-tts")] // MPEG transport stream (not TypeScript)
    fn test_guess_mime_type_modern_formats(#[case] filename: &str, #[case] expected_mime: &str) {
        assert_eq!(guess_mime_type(Path::new(filename)), expected_mime);
    }
}

// Note: Integration tests have been moved to tests/protocol_handlers_integration_tests.rs
// This includes tests for:
// - handle_auroraview_protocol security (directory traversal, file access)
// - handle_custom_protocol with various callbacks
// - Protocol handling with subdirectories
// - Custom protocol with various response codes
