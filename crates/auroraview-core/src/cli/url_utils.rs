//! URL utilities for AuroraView CLI.

use url::Url;

/// Error type for URL operations
#[derive(Debug, thiserror::Error)]
pub enum UrlError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}

/// Normalize URL by adding https:// prefix if missing
///
/// # Arguments
/// * `url_str` - URL string to normalize
///
/// # Returns
/// Normalized URL with proper scheme
///
/// # Examples
/// ```
/// use auroraview_core::cli::normalize_url;
///
/// let url = normalize_url("example.com").unwrap();
/// assert_eq!(url, "https://example.com/");
///
/// let url = normalize_url("http://example.com").unwrap();
/// assert_eq!(url, "http://example.com/");
/// ```
pub fn normalize_url(url_str: &str) -> Result<String, UrlError> {
    // If it already has a scheme, validate and return
    if url_str.contains("://") {
        let url = Url::parse(url_str).map_err(|e| UrlError::InvalidUrl(e.to_string()))?;
        return Ok(url.to_string());
    }

    // Add https:// prefix for URLs without scheme
    let with_scheme = format!("https://{}", url_str);
    let url = Url::parse(&with_scheme).map_err(|e| UrlError::InvalidUrl(e.to_string()))?;
    Ok(url.to_string())
}
