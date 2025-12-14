//! Tests for CLI utilities

use auroraview_core::cli::{normalize_url, rewrite_html_for_custom_protocol};
use rstest::rstest;

// ============================================================================
// URL normalization tests
// ============================================================================

#[rstest]
fn test_normalize_url_without_scheme() {
    let result = normalize_url("example.com").unwrap();
    assert_eq!(result, "https://example.com/");
}

#[rstest]
fn test_normalize_url_with_http() {
    let result = normalize_url("http://example.com").unwrap();
    assert_eq!(result, "http://example.com/");
}

#[rstest]
fn test_normalize_url_with_https() {
    let result = normalize_url("https://example.com/path").unwrap();
    assert_eq!(result, "https://example.com/path");
}

#[rstest]
fn test_normalize_url_with_port() {
    let result = normalize_url("localhost:8080").unwrap();
    assert_eq!(result, "https://localhost:8080/");
}

#[rstest]
fn test_normalize_url_invalid() {
    let result = normalize_url("://invalid");
    assert!(result.is_err());
}

// ============================================================================
// HTML rewriting tests
// ============================================================================

#[rstest]
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

#[rstest]
fn test_preserve_absolute_urls() {
    let html = r#"<link href="https://cdn.example.com/style.css">"#;
    let result = rewrite_html_for_custom_protocol(html);
    assert!(result.contains(r#"href="https://cdn.example.com/style.css""#));
}

#[rstest]
fn test_preserve_anchor_links() {
    let html = "<a href=\"#section\">Link</a>";
    let result = rewrite_html_for_custom_protocol(html);
    assert!(result.contains("href=\"#section\""));
}

#[rstest]
fn test_empty_input() {
    let result = rewrite_html_for_custom_protocol("");
    assert_eq!(result, "");
}
