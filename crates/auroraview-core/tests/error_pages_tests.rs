//! Error pages HTML template tests

use auroraview_core::assets::html::{
    connection_error_page, internal_error_page, loading_with_error, not_found_page,
    python_error_page, startup_error_page,
};
use rstest::rstest;

// ============================================================================
// html_escape coverage (tested indirectly through page generation)
// ============================================================================

#[test]
fn test_html_escape_lt_gt() {
    let page = internal_error_page("<script>alert(1)</script>", None);
    assert!(page.contains("&lt;script&gt;"));
    assert!(!page.contains("<script>alert(1)</script>"));
}

#[test]
fn test_html_escape_ampersand() {
    let page = internal_error_page("a & b error", None);
    assert!(page.contains("a &amp; b error"));
}

#[test]
fn test_html_escape_double_quote() {
    let page = internal_error_page("error \"msg\"", None);
    assert!(page.contains("&quot;"));
    assert!(!page.contains("\"msg\""));
}

#[test]
fn test_html_escape_single_quote() {
    let page = internal_error_page("it's broken", None);
    assert!(page.contains("&#39;s broken"));
}

#[test]
fn test_html_escape_all_chars_combined() {
    let page = internal_error_page("<div class=\"x\">a & b's c</div>", None);
    assert!(page.contains("&lt;div"));
    assert!(page.contains("&amp;"));
    assert!(page.contains("&quot;"));
    assert!(page.contains("&#39;"));
}

#[test]
fn test_html_escape_unicode_passthrough() {
    let page = internal_error_page("错误信息 🚫", None);
    assert!(page.contains("错误信息"));
    assert!(page.contains("🚫"));
}

#[test]
fn test_html_escape_empty_string() {
    let page = internal_error_page("", None);
    assert!(page.contains("500"));
}

#[test]
fn test_html_escape_very_long_input() {
    let long = "x".repeat(10_000);
    let page = internal_error_page(&long, None);
    assert!(page.contains(&long));
}

#[test]
fn test_html_escape_null_byte_passthrough() {
    // null bytes are not HTML-special; they pass through
    let page = internal_error_page("err\x00msg", None);
    assert!(page.contains("500"));
}

// ============================================================================
// not_found_page
// ============================================================================

#[test]
fn test_not_found_page_basic() {
    let page = not_found_page("/missing.js", None);
    assert!(page.contains("404"));
    assert!(page.contains("Not Found"));
    assert!(page.contains("/missing.js"));
    assert!(page.contains("<!DOCTYPE html>"));
}

#[test]
fn test_not_found_page_xss_in_path() {
    let page = not_found_page("<script>xss()</script>", None);
    assert!(page.contains("&lt;script&gt;"));
    assert!(!page.contains("<script>xss()"));
}

#[test]
fn test_not_found_page_with_empty_asset_list() {
    let page = not_found_page("/x", Some(vec![]));
    assert!(page.contains("0 files"));
}

#[test]
fn test_not_found_page_with_assets_shows_all() {
    let assets = vec!["index.html", "app.js", "style.css"];
    let page = not_found_page("/x", Some(assets));
    assert!(page.contains("3 files"));
    assert!(page.contains("index.html"));
    assert!(page.contains("app.js"));
    assert!(page.contains("style.css"));
}

#[test]
fn test_not_found_page_asset_list_capped_at_20() {
    let assets: Vec<&str> = (0..25).map(|_| "file.js").collect();
    let page = not_found_page("/x", Some(assets));
    assert!(page.contains("25 files"));
    assert!(page.contains("and 5 more files"));
}

#[test]
fn test_not_found_page_exactly_20_assets_no_more_label() {
    let assets: Vec<&str> = (0..20).map(|_| "file.js").collect();
    let page = not_found_page("/x", Some(assets));
    assert!(page.contains("20 files"));
    assert!(!page.contains("and 0 more"));
    assert!(!page.contains("more files"));
}

#[test]
fn test_not_found_page_21_assets_shows_and_1_more() {
    let assets: Vec<&str> = (0..21).map(|_| "f.js").collect();
    let page = not_found_page("/x", Some(assets));
    assert!(page.contains("and 1 more files"));
}

#[test]
fn test_not_found_page_xss_in_asset_name() {
    let assets = vec!["<img src=x onerror=alert(1)>"];
    let page = not_found_page("/x", Some(assets));
    assert!(page.contains("&lt;img"));
}

#[rstest]
#[case("/api/v1/data")]
#[case("/images/photo.png")]
#[case("/deeply/nested/path/index.html")]
#[case("")]
fn test_not_found_page_various_paths(#[case] path: &str) {
    let page = not_found_page(path, None);
    assert!(page.contains("404"));
}

// ============================================================================
// internal_error_page
// ============================================================================

#[test]
fn test_internal_error_page_without_details() {
    let page = internal_error_page("Something went wrong", None);
    assert!(page.contains("500"));
    assert!(page.contains("Internal Error"));
    assert!(page.contains("Something went wrong"));
    // details section should not appear in the <body> when None is passed
    // (CSS may define .error-details class, but the <pre> element should be absent)
    assert!(!page.contains("<pre class=\"error-details\">"));
}

#[test]
fn test_internal_error_page_with_details() {
    let page = internal_error_page("DB failure", Some("Connection refused at port 5432"));
    assert!(page.contains("500"));
    assert!(page.contains("DB failure"));
    assert!(page.contains("Connection refused at port 5432"));
    assert!(page.contains("error-details"));
}

#[test]
fn test_internal_error_page_details_xss() {
    let page = internal_error_page("err", Some("<script>xss()</script>"));
    assert!(page.contains("&lt;script&gt;"));
    assert!(!page.contains("<script>xss()"));
}

#[test]
fn test_internal_error_page_message_xss() {
    let page = internal_error_page("<b>bold</b>", None);
    assert!(page.contains("&lt;b&gt;"));
}

#[test]
fn test_internal_error_page_empty_message() {
    let page = internal_error_page("", None);
    assert!(page.contains("500"));
    assert!(page.contains("Internal Error"));
}

#[rstest]
#[case("disk full", Some("ENOSPC"))]
#[case("out of memory", Some("OOM at heap address 0x0"))]
#[case("timeout", None)]
fn test_internal_error_page_variants(#[case] msg: &str, #[case] detail: Option<&str>) {
    let page = internal_error_page(msg, detail);
    assert!(page.contains(msg));
    if let Some(d) = detail {
        assert!(page.contains(d));
    }
}

// ============================================================================
// python_error_page
// ============================================================================

#[test]
fn test_python_error_page_basic() {
    let page = python_error_page("ImportError", "No module named 'xyz'", None);
    assert!(page.contains("Python Error"));
    assert!(page.contains("ImportError"));
    assert!(page.contains("No module named"));
    // Traceback <details> element should not appear when traceback is None
    assert!(!page.contains("Python Traceback"));
}

#[test]
fn test_python_error_page_with_traceback() {
    let tb = "Traceback (most recent call last):\n  File \"main.py\", line 1\nImportError: xyz";
    let page = python_error_page("ImportError", "No module named 'xyz'", Some(tb));
    assert!(page.contains("Python Traceback"));
    assert!(page.contains("Traceback"));
    assert!(page.contains("main.py"));
}

#[test]
fn test_python_error_page_xss_in_type() {
    let page = python_error_page("<SyntaxError>", "msg", None);
    assert!(page.contains("&lt;SyntaxError&gt;"));
}

#[test]
fn test_python_error_page_xss_in_message() {
    let page = python_error_page("TypeError", "<script>xss</script>", None);
    assert!(page.contains("&lt;script&gt;"));
}

#[test]
fn test_python_error_page_xss_in_traceback() {
    let page = python_error_page("Err", "msg", Some("<img src=x onerror=1>"));
    assert!(page.contains("&lt;img"));
}

#[rstest]
#[case("ValueError", "invalid literal for int()", None)]
#[case("RuntimeError", "maximum recursion depth exceeded", Some("  File x.py line 99\nRuntimeError"))]
#[case("AttributeError", "'NoneType' has no attribute 'x'", None)]
fn test_python_error_page_variants(
    #[case] err_type: &str,
    #[case] msg: &str,
    #[case] tb: Option<&str>,
) {
    let page = python_error_page(err_type, msg, tb);
    assert!(page.contains("Python Error"));
    assert!(page.contains(err_type));
    if let Some(t) = tb {
        let first_line = t.lines().next().unwrap_or("");
        assert!(page.contains(first_line));
    }
}

// ============================================================================
// connection_error_page
// ============================================================================

#[test]
fn test_connection_error_page_basic() {
    let page = connection_error_page("http://localhost:8080", "ECONNREFUSED");
    assert!(page.contains("Connection Error"));
    assert!(page.contains("localhost:8080"));
    assert!(page.contains("ECONNREFUSED"));
}

#[test]
fn test_connection_error_page_xss_in_target() {
    let page = connection_error_page("<script>evil()</script>", "err");
    assert!(page.contains("&lt;script&gt;"));
}

#[test]
fn test_connection_error_page_xss_in_error() {
    let page = connection_error_page("target", "<b>bold error</b>");
    assert!(page.contains("&lt;b&gt;"));
}

#[test]
fn test_connection_error_page_has_retry_countdown() {
    let page = connection_error_page("http://x", "timeout");
    assert!(page.contains("countdown"));
    assert!(page.contains("5"));
}

#[rstest]
#[case("http://127.0.0.1:3000", "connection timeout after 30s")]
#[case("ws://localhost:9090/ws", "WebSocket handshake failed")]
#[case("", "unknown host")]
fn test_connection_error_page_variants(#[case] target: &str, #[case] error: &str) {
    let page = connection_error_page(target, error);
    assert!(page.contains("Connection Error"));
}

// ============================================================================
// startup_error_page
// ============================================================================

#[test]
fn test_startup_error_page_minimal() {
    let page = startup_error_page("Python not found", None, None);
    assert!(page.contains("Startup Failed"));
    assert!(page.contains("Python not found"));
    assert!(!page.contains("Python Output"));
    assert!(!page.contains("Entry point"));
}

#[test]
fn test_startup_error_page_with_python_output() {
    let output = "Traceback:\n  ImportError: maya not available";
    let page = startup_error_page("failed", Some(output), None);
    assert!(page.contains("Python Output"));
    assert!(page.contains("ImportError"));
}

#[test]
fn test_startup_error_page_with_entry_point() {
    let page = startup_error_page("failed", None, Some("main:run_app"));
    assert!(page.contains("Entry point"));
    assert!(page.contains("main:run_app"));
}

#[test]
fn test_startup_error_page_with_all_fields() {
    let page = startup_error_page("failed", Some("output"), Some("app:main"));
    assert!(page.contains("Python Output"));
    assert!(page.contains("Entry point"));
    assert!(page.contains("app:main"));
}

#[test]
fn test_startup_error_page_xss_in_error_message() {
    let page = startup_error_page("<script>xss</script>", None, None);
    assert!(page.contains("&lt;script&gt;"));
}

#[test]
fn test_startup_error_page_xss_in_python_output() {
    let page = startup_error_page("err", Some("<img onerror=1>"), None);
    assert!(page.contains("&lt;img"));
}

#[test]
fn test_startup_error_page_xss_in_entry_point() {
    let page = startup_error_page("err", None, Some("<path>"));
    assert!(page.contains("&lt;path&gt;"));
}

#[test]
fn test_startup_error_page_troubleshooting_tips_present() {
    let page = startup_error_page("err", None, None);
    assert!(page.contains("Troubleshooting"));
    assert!(page.contains("Python dependencies"));
    assert!(page.contains("entry point"));
}

#[rstest]
#[case("module 'maya' has no attribute 'cmds'", None, None)]
#[case("syntax error in config.py", Some("SyntaxError at line 5"), Some("config:load"))]
#[case("", Some(""), Some(""))]
fn test_startup_error_page_variants(
    #[case] msg: &str,
    #[case] output: Option<&str>,
    #[case] entry: Option<&str>,
) {
    let page = startup_error_page(msg, output, entry);
    assert!(page.contains("Startup Failed"));
}

// ============================================================================
// loading_with_error
// ============================================================================

#[test]
fn test_loading_with_error_no_error() {
    let page = loading_with_error("Initializing...", None);
    assert!(page.contains("Loading"));
    assert!(page.contains("Initializing..."));
    // The error-message div should not appear when no error is given
    // (CSS defines the class, but the actual element should not be in the body)
    assert!(!page.contains("class=\"error-message\""));
}

#[test]
fn test_loading_with_error_with_error() {
    let page = loading_with_error("Starting up...", Some("WebView2 not installed"));
    assert!(page.contains("Loading"));
    assert!(page.contains("WebView2 not installed"));
    // error-message div element should be present (not just the CSS class)
    assert!(page.contains("class=\"error-message\""));
}

#[test]
fn test_loading_with_error_xss_in_status() {
    let page = loading_with_error("<script>xss</script>", None);
    assert!(page.contains("&lt;script&gt;"));
}

#[test]
fn test_loading_with_error_xss_in_error_message() {
    let page = loading_with_error("ok", Some("<img src=x>"));
    assert!(page.contains("&lt;img"));
}

#[test]
fn test_loading_with_error_has_spinner() {
    let page = loading_with_error("Loading...", None);
    assert!(page.contains("spinner"));
}

#[rstest]
#[case("Connecting to backend", None)]
#[case("Loading assets", Some("asset server timeout"))]
#[case("", Some(""))]
fn test_loading_with_error_variants(#[case] status: &str, #[case] err: Option<&str>) {
    let page = loading_with_error(status, err);
    assert!(page.contains("Loading"));
}

// ============================================================================
// General HTML structure
// ============================================================================

#[rstest]
fn test_all_pages_have_doctype() {
    let pages = vec![
        not_found_page("/x", None),
        internal_error_page("err", None),
        python_error_page("Err", "msg", None),
        connection_error_page("target", "err"),
        startup_error_page("err", None, None),
        loading_with_error("loading", None),
    ];
    for page in &pages {
        assert!(
            page.contains("<!DOCTYPE html>"),
            "Page missing DOCTYPE: {}",
            &page[..100]
        );
        assert!(page.contains("</html>"), "Page missing closing </html>");
        assert!(page.contains("charset=\"UTF-8\""), "Page missing charset");
    }
}

#[test]
fn test_all_pages_have_viewport_meta() {
    let pages = vec![
        not_found_page("/x", None),
        internal_error_page("err", None),
        python_error_page("Err", "msg", None),
        connection_error_page("target", "err"),
        startup_error_page("err", None, None),
        loading_with_error("loading", None),
    ];
    for page in &pages {
        assert!(
            page.contains("viewport"),
            "Page missing viewport meta: {}",
            &page[..100]
        );
    }
}
