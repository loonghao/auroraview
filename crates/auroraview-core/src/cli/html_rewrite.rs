//! HTML rewriting utilities for custom protocol handling.

use regex::Regex;

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

    // Helper function to check if a path is relative (should be rewritten)
    fn is_relative_path(path: &str) -> bool {
        !path.starts_with("http://")
            && !path.starts_with("https://")
            && !path.starts_with("data:")
            && !path.starts_with("//")
            && !path.starts_with("auroraview://")
            && !path.starts_with('#') // Anchor links
    }

    // Helper function to normalize relative path
    // Strips leading "./" prefix for cleaner URLs
    // Keeps "../" as the protocol handler will resolve it
    fn normalize_path(path: &str) -> &str {
        path.strip_prefix("./").unwrap_or(path)
    }

    // Rewrite link href
    let link_re = Regex::new(r#"<link\s+([^>]*)href="([^"]+)""#).unwrap();
    result = link_re
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
    let script_re = Regex::new(r#"<script\s+([^>]*)src="([^"]+)""#).unwrap();
    result = script_re
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
    let img_re = Regex::new(r#"<img\s+([^>]*)src="([^"]+)""#).unwrap();
    result = img_re
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
    let css_url_re = Regex::new(r#"url\(["']?([^"':)]+)["']?\)"#).unwrap();
    result = css_url_re
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rewrite_relative_paths() {
        let html = r#"
        <html>
            <head>
                <link rel="stylesheet" href="./style.css">
                <link rel="stylesheet" href="styles/main.css">
            </head>
            <body>
                <script src="./script.js"></script>
                <script src="js/app.js"></script>
                <img src="./logo.png">
                <img src="images/icon.png">
            </body>
        </html>
        "#;

        let result = rewrite_html_for_custom_protocol(html);

        assert!(result.contains(r#"href="auroraview://style.css""#));
        assert!(result.contains(r#"href="auroraview://styles/main.css""#));
        assert!(result.contains(r#"src="auroraview://script.js""#));
        assert!(result.contains(r#"src="auroraview://js/app.js""#));
        assert!(result.contains(r#"src="auroraview://logo.png""#));
        assert!(result.contains(r#"src="auroraview://images/icon.png""#));
    }

    #[test]
    fn test_preserve_absolute_urls() {
        let html = r#"<link href="https://cdn.example.com/style.css">"#;
        let result = rewrite_html_for_custom_protocol(html);
        assert!(result.contains(r#"href="https://cdn.example.com/style.css""#));
    }

    #[test]
    fn test_preserve_anchor_links() {
        let html = "<a href=\"#section\">Link</a>";
        let result = rewrite_html_for_custom_protocol(html);
        assert!(result.contains("href=\"#section\""));
    }

    #[test]
    fn test_empty_input() {
        let result = rewrite_html_for_custom_protocol("");
        assert_eq!(result, "");
    }
}
