//! AuroraView Assets
//!
//! Centralized frontend assets for AuroraView, built with Vite.
//!
//! # Pages
//!
//! - `loading` - Loading screen with progress indication
//! - `error` - Next.js-style error overlay with diagnostics
//! - `browser` - Simple browser UI with tabs
//! - `browser-controller` - Full-featured browser controller (Edge-like)
//!
//! # Usage
//!
//! ```rust,ignore
//! use auroraview_assets::{Assets, get_asset, get_page_html};
//!
//! // Get raw asset bytes
//! let html = get_asset("loading/index.html").unwrap();
//!
//! // Get page HTML as string
//! let loading_html = get_page_html(Page::Loading).unwrap();
//! let error_html = get_page_html(Page::Error).unwrap();
//! ```

use rust_embed::RustEmbed;
use std::borrow::Cow;

/// Embedded assets from the Vite build output
#[derive(RustEmbed)]
#[folder = "frontend/dist/"]
#[include = "*.html"]
#[include = "*.js"]
#[include = "*.css"]
#[include = "*.svg"]
#[include = "*.png"]
#[include = "*.ico"]
pub struct Assets;

/// Available pages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Page {
    /// Loading screen with Aurora animation and progress
    Loading,
    /// Error overlay with diagnostics (Next.js style)
    Error,
    /// Simple browser UI with basic tabs
    Browser,
    /// Full-featured browser controller (Edge-like)
    BrowserController,
}

impl Page {
    /// Get the HTML file path for this page
    pub fn html_path(&self) -> &'static str {
        match self {
            Page::Loading => "loading/index.html",
            Page::Error => "error/index.html",
            Page::Browser => "browser/index.html",
            Page::BrowserController => "browser-controller/index.html",
        }
    }

    /// Get all available pages
    pub fn all() -> &'static [Page] {
        &[
            Page::Loading,
            Page::Error,
            Page::Browser,
            Page::BrowserController,
        ]
    }
}

/// Error types for asset operations
#[derive(Debug, thiserror::Error)]
pub enum AssetError {
    #[error("Asset not found: {0}")]
    NotFound(String),
    #[error("Invalid UTF-8 in asset: {0}")]
    InvalidUtf8(String),
}

/// Get raw asset bytes by path
pub fn get_asset(path: &str) -> Option<Cow<'static, [u8]>> {
    Assets::get(path).map(|f| f.data)
}

/// Get asset as UTF-8 string
pub fn get_asset_string(path: &str) -> Result<String, AssetError> {
    let data = get_asset(path).ok_or_else(|| AssetError::NotFound(path.to_string()))?;
    String::from_utf8(data.into_owned())
        .map_err(|_| AssetError::InvalidUtf8(path.to_string()))
}

/// Get page HTML content
pub fn get_page_html(page: Page) -> Result<String, AssetError> {
    get_asset_string(page.html_path())
}

/// Get the MIME type for an asset path
pub fn get_mime_type(path: &str) -> &'static str {
    mime_guess::from_path(path)
        .first_raw()
        .unwrap_or("application/octet-stream")
}

/// List all available assets
pub fn list_assets() -> Vec<String> {
    Assets::iter().map(|s| s.to_string()).collect()
}

/// Check if an asset exists
pub fn asset_exists(path: &str) -> bool {
    Assets::get(path).is_some()
}

// Re-export for convenience
pub use rust_embed;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_paths() {
        assert_eq!(Page::Loading.html_path(), "loading/index.html");
        assert_eq!(Page::Error.html_path(), "error/index.html");
        assert_eq!(Page::Browser.html_path(), "browser/index.html");
        assert_eq!(Page::BrowserController.html_path(), "browser-controller/index.html");
    }

    #[test]
    fn test_all_pages() {
        let pages = Page::all();
        assert_eq!(pages.len(), 4);
    }

    #[test]
    fn test_mime_types() {
        assert_eq!(get_mime_type("test.html"), "text/html");
        assert_eq!(get_mime_type("test.js"), "text/javascript");
        assert_eq!(get_mime_type("test.css"), "text/css");
        assert_eq!(get_mime_type("test.svg"), "image/svg+xml");
    }
}
