//! HTML rewriting utilities for custom protocol handling.

use regex::Regex;
use std::sync::LazyLock;

/// Pre-compiled regex for rewriting `<link>` href attributes.
static LINK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<link\s+([^>]*)href="([^"]+)""#).unwrap());

/// Pre-compiled regex for rewriting `<script>` src attributes.
static SCRIPT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<script\s+([^>]*)src="([^"]+)""#).unwrap());

/// Pre-compiled regex for rewriting `<img>` src attributes.
static IMG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<img\s+([^>]*)src="([^"]+)""#).unwrap());

/// Pre-compiled regex for rewriting CSS `url()` references.
static CSS_URL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"url\(["']?([^"':)]+)["']?\)"#).unwrap());

/// Check if a path is relative (should be rewritten).
fn is_relative_path(path: &str) -> bool {
    !path.starts_with("http://")
        && !path.starts_with("https://")
        && !path.starts_with("data:")
        && !path.starts_with("//")
        && !path.starts_with("auroraview://")
        && !path.starts_with('#') // Anchor links
}

/// Normalize relative path by stripping leading "./" prefix.
/// Keeps "../" as the protocol handler will resolve it.
fn normalize_path(path: &str) -> &str {
    path.strip_prefix("./").unwrap_or(path)
}

/// Rewrite HTML to use auroraview:// protocol for relative paths
///
/// This function rewrites HTML content to use the custom auroraview:// protocol
/// for relative resource paths (CSS, JS, images), enabling proper loading of
/// local assets through the custom protocol handler.
///
/// # Arguments
/// * `html` - HTML content to rewrite
///
/// # Returns
/// Rewritten HTML with auroraview:// protocol for relative paths
///
/// # Examples
/// ```
/// use auroraview_core::cli::rewrite_html_for_custom_protocol;
///
/// let html = r#"<link href="style.css" rel="stylesheet">"#;
/// let rewritten = rewrite_html_for_custom_protocol(html);
/// assert!(rewritten.contains(r#"href="auroraview://style.css""#));
/// ```
pub fn rewrite_html_for_custom_protocol(html: &str) -> String {
    let mut result = html.to_string();

    // Rewrite link href
    result = LINK_RE
        .replace_all(&result, |caps: &regex::Captures| {
            let attrs = &caps[1];
            let path = &caps[2];
            if is_relative_path(path) {
                let normalized = normalize_path(path);
                format!(r#"<link {}href="auroraview://{}""#, attrs, normalized)
            } else {
                caps[0].to_string()
            }
        })
        .to_string();

    // Rewrite script src
    result = SCRIPT_RE
        .replace_all(&result, |caps: &regex::Captures| {
            let attrs = &caps[1];
            let path = &caps[2];
            if is_relative_path(path) {
                let normalized = normalize_path(path);
                format!(r#"<script {}src="auroraview://{}""#, attrs, normalized)
            } else {
                caps[0].to_string()
            }
        })
        .to_string();

    // Rewrite img src
    result = IMG_RE
        .replace_all(&result, |caps: &regex::Captures| {
            let attrs = &caps[1];
            let path = &caps[2];
            if is_relative_path(path) {
                let normalized = normalize_path(path);
                format!(r#"<img {}src="auroraview://{}""#, attrs, normalized)
            } else {
                caps[0].to_string()
            }
        })
        .to_string();

    // Rewrite CSS url()
    result = CSS_URL_RE
        .replace_all(&result, |caps: &regex::Captures| {
            let path = &caps[1];
            if is_relative_path(path) {
                let normalized = normalize_path(path);
                format!(r#"url("auroraview://{}")"#, normalized)
            } else {
                caps[0].to_string()
            }
        })
        .to_string();

    result
}
