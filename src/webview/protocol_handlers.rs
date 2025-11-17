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
            return Response::builder()
                .status(500)
                .body(Cow::Borrowed(b"Internal Server Error" as &[u8]))
                .unwrap();
        }
    };

    let canonical_full_path = match full_path.canonicalize() {
        Ok(p) => p,
        Err(_) => {
            // File doesn't exist or can't be accessed
            tracing::warn!("[Protocol] File not found or inaccessible: {:?}", full_path);
            return Response::builder()
                .status(404)
                .body(Cow::Borrowed(b"Not Found" as &[u8]))
                .unwrap();
        }
    };

    if !canonical_full_path.starts_with(&canonical_asset_root) {
        tracing::warn!(
            "[Protocol] Directory traversal attempt: {:?} not in {:?}",
            canonical_full_path,
            canonical_asset_root
        );
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

    #[test]
    fn test_guess_mime_type() {
        // Test common web file types
        assert_eq!(guess_mime_type(Path::new("test.html")), "text/html");
        assert_eq!(guess_mime_type(Path::new("test.css")), "text/css");
        assert_eq!(
            guess_mime_type(Path::new("test.js")),
            "text/javascript" // mime_guess uses text/javascript (RFC 9239)
        );
        assert_eq!(guess_mime_type(Path::new("test.png")), "image/png");

        // Test unknown extension defaults to octet-stream
        assert_eq!(
            guess_mime_type(Path::new("test.unknown")),
            "application/octet-stream"
        );

        // Test additional file types supported by mime_guess
        assert_eq!(guess_mime_type(Path::new("test.json")), "application/json");
        assert_eq!(guess_mime_type(Path::new("test.svg")), "image/svg+xml");
        assert_eq!(guess_mime_type(Path::new("test.woff2")), "font/woff2");
        assert_eq!(guess_mime_type(Path::new("test.mp4")), "video/mp4");
    }

    #[test]
    fn test_guess_mime_type_dcc_formats() {
        // Test DCC-specific file formats
        // Note: mime_guess returns actual MIME types from its database
        assert_eq!(
            guess_mime_type(Path::new("model.fbx")),
            "application/octet-stream" // FBX not in mime_guess database
        );
        assert_eq!(
            guess_mime_type(Path::new("scene.usd")),
            "application/octet-stream" // USD not in mime_guess database
        );
        assert_eq!(
            guess_mime_type(Path::new("texture.exr")),
            "application/octet-stream" // EXR not in mime_guess database
        );
        assert_eq!(
            guess_mime_type(Path::new("geometry.obj")),
            "application/x-tgif" // OBJ is registered as TGIF format in mime_guess
        );
    }

    #[test]
    fn test_guess_mime_type_modern_formats() {
        // Test modern web formats
        assert_eq!(guess_mime_type(Path::new("image.avif")), "image/avif");
        assert_eq!(guess_mime_type(Path::new("image.webp")), "image/webp");
        assert_eq!(guess_mime_type(Path::new("app.wasm")), "application/wasm");
        // TypeScript (.ts) shares extension with MPEG transport stream
        // mime_guess returns the video MIME type, not TypeScript
        assert_eq!(
            guess_mime_type(Path::new("script.ts")),
            "video/vnd.dlna.mpeg-tts" // MPEG transport stream (not TypeScript)
        );
    }

    #[test]
    fn test_handle_auroraview_protocol_security() {
        use std::fs;
        use tempfile::TempDir;

        // Create temporary directory structure
        let temp_dir = TempDir::new().unwrap();
        let asset_root = temp_dir.path();

        // Create a file inside asset_root
        let safe_file = asset_root.join("safe.txt");
        fs::write(&safe_file, b"Safe content").unwrap();

        // Create a file outside asset_root
        let outside_dir = TempDir::new().unwrap();
        let unsafe_file = outside_dir.path().join("unsafe.txt");
        fs::write(&unsafe_file, b"Unsafe content").unwrap();

        // Test 1: Valid request within asset_root
        let request = Request::builder()
            .method("GET")
            .uri("auroraview://safe.txt")
            .body(vec![])
            .unwrap();

        let response = handle_auroraview_protocol(asset_root, request);
        assert_eq!(response.status(), 200);

        // Test 2: Directory traversal attempt (should be blocked)
        let request = Request::builder()
            .method("GET")
            .uri("auroraview://../../../etc/passwd")
            .body(vec![])
            .unwrap();

        let response = handle_auroraview_protocol(asset_root, request);
        // Should return 403 Forbidden or 404 Not Found
        assert!(response.status() == 403 || response.status() == 404);

        // Test 3: Non-GET request
        let request = Request::builder()
            .method("POST")
            .uri("auroraview://safe.txt")
            .body(vec![])
            .unwrap();

        let response = handle_auroraview_protocol(asset_root, request);
        assert_eq!(response.status(), 405); // Method Not Allowed
    }

    #[test]
    fn test_handle_custom_protocol() {
        use std::sync::Arc;

        // Create a simple callback
        // Note: The URI passed to callback is the full URI string from request.uri().to_string()
        let callback = Arc::new(|uri: &str| -> Option<(Vec<u8>, String, u16)> {
            if uri == "test://hello.txt" || uri == "test://hello.txt/" {
                Some((b"Hello, World!".to_vec(), "text/plain".to_string(), 200))
            } else {
                None
            }
        });

        // Test 1: Successful request
        let request = Request::builder()
            .uri("test://hello.txt")
            .body(vec![])
            .unwrap();

        let response = handle_custom_protocol(&*callback, request);
        assert_eq!(response.status(), 200);
        assert_eq!(
            response.headers().get("Content-Type").unwrap(),
            "text/plain"
        );

        // Test 2: Not found
        let request = Request::builder()
            .uri("test://notfound.txt")
            .body(vec![])
            .unwrap();

        let response = handle_custom_protocol(&*callback, request);
        assert_eq!(response.status(), 404);
    }
}
