//! Static assets for AuroraView
//!
//! This module provides embedded static assets including:
//! - Loading HTML page
//! - JavaScript utilities (event bridge, context menu, etc.)
//! - BOM (Browser Object Model) scripts

use rust_embed::RustEmbed;

/// Embedded static assets
#[derive(RustEmbed)]
#[folder = "src/assets/"]
pub struct Assets;

/// Get the loading HTML page content
pub fn get_loading_html() -> String {
    Assets::get("html/loading.html")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_else(|| {
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            margin: 0;
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
            color: white;
            font-family: system-ui, -apple-system, sans-serif;
        }
        .spinner {
            width: 50px;
            height: 50px;
            border: 3px solid rgba(255,255,255,0.3);
            border-radius: 50%;
            border-top-color: #fff;
            animation: spin 1s linear infinite;
        }
        @keyframes spin {
            to { transform: rotate(360deg); }
        }
        .container { text-align: center; }
        h1 { font-size: 1.5rem; margin-top: 1rem; }
    </style>
</head>
<body>
    <div class="container">
        <div class="spinner"></div>
        <h1>Loading...</h1>
    </div>
</body>
</html>"#
                .to_string()
        })
}

/// Get the error HTML page content
///
/// Returns a styled error page that matches the AuroraView design language.
/// The page accepts URL parameters to customize the error display:
/// - `code`: HTTP status code (e.g., "500", "404")
/// - `title`: Error title (e.g., "Internal Server Error")
/// - `message`: User-friendly error message
/// - `details`: Technical details (shown in a code block)
/// - `url`: The URL that caused the error (for retry functionality)
pub fn get_error_html() -> String {
    Assets::get("html/error.html")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_else(|| {
            // Fallback minimal error page
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            margin: 0;
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
            color: white;
            font-family: system-ui, -apple-system, sans-serif;
        }
        .container { text-align: center; }
        h1 { font-size: 4rem; margin: 0; color: #f36262; }
        h2 { font-size: 1.5rem; margin: 1rem 0; }
        p { opacity: 0.8; }
        button {
            margin-top: 1rem;
            padding: 10px 24px;
            background: #f36262;
            border: none;
            border-radius: 6px;
            color: white;
            cursor: pointer;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>Error</h1>
        <h2>Something went wrong</h2>
        <p>An unexpected error occurred.</p>
        <button onclick="location.reload()">Try Again</button>
    </div>
</body>
</html>"#
                .to_string()
        })
}

/// Build an error page HTML with specific error information
///
/// This function generates a complete error page by injecting error details
/// into the base error template via URL parameters.
///
/// # Arguments
/// * `code` - HTTP status code (e.g., 500, 404)
/// * `title` - Error title
/// * `message` - User-friendly error message
/// * `details` - Optional technical details
/// * `url` - Optional URL that caused the error
pub fn build_error_page(
    code: u16,
    title: &str,
    message: &str,
    details: Option<&str>,
    url: Option<&str>,
) -> String {
    let base_html = get_error_html();

    // Build JavaScript to update the error display
    let js_update = format!(
        r#"<script>
        (function() {{
            // Store error info globally for copy function
            window._errorInfo = {{
                code: '{}',
                title: '{}',
                message: '{}',
                details: '{}',
                url: '{}'
            }};

            document.addEventListener('DOMContentLoaded', function() {{
                var info = window._errorInfo;
                document.getElementById('error-code').textContent = info.code;
                document.getElementById('error-title').textContent = info.title;
                document.getElementById('error-message').textContent = info.message;

                var detailsWrapper = document.getElementById('details-wrapper');
                var detailsEl = document.getElementById('error-details');

                if (info.details || info.url) {{
                    var text = '';
                    if (info.url) text += 'URL: ' + info.url + '\\n';
                    if (info.details) text += info.details;
                    detailsEl.textContent = text.trim();
                    detailsWrapper.style.display = 'block';
                }} else {{
                    detailsWrapper.style.display = 'none';
                }}
            }});

            // Override getErrorInfo to use injected data
            window.getErrorInfo = function() {{
                return window._errorInfo;
            }};
        }})();
        </script>"#,
        code,
        escape_js_string(title),
        escape_js_string(message),
        escape_js_string(details.unwrap_or("")),
        escape_js_string(url.unwrap_or(""))
    );

    // Insert the JavaScript before </body>
    base_html.replace("</body>", &format!("{}</body>", js_update))
}

/// Escape a string for safe inclusion in JavaScript
fn escape_js_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

/// Get the event bridge JavaScript code
pub fn get_event_bridge_js() -> String {
    Assets::get("js/core/event_bridge.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Get the context menu JavaScript code
pub fn get_context_menu_js() -> String {
    Assets::get("js/features/context_menu.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Get the emit event JavaScript code
pub fn get_emit_event_js() -> String {
    Assets::get("js/runtime/emit_event.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Get the load URL JavaScript code
pub fn get_load_url_js() -> String {
    Assets::get("js/runtime/load_url.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

// ========================================
// BOM (Browser Object Model) Scripts
// ========================================

/// Get the navigation tracker JavaScript code
pub fn get_navigation_tracker_js() -> String {
    Assets::get("js/bom/navigation_tracker.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Get the DOM events JavaScript code
pub fn get_dom_events_js() -> String {
    Assets::get("js/bom/dom_events.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Get the browsing data JavaScript code
pub fn get_browsing_data_js() -> String {
    Assets::get("js/bom/browsing_data.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Get the navigation API JavaScript code
pub fn get_navigation_api_js() -> String {
    Assets::get("js/bom/navigation_api.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Get the zoom API JavaScript code
pub fn get_zoom_api_js() -> String {
    Assets::get("js/bom/zoom_api.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Get the file drop handler JavaScript code
///
/// This script provides file drag and drop handling capabilities.
/// It intercepts drag/drop events and sends file information to Python.
/// Events emitted:
/// - file_drop_hover: When files are dragged over the window
/// - file_drop: When files are dropped
/// - file_drop_cancelled: When drag operation is cancelled
/// - file_paste: When files are pasted from clipboard
pub fn get_file_drop_js() -> String {
    Assets::get("js/bom/file_drop.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

// ========================================
// Core Bridge Scripts
// ========================================

/// Get the bridge stub JavaScript code
///
/// This stub creates a minimal window.auroraview namespace before the full
/// event bridge is loaded. Use this in DCC environments where timing may vary.
///
/// The stub:
/// - Creates a placeholder `window.auroraview` with queuing support
/// - Queues any `call()`, `send_event()`, `on()` calls made before bridge init
/// - Provides `whenReady()` Promise API for safe async initialization
/// - Automatically replays queued calls when real bridge initializes
///
/// # Example
///
/// ```javascript
/// // In DCC frontend code (before bridge is ready)
/// window.auroraview.whenReady().then(function(av) {
///     av.call('api.myMethod', { param: 'value' });
/// });
/// ```
pub fn get_bridge_stub_js() -> String {
    Assets::get("js/core/bridge_stub.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Get the state bridge JavaScript code
pub fn get_state_bridge_js() -> String {
    Assets::get("js/core/state_bridge.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Get the command bridge JavaScript code
pub fn get_command_bridge_js() -> String {
    Assets::get("js/core/command_bridge.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Get the channel bridge JavaScript code
pub fn get_channel_bridge_js() -> String {
    Assets::get("js/core/channel_bridge.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Get the event utilities JavaScript code
///
/// This script provides utility functions for event handling:
/// - debounce: Delays function execution until after wait milliseconds
/// - throttle: Limits function execution to at most once per wait milliseconds
/// - once: Restricts function to single invocation
/// - onDebounced/onThrottled: Convenience wrappers for event handlers
pub fn get_event_utils_js() -> String {
    Assets::get("js/core/event_utils.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

// ========================================
// Feature Scripts
// ========================================

/// Get the screenshot JavaScript code
///
/// This script provides screenshot capture functionality using html2canvas.
/// It dynamically loads html2canvas from CDN and provides methods for:
/// - Full page screenshots
/// - Element screenshots
/// - Viewport screenshots
pub fn get_screenshot_js() -> String {
    Assets::get("js/features/screenshot.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Get the network interception JavaScript code
///
/// This script provides network request interception and mocking capabilities.
/// It intercepts fetch() and XMLHttpRequest to enable:
/// - Request interception with pattern matching
/// - Response mocking
/// - Network monitoring and logging
pub fn get_network_intercept_js() -> String {
    Assets::get("js/features/network_intercept.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Get the test callback JavaScript code
///
/// This script provides callback mechanism for AuroraTest framework
/// to receive JavaScript evaluation results asynchronously.
pub fn get_test_callback_js() -> String {
    Assets::get("js/features/test_callback.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

// ========================================
// Plugin Scripts
// ========================================

/// Get the file system plugin JavaScript code
pub fn get_fs_plugin_js() -> String {
    Assets::get("js/plugins/fs.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Get the dialog plugin JavaScript code
pub fn get_dialog_plugin_js() -> String {
    Assets::get("js/plugins/dialog.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Get the clipboard plugin JavaScript code
pub fn get_clipboard_plugin_js() -> String {
    Assets::get("js/plugins/clipboard.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Get the shell plugin JavaScript code
pub fn get_shell_plugin_js() -> String {
    Assets::get("js/plugins/shell.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Get plugin JavaScript by name
pub fn get_plugin_js(name: &str) -> Option<String> {
    match name {
        "fs" => Some(get_fs_plugin_js()),
        "dialog" => Some(get_dialog_plugin_js()),
        "clipboard" => Some(get_clipboard_plugin_js()),
        "shell" => Some(get_shell_plugin_js()),
        _ => None,
    }
}

/// List available plugin names
pub fn plugin_names() -> &'static [&'static str] {
    &["fs", "dialog", "clipboard", "shell"]
}

/// Get all plugin JavaScript code concatenated
pub fn get_all_plugins_js() -> String {
    let mut scripts = Vec::new();

    // Add all plugins
    for name in plugin_names() {
        if let Some(js) = get_plugin_js(name) {
            if !js.is_empty() {
                scripts.push(js);
            }
        }
    }

    scripts.join("\n\n")
}

// ========================================
// Generic Asset Access
// ========================================

/// Get any JavaScript asset by path
///
/// # Arguments
/// * `path` - Path relative to js/ directory (e.g., "core/event_bridge.js")
pub fn get_js_asset(path: &str) -> Option<String> {
    let full_path = format!("js/{}", path);
    Assets::get(&full_path).map(|f| String::from_utf8_lossy(&f.data).to_string())
}

/// Get TypeScript definition file
pub fn get_typescript_definitions() -> String {
    Assets::get("types/auroraview.d.ts")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Build JavaScript to load a URL
pub fn build_load_url_script(url: &str) -> String {
    format!(
        r#"window.location.href = "{}";"#,
        url.replace('\\', "\\\\").replace('"', "\\\"")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loading_html_not_empty() {
        let html = get_loading_html();
        assert!(!html.is_empty());
        assert!(html.contains("Loading") || html.contains("loading"));
    }

    #[test]
    fn test_error_html_not_empty() {
        let html = get_error_html();
        assert!(!html.is_empty());
        assert!(html.contains("Error") || html.contains("error"));
    }

    #[test]
    fn test_build_error_page() {
        let html = build_error_page(
            500,
            "Internal Server Error",
            "Something went wrong",
            Some("Details here"),
            Some("https://example.com"),
        );
        assert!(html.contains("500"));
        assert!(html.contains("Internal Server Error"));
        assert!(html.contains("Something went wrong"));
        assert!(html.contains("Details here"));
        assert!(html.contains("https://example.com"));
    }

    #[test]
    fn test_build_error_page_without_details() {
        let html = build_error_page(404, "Not Found", "Page not found", None, None);
        assert!(html.contains("404"));
        assert!(html.contains("Not Found"));
        assert!(html.contains("Page not found"));
    }

    #[test]
    fn test_build_load_url_script() {
        let script = build_load_url_script("https://example.com");
        assert!(script.contains("https://example.com"));
        assert!(script.contains("window.location.href"));
    }

    #[test]
    fn test_bom_scripts_available() {
        // These may be empty if assets aren't embedded, but shouldn't panic
        let _ = get_navigation_tracker_js();
        let _ = get_dom_events_js();
        let _ = get_browsing_data_js();
        let _ = get_navigation_api_js();
        let _ = get_zoom_api_js();
    }

    #[test]
    fn test_file_drop_js_available() {
        let js = get_file_drop_js();
        // Should contain file drop handler code
        assert!(js.contains("file_drop") || js.is_empty());
    }

    #[test]
    fn test_event_utils_js_available() {
        let js = get_event_utils_js();
        // Should contain debounce/throttle utilities
        assert!(js.contains("debounce") || js.is_empty());
    }

    #[test]
    fn test_event_bridge_js_available() {
        let js = get_event_bridge_js();
        // Should contain auroraview bridge code
        assert!(js.contains("auroraview") || js.is_empty());
    }

    #[test]
    fn test_plugin_names() {
        let names = plugin_names();
        assert!(names.contains(&"fs"));
        assert!(names.contains(&"dialog"));
        assert!(names.contains(&"clipboard"));
        assert!(names.contains(&"shell"));
        assert_eq!(names.len(), 4);
    }

    #[test]
    fn test_get_plugin_js_valid() {
        // Test valid plugin names
        assert!(get_plugin_js("fs").is_some());
        assert!(get_plugin_js("dialog").is_some());
        assert!(get_plugin_js("clipboard").is_some());
        assert!(get_plugin_js("shell").is_some());
    }

    #[test]
    fn test_get_plugin_js_invalid() {
        // Test invalid plugin name
        assert!(get_plugin_js("nonexistent").is_none());
        assert!(get_plugin_js("").is_none());
    }

    #[test]
    fn test_get_all_plugins_js() {
        let all_js = get_all_plugins_js();
        // Should contain code from all plugins
        assert!(!all_js.is_empty() || plugin_names().is_empty());
    }

    #[test]
    fn test_escape_js_string() {
        // Test escaping special characters
        assert_eq!(escape_js_string("hello"), "hello");
        assert_eq!(escape_js_string("it's"), "it\\'s");
        assert_eq!(escape_js_string("line1\nline2"), "line1\\nline2");
        assert_eq!(escape_js_string("path\\to\\file"), "path\\\\to\\\\file");
        assert_eq!(escape_js_string("say \"hi\""), "say \\\"hi\\\"");
    }

    #[test]
    fn test_bridge_stub_js_available() {
        let js = get_bridge_stub_js();
        // Should contain stub code for early initialization
        assert!(js.contains("auroraview") || js.is_empty());
    }

    #[test]
    fn test_state_bridge_js_available() {
        let js = get_state_bridge_js();
        // Should be available (may be empty if not embedded)
        let _ = js;
    }

    #[test]
    fn test_channel_bridge_js_available() {
        let js = get_channel_bridge_js();
        // Should be available
        let _ = js;
    }

    #[test]
    fn test_screenshot_js_available() {
        let js = get_screenshot_js();
        // Should contain screenshot functionality
        assert!(js.contains("screenshot") || js.is_empty());
    }

    #[test]
    fn test_network_intercept_js_available() {
        let js = get_network_intercept_js();
        // Should contain network interception code
        let _ = js;
    }

    #[test]
    fn test_get_js_asset() {
        // Test getting asset by path
        let result = get_js_asset("core/event_bridge.js");
        // May be Some or None depending on whether assets are embedded
        let _ = result;
    }

    #[test]
    fn test_get_js_asset_invalid_path() {
        let result = get_js_asset("nonexistent/file.js");
        assert!(result.is_none());
    }
}
