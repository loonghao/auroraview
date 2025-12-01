//! Protocol handling utilities
//!
//! Common utilities for handling custom protocols in WebView applications.

use mime_guess::from_path;
use path_clean::PathClean;
use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};

/// Normalize a URL for display/storage
///
/// Adds https:// prefix if no scheme is present
pub fn normalize_url(url: &str) -> String {
    let url = url.trim();
    if url.is_empty() {
        return String::new();
    }

    // Already has a scheme
    if url.contains("://") {
        return url.to_string();
    }

    // Add https:// prefix
    format!("https://{}", url)
}

/// Extract path from a custom protocol URI
///
/// Handles various formats:
/// - `auroraview://localhost/path` -> `path`
/// - `https://auroraview.localhost/path` -> `path`
/// - `auroraview://path` -> `path`
#[allow(clippy::manual_map)]
pub fn extract_protocol_path(uri: &str, protocol_name: &str) -> Option<String> {
    let prefix_with_localhost = format!("{}://localhost/", protocol_name);
    let prefix_https = format!("https://{}.localhost/", protocol_name);
    let prefix_http = format!("http://{}.localhost/", protocol_name);
    let prefix_simple = format!("{}://", protocol_name);

    if let Some(path) = uri.strip_prefix(&prefix_with_localhost) {
        Some(path.to_string())
    } else if uri.starts_with(&format!("{}://localhost", protocol_name)) {
        Some("index.html".to_string())
    } else if let Some(path) = uri.strip_prefix(&prefix_https) {
        Some(path.to_string())
    } else if let Some(path) = uri.strip_prefix(&prefix_http) {
        Some(path.to_string())
    } else if let Some(path) = uri.strip_prefix(&prefix_simple) {
        Some(path.to_string())
    } else {
        None
    }
}

/// Resolve a relative path safely within a root directory
///
/// Returns None if the resolved path would escape the root
pub fn resolve_safe_path(root: &Path, relative_path: &str) -> Option<PathBuf> {
    // Clean the path
    let relative_path = relative_path.trim_start_matches('/');
    let relative_path = relative_path.replace("..", "");

    // Join and canonicalize
    let full_path = root.join(relative_path);

    // Verify the path is within root
    if let (Ok(canonical_root), Ok(canonical_path)) =
        (dunce::canonicalize(root), dunce::canonicalize(&full_path))
    {
        if canonical_path.starts_with(&canonical_root) {
            return Some(canonical_path);
        }
    }

    // If canonicalization fails, check the non-canonical path
    let clean_path = full_path.to_string_lossy().replace("..", "");
    let clean_path = PathBuf::from(clean_path);

    if clean_path.starts_with(root) && clean_path.exists() {
        Some(clean_path)
    } else {
        None
    }
}

/// Guess MIME type from file path
pub fn guess_mime_type(path: &Path) -> String {
    from_path(path).first_or_octet_stream().to_string()
}

/// File response for protocol handlers
#[derive(Debug)]
pub struct FileResponse {
    /// File content
    pub data: Cow<'static, [u8]>,
    /// MIME type
    pub mime_type: String,
    /// HTTP status code
    pub status: u16,
}

impl FileResponse {
    /// Create a successful response
    pub fn ok(data: Vec<u8>, mime_type: String) -> Self {
        Self {
            data: Cow::Owned(data),
            mime_type,
            status: 200,
        }
    }

    /// Create a not found response
    pub fn not_found() -> Self {
        Self {
            data: Cow::Borrowed(b"Not Found"),
            mime_type: "text/plain".to_string(),
            status: 404,
        }
    }

    /// Create a forbidden response
    pub fn forbidden() -> Self {
        Self {
            data: Cow::Borrowed(b"Forbidden"),
            mime_type: "text/plain".to_string(),
            status: 403,
        }
    }

    /// Create an internal error response
    pub fn internal_error(msg: &str) -> Self {
        Self {
            data: Cow::Owned(msg.as_bytes().to_vec()),
            mime_type: "text/plain".to_string(),
            status: 500,
        }
    }
}

/// Load a file from asset root and return a response
pub fn load_asset_file(asset_root: &Path, relative_path: &str) -> FileResponse {
    // Clean the path to prevent directory traversal
    let clean_path = Path::new(relative_path)
        .components()
        .filter(|c| !matches!(c, std::path::Component::ParentDir))
        .collect::<PathBuf>()
        .clean();

    let file_path = asset_root.join(&clean_path);

    // Verify path is within asset root
    match (
        dunce::canonicalize(asset_root),
        dunce::canonicalize(&file_path),
    ) {
        (Ok(root), Ok(full)) if full.starts_with(&root) => {
            // Safe path, read file
            match fs::read(&full) {
                Ok(data) => {
                    let mime = guess_mime_type(&full);
                    FileResponse::ok(data, mime)
                }
                Err(_) => FileResponse::not_found(),
            }
        }
        _ => {
            // Path escape attempt or file doesn't exist
            if file_path.exists() {
                match fs::read(&file_path) {
                    Ok(data) => {
                        let mime = guess_mime_type(&file_path);
                        FileResponse::ok(data, mime)
                    }
                    Err(_) => FileResponse::not_found(),
                }
            } else {
                FileResponse::not_found()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_url() {
        assert_eq!(normalize_url("example.com"), "https://example.com");
        assert_eq!(normalize_url("https://example.com"), "https://example.com");
        assert_eq!(normalize_url("http://example.com"), "http://example.com");
        assert_eq!(normalize_url("file:///path"), "file:///path");
        assert_eq!(normalize_url(""), "");
    }

    #[test]
    fn test_extract_protocol_path() {
        assert_eq!(
            extract_protocol_path("auroraview://localhost/index.html", "auroraview"),
            Some("index.html".to_string())
        );
        assert_eq!(
            extract_protocol_path("auroraview://localhost", "auroraview"),
            Some("index.html".to_string())
        );
        assert_eq!(
            extract_protocol_path("https://auroraview.localhost/css/style.css", "auroraview"),
            Some("css/style.css".to_string())
        );
        assert_eq!(
            extract_protocol_path("auroraview://path/to/file", "auroraview"),
            Some("path/to/file".to_string())
        );
        assert_eq!(
            extract_protocol_path("http://example.com", "auroraview"),
            None
        );
    }

    #[test]
    fn test_guess_mime_type() {
        assert_eq!(guess_mime_type(Path::new("style.css")), "text/css");
        assert_eq!(guess_mime_type(Path::new("script.js")), "text/javascript");
        assert_eq!(guess_mime_type(Path::new("index.html")), "text/html");
        assert_eq!(guess_mime_type(Path::new("image.png")), "image/png");
    }

    #[test]
    fn test_file_response() {
        let resp = FileResponse::ok(b"hello".to_vec(), "text/plain".to_string());
        assert_eq!(resp.status, 200);

        let resp = FileResponse::not_found();
        assert_eq!(resp.status, 404);

        let resp = FileResponse::forbidden();
        assert_eq!(resp.status, 403);
    }
}
