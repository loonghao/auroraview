//! Tests for CLI utilities

use auroraview_core::cli::{normalize_url, rewrite_html_for_custom_protocol};
use rstest::rstest;

// ============================================================================
// URL normalization — original tests
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
// URL normalization — edge cases
// ============================================================================

#[rstest]
fn test_normalize_url_localhost_no_scheme() {
    let result = normalize_url("localhost").unwrap();
    // url crate may append trailing slash
    assert!(result.starts_with("https://localhost"));
}

#[rstest]
fn test_normalize_url_localhost_with_https() {
    let result = normalize_url("https://localhost").unwrap();
    assert!(result.starts_with("https://localhost"));
}

#[rstest]
fn test_normalize_url_with_path_and_query() {
    let result = normalize_url("https://example.com/api?foo=bar&baz=1").unwrap();
    assert_eq!(result, "https://example.com/api?foo=bar&baz=1");
}

#[rstest]
fn test_normalize_url_with_fragment() {
    let result = normalize_url("https://example.com/page#section").unwrap();
    assert_eq!(result, "https://example.com/page#section");
}

#[rstest]
fn test_normalize_url_with_subdomain() {
    let result = normalize_url("api.example.com").unwrap();
    assert_eq!(result, "https://api.example.com/");
}

#[rstest]
fn test_normalize_url_https_with_port() {
    let result = normalize_url("https://localhost:3000").unwrap();
    assert_eq!(result, "https://localhost:3000/");
}

#[rstest]
fn test_normalize_url_http_with_port() {
    let result = normalize_url("http://127.0.0.1:8080").unwrap();
    assert_eq!(result, "http://127.0.0.1:8080/");
}

#[rstest]
fn test_normalize_url_ipv4_no_scheme() {
    let result = normalize_url("192.168.1.1").unwrap();
    assert_eq!(result, "https://192.168.1.1/");
}

#[rstest]
fn test_normalize_url_with_credentials_in_url() {
    // URLs with user info are valid per RFC 3986
    let result = normalize_url("https://user:pass@example.com");
    // Should parse successfully
    assert!(result.is_ok());
}

#[rstest]
fn test_normalize_url_file_scheme() {
    let result = normalize_url("file:///C:/path/to/file.html").unwrap();
    assert!(result.starts_with("file:///"));
}

#[rstest]
fn test_normalize_url_deep_path() {
    let result = normalize_url("https://example.com/a/b/c/d/e/f.html").unwrap();
    assert_eq!(result, "https://example.com/a/b/c/d/e/f.html");
}

#[rstest]
fn test_normalize_url_encoded_chars() {
    let result = normalize_url("https://example.com/path%20with%20spaces").unwrap();
    assert!(result.contains("example.com"));
}

#[rstest]
fn test_normalize_url_no_scheme_with_path() {
    let result = normalize_url("example.com/some/path").unwrap();
    assert_eq!(result, "https://example.com/some/path");
}

#[rstest]
fn test_normalize_url_no_scheme_with_query() {
    let result = normalize_url("example.com?search=rust").unwrap();
    assert!(result.starts_with("https://example.com"));
    assert!(result.contains("search=rust"));
}

// ============================================================================
// URL normalization — rstest parametrized
// ============================================================================

#[rstest]
#[case("example.com", "https://example.com/")]
#[case("http://example.com", "http://example.com/")]
#[case("https://example.com/path", "https://example.com/path")]
#[case("localhost:8080", "https://localhost:8080/")]
fn test_normalize_url_valid_cases(#[case] input: &str, #[case] expected: &str) {
    let result = normalize_url(input).unwrap();
    assert_eq!(result, expected);
}

#[rstest]
#[case("://missing-host")]
#[case(":not-a-url")]
fn test_normalize_url_invalid_cases(#[case] input: &str) {
    let result = normalize_url(input);
    assert!(result.is_err(), "Expected error for input: {}", input);
}

#[rstest]
#[case("rust-lang.org")]
#[case("github.com")]
#[case("crates.io")]
#[case("docs.rs")]
fn test_normalize_url_well_known_domains_no_scheme(#[case] domain: &str) {
    let result = normalize_url(domain).unwrap();
    assert!(result.starts_with("https://"), "Should add https:// prefix");
    assert!(result.contains(domain));
}

// ============================================================================
// HTML rewriting — original tests
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

// ============================================================================
// HTML rewriting — edge cases
// ============================================================================

#[rstest]
fn test_rewrite_preserves_http_urls() {
    let html = r#"<script src="http://cdn.example.com/app.js"></script>"#;
    let result = rewrite_html_for_custom_protocol(html);
    assert!(
        result.contains("http://cdn.example.com/app.js"),
        "http:// URLs should not be rewritten"
    );
}

#[rstest]
fn test_rewrite_preserves_data_urls() {
    let html = r#"<img src="data:image/png;base64,abc123">"#;
    let result = rewrite_html_for_custom_protocol(html);
    assert!(result.contains("data:image/png;base64,abc123"));
}

#[rstest]
fn test_rewrite_script_module_type() {
    let html = r#"<script type="module" src="./main.js"></script>"#;
    let result = rewrite_html_for_custom_protocol(html);
    // Relative path should be rewritten
    assert!(result.contains("auroraview://main.js"));
}

#[rstest]
fn test_rewrite_nested_paths() {
    let html = r#"<link href="assets/css/theme.min.css">"#;
    let result = rewrite_html_for_custom_protocol(html);
    assert!(result.contains("auroraview://assets/css/theme.min.css"));
}

#[rstest]
fn test_rewrite_html_without_html_tags() {
    // Plain text should pass through unchanged
    let text = "This is plain text without HTML tags";
    let result = rewrite_html_for_custom_protocol(text);
    assert_eq!(result, text);
}

#[rstest]
fn test_rewrite_img_with_absolute_url() {
    let html = r#"<img src="https://example.com/image.png" alt="test">"#;
    let result = rewrite_html_for_custom_protocol(html);
    assert!(result.contains("https://example.com/image.png"));
}

#[rstest]
fn test_rewrite_multiple_same_resources() {
    let html = r#"<img src="./logo.png"><img src="./logo.png">"#;
    let result = rewrite_html_for_custom_protocol(html);
    // Both occurrences should be rewritten
    let count = result.matches("auroraview://logo.png").count();
    assert_eq!(count, 2, "Both identical relative paths should be rewritten");
}

#[rstest]
fn test_rewrite_preserves_html_structure() {
    let html = "<html><head></head><body><p>Hello</p></body></html>";
    let result = rewrite_html_for_custom_protocol(html);
    assert!(result.contains("<p>Hello</p>"));
}

// ============================================================================
// Additional coverage R9
// ============================================================================

#[rstest]
fn test_normalize_url_empty_string() {
    // empty string should fail or return error
    let result = normalize_url("");
    // empty string is not a valid URL — could be error or default-append https
    // We just verify it doesn't panic
    let _ = result;
}

#[rstest]
fn test_normalize_url_whitespace_only() {
    // whitespace string should fail
    let result = normalize_url("   ");
    let _ = result;
}

#[rstest]
fn test_normalize_url_ftp_scheme_passthrough() {
    // ftp:// is a known scheme, should pass through
    let result = normalize_url("ftp://files.example.com/pub");
    // either works or returns an error; just should not panic
    let _ = result;
}

#[rstest]
fn test_rewrite_preserves_auroraview_protocol() {
    // already-rewritten auroraview:// URLs should not be double-rewritten
    let html = r#"<script src="auroraview://app.js"></script>"#;
    let result = rewrite_html_for_custom_protocol(html);
    assert!(result.contains("auroraview://app.js"));
    // must not become auroraview://auroraview://app.js
    assert!(!result.contains("auroraview://auroraview://"));
}

#[rstest]
fn test_rewrite_link_with_multiple_attrs() {
    let html = r#"<link rel="stylesheet" type="text/css" href="theme.css" media="screen">"#;
    let result = rewrite_html_for_custom_protocol(html);
    assert!(result.contains("auroraview://theme.css"));
    assert!(result.contains(r#"rel="stylesheet""#));
    assert!(result.contains(r#"type="text/css""#));
}

#[rstest]
fn test_rewrite_script_with_integrity() {
    let html = r#"<script src="https://cdn.example.com/lib.js" integrity="sha384-abc"></script>"#;
    let result = rewrite_html_for_custom_protocol(html);
    // absolute https:// should NOT be rewritten
    assert!(result.contains("https://cdn.example.com/lib.js"));
}

#[rstest]
fn test_rewrite_img_empty_src() {
    let html = r#"<img src="" alt="empty">"#;
    let result = rewrite_html_for_custom_protocol(html);
    // empty src: behavior is defined by the implementation; just should not panic
    let _ = result;
}

#[rstest]
fn test_normalize_url_with_multiple_query_params() {
    let result = normalize_url("https://api.example.com/v1/data?key=val&foo=bar&baz=qux").unwrap();
    assert!(result.contains("key=val"));
    assert!(result.contains("foo=bar"));
    assert!(result.contains("baz=qux"));
}

#[rstest]
fn test_normalize_url_loopback() {
    let result = normalize_url("127.0.0.1:3000").unwrap();
    assert!(result.contains("127.0.0.1"));
    assert!(result.contains("3000"));
}

#[rstest]
fn test_rewrite_large_html() {
    // Build a large HTML document with many relative assets
    let mut html = String::from("<html><head>");
    for i in 0..50 {
        html.push_str(&format!(r#"<link href="style{}.css">"#, i));
    }
    html.push_str("</head><body>");
    for i in 0..50 {
        html.push_str(&format!(r#"<script src="script{}.js"></script>"#, i));
    }
    html.push_str("</body></html>");

    let result = rewrite_html_for_custom_protocol(&html);
    // Verify first and last assets are rewritten
    assert!(result.contains("auroraview://style0.css"));
    assert!(result.contains("auroraview://style49.css"));
    assert!(result.contains("auroraview://script0.js"));
    assert!(result.contains("auroraview://script49.js"));
}

// ============================================================================
// R15 Extensions
// ============================================================================

#[rstest]
fn test_normalize_url_returns_ok_for_http() {
    let result = normalize_url("http://example.com");
    assert!(result.is_ok(), "http URL should be Ok");
}

#[rstest]
fn test_normalize_url_returns_ok_for_https() {
    let result = normalize_url("https://example.com");
    assert!(result.is_ok(), "https URL should be Ok");
}

#[rstest]
fn test_normalize_url_result_is_not_empty_string() {
    let result = normalize_url("https://example.com");
    let url = result.unwrap();
    assert!(!url.is_empty());
}

#[rstest]
fn test_rewrite_empty_body() {
    let html = "<html><body></body></html>";
    let result = rewrite_html_for_custom_protocol(html);
    assert!(!result.is_empty());
}

#[rstest]
fn test_rewrite_preserves_absolute_https_url() {
    let html = r#"<html><body><script src="https://cdn.example.com/lib.js"></script></body></html>"#;
    let result = rewrite_html_for_custom_protocol(html);
    // Absolute HTTPS URLs should not be rewritten
    assert!(result.contains("https://cdn.example.com/lib.js"));
}

#[rstest]
fn test_normalize_url_localhost_with_port_produces_valid_url() {
    let result = normalize_url("localhost:3000");
    assert!(result.is_ok(), "localhost:3000 should be parseable");
    let url = result.unwrap();
    assert!(url.contains("localhost"));
}
