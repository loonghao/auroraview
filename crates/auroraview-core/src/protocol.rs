//! Protocol handling utilities
//!
//! Common utilities for handling custom protocols in WebView applications.
//!
//! ## AuroraView Protocol URL Format
//!
//! Local files are accessed through the custom protocol with type prefixes:
//!
//! - `type:file` - Converted from file:// URLs
//!   - `file:///C:/path/to/file.ext` → `https://auroraview.localhost/type:file/C:/path/to/file.ext`
//!
//! - `type:local` - Converted from local file paths
//!   - `C:/path/to/file.ext` → `https://auroraview.localhost/type:local/C:/path/to/file.ext`
//!   - `/path/to/file.ext` → `https://auroraview.localhost/type:local/path/to/file.ext`
//!
//! The type prefix helps distinguish the source of the path for debugging and logging.

use mime_guess::from_path;
use path_clean::PathClean;
use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};

// ============================================================================
// Protocol Constants
// ============================================================================

/// The base hostname for AuroraView custom protocol
pub const AURORAVIEW_HOST: &str = "auroraview.localhost";

/// Protocol type prefix for file:// URL conversions
pub const PROTOCOL_TYPE_FILE: &str = "type:file";

/// Protocol type prefix for local path conversions
pub const PROTOCOL_TYPE_LOCAL: &str = "type:local";

// ============================================================================
// URL Conversion Functions
// ============================================================================

/// Convert a file:// URL to auroraview protocol format
///
/// This converts `file:///path/to/file` to `https://auroraview.localhost/type:file/path/to/file`
/// which allows loading local files through the custom protocol handler.
///
/// # Arguments
/// * `file_url` - A file:// URL string
///
/// # Returns
/// An auroraview protocol URL string with `type:file` prefix
///
/// # Examples
/// ```
/// use auroraview_core::protocol::file_url_to_auroraview;
///
/// let url = file_url_to_auroraview("file:///C:/path/to/file.html");
/// assert_eq!(url, "https://auroraview.localhost/type:file/C:/path/to/file.html");
/// ```
pub fn file_url_to_auroraview(file_url: &str) -> String {
    // Extract path from file:// URL
    let path = if let Some(stripped) = file_url.strip_prefix("file:///") {
        stripped
    } else if let Some(stripped) = file_url.strip_prefix("file://") {
        stripped
    } else {
        file_url
    };

    // Normalize path separators
    let normalized_path = path.replace('\\', "/");

    format!(
        "https://{}/{}/{}",
        AURORAVIEW_HOST, PROTOCOL_TYPE_FILE, normalized_path
    )
}

/// Convert a local file path to auroraview protocol format
///
/// This converts local paths to `https://auroraview.localhost/type:local/path/to/file`
/// which allows loading local files through the custom protocol handler.
///
/// # Arguments
/// * `local_path` - A local file path string (e.g., `C:/path/to/file` or `/path/to/file`)
///
/// # Returns
/// An auroraview protocol URL string with `type:local` prefix
///
/// # Examples
/// ```
/// use auroraview_core::protocol::local_path_to_auroraview;
///
/// let url = local_path_to_auroraview("C:/path/to/file.html");
/// assert_eq!(url, "https://auroraview.localhost/type:local/C:/path/to/file.html");
///
/// let url = local_path_to_auroraview("/path/to/file.html");
/// assert_eq!(url, "https://auroraview.localhost/type:local/path/to/file.html");
/// ```
pub fn local_path_to_auroraview(local_path: &str) -> String {
    // Normalize path separators
    let normalized_path = local_path.replace('\\', "/");

    // Remove leading slash for consistency (Unix paths start with /)
    let path = normalized_path.trim_start_matches('/');

    format!(
        "https://{}/{}/{}",
        AURORAVIEW_HOST, PROTOCOL_TYPE_LOCAL, path
    )
}

/// Check if a path string matches a protocol type prefix
///
/// # Arguments
/// * `path` - The path to check (e.g., "type:file/C:/path/to/file")
/// * `protocol_type` - The protocol type to match (e.g., PROTOCOL_TYPE_FILE)
///
/// # Returns
/// The remaining path after the prefix if matched, None otherwise
pub fn strip_protocol_type<'a>(path: &'a str, protocol_type: &str) -> Option<&'a str> {
    let prefix = format!("{}/", protocol_type);
    path.strip_prefix(&prefix)
}

/// Check if a URL is an auroraview protocol URL
///
/// # Examples
/// ```
/// use auroraview_core::protocol::is_auroraview_url;
///
/// assert!(is_auroraview_url("https://auroraview.localhost/type:file/C:/path"));
/// assert!(is_auroraview_url("auroraview://localhost/index.html"));
/// assert!(!is_auroraview_url("https://example.com"));
/// ```
pub fn is_auroraview_url(url: &str) -> bool {
    url.contains("auroraview.localhost") || url.starts_with("auroraview://")
}

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

// ============================================================================
// In-Memory Asset Protocol Handler
// ============================================================================

/// In-memory asset store for protocol handlers
///
/// This struct provides a way to serve assets from memory (e.g., embedded assets
/// in packed applications) through the custom protocol handler.
#[derive(Clone)]
pub struct MemoryAssets {
    /// Map of path -> content
    assets: std::collections::HashMap<String, Vec<u8>>,
    /// Optional loading HTML for special `__loading__` path
    loading_html: Option<String>,
}

impl MemoryAssets {
    /// Create a new empty asset store
    pub fn new() -> Self {
        Self {
            assets: std::collections::HashMap::new(),
            loading_html: None,
        }
    }

    /// Create from a HashMap of assets
    pub fn from_map(assets: std::collections::HashMap<String, Vec<u8>>) -> Self {
        Self {
            assets,
            loading_html: None,
        }
    }

    /// Create from a Vec of (path, content) tuples
    pub fn from_vec(assets: Vec<(String, Vec<u8>)>) -> Self {
        Self {
            assets: assets.into_iter().collect(),
            loading_html: None,
        }
    }

    /// Set the loading HTML for the `__loading__` path
    pub fn with_loading_html(mut self, html: String) -> Self {
        self.loading_html = Some(html);
        self
    }

    /// Add an asset
    pub fn insert(&mut self, path: String, content: Vec<u8>) {
        self.assets.insert(path, content);
    }

    /// Get the number of assets
    pub fn len(&self) -> usize {
        self.assets.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.assets.is_empty()
    }

    /// Handle a protocol request and return a response
    ///
    /// This method handles:
    /// - `__loading__` - Returns the loading HTML if set
    /// - Empty path or `/` - Returns `index.html`
    /// - Other paths - Looks up in assets with fallback variations
    pub fn handle_request(&self, path: &str) -> FileResponse {
        let path = path.trim_start_matches('/');

        tracing::debug!("MemoryAssets: handling request for '{}'", path);

        // Handle special loading page
        if path == "__loading__" {
            if let Some(ref html) = self.loading_html {
                tracing::debug!("MemoryAssets: serving loading page ({} bytes)", html.len());
                return FileResponse {
                    data: Cow::Owned(html.clone().into_bytes()),
                    mime_type: "text/html; charset=utf-8".to_string(),
                    status: 200,
                };
            }
        }

        // Default to index.html for root path
        let path = if path.is_empty() { "index.html" } else { path };

        // Try different path variations
        let content = self
            .assets
            .get(path)
            .or_else(|| self.assets.get(&format!("frontend/{}", path)))
            .or_else(|| {
                // Try finding a path that ends with the requested path
                self.assets
                    .iter()
                    .find(|(p, _)| p.ends_with(&format!("/{}", path)))
                    .map(|(_, content)| content)
            });

        match content {
            Some(data) => {
                let mime = guess_mime_type(Path::new(path));
                tracing::debug!(
                    "MemoryAssets: serving '{}' ({} bytes, {})",
                    path,
                    data.len(),
                    mime
                );
                FileResponse {
                    data: Cow::Owned(data.clone()),
                    mime_type: mime,
                    status: 200,
                }
            }
            None => {
                tracing::warn!("MemoryAssets: asset not found: '{}'", path);
                FileResponse::not_found()
            }
        }
    }

    /// List all asset paths (for debugging)
    pub fn list_paths(&self) -> Vec<&String> {
        self.assets.keys().collect()
    }
}

impl Default for MemoryAssets {
    fn default() -> Self {
        Self::new()
    }
}
