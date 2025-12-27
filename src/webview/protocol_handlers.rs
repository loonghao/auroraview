//! Protocol handlers for custom URI schemes
//!
//! ## AuroraView Protocol URL Format
//!
//! The auroraview protocol supports several path prefixes:
//!
//! - `type:file` - For file:// URL conversions
//!   - `https://auroraview.localhost/type:file/C:/path/to/file.ext`
//!
//! - `type:local` - For local path conversions
//!   - `https://auroraview.localhost/type:local/C:/path/to/file.ext`
//!
//! - `extension/{extensionId}/` - For Chrome extension resources
//!   - `https://auroraview.localhost/extension/my-extension/sidepanel.html`
//!   - Maps to `%LOCALAPPDATA%/AuroraView/Extensions/{extensionId}/{path}`
//!
//! Both type prefixes allow loading arbitrary local files through the custom protocol.
//! The type prefix helps distinguish the source of the path for debugging.

use auroraview_core::assets::build_error_page;
use mime_guess::from_path;
use path_clean::PathClean;
use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};
use wry::http::{Request, Response};

// Import protocol type constants
use crate::utils::{PROTOCOL_TYPE_FILE, PROTOCOL_TYPE_LOCAL};

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

    // Check for type:file/ or type:local/ prefix - allows loading arbitrary local files
    // Format: https://auroraview.localhost/type:file/C:/path/to/file.ext
    //         https://auroraview.localhost/type:local/C:/path/to/file.ext
    // Both bypass the asset_root restriction for local file access
    let type_file_prefix = format!("{}/", PROTOCOL_TYPE_FILE);
    let type_local_prefix = format!("{}/", PROTOCOL_TYPE_LOCAL);

    if let Some(file_path) = path.strip_prefix(&type_file_prefix) {
        tracing::debug!("[Protocol] {} request: {}", PROTOCOL_TYPE_FILE, file_path);
        return handle_file_path_request(file_path);
    }

    if let Some(file_path) = path.strip_prefix(&type_local_prefix) {
        tracing::debug!("[Protocol] {} request: {}", PROTOCOL_TYPE_LOCAL, file_path);
        return handle_file_path_request(file_path);
    }

    // Check for extension/ prefix - serves Chrome extension resources
    // Format: https://auroraview.localhost/extension/{extensionId}/{path}
    // Maps to: %LOCALAPPDATA%/AuroraView/Extensions/{extensionId}/{path}
    if let Some(ext_path) = path.strip_prefix("extension/") {
        tracing::debug!("[Protocol] extension request: {}", ext_path);
        return handle_extension_request(ext_path);
    }

    // Build full path
    // Parse the path and determine if it's absolute
    let full_path = parse_protocol_path(path, asset_root);

    tracing::debug!(
        "[Protocol] auroraview:// request: {} -> {:?}",
        uri,
        full_path
    );

    // Determine if this is an HTML page request for error page rendering
    let is_html = is_html_page_request(path);

    // Security check: prevent directory traversal
    // Canonicalize both paths to resolve .. and symlinks
    let canonical_asset_root = match asset_root.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("[Protocol] Failed to canonicalize asset_root: {}", e);
            #[cfg(test)]
            eprintln!("[Protocol] ERROR: asset_root.canonicalize() failed: {}", e);
            return build_error_response(
                500,
                "Internal Server Error",
                "Failed to resolve asset root directory.",
                Some(&format!("Error: {}", e)),
                Some(&uri_str),
                is_html,
            );
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
            return build_error_response(
                404,
                "Not Found",
                &format!("The requested resource '{}' could not be found.", path),
                Some(&format!("Path: {:?}", full_path)),
                Some(&uri_str),
                is_html,
            );
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
        return build_error_response(
            403,
            "Forbidden",
            "Access to this resource is not allowed.",
            Some("Directory traversal attempt detected."),
            Some(&uri_str),
            is_html,
        );
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
            build_error_response(
                404,
                "Not Found",
                &format!("The requested resource '{}' could not be found.", path),
                Some(&format!("Error: {}", e)),
                Some(&uri_str),
                is_html,
            )
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

/// Handle file path requests via /file/ prefix in auroraview protocol
///
/// This allows loading arbitrary local files through the custom protocol:
/// - URL: `https://auroraview.localhost/file/C:/path/to/file.ext`
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

/// Handle extension resource requests
///
/// Maps URLs like `https://auroraview.localhost/extension/{extensionId}/{path}`
/// to local files in `%LOCALAPPDATA%/AuroraView/Extensions/{extensionId}/{path}`
///
/// This allows Chrome extensions to load their resources through the custom protocol,
/// avoiding the "Not allowed to load local resource" error for file:// URLs.
fn handle_extension_request(ext_path: &str) -> Response<Cow<'static, [u8]>> {
    tracing::debug!("[Protocol] extension request: {}", ext_path);

    // Parse extension ID and resource path
    // Format: {extensionId}/{path/to/resource}
    let parts: Vec<&str> = ext_path.splitn(2, '/').collect();
    if parts.is_empty() {
        tracing::warn!("[Protocol] Invalid extension path: {}", ext_path);
        return Response::builder()
            .status(400)
            .body(Cow::Borrowed(
                b"Bad Request: Invalid extension path" as &[u8],
            ))
            .unwrap();
    }

    let extension_id = parts[0];
    let resource_path = if parts.len() > 1 {
        parts[1]
    } else {
        "index.html"
    };

    // Get the extensions directory
    // On Windows: %LOCALAPPDATA%/AuroraView/Extensions
    // On macOS: ~/Library/Application Support/AuroraView/Extensions
    // On Linux: ~/.local/share/AuroraView/Extensions
    let extensions_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("AuroraView")
        .join("Extensions");

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
            return Response::builder()
                .status(404)
                .body(Cow::Borrowed(b"Extension not found" as &[u8]))
                .unwrap();
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
            return Response::builder()
                .status(404)
                .body(Cow::Borrowed(b"Resource not found" as &[u8]))
                .unwrap();
        }
    };

    // Verify the resource is within the extension directory (prevent directory traversal)
    if !canonical_full_path.starts_with(&canonical_ext_dir) {
        tracing::warn!(
            "[Protocol] Directory traversal attempt in extension: {:?}",
            full_path
        );
        return Response::builder()
            .status(403)
            .body(Cow::Borrowed(b"Forbidden: Directory traversal" as &[u8]))
            .unwrap();
    }

    // Read and serve the file
    match fs::read(&full_path) {
        Ok(data) => {
            let mime_type = guess_mime_type(&full_path);
            tracing::debug!(
                "[Protocol] Loaded extension resource: {} ({} bytes, {})",
                ext_path,
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
            tracing::warn!(
                "[Protocol] Failed to read extension resource: {:?} ({})",
                full_path,
                e
            );
            Response::builder()
                .status(404)
                .body(Cow::Borrowed(b"Resource not found" as &[u8]))
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

/// Build an error response with a styled HTML error page
///
/// This function creates an HTTP response with a custom error page that matches
/// the AuroraView design language. For non-HTML requests (like CSS, JS, images),
/// it returns a simple text response instead.
///
/// # Arguments
/// * `status` - HTTP status code
/// * `title` - Error title (e.g., "Internal Server Error")
/// * `message` - User-friendly error message
/// * `details` - Optional technical details
/// * `url` - Optional URL that caused the error
/// * `is_html_request` - Whether this is likely an HTML page request
pub fn build_error_response(
    status: u16,
    title: &str,
    message: &str,
    details: Option<&str>,
    url: Option<&str>,
    is_html_request: bool,
) -> Response<Cow<'static, [u8]>> {
    if is_html_request {
        // Return styled HTML error page
        let html = build_error_page(status, title, message, details, url);
        Response::builder()
            .status(status)
            .header("Content-Type", "text/html; charset=utf-8")
            .body(Cow::Owned(html.into_bytes()))
            .unwrap()
    } else {
        // Return simple text for non-HTML resources
        Response::builder()
            .status(status)
            .header("Content-Type", "text/plain; charset=utf-8")
            .body(Cow::Owned(format!("{}: {}", status, title).into_bytes()))
            .unwrap()
    }
}

/// Check if a request path looks like an HTML page request
///
/// Returns true for paths that:
/// - End with .html or .htm
/// - Have no extension (likely a route)
/// - Are the root path
fn is_html_page_request(path: &str) -> bool {
    let path_lower = path.to_lowercase();

    // Check for HTML extensions
    if path_lower.ends_with(".html") || path_lower.ends_with(".htm") {
        return true;
    }

    // Check for paths without extensions (likely routes)
    let last_segment = path.rsplit('/').next().unwrap_or(path);
    if !last_segment.contains('.') {
        return true;
    }

    // Root path
    if path.is_empty() || path == "/" || path == "index.html" {
        return true;
    }

    false
}

// Note: All tests have been moved to tests/protocol_handlers_integration_tests.rs
// This includes tests for:
// - handle_auroraview_protocol security (directory traversal, file access)
// - handle_custom_protocol with various callbacks
// - Protocol handling with subdirectories
// - Custom protocol with various response codes
