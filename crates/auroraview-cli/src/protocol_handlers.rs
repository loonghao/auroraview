//! Protocol handlers for custom URI schemes

use mime_guess::from_path;
use path_clean::PathClean;
use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};
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
    // Platform differences:
    // - macOS/Linux: "auroraview://file.txt" -> uri.path() returns the file path
    // - Windows: wry maps "auroraview" to "http://auroraview.xxx" format
    //   e.g., "http://auroraview.index.html" for "auroraview://index.html"
    //
    // We need to handle both cases
    let uri_str = uri.to_string();

    // Write debug info to log file (only in non-test builds)
    // This is used for debugging in DCC applications like Maya where stderr is redirected
    #[cfg(not(test))]
    {
        let _ = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(std::env::temp_dir().join("auroraview_debug.log"))
            .and_then(|mut f| {
                use std::io::Write;
                writeln!(f, "[DEBUG] Protocol handler called:")
                    .and_then(|_| writeln!(f, "  uri_str = {}", uri_str))
                    .and_then(|_| writeln!(f, "  uri.path() = {}", uri.path()))
                    .and_then(|_| writeln!(f, "  uri.host() = {:?}", uri.host()))
            });
    }

    // On Windows with wry, the URL format goes through these transformations:
    // 1. Python sends: https://auroraview.localhost/index.html
    // 2. wry converts to: auroraview://localhost/index.html (for the protocol handler)
    // 3. We need to extract: /index.html (the path after the host)
    //
    // For relative paths like ./assets/xxx, the browser resolves them as:
    // - Base: https://auroraview.localhost/index.html
    // - Relative: ./assets/xxx
    // - Result: https://auroraview.localhost/assets/xxx
    // - wry converts to: auroraview://localhost/assets/xxx
    //
    // So we need to extract the path component, which is everything after the host.
    let path = if let Some(stripped) = uri_str.strip_prefix("auroraview://localhost/") {
        // Windows format (converted by wry): auroraview://localhost/path
        stripped
    } else if uri_str.starts_with("auroraview://localhost") {
        // Windows format without trailing path (root)
        "index.html"
    } else if let Some(stripped) = uri_str.strip_prefix("https://auroraview.localhost/") {
        // Windows format with HTTPS (before wry conversion)
        stripped
    } else if let Some(stripped) = uri_str.strip_prefix("http://auroraview.localhost/") {
        // Windows format with HTTP (before wry conversion)
        stripped
    } else if let Some(idx) = uri_str.find("://") {
        // macOS/Linux format: auroraview://path/to/file
        // Or Windows fallback: auroraview://xxx
        let after_scheme = &uri_str[idx + 3..];
        after_scheme.trim_end_matches('/')
    } else {
        // Fallback to path() method
        uri.path().trim_start_matches('/')
    };

    // Trim trailing slashes from path
    let path = path.trim_end_matches('/');

    // Default to index.html if path is empty (root access)
    let path = if path.is_empty() { "index.html" } else { path };

    // Check for /file/ prefix - allows loading arbitrary local files
    // Format: https://auroraview.localhost/file/C:/path/to/file.ext
    // This bypasses the asset_root restriction for local file access
    if let Some(file_path) = path.strip_prefix("file/") {
        return handle_file_path_request(file_path);
    }

    // Build full path
    // Parse the path and determine if it's absolute
    let full_path = parse_protocol_path(path, asset_root);

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

/// Handle file:// protocol requests
///
/// Maps URLs like `file:///C:/path/to/file.txt` to local file system
/// WARNING: This bypasses WebView's default security restrictions
pub fn handle_file_protocol(request: Request<Vec<u8>>) -> Response<Cow<'static, [u8]>> {
    // Only handle GET requests
    if request.method() != "GET" {
        return Response::builder()
            .status(405)
            .body(Cow::Borrowed(b"Method Not Allowed" as &[u8]))
            .unwrap();
    }

    let uri = request.uri().to_string();
    tracing::debug!("[Protocol] file:// request: {}", uri);

    // Extract path from file:// URI
    // Examples:
    // - "file:///C:/path/file.txt" -> "C:/path/file.txt"
    // - "file:///path/to/file.txt" -> "/path/to/file.txt" (Unix)
    let path_str = if let Some(idx) = uri.find("file://") {
        let after_scheme = &uri[idx + 7..]; // Skip "file://"

        // On Windows, file:///C:/... has three slashes
        // On Unix, file:///path/... also has three slashes
        #[cfg(target_os = "windows")]
        {
            // Remove leading slash for Windows paths: /C:/... -> C:/...
            after_scheme.trim_start_matches('/')
        }
        #[cfg(not(target_os = "windows"))]
        {
            // Keep leading slash for Unix paths
            after_scheme
        }
    } else {
        tracing::warn!("[Protocol] Invalid file:// URI: {}", uri);
        return Response::builder()
            .status(400)
            .body(Cow::Borrowed(b"Bad Request" as &[u8]))
            .unwrap();
    };

    // URL decode the path (handle %20 etc.)
    let decoded_path = match urlencoding::decode(path_str) {
        Ok(p) => p.to_string(),
        Err(e) => {
            tracing::warn!("[Protocol] Failed to decode path: {} ({})", path_str, e);
            return Response::builder()
                .status(400)
                .body(Cow::Borrowed(b"Bad Request" as &[u8]))
                .unwrap();
        }
    };

    let file_path = Path::new(&decoded_path);
    tracing::debug!("[Protocol] Resolved file path: {:?}", file_path);

    // Read file
    match fs::read(file_path) {
        Ok(data) => {
            let mime_type = guess_mime_type(file_path);
            tracing::debug!(
                "[Protocol] Loaded file:// {} ({} bytes, {})",
                decoded_path,
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
            tracing::warn!("[Protocol] File not found: {:?} ({})", file_path, e);
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
#[allow(dead_code)]
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

/// Handle /file/ protocol requests for loading arbitrary local files
///
/// This allows loading arbitrary local files through the custom protocol:
/// - URL: https://auroraview.localhost/file/C:/path/to/file.ext
/// - Returns the file content with appropriate MIME type
///
/// **Security Note**: This bypasses asset_root restrictions. Use with caution.
fn handle_file_path_request(file_path: &str) -> Response<Cow<'static, [u8]>> {
    tracing::debug!("[Protocol] /file/ request: {}", file_path);

    // URL decode the path (handle %20 etc.)
    let decoded_path = match urlencoding::decode(file_path) {
        Ok(p) => p.to_string(),
        Err(e) => {
            tracing::warn!(
                "[Protocol] Failed to decode file path: {} ({})",
                file_path,
                e
            );
            return Response::builder()
                .status(400)
                .body(Cow::Borrowed(
                    b"Bad Request: Invalid path encoding" as &[u8],
                ))
                .unwrap();
        }
    };

    // Normalize Windows path without colon (e.g., "C/path/..." -> "C:/path/...")
    let normalized_path = if is_windows_absolute_path_without_colon(&decoded_path) {
        normalize_windows_path_without_colon(&decoded_path)
    } else {
        decoded_path
    };

    let path = Path::new(&normalized_path);
    tracing::debug!("[Protocol] Resolved /file/ path: {:?}", path);

    // Read file
    match fs::read(path) {
        Ok(data) => {
            let mime_type = guess_mime_type(path);
            tracing::debug!(
                "[Protocol] Loaded /file/ {} ({} bytes, {})",
                normalized_path,
                data.len(),
                mime_type
            );

            Response::builder()
                .status(200)
                .header("Content-Type", mime_type.as_str())
                .header("Access-Control-Allow-Origin", "*")
                .body(Cow::Owned(data))
                .unwrap()
        }
        Err(e) => {
            tracing::warn!("[Protocol] /file/ not found: {:?} ({})", path, e);
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

/// Parse a protocol path and resolve it to a full file system path
///
/// This function handles both relative and absolute paths in the protocol URL:
/// - Relative paths (e.g., "css/style.css") are joined with asset_root
/// - Absolute paths (e.g., "c/users/..." or "/path/...") are used directly
///
/// For Windows paths in the format "c/users/..." (lowercase drive, no colon),
/// this function normalizes them to "C:/users/..." format.
pub fn parse_protocol_path(path_str: &str, asset_root: &Path) -> PathBuf {
    // Try to parse as a path
    let path = PathBuf::from(path_str);

    // Check if it's an absolute path using standard library
    if path.is_absolute() {
        // Already absolute (e.g., "/path/..." on Unix or "C:\path\..." on Windows)
        return path.clean();
    }

    // Check for Windows-style absolute path without colon (e.g., "c/users/...")
    // This is a special case where the URL protocol strips the colon
    if is_windows_absolute_path_without_colon(path_str) {
        let normalized = normalize_windows_path_without_colon(path_str);
        return PathBuf::from(normalized).clean();
    }

    // Relative path - join with asset_root and clean
    asset_root.join(path).clean()
}

/// Check if a path string is a Windows absolute path without colon
///
/// Detects patterns like "c/users/..." where the drive letter is followed
/// directly by a forward slash instead of a colon.
pub fn is_windows_absolute_path_without_colon(path: &str) -> bool {
    if path.len() < 2 {
        return false;
    }

    let chars: Vec<char> = path.chars().collect();

    // Check for pattern: [a-zA-Z]/...
    chars[0].is_ascii_alphabetic() && chars[1] == '/'
}

/// Normalize a Windows path without colon to standard format
///
/// Converts "c/users/..." to "C:/users/..."
pub fn normalize_windows_path_without_colon(path: &str) -> String {
    if path.len() < 2 {
        return path.to_string();
    }

    let drive_letter = path.chars().next().unwrap().to_ascii_uppercase();
    let rest = &path[1..]; // This includes the leading "/"

    // Convert to standard Windows path format: C:/users/...
    format!("{}:{}", drive_letter, rest)
}

// Note: All tests have been moved to tests/protocol_handlers_integration_tests.rs
// This includes tests for:
// - handle_auroraview_protocol security (directory traversal, file access)
// - handle_custom_protocol with various callbacks
// - Protocol handling with subdirectories
// - Custom protocol with various response codes
